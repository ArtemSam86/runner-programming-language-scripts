use crate::{
    app_state::AppState,
    db,
    jwt,
    error::AppError,
    models::*,
    script_runner,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use futures::future::join_all;
use mongodb::bson::{doc, DateTime as BsonDateTime};
use std::{collections::HashMap, sync::Arc, time::SystemTime};
use tokio::fs;
use tracing::info;

fn bson_to_chrono(bson: BsonDateTime) -> DateTime<Utc> {
    let millis = bson.timestamp_millis();
    DateTime::from_timestamp_millis(millis).expect("Invalid BSON timestamp")
}

/// Получить список скриптов с фильтрацией и сортировкой
#[utoipa::path(
    get,
    path = "/scripts",
    params(SearchQuery),
    responses(
        (status = 200, description = "Список скриптов", body = Vec<ScriptMetadata>),
        (status = 401, description = "Не авторизован")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "scripts"
)]
pub async fn list_scripts(
    State(state): State<Arc<AppState>>,
    Query(search_query): Query<SearchQuery>,
) -> Result<Json<Vec<ScriptMetadata>>, AppError> {
    info!("Listing scripts with metadata (including code)");

    let docs = db::get_all_scripts(&state.db).await?;
    let mut metadatas: Vec<ScriptMetadata> = docs
        .into_iter()
        .map(|doc| ScriptMetadata {
            name: doc.name,
            code: Some(doc.code),
            description: doc.description,
            result: doc.result,
            size: doc.size,
            created: bson_to_chrono(doc.created),
            modified: bson_to_chrono(doc.modified),
        })
        .collect();

    // Фильтрация по поисковому запросу
    if let Some(query) = &search_query.query {
        let q = query.to_lowercase();
        if !q.is_empty() {
            metadatas.retain(|m| {
                m.name.to_lowercase().contains(&q)
                    || m.code.as_deref().unwrap_or("").to_lowercase().contains(&q)
                    || m.size.to_string().contains(&q)
                    || m.created.to_string().to_lowercase().contains(&q)
                    || m.modified.to_string().to_lowercase().contains(&q)
                    || m.description.as_deref().unwrap_or("").to_lowercase().contains(&q)
                    || m.result.as_deref().unwrap_or("").to_lowercase().contains(&q)
            });
        }
    }

    // Параметры сортировки (по умолчанию — по имени, по возрастанию)
    let sort_by = search_query.sort_by.as_deref().unwrap_or("name");
    let sort_order = search_query.sort_order.as_deref().unwrap_or("asc");
    let descending = sort_order.eq_ignore_ascii_case("desc");

    // Сортировка по выбранному полю
    metadatas.sort_by(|a, b| {
        let cmp = match sort_by {
            "name" => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            "size" => a.size.cmp(&b.size),
            "created" => a.created.cmp(&b.created),
            "modified" => a.modified.cmp(&b.modified),
            _ => std::cmp::Ordering::Equal, // неизвестное поле не меняет порядок
        };
        if descending {
            cmp.reverse()
        } else {
            cmp
        }
    });

    Ok(Json(metadatas))
}

/// Получить конкретный скрипт по имени
#[utoipa::path(
    get,
    path = "/scripts/{name}",
    params(
        ("name" = String, Path, description = "Имя файла скрипта")
    ),
    responses(
        (status = 200, description = "Данные скрипта", body = ScriptMetadata),
        (status = 404, description = "Скрипт не найден"),
        (status = 401, description = "Не авторизован")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "scripts"
)]
pub async fn get_script(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<ScriptMetadata>, AppError> {
    info!("Get script {}", name);

    let doc = db::get_script_by_name(&state.db, &name)
        .await?
        .ok_or_else(|| AppError::ScriptNotFound(name.clone()))?;

    let path = state.scripts_dir.join(&name);
    let code = fs::read_to_string(&path).await?;

    Ok(Json(ScriptMetadata {
        name: doc.name,
        code: Some(code),
        description: doc.description,
        result: doc.result,
        size: doc.size,
        created: bson_to_chrono(doc.created),
        modified: bson_to_chrono(doc.modified),
    }))
}

/// Создать новый скрипт
#[utoipa::path(
    post,
    path = "/scripts",
    request_body = CreateScriptRequest,
    responses(
        (status = 201, description = "Скрипт создан"),
        (status = 400, description = "Некорректное имя скрипта"),
        (status = 409, description = "Скрипт уже существует"),
        (status = 401, description = "Не авторизован")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "scripts"
)]
pub async fn create_script(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateScriptRequest>,
) -> Result<StatusCode, AppError> {
    info!("Creating script {}", payload.name);

    if payload.name.contains('/') || payload.name.contains('\\') || !payload.name.ends_with(".py") {
        return Err(AppError::InvalidScriptName(
            "Name must be a simple .py filename".to_string(),
        ));
    }

    let path = state.scripts_dir.join(&payload.name);
    if path.exists() {
        return Err(AppError::Internal("Script already exists".into()));
    }

    // Сохраняем файл
    fs::write(&path, &payload.code).await?;

    // Метаданные файла
    let meta = fs::metadata(&path).await?;
    let created: DateTime<Utc> = meta
        .created()
        .unwrap_or_else(|_| SystemTime::now())
        .into();
    let modified: DateTime<Utc> = meta
        .modified()
        .unwrap_or_else(|_| SystemTime::now())
        .into();

    // Документ в БД – преобразуем chrono в bson
    let doc = db::ScriptDoc {
        id: None,
        name: payload.name,
        code: payload.code,
        description: payload.description,
        result: payload.result,
        size: meta.len(),
        created: mongodb::bson::DateTime::from_millis(created.timestamp_millis()),
        modified: mongodb::bson::DateTime::from_millis(modified.timestamp_millis()),
    };

    db::insert_script(&state.db, doc).await?;

    // Обновляем список в памяти
    let mut scripts = state.scripts.lock().await;
    scripts.push(path);

    Ok(StatusCode::CREATED)
}

