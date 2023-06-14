use sqlx::Transaction;
use uuid::Uuid;

use crate::{models::manifest::Manifest, registry_error::RegistryResult};

use super::DB;

pub async fn insert(
    transaction: &mut Transaction<'_, DB>,
    repository: String,
    blob_id: Uuid,
    tag: String,
) -> RegistryResult<Manifest> {
    Ok(sqlx::query_as!(
        Manifest,
        r#"
INSERT INTO manifest(repository, tag, blob_id)
VALUES              ($1,         $2,  $3)
RETURNING id, repository, tag, blob_id, created_at
        "#,
        repository,
        tag,
        blob_id,
    )
    .fetch_one(transaction)
    .await?)
}
