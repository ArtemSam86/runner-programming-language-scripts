use std::{
    collections::HashMap,
    path::PathBuf,
    time::{Duration, Instant, SystemTime},
};
use mongodb::Database;
use tokio::sync::{Mutex, Semaphore};

#[derive(Clone)]
pub struct CachedResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub timestamp: Instant,
    pub script_mtime: SystemTime,
}

pub struct AppState {
    pub scripts_dir: PathBuf,
    pub db: Database,
    pub scripts: Mutex<Vec<PathBuf>>,
    pub semaphore: Semaphore,
    pub cache: Mutex<HashMap<String, CachedResult>>,
    pub cache_ttl: Duration,
}

impl AppState {
    pub fn new(
        scripts_dir: PathBuf,
        db: Database,
        max_concurrent: usize,
        cache_ttl: Duration,
    ) -> Self {
        Self {
            scripts_dir,
            db,
            scripts: Mutex::new(Vec::new()),
            semaphore: Semaphore::new(max_concurrent),
            cache: Mutex::new(HashMap::new()),
            cache_ttl,
        }
    }
}