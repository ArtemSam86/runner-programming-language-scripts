use async_trait::async_trait;
use chrono::Utc;
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, Bson, DateTime as BsonDateTime, Document},
    Database,
};
use tracing::info;

use super::Migration;

pub struct V2FixDateFields;

#[async_trait]
impl Migration for V2FixDateFields {
    fn name(&self) -> &'static str {
        "v2_fix_date_fields"
    }

    async fn up(&self, db: &Database) -> anyhow::Result<()> {
        let collection = db.collection::<Document>("scripts");

        // Находим все документы, где created или modified не являются датами (включая null и отсутствие поля)
        let cursor = collection.find(doc! {
            "$or": [
                { "created": { "$not": { "$type": "date" } } },
                { "created": { "$exists": false } },
                { "modified": { "$not": { "$type": "date" } } },
                { "modified": { "$exists": false } }
            ]
        }).await?;

        let docs: Vec<Document> = cursor.try_collect().await?;
        let mut updated = 0;

        for mut doc in docs {
            let mut changed = false;

            // Обрабатываем created
            if let Some(created) = doc.get("created") {
                if !matches!(created, Bson::DateTime(_)) {
                    if let Some(new_date) = convert_to_bson_datetime(created) {
                        doc.insert("created", new_date);
                        changed = true;
                    } else {
                        // Если не удалось преобразовать, ставим текущее время
                        doc.insert("created", BsonDateTime::now());
                        changed = true;
                    }
                }
            } else {
                // Поле отсутствует - добавляем с текущим временем
                doc.insert("created", BsonDateTime::now());
                changed = true;
            }

            // Обрабатываем modified
            if let Some(modified) = doc.get("modified") {
                if !matches!(modified, Bson::DateTime(_)) {
                    if let Some(new_date) = convert_to_bson_datetime(modified) {
                        doc.insert("modified", new_date);
                        changed = true;
                    } else {
                        doc.insert("modified", BsonDateTime::now());
                        changed = true;
                    }
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

        info!("Fixed date fields for {} documents", updated);
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
        _ => None,
    }
}