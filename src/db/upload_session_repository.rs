use sqlx::Transaction;
use uuid::Uuid;

use crate::{models::upload_session::UploadSession, registry_error::RegistryResult};

use super::DB;

pub async fn insert(
    transaction: &mut Transaction<'_, DB>,
    repository_id: Uuid,
) -> RegistryResult<UploadSession> {
    Ok(sqlx::query_as!(
        UploadSession,
        r#"
INSERT INTO upload_session(repository_id)
VALUES                    ($1)
RETURNING id, repository_id, created_at
    "#,
        repository_id
    )
    .fetch_one(transaction)
    .await?)
}
