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
    .fetch_one(&mut **transaction)
    .await?)
}

pub async fn find_by_manifest_and_blob(
    transaction: &mut Transaction<'_, DB>,
    manifest_id: Uuid,
    blob_id: Uuid,
) -> RegistryResult<Option<ManifestLayer>> {
    Ok(sqlx::query_as!(
        ManifestLayer,
        r#"
SELECT manifest_id, blob_id, media_type, size, created_at
FROM manifest_layer
WHERE manifest_id = $1 AND blob_id = $2
        "#,
        manifest_id,
        blob_id
    )
    .fetch_optional(&mut **transaction)
    .await?)
}

pub async fn delete_all_for_manifest(
    transaction: &mut Transaction<'_, DB>,
    manifest_id: Uuid,
) -> RegistryResult<()> {
    sqlx::query_as!(
        ManifestLayer,
        r#"
DELETE
FROM manifest_layer
WHERE manifest_id = $1   
        "#,
        manifest_id
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
}
