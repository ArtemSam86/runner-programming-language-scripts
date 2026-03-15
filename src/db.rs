use anyhow::anyhow;
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, Document, DateTime as BsonDateTime},
    options::ClientOptions,
    Client, Collection, Database
};
use serde::{Deserialize, Serialize};
use bcrypt::{hash, verify, DEFAULT_COST};
use crate::error::AppError;
use crate::models::User;
use tracing::info;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScriptDoc {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub result: Option<String>,
    pub size: u64,
    pub created: BsonDateTime,
    pub modified: BsonDateTime,
}

pub async fn init_db(uri: &str, db_name: &str) -> Result<Database, mongodb::error::Error> {
    let mut client_options = ClientOptions::parse(uri).await?;
    client_options.app_name = Some("script-server".to_string());
    let client = Client::with_options(client_options)?;
    Ok(client.database(db_name))
}

fn scripts_collection(db: &Database) -> Collection<ScriptDoc> {
    db.collection::<ScriptDoc>("scripts")
}

pub async fn get_script_by_name(
    db: &Database,
    name: &str,
) -> Result<Option<ScriptDoc>, mongodb::error::Error> {
    let collection = scripts_collection(db);
    collection.find_one(doc! { "name": name }).await
}

pub async fn insert_script(
    db: &Database,
    doc: ScriptDoc,
) -> Result<(), mongodb::error::Error> {
    let collection = scripts_collection(db);
    collection.insert_one(doc).await?;
    Ok(())
}

pub async fn update_script(
    db: &Database,
    name: &str,
    update_doc: Document,
) -> Result<(), mongodb::error::Error> {
    let collection = scripts_collection(db);
    collection
        .update_one(doc! { "name": name }, doc! { "$set": update_doc })
        .await?;
    Ok(())
}

pub async fn delete_script(
    db: &Database,
    name: &str,
) -> Result<(), mongodb::error::Error> {
    let collection = scripts_collection(db);
    collection.delete_one(doc! { "name": name }).await?;
    Ok(())
}

pub async fn get_all_scripts(db: &Database) -> Result<Vec<ScriptDoc>, mongodb::error::Error> {
    let collection = db.collection::<Document>("scripts");
    let mut cursor = collection.find(doc! {}).await?;
    let mut result = Vec::new();

    while let Some(document) = cursor.try_next().await? {
        match mongodb::bson::from_document::<ScriptDoc>(document.clone()) {
            Ok(script) => result.push(script),
            Err(e) => {
                eprintln!("Failed to deserialize document: {:?}", document);
                eprintln!("Error: {}", e);
                return Err(e.into());
            }
        }
    }
    Ok(result)
}

// Создание нового пользователя
pub async fn create_user(
    db: &Database,
    username: &str,
    password: &str,
) -> Result<User, AppError> {
    let collection: Collection<User> = db.collection("users");

    // Проверяем, не существует ли уже пользователь
    if collection
        .find_one(doc! { "username": username })
        .await?
        .is_some()
    {
        return Err(AppError::UserAlreadyExists(username.to_string()));
    }

    let password_hash = hash(password, DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("Bcrypt error: {}", e)))?;

    let now = BsonDateTime::now();
    let user = User {
        id: None,
        username: username.to_string(),
        password_hash,
        created_at: now,
    };

    let result = collection.insert_one(&user).await?;
    let inserted_id = result.inserted_id.as_object_id().unwrap();

    // Возвращаем пользователя с заполненным id
    Ok(User {
        id: Some(inserted_id),
        ..user
    })
}

// Получение пользователя по имени
pub async fn get_user_by_username(
    db: &Database,
    username: &str,
) -> Result<Option<User>, mongodb::error::Error> {
    let collection: Collection<User> = db.collection("users");
    collection.find_one(doc! { "username": username }).await
}

// Проверка пароля пользователя
pub async fn verify_user_password(
    db: &Database,
    username: &str,
    password: &str,
) -> Result<bool, AppError> {
    if let Some(user) = get_user_by_username(db, username).await? {
        let valid = verify(password, &user.password_hash)
            .map_err(|e| AppError::Internal(format!("Bcrypt error: {}", e)))?;
        Ok(valid)
    } else {
        Ok(false)
    }
}

/// Создаёт суперадминистратора, если в базе нет ни одного пользователя.
/// Использует переменные окружения SUPER_ADMIN_NAME и SUPER_ADMIN_PASSWORD.
pub async fn ensure_superadmin(db: &Database) -> anyhow::Result<()> {
    let collection = db.collection::<Document>("users");
    let count = collection.count_documents(doc! {}).await?;

    if count == 0 {
        info!("No users found. Creating superadmin from environment variables.");

        let superadmin_name = std::env::var("SUPER_ADMIN_NAME")
            .map_err(|_| anyhow!("SUPER_ADMIN_NAME must be set when no users exist"))?;
        let superadmin_password = std::env::var("SUPER_ADMIN_PASSWORD")
            .map_err(|_| anyhow!("SUPER_ADMIN_PASSWORD must be set when no users exist"))?;

        let password_hash = hash(&superadmin_password, DEFAULT_COST)
            .map_err(|e| anyhow!("Bcrypt error: {}", e))?;

        let now = BsonDateTime::now();

        let doc = doc! {
            "username": superadmin_name,
            "password_hash": password_hash,
            "created_at": now,
        };

        collection.insert_one(doc).await?;
        info!("Superadmin created successfully.");
    } else {
        info!("Users already exist. Skipping superadmin creation.");
    }
    Ok(())
}