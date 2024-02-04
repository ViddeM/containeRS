use rocket::http::ContentType;
use sqlx::Pool;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use uuid::Uuid;

use crate::{
    config::Config,
    db::{self, blob_repository, manifest_layer_repository, manifest_repository, DB},
    registry_error::{RegistryError, RegistryResult},
    types::manifest::{DockerImageManifestV2, APPLICATION_CONTENT_TYPE_TOP},
};

pub async fn upload_manifest(
    db_pool: &Pool<DB>,
    config: &Config,
    namespace: &str,
    reference: &str,
    manifest_type: &ContentType,
    data: Vec<u8>,
) -> RegistryResult<(Uuid, String)> {
    let calculated_digest = format!("sha256:{}", sha256::digest(data.as_slice()));

    let image_manifest = DockerImageManifestV2::parse(manifest_type, data.clone())?;

    let mut transaction = db::new_transaction(db_pool).await?;

    let image_blob = blob_repository::find_by_repository_and_digest(
        &mut transaction,
        namespace,
        &image_manifest.config.digest,
    )
    .await?
    .ok_or(RegistryError::InvalidDigest)?;

    let manifest = match manifest_repository::find_by_repository_and_tag(
        &mut transaction,
        namespace,
        reference,
    )
    .await?
    {
        Some(m) => {
            warn!("Manifest already exists, overwriting");
            m
        }
        None => {
            let content_type = manifest_type.to_string();
            let Some(content_type_sub) = content_type.strip_prefix("application/") else {
                error!("Media type does not start with `application/`! (Got {manifest_type})");
                return Err(RegistryError::InvalidManifestSchema(
                    "Expected application/".to_string(),
                ));
            };
            manifest_repository::insert(
                &mut transaction,
                namespace,
                image_blob.id,
                reference,
                &calculated_digest,
                APPLICATION_CONTENT_TYPE_TOP,
                content_type_sub,
            )
            .await?
        }
    };

    for layer in image_manifest.layers.iter() {
        let blob = blob_repository::find_by_repository_and_digest(
            &mut transaction,
            namespace,
            &layer.digest,
        )
        .await?
        .ok_or(RegistryError::InvalidDigest)?;

        match manifest_layer_repository::find_by_manifest_and_blob(
            &mut transaction,
            manifest.id.clone(),
            blob.id.clone(),
        )
        .await?
        {
            Some(_) => {
                info!("Manifest layer already exists, skipping DB insertion");
            }
            None => {
                manifest_layer_repository::insert(
                    &mut transaction,
                    manifest.id,
                    blob.id,
                    layer.media_type.clone(),
                    layer.size,
                )
                .await?;
            }
        }
    }

    save_file(manifest.id, config, data)?;

    transaction.commit().await?;

    Ok((manifest.id, calculated_digest))
}

fn get_manifests_dir(config: &Config) -> PathBuf {
    Path::new(&config.storage_directory).join("manifests")
}

fn to_file_path(dir_path: PathBuf, manifest_id: Uuid) -> PathBuf {
    let mut file_path = dir_path.join(manifest_id.to_string());
    file_path.set_extension("json");
    file_path
}

pub fn get_manifest_file_path(config: &Config, manifest_id: Uuid) -> PathBuf {
    let dir = get_manifests_dir(config);
    to_file_path(dir, manifest_id)
}

fn save_file(manifest_id: Uuid, config: &Config, data: Vec<u8>) -> RegistryResult<()> {
    let manifests_dir = get_manifests_dir(config);

    info!("Creating directories {manifests_dir:?}");
    fs::create_dir_all(manifests_dir.clone())?;

    let file_path = to_file_path(manifests_dir, manifest_id);

    if file_path.exists() {
        info!("Manifest already exists at path {file_path:?}");
        return Ok(());
    }

    info!("Creating file manifest file {file_path:?}");
    let mut file = fs::File::create(file_path)?;

    file.write_all(&data)?;

    Ok(())
}
