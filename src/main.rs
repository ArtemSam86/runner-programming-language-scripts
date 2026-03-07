use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, HeaderValue},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use bytes::Bytes;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr, // <-- для явного указания типа адреса
    path::{Path as StdPath, PathBuf},
    sync::Arc,
    time::{Duration, Instant, SystemTime},
};
use thiserror::Error;
use tokio::{
    fs,
    io::AsyncWriteExt,
    net::TcpListener,
    process::Command,
    sync::{Mutex, Semaphore},
    time::timeout,
};
use chrono::{DateTime, Local, Utc};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// ------------------------------------------------------------
// Ошибки приложения
// ------------------------------------------------------------
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Script '{0}' not found")]
    ScriptNotFound(String),
    #[error("Script name invalid: {0}")]
    InvalidScriptName(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Script execution timed out")]
    Timeout,
    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, msg) = match self {
            AppError::ScriptNotFound(name) => (
                StatusCode::NOT_FOUND,
                format!("Script '{}' not found", name),
            ),
            AppError::InvalidScriptName(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Io(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("IO error: {}", e),
            ),
            AppError::Json(e) => (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e)),
            AppError::Utf8(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("UTF-8 error: {}", e),
            ),
            AppError::Timeout => (
                StatusCode::GATEWAY_TIMEOUT,
                "Script execution timed out".to_string(),
            ),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        (status, msg).into_response()
    }
}

// ------------------------------------------------------------
// Типы для кэширования
// ------------------------------------------------------------
#[derive(Clone)]
struct CachedResult {
    stdout: String,
    stderr: String,
    exit_code: i32,
    timestamp: Instant,
    script_mtime: SystemTime,
}

// ------------------------------------------------------------
// Состояние приложения
// ------------------------------------------------------------
pub struct AppState {
    scripts_dir: PathBuf,
    descriptions_path: PathBuf,
    scripts: Mutex<Vec<PathBuf>>,
    semaphore: Semaphore,
    cache: Mutex<HashMap<String, CachedResult>>,
    cache_ttl: Duration,
}

impl AppState {
    fn new(scripts_dir: PathBuf, max_concurrent: usize, cache_ttl: Duration) -> Arc<Self> {
        let descriptions_path = scripts_dir.join("descriptions.json");
        Arc::new(Self {
            scripts_dir,
            descriptions_path,
            scripts: Mutex::new(Vec::new()),
            semaphore: Semaphore::new(max_concurrent),
            cache: Mutex::new(HashMap::new()),
            cache_ttl,
        })
    }