/// Обновить существующий скрипт
#[utoipa::path(
    put,
    path = "/scripts/{name}",
    params(
        ("name" = String, Path, description = "Имя файла скрипта")
    ),
    request_body = UpdateScriptRequest,
    responses(
        (status = 200, description = "Обновлённые данные скрипта", body = ScriptMetadata),
        (status = 404, description = "Скрипт не найден"),
        (status = 401, description = "Не авторизован")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "scripts"
)]
pub async fn update_script(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<UpdateScriptRequest>,
) -> Result<Json<ScriptMetadata>, AppError> {
    info!("Updating script {}", name);

    let path = state.scripts_dir.join(&name);
    if !path.exists() {
        return Err(AppError::ScriptNotFound(name));
    }

    // Если передан code, обновляем файл
    if let Some(ref code) = payload.code {
        fs::write(&path, code).await?;
    }

    // Метаданные файла (всегда обновляем размер и mtime)
    let meta = fs::metadata(&path).await?;
    let modified: DateTime<Utc> = meta
        .modified()
        .unwrap_or_else(|_| SystemTime::now())
        .into();

    let mut update_doc = doc! {
        "size": meta.len() as i64,
        "modified": BsonDateTime::from_millis(modified.timestamp_millis()),
    };

    if let Some(code) = payload.code {
        update_doc.insert("code", code);
    }
    if let Some(desc) = payload.description {
        update_doc.insert(
            "description",
            if desc.is_empty() { None } else { Some(desc) },
        );
    }
    if let Some(res) = payload.result {
        update_doc.insert(
            "result",
            if res.is_empty() { None } else { Some(res) },
        );
    }

    db::update_script(&state.db, &name, update_doc).await?;

    // Если нужно будет, чтобы запрос возвращал измененный скрипт
    get_script(State(state), Path(name)).await
    // Ok(StatusCode::OK)
}

/// Удалить скрипт
#[utoipa::path(
    delete,
    path = "/scripts/{name}",
    params(
        ("name" = String, Path, description = "Имя файла скрипта")
    ),
    responses(
        (status = 204, description = "Скрипт удалён"),
        (status = 404, description = "Скрипт не найден"),
        (status = 401, description = "Не авторизован")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "scripts"
)]
pub async fn delete_script(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<StatusCode, AppError> {
    info!("Deleting script {}", name);

    let path = state.scripts_dir.join(&name);
    if path.exists() {
        fs::remove_file(&path).await?;
    }

    db::delete_script(&state.db, &name).await?;

    let mut scripts = state.scripts.lock().await;
    scripts.retain(|p| p != &path);

    Ok(StatusCode::NO_CONTENT)
}

/// Запустить несколько скриптов (по именам) с одинаковыми данными
#[utoipa::path(
    post,
    path = "/run",
    params(RunQuery),
    request_body = RunRequest,
    responses(
        (status = 200, description = "Результаты выполнения", body = RunResponse),
        (status = 401, description = "Не авторизован")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "execution"
)]
pub async fn run_scripts(
    State(state): State<Arc<AppState>>,
    Query(query): Query<RunQuery>,
    Json(payload): Json<RunRequest>,
) -> Result<Json<RunResponse>, AppError> {
    info!("Running scripts with data");

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
            let result = script_runner::run_script(state, &name, args, input_bytes).await;
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

/// Запустить один скрипт по имени
#[utoipa::path(
    post,
    path = "/run/{name}",
    params(
        ("name" = String, Path, description = "Имя файла скрипта")
    ),
    request_body = RunRequest,
    responses(
        (status = 200, description = "Результат выполнения", body = ScriptResult),
        (status = 404, description = "Скрипт не найден"),
        (status = 401, description = "Не авторизован")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "execution"
)]
pub async fn run_single_script(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<RunRequest>,
) -> Result<Json<ScriptResult>, AppError> {
    info!("Running single script {}", name);

    let input_bytes = Bytes::from(serde_json::to_vec(&payload.data)?);
    let args = payload.args.unwrap_or_default();
    let result = script_runner::run_script(state, &name, args, input_bytes).await?;
    Ok(Json(result))
}

/// Регистрация нового пользователя
#[utoipa::path(
    post,
    path = "/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Пользователь создан"),
        (status = 409, description = "Пользователь уже существует"),
        (status = 400, description = "Некорректные данные")
    ),
    tag = "auth"
)]
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<StatusCode, AppError> {
    info!("Registering user: {}", payload.username);
    db::create_user(&state.db, &payload.username, &payload.password).await?;
    Ok(StatusCode::CREATED)
}

/// Вход в систему (получение JWT токена)
#[utoipa::path(
    post,
    path = "/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Успешный вход", body = LoginResponse),
        (status = 401, description = "Неверные учетные данные")
    ),
    tag = "auth"
)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    info!("Login attempt for user: {}", payload.username);

    let valid = db::verify_user_password(&state.db, &payload.username, &payload.password).await?;
    if !valid {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    let token = jwt::create_token(&payload.username)
        .map_err(|e| AppError::Internal(format!("Token creation failed: {}", e)))?;

    Ok(Json(LoginResponse {
        token,
        username: payload.username,
    }))
}