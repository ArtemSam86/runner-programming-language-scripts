use async_trait::async_trait;
use chrono::Utc;
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, Bson, DateTime as BsonDateTime, Document},
    Database,
};
use tracing::info;

use super::Migration;

pub struct V3ForceFixDates;

#[async_trait]
impl Migration for V3ForceFixDates {
    fn name(&self) -> &'static str {
        "v3_force_fix_dates"
    }

    async fn up(&self, db: &Database) -> anyhow::Result<()> {
        let collection = db.collection::<Document>("scripts");
        let cursor = collection.find(doc! {}).await?; // все документы
        let docs: Vec<Document> = cursor.try_collect().await?;
        let mut updated = 0;

        for mut doc in docs {
            let mut changed = false;

            // Принудительно преобразуем created в BSON Date
            if let Some(created) = doc.get("created") {
                if let Some(new_date) = convert_to_bson_datetime(created) {
                    doc.insert("created", new_date);
                    changed = true;
                }
            } else {
                // Если поля нет, создаём с текущей датой
                doc.insert("created", BsonDateTime::now());
                changed = true;
            }

            // Принудительно преобразуем modified в BSON Date
            if let Some(modified) = doc.get("modified") {
                if let Some(new_date) = convert_to_bson_datetime(modified) {
                    doc.insert("modified", new_date);
                    changed = true;
                }
            } else {
                doc.insert("modified", BsonDateTime::now());
                changed = true;
            }

            if changed {
                if let Some(id) = doc.get("_id").and_then(|id| id.as_object_id()) {
                    collection.replace_one(doc! { "_id": id }, &doc).await?;
                    updated += 1;
                }
            }
        }

        info!("V3: Force-fixed date fields for {} documents", updated);
        Ok(())
    }

    async fn down(&self, _db: &Database) -> anyhow::Result<()> {
        Ok(())
    }
}

fn convert_to_bson_datetime(value: &Bson) -> Option<Bson> {
    match value {
        Bson::String(s) => {
            chrono::DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .or_else(|_| chrono::DateTime::parse_from_rfc2822(s).map(|dt| dt.with_timezone(&Utc)))
                .ok()
                .map(|dt| Bson::DateTime(BsonDateTime::from_millis(dt.timestamp_millis())))
        }
        Bson::Document(doc) if doc.contains_key("$date") => {
            if let Ok(date_str) = doc.get_str("$date") {
                convert_to_bson_datetime(&Bson::String(date_str.to_owned()))
            } else if let Ok(date_obj) = doc.get_document("$date") {
                if let Ok(long_str) = date_obj.get_str("$numberLong") {
                    if let Ok(millis) = long_str.parse::<i64>() {
                        Some(Bson::DateTime(BsonDateTime::from_millis(millis)))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
        Bson::DateTime(_) => Some(value.clone()), // уже дата – оставляем
        _ => None,
    }
}