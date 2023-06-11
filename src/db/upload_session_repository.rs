use sqlx::Transaction;
use uuid::Uuid;

use crate::{models::upload_session::UploadSession, registry_error::RegistryResult};

use super::DB;

pub async fn insert(
    transaction: &mut Transaction<'_, DB>,
    repository: String,
) -> RegistryResult<UploadSession> {
    Ok(sqlx::query_as!(
        UploadSession,
        r#"
INSERT INTO upload_session(repository)
VALUES                    ($1)
RETURNING id, repository, created_at
    "#,
        repository
    )
    .fetch_one(transaction)
    .await?)
}

pub async fn find_by_repository_and_id(
    transaction: &mut Transaction<'_, DB>,
    repository: String,
    session_id: Uuid,
) -> RegistryResult<Option<UploadSession>> {
    Ok(sqlx::query_as!(
        UploadSession,
        r#"
SELECT id, repository, created_at
FROM upload_session
WHERE id = $1 AND repository = $2
    "#,
        session_id,
        repository
    )
    .fetch_optional(transaction)
    .await?)
}
