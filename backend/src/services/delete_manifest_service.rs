use std::fs;

use sqlx::Pool;
use uuid::Uuid;

use crate::{
    config::Config,
    db::{self, manifest_layer_repository, manifest_repository, DB},
    registry_error::{RegistryError, RegistryResult},
};

use super::upload_manifest_service::get_manifest_file_path;

pub async fn delete_tag(db_pool: &Pool<DB>, name: &str, tag: &str) -> RegistryResult<()> {
    let mut transaction = db::new_transaction(db_pool).await?;

    if let Err(err) = manifest_repository::delete_tag(&mut transaction, name, tag).await {
        warn!("Failed to set tag to null in {name} / {tag} due to err: {err:?}");
        return Err(RegistryError::FailedToDeleteTag);
    }

    Ok(())
}

pub async fn delete_manifest(
    db_pool: &Pool<DB>,
    config: &Config,
    name: &str,
    digest: &str,
) -> RegistryResult<()> {
    let mut transaction = db::new_transaction(db_pool).await?;

    let manifests =
        manifest_repository::find_by_repository_and_digest(&mut transaction, name, digest).await?;

    if manifests.is_empty() {
        warn!("Manifest not found in {name} / {digest}");
        return Err(RegistryError::ManifestNotFound);
    };

    for manifest in manifests.into_iter() {
        manifest_layer_repository::delete_all_for_manifest(&mut transaction, manifest.id).await?;

        manifest_repository::delete_manifest(&mut transaction, manifest.id).await?;

        delete_manifest_file(config, manifest.id)?;
    }

    transaction.commit().await?;

    Ok(())
}

fn delete_manifest_file(config: &Config, manifest_id: Uuid) -> RegistryResult<()> {
    let file_path = get_manifest_file_path(config, manifest_id);

    if !file_path.exists() {
        error!("Manifest file for manifest {manifest_id} does not exist at path {file_path:?}!");
        return Err(RegistryError::ManifestFileNotFound);
    }

    fs::remove_file(file_path)?;

    Ok(())
}
