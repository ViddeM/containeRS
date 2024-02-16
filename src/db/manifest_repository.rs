use sqlx::Transaction;
use uuid::Uuid;

use crate::{models::manifest::Manifest, registry_error::RegistryResult};

use super::DB;

pub async fn insert(
    transaction: &mut Transaction<'_, DB>,
    repository: &str,
    blob_id: Uuid,
    tag: Option<&str>,
    digest: &str,
    content_type_top: &str,
    content_type_sub: &str,
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
    repository: &str,
    tag: Option<&str>,
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

pub async fn find_first_by_repository_and_digest(
    transaction: &mut Transaction<'_, DB>,
    repository: &str,
    digest: &str,
) -> RegistryResult<Option<Manifest>> {
    Ok(sqlx::query_as!(
        Manifest,
        r#"
SELECT m.id, m.repository, m.tag, m.blob_id, m.digest, m.content_type_top, m.content_type_sub, m.created_at
FROM manifest m
WHERE m.repository = $1 AND m.digest = $2
        "#,
        repository,
        digest
    )
    .fetch_optional(&mut **transaction)
    .await?)
}

pub async fn find_by_repository_and_digest(
    transaction: &mut Transaction<'_, DB>,
    repository: &str,
    digest: &str,
) -> RegistryResult<Vec<Manifest>> {
    Ok(sqlx::query_as!(
        Manifest,
        r#"
SELECT m.id, m.repository, m.tag, m.blob_id, m.digest, m.content_type_top, m.content_type_sub, m.created_at
FROM manifest m
WHERE m.repository = $1 AND m.digest = $2
        "#,
        repository,
        digest
    )
    .fetch_all(&mut **transaction)
    .await?)
}

pub async fn find_all_by_repository(
    transaction: &mut Transaction<'_, DB>,
    repository: &str,
) -> RegistryResult<Vec<Manifest>> {
    Ok(sqlx::query_as!(
        Manifest,
        r#"
SELECT id, repository, tag, blob_id, digest, content_type_top, content_type_sub, created_at
FROM manifest
WHERE repository = $1
ORDER BY tag ASC
        "#,
        repository
    )
    .fetch_all(&mut **transaction)
    .await?)
}

pub async fn find_all_by_repository_max(
    transaction: &mut Transaction<'_, DB>,
    repository: &str,
    n: i64,
) -> RegistryResult<Vec<Manifest>> {
    Ok(sqlx::query_as!(
        Manifest,
        r#"
SELECT id, repository, tag, blob_id, digest, content_type_top, content_type_sub, created_at
FROM manifest
WHERE repository = $1
ORDER BY tag ASC
LIMIT $2
        "#,
        repository,
        n
    )
    .fetch_all(&mut **transaction)
    .await?)
}

pub async fn find_all_by_repository_last(
    transaction: &mut Transaction<'_, DB>,
    repository: &str,
    last: &str,
) -> RegistryResult<Vec<Manifest>> {
    Ok(sqlx::query_as!(
        Manifest,
        r#"
SELECT id, repository, tag, blob_id, digest, content_type_top, content_type_sub, created_at
FROM manifest
WHERE repository = $1 AND tag > $2
ORDER BY tag ASC
        "#,
        repository,
        last,
    )
    .fetch_all(&mut **transaction)
    .await?)
}

pub async fn find_all_by_repository_last_max(
    transaction: &mut Transaction<'_, DB>,
    repository: &str,
    last: &str,
    n: i64,
) -> RegistryResult<Vec<Manifest>> {
    Ok(sqlx::query_as!(
        Manifest,
        r#"
SELECT id, repository, tag, blob_id, digest, content_type_top, content_type_sub, created_at
FROM manifest
WHERE repository = $1 AND tag > $2
ORDER BY tag ASC
LIMIT $3
        "#,
        repository,
        last,
        n
    )
    .fetch_all(&mut **transaction)
    .await?)
}

pub async fn find_all_by_blob_id_and_repository(
    transaction: &mut Transaction<'_, DB>,
    repository: &str,
    blob_id: Uuid,
) -> RegistryResult<Vec<Manifest>> {
    Ok(sqlx::query_as!(
        Manifest,
        r#"
SELECT id, repository, tag, blob_id, digest, content_type_top, content_type_sub, created_at
FROM manifest
WHERE blob_id = $1 AND repository = $2
        "#,
        blob_id,
        repository
    )
    .fetch_all(&mut **transaction)
    .await?)
}

pub async fn delete_manifest(
    transaction: &mut Transaction<'_, DB>,
    id: Uuid,
) -> RegistryResult<()> {
    sqlx::query_as!(
        Manifest,
        r#"
DELETE
FROM manifest
WHERE id = $1
    "#,
        id
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

pub async fn delete_tag(
    transaction: &mut Transaction<'_, DB>,
    name: &str,
    tag: &str,
) -> RegistryResult<()> {
    sqlx::query_as!(
        Manifest,
        r#"
UPDATE manifest
SET tag = NULL
WHERE repository = $1 AND tag = $2
        "#,
        name,
        tag
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
}
