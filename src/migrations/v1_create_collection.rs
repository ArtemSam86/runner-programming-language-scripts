use async_trait::async_trait;
use mongodb::{
    bson::doc,
    options::IndexOptions,
    Database, IndexModel,
};
use tracing::info;

use super::Migration;

pub struct V1CreateCollection;

#[async_trait]
impl Migration for V1CreateCollection {
    fn name(&self) -> &'static str {
        "v1_create_collection"
    }

    async fn up(&self, db: &Database) -> anyhow::Result<()> {
        let collection = db.collection::<mongodb::bson::Document>("scripts");
        let index = IndexModel::builder()
            .keys(doc! { "name": 1 })
            .options(IndexOptions::builder().unique(Some(true)).build())
            .build();
        collection.create_index(index).await?;
        info!("Created unique index on scripts.name");
        Ok(())
    }

    async fn down(&self, db: &Database) -> anyhow::Result<()> {
        let collection = db.collection::<mongodb::bson::Document>("scripts");
        collection.drop().await?;
        info!("Dropped scripts collection");
        Ok(())
    }
}