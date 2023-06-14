use sqlx::Transaction;
use uuid::Uuid;

use crate::{models::upload_session::UploadSession, registry_error::RegistryResult};

use super::DB;

pub async fn insert(
    transaction: &mut Transaction<'_, DB>,
    prev_session: Option<Uuid>,
    digest: Option<String>,
    repository: String,
) -> RegistryResult<UploadSession> {
    Ok(sqlx::query_as!(
        UploadSession,
        r#"
INSERT INTO upload_session(digest, repository, previous_session)
VALUES                    ($1,     $2,         $3)
RETURNING id, previous_session, digest, repository, created_at, is_finished
    "#,
        digest,
        repository,
        prev_session,
    )
    .fetch_one(transaction)
    .await?)
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
    .execute(transaction)
    .await?;

    Ok(())
}

pub async fn find_by_repository_and_id(
    transaction: &mut Transaction<'_, DB>,
    repository: String,
    session_id: Uuid,
) -> RegistryResult<Option<UploadSession>> {
    Ok(sqlx::query_as!(
        UploadSession,
        r#"
SELECT id, previous_session, digest, repository, created_at, is_finished
FROM upload_session
WHERE id = $1 AND repository = $2
    "#,
        session_id,
        repository
    )
    .fetch_optional(transaction)
    .await?)
}
