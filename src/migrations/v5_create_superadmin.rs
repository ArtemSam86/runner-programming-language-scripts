use async_trait::async_trait;
use bcrypt::{hash, DEFAULT_COST};
use mongodb::{
    bson::{doc, DateTime as BsonDateTime},
    Database,
};
use std::env;
use tracing::info;

use super::Migration;

pub struct V5CreateSuperadmin;

#[async_trait]
impl Migration for V5CreateSuperadmin {
    fn name(&self) -> &'static str {
        "v5_create_superadmin"
    }

    async fn up(&self, db: &Database) -> anyhow::Result<()> {
        let collection = db.collection::<mongodb::bson::Document>("users");
        let count = collection.count_documents(doc! {}).await?;

        if count == 0 {
            info!("No users found. Creating superadmin from environment variables.");

            let superadmin_name = env::var("SUPER_ADMIN_NAME")
                .map_err(|_| anyhow::anyhow!("SUPER_ADMIN_NAME must be set"))?;
            let superadmin_password = env::var("SUPER_ADMIN_PASSWORD")
                .map_err(|_| anyhow::anyhow!("SUPER_ADMIN_PASSWORD must be set"))?;

            let password_hash = hash(&superadmin_password, DEFAULT_COST)
                .map_err(|e| anyhow::anyhow!("Bcrypt error: {}", e))?;

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

    async fn down(&self, _db: &Database) -> anyhow::Result<()> {
        Ok(())
    }
}