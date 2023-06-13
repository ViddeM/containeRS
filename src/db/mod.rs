use sqlx::{Pool, Postgres, Transaction};

use crate::registry_error::{RegistryError, RegistryResult};

pub mod blob_repository;
pub mod manifest_layer_repository;
pub mod manifest_repository;
pub mod repository_repository;
pub mod upload_session_repository;

pub type DB = Postgres;

pub async fn new_transaction(db_pool: &Pool<DB>) -> RegistryResult<Transaction<'_, DB>> {
    match db_pool.begin().await {
        Ok(transaction) => Ok(transaction),
        Err(err) => {
            error!("Failed to create transaction: {:?}", err);
            Err(RegistryError::SqlxError(err))
        }
    }
}
