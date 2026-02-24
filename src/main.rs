use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
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
    scripts: Mutex<Vec<PathBuf>>,
    semaphore: Semaphore,
    cache: Mutex<HashMap<String, CachedResult>>,
    cache_ttl: Duration,
}

impl AppState {
    fn new(scripts_dir: PathBuf, max_concurrent: usize, cache_ttl: Duration) -> Arc<Self> {
        Arc::new(Self {
            scripts_dir,
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

// ------------------------------------------------------------
// Обработчики
// ------------------------------------------------------------
async fn list_scripts(State(state): State<Arc<AppState>>) -> Result<Json<Vec<String>>, AppError> {
    let scripts = state.scripts.lock().await;
    let names = scripts
        .iter()
        .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(String::from))
        .collect();
    Ok(Json(names))
}

async fn create_script(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateScriptRequest>,
) -> Result<StatusCode, AppError> {
    state.save_script(&payload.name, &payload.code).await?;
    Ok(StatusCode::CREATED)
}

async fn update_script(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<UpdateScriptRequest>,
) -> Result<StatusCode, AppError> {
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
    state.delete_script(&name).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn run_scripts(
    State(state): State<Arc<AppState>>,
    Query(query): Query<RunQuery>,
    Json(payload): Json<RunRequest>,
) -> Result<Json<RunResponse>, AppError> {
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
    let input_bytes = Bytes::from(serde_json::to_vec(&payload.data)?);
    let args = payload.args.unwrap_or_default();
    let result = state.run_script(&name, args, input_bytes).await?;
    Ok(Json(result))
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

    let app = Router::new()
        .route("/scripts", get(list_scripts).post(create_script))
        .route("/scripts/:name", put(update_script).delete(delete_script))
        .route("/run", post(run_scripts))
        .route("/run/:name", post(run_single_script))
        .with_state(state);

    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap(); // явно указываем тип
    let listener = TcpListener::bind(addr).await.unwrap();
    info!("Server listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
