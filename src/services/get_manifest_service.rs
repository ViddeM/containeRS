use std::path::Path;

use rocket::fs::NamedFile;
use sqlx::Pool;
use uuid::Uuid;

use crate::{
    config::Config,
    db::{self, blob_repository, manifest_repository, DB},
    models::{blob::Blob, manifest::Manifest},
    registry_error::RegistryResult,
};

pub struct ManifestInfo {
    pub manifest: Manifest,
    pub blob: Blob,
    pub named_file: NamedFile,
}

pub async fn find_manifest(
    db_pool: &Pool<DB>,
    namespace: &str,
    reference: &str,
    config: &Config,
) -> RegistryResult<Option<ManifestInfo>> {
    let mut transaction = db::new_transaction(db_pool).await?;

    let manifest = if reference.starts_with("sha256:") {
        info!("Identified as a digest {reference}, retrieving manifest from that");
        manifest_repository::find_by_repository_and_reference_optional(
            &mut transaction,
            namespace,
            reference,
        )
        .await?
    } else {
        info!("Assumed to be tag {reference}, retrieving manifest from that");
        manifest_repository::find_by_repository_and_tag(&mut transaction, namespace, reference)
            .await?
    };

    let manifest = if let Some(m) = manifest {
        m
    } else {
        return Ok(None);
    };

    let blob =
        blob_repository::find_by_repository_and_id(&mut transaction, namespace, manifest.blob_id)
            .await?;

    let blob = if let Some(b) = blob {
        b
    } else {
        error!("Manifest blob not found! Blob ID {}", manifest.blob_id);
        return Ok(None);
    };

    transaction.commit().await?;

    let file = manifest_file(config, manifest.id).await?;

    Ok(Some(ManifestInfo {
        manifest,
        blob,
        named_file: file,
    }))
}

async fn manifest_file(config: &Config, manifest_id: Uuid) -> RegistryResult<NamedFile> {
    let file_path = format!(
        "{}/manifests/{}.json",
        config.storage_directory,
        manifest_id.to_string()
    );
    info!("Looking for manifest at path {file_path}");
    let manifest_path = Path::new(file_path.as_str());
    let file = NamedFile::open(manifest_path).await?;

    Ok(file)
}