    // Сканирование директории и обновление списка скриптов
    async fn scan_scripts(&self) {
        let mut scripts = self.scripts.lock().await;
        *scripts = match fs::read_dir(&self.scripts_dir).await {
            Ok(mut entries) => {
                let mut list = Vec::new();
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();
                    if path.extension().and_then(|ext| ext.to_str()) == Some("py") {
                        list.push(path);
                    }
                }
                list
            }
            Err(e) => {
                error!("Failed to read scripts dir: {}", e);
                Vec::new()
            }
        };
        info!("Scanned scripts: found {} scripts", scripts.len());
    }

    // Получение mtime файла
    async fn get_mtime(&self, path: &StdPath) -> Option<SystemTime> {
        fs::metadata(path)
            .await
            .ok()
            .and_then(|meta| meta.modified().ok())
    }

    // Сохранить новый скрипт
    async fn save_script(&self, name: &str, code: &str) -> Result<(), AppError> {
        if name.contains('/') || name.contains('\\') || !name.ends_with(".py") {
            return Err(AppError::InvalidScriptName(
                "Name must be a simple .py filename".to_string(),
            ));
        }
        let path = self.scripts_dir.join(name);
        fs::write(&path, code).await?;

        let mut scripts = self.scripts.lock().await;
        if !scripts.contains(&path) {
            scripts.push(path);
        }
        Ok(())
    }

    // Удалить скрипт
    async fn delete_script(&self, name: &str) -> Result<(), AppError> {
        let path = self.scripts_dir.join(name);
        if fs::remove_file(&path).await.is_err() {
            return Err(AppError::ScriptNotFound(name.to_string()));
        }
        let mut scripts = self.scripts.lock().await;
        scripts.retain(|p| p != &path);
        Ok(())
    }

    // Запуск одного скрипта с данными и аргументами (с кэшированием)
    async fn run_script(
        self: Arc<Self>,
        script_name: &str,
        args: Vec<String>,
        input_bytes: Bytes,
    ) -> Result<ScriptResult, AppError> {
        let script_path = self.scripts_dir.join(script_name);

        {
            let scripts = self.scripts.lock().await;
            if !scripts.contains(&script_path) {
                return Err(AppError::ScriptNotFound(script_name.to_string()));
            }
        }

        let current_mtime = self.get_mtime(&script_path).await;

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        args.hash(&mut hasher);
        input_bytes.hash(&mut hasher);
        let cache_key = format!("{}:{:x}", script_name, hasher.finish());

        // Проверка кэша
        {
            let mut cache = self.cache.lock().await;
            if let Some(cached) = cache.get(&cache_key) {
                if cached.timestamp.elapsed() < self.cache_ttl
                    && current_mtime
                        .map(|m| m == cached.script_mtime)
                        .unwrap_or(false)
                {
                    info!("Cache hit for {}", script_name);
                    return Ok(ScriptResult {
                        stdout: cached.stdout.clone(),
                        stderr: cached.stderr.clone(),
                        exit_code: cached.exit_code,
                        timed_out: false,
                    });
                } else {
                    cache.remove(&cache_key);
                }
            }
        }

        let _permit = self.semaphore.acquire().await.unwrap();

        let script_path = script_path.clone();
        let script_name = script_name.to_string();

        let run_fut = async {
            let mut child = Command::new("python3")
                .arg("-u")
                .arg(&script_path)
                .args(&args)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()?;

            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(&input_bytes).await?;
                stdin.flush().await?;
            }
            drop(child.stdin.take());

            let output = child.wait_with_output().await?;
            Ok::<_, std::io::Error>(output)
        };

        let result = timeout(Duration::from_secs(30), run_fut).await;

        let (stdout, stderr, exit_code, timed_out) = match result {
            Ok(Ok(output)) => (
                String::from_utf8(output.stdout)?,
                String::from_utf8(output.stderr)?,
                output.status.code().unwrap_or(-1),
                false,
            ),
            Ok(Err(e)) => return Err(AppError::Io(e)),
            Err(_elapsed) => {
                warn!("Script {} timed out", script_name);
                return Err(AppError::Timeout);
            }
        };

        if let Some(mtime) = current_mtime {
            let mut cache = self.cache.lock().await;
            cache.insert(
                cache_key,
                CachedResult {
                    stdout: stdout.clone(),
                    stderr: stderr.clone(),
                    exit_code,
                    timestamp: Instant::now(),
                    script_mtime: mtime,
                },
            );
        }

        Ok(ScriptResult {
            stdout,
            stderr,
            exit_code,
            timed_out,
        })
    }
    
    async fn load_descriptions(&self) -> Result<HashMap<String, String>, AppError> {
        if !self.descriptions_path.exists() {
            return Ok(HashMap::new());
        }
        let content = fs::read_to_string(&self.descriptions_path).await?;
        if content.trim().is_empty() {
            return Ok(HashMap::new());
        }
        let descriptions = serde_json::from_str(&content)?;
        Ok(descriptions)
    }

    async fn save_descriptions(&self, descriptions: &HashMap<String, String>) -> Result<(), AppError> {
        let content = serde_json::to_string_pretty(descriptions)?;
        fs::write(&self.descriptions_path, content).await?;
        Ok(())
    }

    async fn update_descriptions(&self, updates: HashMap<String, String>) -> Result<HashMap<String, String>, AppError> {
        let mut current = self.load_descriptions().await?;
        for (name, desc) in updates {
            if desc.trim().is_empty() {
                current.remove(&name);
            } else {
                current.insert(name, desc);
            }
        }
        self.save_descriptions(&current).await?;
        Ok(current)
    }
}

