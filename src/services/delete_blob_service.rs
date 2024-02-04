use std::fs;

use sqlx::Pool;

use crate::{
    config::Config,
    db::{self, blob_repository, DB},
    registry_error::{RegistryError, RegistryResult},
};

use super::upload_blob_service::get_blob_file_path;

pub async fn delete_blob(
    db_pool: &Pool<DB>,
    config: &Config,
    name: &str,
    digest: &str,
) -> RegistryResult<()> {
    let mut transaction = db::new_transaction(db_pool).await?;

    let Some(blob) =
        blob_repository::find_by_repository_and_digest(&mut transaction, name, digest).await?
    else {
        return Err(RegistryError::BlobNotFound);
    };

    blob_repository::delete_blob(&mut transaction, blob.id).await?;

    let remaining_references =
        blob_repository::find_blobs_by_digest(&mut transaction, digest).await?;

    if remaining_references.is_empty() {
        info!("Last reference to blob with digest {digest} remove, deleting file");
        delete_blob_file(config, digest)?;
    }

    transaction.commit().await?;

    Ok(())
}

fn delete_blob_file(config: &Config, digest: &str) -> RegistryResult<()> {
    let file_path = get_blob_file_path(config, digest);
    if !file_path.exists() {
        error!("blob file with digest {digest} did not exist at path {file_path:?}");
        return Err(RegistryError::BlobFileNotFound);
    }

    fs::remove_file(file_path.as_path()).map_err(|err| {
        error!("Failed to remove blob file at path {file_path:?} due to err: {err:?}");
        err
    })?;

    Ok(())
}
