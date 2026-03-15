use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::DateTime as BsonDateTime;
use utoipa::{ToSchema, IntoParams};

// Ответ для списка скриптов (без кода)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ScriptMetadata {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    pub description: Option<String>,
    pub result: Option<String>,
    pub size: u64,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
}

// Запрос на создание скрипта
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateScriptRequest {
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub result: Option<String>,
}

// Запрос на обновление скрипта
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateScriptRequest {
    pub code: Option<String>,
    pub description: Option<String>,
    pub result: Option<String>,
}

// Запрос на выполнение
#[derive(Debug, Deserialize, ToSchema)]
pub struct RunRequest {
    pub data: serde_json::Value,
    pub args: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct RunQuery {
    pub names: Option<String>,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct ScriptResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub timed_out: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RunResponse {
    pub results: HashMap<String, ScriptResult>,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct SearchQuery {
    pub query: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

// Модель пользователя (хранится в БД)
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    pub password_hash: String,
    pub created_at: BsonDateTime,
}

// Запрос на регистрацию
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

// Запрос на логин (уже есть, но добавим для полноты)
#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

// Ответ с токеном
#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub username: String,
}