// ------------------------------------------------------------
// Модели запросов/ответов
// ------------------------------------------------------------
#[derive(Debug, Deserialize)]
pub struct RunRequest {
    data: serde_json::Value,
    args: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct RunQuery {
    names: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub query: Option<String>,  // может быть None или пустой строкой
}

#[derive(Debug, Serialize, Clone)]
pub struct ScriptResult {
    stdout: String,
    stderr: String,
    exit_code: i32,
    timed_out: bool,
}

#[derive(Debug, Serialize)]
pub struct RunResponse {
    results: HashMap<String, ScriptResult>,
}

#[derive(Debug, Deserialize)]
pub struct CreateScriptRequest {
    name: String,
    code: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateScriptRequest {
    code: String,
}

#[derive(Serialize)]
pub struct ScriptMetadata {
    name: String,
    size: u64,
    modified: DateTime<Utc>,
    created: DateTime<Utc>,
    description: Option<String>,
}

// ------------------------------------------------------------
// Обработчики
// ------------------------------------------------------------
pub async fn list_scripts(
    State(state): State<Arc<AppState>>,
    Query(search_query): Query<SearchQuery>,
) -> Result<Json<Vec<ScriptMetadata>>, AppError> {
    info!("Get search scripts with metadata");

    // Загружаем описания
    let descriptions = state.load_descriptions().await?;

    // Получаем список путей к скриптам
    let paths = {
        let scripts = state.scripts.lock().await;
        scripts.clone()
    };

    let mut metadatas = Vec::with_capacity(paths.len());

    for path in paths {
        // Получаем имя файла
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| AppError::Internal("Invalid file name".to_string()))?
            .to_string();

        // Получаем метаданные файла
        let meta = fs::metadata(&path).await?;

        // Преобразуем временные метки
        let datetime_modified: DateTime<Local> = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH).into();
        let modified = datetime_modified.to_utc();

        let datetime_created: DateTime<Local> = meta.created().unwrap_or(SystemTime::UNIX_EPOCH).into();
        let created = datetime_created.to_utc();

        let description = descriptions.get(&file_name).cloned(); // получаем описание

        metadatas.push(ScriptMetadata {
            name: file_name,
            description,
            size: meta.len(),
            modified,
            created,
        });
    }

    // Фильтрация по всем полям, если задан поисковый запрос
    if let Some(query) = &search_query.query {
        let query_lower = query.to_lowercase();
        if !query_lower.is_empty() {
            metadatas.retain(|m| {
                m.name.to_lowercase().contains(&query_lower)
                    || m.size.to_string().contains(query)
                    || m.modified.to_string().to_lowercase().contains(&query_lower)
                    || m.created.to_string().to_lowercase().contains(&query_lower)
                    || m.description.as_ref().map(|d| d.to_lowercase()
                    .contains(&query_lower)).unwrap_or(false)
            });
        }
    }

    // Сортировка по алфавиту (по имени файла)
    // metadatas.sort_by(|a, b| a.name.cmp(&b.name));
    // Для регистронезависимой сортировки можно использовать:
    metadatas.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(Json(metadatas))
}

async fn get_script(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    info!("Get script {}", &name);
    let path = state.scripts_dir.join(&name);
    // Проверяем, что файл существует и входит в список известных скриптов
    {
        let scripts = state.scripts.lock().await;
        if !scripts.contains(&path) {
            return Err(AppError::ScriptNotFound(name));
        }
    }
    let code = fs::read_to_string(&path).await?;
    Ok(Json(serde_json::json!({
        "name": name,
        "code": code
    })))
}

async fn create_script(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateScriptRequest>,
) -> Result<StatusCode, AppError> {
    info!("Creating script {}", &payload.name);
    state.save_script(&payload.name, &payload.code).await?;
    Ok(StatusCode::CREATED)
}

async fn update_script(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<UpdateScriptRequest>,
) -> Result<StatusCode, AppError> {
    info!("Updating script {}", &payload.code);
    let path = state.scripts_dir.join(&name);
    if !path.exists() {
        return Err(AppError::ScriptNotFound(name));
    }
    fs::write(&path, &payload.code).await?;
    Ok(StatusCode::OK)
}

