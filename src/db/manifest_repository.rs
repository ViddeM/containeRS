use sqlx::Transaction;
use uuid::Uuid;

use crate::{models::manifest::Manifest, registry_error::RegistryResult};

use super::DB;

pub async fn insert(
    transaction: &mut Transaction<'_, DB>,
    repository: String,
    blob_id: Uuid,
    tag: String,
    digest: &str,
    content_type_top: String,
    content_type_sub: String,
) -> RegistryResult<Manifest> {
    Ok(sqlx::query_as!(
        Manifest,
        r#"
INSERT INTO manifest(repository, tag, blob_id, digest, content_type_top, content_type_sub)
VALUES              ($1,         $2,  $3,      $4,     $5,               $6)
RETURNING id, repository, tag, blob_id, digest, content_type_top, content_type_sub, created_at
        "#,
        repository,
        tag,
        blob_id,
        digest,
        content_type_top,
        content_type_sub,
    )
    .fetch_one(&mut **transaction)
    .await?)
}

pub async fn find_by_repository_and_tag(
    transaction: &mut Transaction<'_, DB>,
    repository: String,
    tag: String,
) -> RegistryResult<Option<Manifest>> {
    Ok(sqlx::query_as!(
        Manifest,
        r#"
SELECT id, repository, tag, blob_id, digest, content_type_top, content_type_sub, created_at
FROM manifest
WHERE repository = $1 AND tag = $2
    "#,
        repository,
        tag
    )
    .fetch_optional(&mut **transaction)
    .await?)
}

pub async fn find_by_repository_and_digest(
    transaction: &mut Transaction<'_, DB>,
    repository: String,
    digest: String,
) -> RegistryResult<Manifest> {
    Ok(sqlx::query_as!(
        Manifest,
        r#"
SELECT m.id, m.repository, m.tag, m.blob_id, m.digest, m.content_type_top, m.content_type_sub, m.created_at
FROM manifest m
INNER JOIN blob b ON m.blob_id = b.id
WHERE m.repository = $1 AND b.digest = $2
        "#,
        repository,
        digest
    )
    .fetch_one(&mut **transaction)
    .await?)
}

pub async fn find_by_repository_and_reference_optional(
    transaction: &mut Transaction<'_, DB>,
    repository: String,
    reference: String,
) -> RegistryResult<Option<Manifest>> {
    Ok(sqlx::query_as!(
        Manifest,
        r#"
SELECT m.id, m.repository, m.tag, m.blob_id, m.digest, m.content_type_top, m.content_type_sub, m.created_at
FROM manifest m
WHERE m.repository = $1 AND m.tag = $2
        "#,
        repository,
        reference
    )
    .fetch_optional(&mut **transaction)
    .await?)
}

pub async fn find_all_by_repository(
    transaction: &mut Transaction<'_, DB>,
    repository: String,
) -> RegistryResult<Vec<Manifest>> {
    Ok(sqlx::query_as!(
        Manifest,
        r#"
SELECT id, repository, tag, blob_id, digest, content_type_top, content_type_sub, created_at
FROM manifest
WHERE repository = $1
        "#,
        repository
    )
    .fetch_all(&mut **transaction)
    .await?)
}
