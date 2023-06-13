use sqlx::Pool;

use crate::{
    db::{self, blob_repository, DB},
    registry_error::RegistryResult,
};

pub async fn find_blob_by_digest(
    db_pool: &Pool<DB>,
    namespace: String,
    digest: String,
) -> RegistryResult<Option<String>> {
    let mut transaction = db::new_transaction(db_pool).await?;

    let blob =
        blob_repository::find_by_repository_and_digest(&mut transaction, namespace, digest).await?;

    transaction.commit().await?;

    Ok(blob.map(|b| b.digest))
}
