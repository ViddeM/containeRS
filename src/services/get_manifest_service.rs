use std::path::Path;

use rocket::fs::NamedFile;
use sqlx::Pool;
use uuid::Uuid;

use crate::{
    config::Config,
    db::{self, blob_repository, manifest_repository, DB},
    models::{blob::Blob, manifest::Manifest},
    registry_error::{RegistryError, RegistryResult},
};

pub async fn find_manifest(
    db_pool: &Pool<DB>,
    namespace: String,
    reference: String,
    config: &Config,
) -> RegistryResult<Option<(Manifest, Blob, NamedFile)>> {
    let mut transaction = db::new_transaction(db_pool).await?;

    let manifest = if let Some(digest) = reference.strip_prefix("sha256:") {
        Some(
            manifest_repository::find_by_repository_and_digest(
                &mut transaction,
                namespace.clone(),
                digest.to_string(),
            )
            .await?,
        )
    } else {
        manifest_repository::find_by_repository_and_tag(
            &mut transaction,
            namespace.clone(),
            reference,
        )
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

    Ok(Some((manifest, blob, file)))
}

async fn manifest_file(config: &Config, manifest_id: Uuid) -> RegistryResult<NamedFile> {
    let file_path = format!(
        "{}/manifests/{}.json",
        config.storage_directory,
        manifest_id.to_string()
    );
    println!("Looking for manifest at path {file_path}");
    let manifest_path = Path::new(file_path.as_str());
    let file = NamedFile::open(manifest_path).await?;

    Ok(file)
}
