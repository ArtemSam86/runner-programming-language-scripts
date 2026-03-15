use crate::{
    app_state::{AppState, CachedResult},
    db,
    error::AppError,
    models::ScriptResult,
};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use mongodb::bson::{doc};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
    time::{Duration, Instant, SystemTime},
};
use tokio::{
    fs,
    io::AsyncWriteExt,
    process::Command,
    time::timeout,
};
use tracing::{info, warn};

async fn get_mtime(path: &std::path::Path) -> Option<SystemTime> {
    fs::metadata(path).await.ok().and_then(|m| m.modified().ok())
}

pub async fn run_script(
    state: Arc<AppState>,
    script_name: &str,
    args: Vec<String>,
    input_bytes: Bytes,
) -> Result<ScriptResult, AppError> {
    let script_path = state.scripts_dir.join(script_name);

    {
        let scripts = state.scripts.lock().await;
        if !scripts.contains(&script_path) {
            return Err(AppError::ScriptNotFound(script_name.to_string()));
        }
    }

    let current_mtime = get_mtime(&script_path).await;

    // Ключ кэша
    let mut hasher = DefaultHasher::new();
    args.hash(&mut hasher);
    input_bytes.hash(&mut hasher);
    let cache_key = format!("{}:{:x}", script_name, hasher.finish());

    // Проверка кэша
    {
        let mut cache = state.cache.lock().await;
        if let Some(cached) = cache.get(&cache_key) {
            if cached.timestamp.elapsed() < state.cache_ttl
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

    let _permit = state.semaphore.acquire().await.unwrap();

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
        Err(_) => {
            warn!("Script {} timed out", script_name);
            return Err(AppError::Timeout);
        }
    };

    if let Some(mtime) = current_mtime {
        let mut cache = state.cache.lock().await;
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

// Фоновое сканирование
pub async fn scan_scripts(state: Arc<AppState>) {
    let mut current_files = Vec::new();
    if let Ok(mut entries) = fs::read_dir(&state.scripts_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("py") {
                current_files.push(path);
            }
        }
    }

    let db_docs = match db::get_all_scripts(&state.db).await {
        Ok(docs) => docs,
        Err(e) => {
            warn!("Failed to get scripts from DB during scan: {}", e);
            return;
        }
    };

    for path in &current_files {
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };
        let meta = match fs::metadata(path).await {
            Ok(m) => m,
            Err(_) => continue,
        };
        let modified: DateTime<Utc> = meta
            .modified()
            .unwrap_or_else(|_| SystemTime::now())
            .into();

        if let Some(doc) = db_docs.iter().find(|d| d.name == file_name) {
            // Сравниваем по миллисекундам
            if doc.modified.timestamp_millis() < modified.timestamp_millis() {
                let code = match fs::read_to_string(path).await {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                let update = doc! {
                    "code": code,
                    "size": meta.len() as i64,
                    "modified": mongodb::bson::DateTime::from_millis(modified.timestamp_millis()),
                };
                if let Err(e) = db::update_script(&state.db, &file_name, update).await {
                    warn!("Failed to update script in DB: {}", e);
                }
            }
        } else {
            // создание нового документа (уже исправлено)
            let created: DateTime<Utc> = meta
                .created()
                .unwrap_or_else(|_| SystemTime::now())
                .into();
            let doc = db::ScriptDoc {
                id: None,
                name: file_name,
                code: match fs::read_to_string(path).await {
                    Ok(c) => c,
                    Err(_) => continue,
                },
                description: None,
                result: None,
                size: meta.len(),
                created: mongodb::bson::DateTime::from_millis(created.timestamp_millis()),
                modified: mongodb::bson::DateTime::from_millis(modified.timestamp_millis()),
            };
            if let Err(e) = db::insert_script(&state.db, doc).await {
                warn!("Failed to insert new script into DB: {}", e);
            }
        }
    }

    // Удаляем из БД записи, для которых нет файлов
    for doc in db_docs {
        if !current_files
            .iter()
            .any(|p| p.file_name().and_then(|n| n.to_str()) == Some(&doc.name))
        {
            if let Err(e) = db::delete_script(&state.db, &doc.name).await {
                warn!("Failed to delete script from DB: {}", e);
            }
        }
    }

    // Обновляем список в памяти
    let mut scripts = state.scripts.lock().await;
    *scripts = current_files;
}