use sqlx::Transaction;
use uuid::Uuid;

use crate::{models::blob::Blob, registry_error::RegistryResult};

use super::DB;

pub async fn insert(
    transaction: &mut Transaction<'_, DB>,
    repository: String,
    digest: String,
) -> RegistryResult<Blob> {
    Ok(sqlx::query_as!(
        Blob,
        r#"
INSERT INTO blob(repository, digest)
VALUES          ($1,         $2)
RETURNING id, repository, digest, created_at
    "#,
        repository,
        digest,
    )
    .fetch_one(transaction)
    .await?)
}

pub async fn find_by_repository_and_id(
    transaction: &mut Transaction<'_, DB>,
    repository: String,
    blob_id: Uuid,
) -> RegistryResult<Option<Blob>> {
    Ok(sqlx::query_as!(
        Blob,
        r#"
SELECT id, repository, digest, created_at
FROM blob
WHERE id = $1 AND repository = $2
    "#,
        blob_id,
        repository
    )
    .fetch_optional(transaction)
    .await?)
}
