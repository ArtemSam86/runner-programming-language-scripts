use async_trait::async_trait;
use mongodb::{bson::doc, Database};
use tracing::info;

mod v1_create_collection;
mod v2_fix_date_fields;
mod v3_force_fix_dates;
mod v4_create_user_collection;
mod v5_create_superadmin;

use v1_create_collection::V1CreateCollection;
use v2_fix_date_fields::V2FixDateFields;
use v3_force_fix_dates::V3ForceFixDates;
use v4_create_user_collection::V4CreateUserCollection;
use v5_create_superadmin::V5CreateSuperadmin;

#[async_trait]
trait Migration: Send + Sync {
    fn name(&self) -> &'static str;
    async fn up(&self, db: &Database) -> anyhow::Result<()>;
    #[allow(dead_code)]
    async fn down(&self, db: &Database) -> anyhow::Result<()>;
}

pub async fn run_migrations(db: &Database) -> anyhow::Result<()> {
    let migration_collection = db.collection::<mongodb::bson::Document>("migrations");

    let migrations: Vec<Box<dyn Migration>> = vec![
        Box::new(V1CreateCollection),
        Box::new(V2FixDateFields),
        Box::new(V3ForceFixDates),
        Box::new(V4CreateUserCollection),
        Box::new(V5CreateSuperadmin),
    ];

    for migration in migrations {
        let name = migration.name();
        let count = migration_collection
            .count_documents(doc! { "name": name })
            .await?;

        if count == 0 {
            info!("Running migration: {}", name);
            migration.up(db).await?;
            migration_collection
                .insert_one(doc! { "name": name, "applied_at": mongodb::bson::DateTime::now() })
                .await?;
            info!("Migration {} completed", name);
        } else {
            info!("Migration {} already applied, skipping", name);
        }
    }
    Ok(())
}