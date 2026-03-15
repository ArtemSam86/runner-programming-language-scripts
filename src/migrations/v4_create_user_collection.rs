use async_trait::async_trait;
use mongodb::{
    bson::doc,
    options::IndexOptions,
    Database, IndexModel,
};
use tracing::info;

use super::Migration;

pub struct V4CreateUserCollection;

#[async_trait]
impl Migration for V4CreateUserCollection {
    fn name(&self) -> &'static str {
        "v4_create_user_collection"
    }

    async fn up(&self, db: &Database) -> anyhow::Result<()> {
        let collection = db.collection::<mongodb::bson::Document>("users");
        let index = IndexModel::builder()
            .keys(doc! { "username": 1 })
            .options(IndexOptions::builder().unique(Some(true)).build())
            .build();
        collection.create_index(index).await?;
        info!("Created unique index on users.username");
        Ok(())
    }

    async fn down(&self, db: &Database) -> anyhow::Result<()> {
        let collection = db.collection::<mongodb::bson::Document>("users");
        collection.drop().await?;
        Ok(())
    }
}