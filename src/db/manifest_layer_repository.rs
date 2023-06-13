use sqlx::Transaction;
use uuid::Uuid;

use crate::{models::manifest_layer::ManifestLayer, registry_error::RegistryResult};

use super::DB;

pub async fn insert(
    transaction: &mut Transaction<'_, DB>,
    manifest_id: Uuid,
    blob_id: Uuid,
    media_type: String,
    size: i64,
) -> RegistryResult<ManifestLayer> {
    Ok(sqlx::query_as!(
        ManifestLayer,
        r#"
INSERT INTO manifest_layer(manifest_id, blob_id, media_type, size)
VALUES                    ($1,          $2,      $3,         $4)
RETURNING manifest_id, blob_id, media_type, size, created_at
        "#,
        manifest_id,
        blob_id,
        media_type,
        size
    )
    .fetch_one(transaction)
    .await?)
}
