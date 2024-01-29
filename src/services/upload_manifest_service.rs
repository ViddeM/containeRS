use sqlx::Pool;
use std::{fs, io::Write, path::Path};
use uuid::Uuid;

use crate::{
    config::Config,
    db::{self, blob_repository, manifest_layer_repository, manifest_repository, DB},
    registry_error::{RegistryError, RegistryResult},
    types::manifest::{
        DockerImageManifestV2, APPLICATION_CONTENT_TYPE_TOP,
        DOCKER_IMAGE_MANIFEST_V2_CONTENT_TYPE_SUB,
    },
};

pub async fn upload_manifest(
    namespace: String,
    reference: String,
    content_length: usize,
    content_type: String,
    data: Vec<u8>,
    config: &Config,
    db_pool: &Pool<DB>,
) -> RegistryResult<(Uuid, String)> {
    if content_length != data.len() {
        error!(
            "Invalid content length, got {content_length} but data is {}",
            data.len()
        );
        return Err(RegistryError::InvalidContentLength);
    }

    let calculated_digest = format!("sha256:{}", sha256::digest(data.as_slice()));

    let image_manifest = DockerImageManifestV2::parse(content_type, data.clone())?;

    let mut transaction = db::new_transaction(db_pool).await?;

    let image_blob = blob_repository::find_by_repository_and_digest(
        &mut transaction,
        namespace.clone(),
        &image_manifest.config.digest,
    )
    .await?
    .ok_or(RegistryError::InvalidDigest)?;

    let manifest = match manifest_repository::find_by_repository_and_tag(
        &mut transaction,
        namespace.clone(),
        reference.clone(),
    )
    .await?
    {
        Some(m) => {
            warn!("Manifest already exists, overwriting");
            m
        }
        None => {
            manifest_repository::insert(
                &mut transaction,
                namespace.clone(),
                image_blob.id,
                reference.clone(),
                &calculated_digest,
                APPLICATION_CONTENT_TYPE_TOP.to_string(),
                DOCKER_IMAGE_MANIFEST_V2_CONTENT_TYPE_SUB.to_string(),
            )
            .await?
        }
    };

    for layer in image_manifest.layers.iter() {
        let blob = blob_repository::find_by_repository_and_digest(
            &mut transaction,
            namespace.clone(),
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

fn save_file(manifest_id: Uuid, config: &Config, data: Vec<u8>) -> RegistryResult<()> {
    let path_name = format!("{}/manifests", config.storage_directory);
    let path = Path::new(path_name.as_str());
    fs::create_dir_all(path)?;
    info!("Creating directories {path:?}");

    let file_path_name = format!("{}.json", manifest_id.to_string());
    let file_path = Path::new(file_path_name.as_str());
    let mut file = fs::File::create(path.join(file_path))?;
    info!("File stored at {file_path:?}");

    file.write_all(&data)?;

    Ok(())
}
