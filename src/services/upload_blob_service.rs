use sqlx::{Pool, Transaction};
use uuid::Uuid;

use crate::{
    db::{self, repository_repository, upload_session_repository, DB},
    models::repository::Repository,
    registry_error::{RegistryError, RegistryResult},
};

const PG_UNIQUE_CONSTRAINT_ERROR_CODE: &str = "23505";

pub async fn create_session(db_pool: &Pool<DB>, namespace: String) -> RegistryResult<Uuid> {
    let mut transaction = db::new_transaction(db_pool).await?;

    let repository = match repository_repository::insert(&mut transaction, namespace.clone()).await
    {
        Ok(r) => r,
        Err(RegistryError::SqlxError(err)) => {
            // Reset the transaction as the old one is cancelled.
            transaction.rollback().await?;
            transaction = db::new_transaction(db_pool).await?;
            get_repository_if_exists(err, &mut transaction, namespace).await?
        }
        Err(e) => return Err(e),
    };

    let session =
        upload_session_repository::insert(&mut transaction, repository.namespace_name).await?;

    transaction.commit().await?;

    Ok(session.id)
}

async fn get_repository_if_exists(
    err: sqlx::Error,
    transaction: &mut Transaction<'_, DB>,
    namespace: String,
) -> RegistryResult<Repository> {
    if let Some(db_err) = err.as_database_error() {
        if let Some(code) = db_err.code() {
            if code.to_string().as_str() == PG_UNIQUE_CONSTRAINT_ERROR_CODE {
                // The repository already exists, let's get it!
                return Ok(repository_repository::find_by_name(transaction, namespace).await?);
            }
        }
    }

    return Err(RegistryError::SqlxError(err));
}

pub async fn upload_blob(
    db_pool: &Pool<DB>,
    namespace: String,
    session_id: Uuid,
) -> RegistryResult<()> {
    let mut transaction = db::new_transaction(db_pool).await?;

    if upload_session_repository::find_by_repository_and_id(&mut transaction, namespace, session_id)
        .await?
        .is_none()
    {
        return Err(RegistryError::SessionNotFound);
    }

    Ok(())
}
