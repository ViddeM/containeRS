use sqlx::Transaction;
use uuid::Uuid;

use crate::{models::blob::Blob, registry_error::RegistryResult};

use super::DB;

pub async fn insert(
    transaction: &mut Transaction<'_, DB>,
    repository: &str,
    digest: &str,
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
    .fetch_one(&mut **transaction)
    .await?)
}

pub async fn find_by_repository_and_id(
    transaction: &mut Transaction<'_, DB>,
    repository: &str,
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
    .fetch_optional(&mut **transaction)
    .await?)
}

pub async fn find_by_repository_and_digest(
    transaction: &mut Transaction<'_, DB>,
    repository: &str,
    digest: &str,
) -> RegistryResult<Option<Blob>> {
    Ok(sqlx::query_as!(
        Blob,
        r#"
SELECT id, repository, digest, created_at
FROM blob
WHERE digest = $1 AND repository = $2
    "#,
        digest,
        repository
    )
    .fetch_optional(&mut **transaction)
    .await?)
}

pub async fn delete_blob(transaction: &mut Transaction<'_, DB>, id: Uuid) -> RegistryResult<()> {
    sqlx::query_as!(
        Blob,
        r#"
DELETE
FROM blob
WHERE id = $1
        "#,
        id
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

pub async fn find_blobs_by_digest(
    transaction: &mut Transaction<'_, DB>,
    digest: &str,
) -> RegistryResult<Vec<Blob>> {
    Ok(sqlx::query_as!(
        Blob,
        r#"
SELECT id, repository, digest, created_at
FROM blob
WHERE digest = $1
    "#,
        digest
    )
    .fetch_all(&mut **transaction)
    .await?)
}
