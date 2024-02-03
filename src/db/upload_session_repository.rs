use sqlx::Transaction;
use uuid::Uuid;

use crate::{models::upload_session::UploadSession, registry_error::RegistryResult};

use super::DB;

pub async fn insert(
    transaction: &mut Transaction<'_, DB>,
    prev_session: Option<Uuid>,
    starting_byte_index: i32,
    repository: String,
) -> RegistryResult<UploadSession> {
    Ok(sqlx::query_as!(
        UploadSession,
        r#"
INSERT INTO upload_session(digest, starting_byte_index, repository, previous_session)
VALUES                    (null,   $1,                  $2,         $3)
RETURNING id, previous_session, starting_byte_index, digest, repository, created_at, is_finished
    "#,
        starting_byte_index,
        repository,
        prev_session,
    )
    .fetch_one(&mut **transaction)
    .await?)
}

pub async fn set_digest(
    transaction: &mut Transaction<'_, DB>,
    id: Uuid,
    digest: String,
) -> RegistryResult<()> {
    sqlx::query_as!(
        UploadSession,
        r#"
UPDATE upload_session
SET digest = $1
WHERE id = $2
        "#,
        digest,
        id
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

pub async fn set_finished(
    transaction: &mut Transaction<'_, DB>,
    session_id: Uuid,
    repository: String,
) -> RegistryResult<()> {
    sqlx::query_as!(
        UploadSession,
        r#"
UPDATE upload_session
SET is_finished=TRUE
WHERE id = $1 AND repository = $2
        "#,
        session_id,
        repository
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

pub async fn find_by_repository_and_id(
    transaction: &mut Transaction<'_, DB>,
    repository: &str,
    session_id: Uuid,
) -> RegistryResult<Option<UploadSession>> {
    Ok(sqlx::query_as!(
        UploadSession,
        r#"
SELECT id, previous_session, starting_byte_index, digest, repository, created_at, is_finished
FROM upload_session
WHERE id = $1 AND repository = $2
    "#,
        session_id,
        repository
    )
    .fetch_optional(&mut **transaction)
    .await?)
}

pub async fn try_find_by_previous_id(
    transaction: &mut Transaction<'_, DB>,
    repository: &str,
    session_id: Uuid,
) -> RegistryResult<Option<UploadSession>> {
    Ok(sqlx::query_as!(
        UploadSession,
        r#"
SELECT id, previous_session, starting_byte_index, digest, repository, created_at, is_finished
FROM upload_session
WHERE previous_session = $1 AND repository = $2
    "#,
        session_id,
        repository
    )
    .fetch_optional(&mut **transaction)
    .await?)
}