async fn delete_script(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<StatusCode, AppError> {
    info!("Delete script {}", &name);
    state.delete_script(&name).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn run_scripts(
    State(state): State<Arc<AppState>>,
    Query(query): Query<RunQuery>,
    Json(payload): Json<RunRequest>,
) -> Result<Json<RunResponse>, AppError> {
    info!("Run scripts {}", &payload.data);
    let target_names: Vec<String> = match query.names {
        Some(names_str) => names_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        None => {
            let scripts = state.scripts.lock().await;
            scripts
                .iter()
                .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(String::from))
                .collect()
        }
    };

    if target_names.is_empty() {
        return Ok(Json(RunResponse {
            results: HashMap::new(),
        }));
    }

    let input_bytes = Bytes::from(serde_json::to_vec(&payload.data)?);
    let args = payload.args.unwrap_or_default();

    let state = Arc::clone(&state);
    let futures = target_names.into_iter().map(move |name| {
        let state = Arc::clone(&state);
        let input_bytes = input_bytes.clone();
        let args = args.clone();
        async move {
            let result = state.run_script(&name, args, input_bytes).await;
            (name, result)
        }
    });

    let results_vec = join_all(futures).await;
    let mut results = HashMap::new();
    for (name, res) in results_vec {
        match res {
            Ok(r) => {
                results.insert(name, r);
            }
            Err(e) => {
                results.insert(
                    name,
                    ScriptResult {
                        stdout: String::new(),
                        stderr: format!("Error: {}", e),
                        exit_code: -1,
                        timed_out: false,
                    },
                );
            }
        }
    }

    Ok(Json(RunResponse { results }))
}

async fn run_single_script(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<RunRequest>,
) -> Result<Json<ScriptResult>, AppError> {
    info!("Run single script {}", &payload.data);
    let input_bytes = Bytes::from(serde_json::to_vec(&payload.data)?);
    let args = payload.args.unwrap_or_default();
    let result = state.run_script(&name, args, input_bytes).await?;
    Ok(Json(result))
}

async fn get_descriptions(
    State(state): State<Arc<AppState>>,
) -> Result<Json<HashMap<String, String>>, AppError> {
    let descriptions = state.load_descriptions().await?;
    Ok(Json(descriptions))
}

async fn update_descriptions(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<HashMap<String, String>>,
) -> Result<Json<HashMap<String, String>>, AppError> {
    let updated = state.update_descriptions(payload).await?;
    Ok(Json(updated))
}
// ------------------------------------------------------------
// Запуск сервера
// ------------------------------------------------------------
#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let scripts_dir = PathBuf::from("./scripts");
    if !scripts_dir.exists() {
        fs::create_dir_all(&scripts_dir)
            .await
            .expect("Failed to create scripts directory");
    }

    let state = AppState::new(scripts_dir, 4, Duration::from_secs(30));

    // Фоновое сканирование
    let scanner_state = Arc::clone(&state);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            scanner_state.scan_scripts().await;
        }
    });

    // Настройка CORS
    let origins = std::env::var("ALLOWED_ORIGINS").ok();
    let (allow_origin, is_any) = if let Some(origins_str) = origins {
        if origins_str == "*" {
            (AllowOrigin::any(), true)
        } else {
            let origins: Vec<HeaderValue> = origins_str
                .split(',')
                .map(|s| s.trim().parse().expect("Invalid origin format"))
                .collect();
            (AllowOrigin::list(origins), false)
        }
    } else {
        (AllowOrigin::any(), true)
    };

    let mut cors = CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
            axum::http::header::ACCEPT,
        ]);

    if !is_any && std::env::var("CORS_ALLOW_CREDENTIALS").as_deref() == Ok("true") {
        cors = cors.allow_credentials(true);
    }

    let app = Router::new()
        .route("/scripts", get(list_scripts).post(create_script))
        .route("/scripts/descriptions", get(get_descriptions).post(update_descriptions))
        .route("/scripts/{name}", get(get_script).put(update_script).delete(delete_script))
        .route("/run", post(run_scripts))
        .route("/run/{name}", post(run_single_script))
        .layer(cors)
        .with_state(state);

    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap(); // явно указываем тип
    let listener = TcpListener::bind(addr).await.unwrap();
    info!("Server listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
