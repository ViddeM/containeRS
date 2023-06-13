use std::{fs, io::Write, path::Path};

use serde::Deserialize;
use serde_json::value::RawValue;
use sqlx::Pool;

use crate::{
    config::Config,
    db::{self, DB},
    registry_error::{RegistryError, RegistryResult},
};

pub async fn upload_manifest(
    namespace: String,
    reference: String,
    content_length: usize,
    content_type: String,
    data: Vec<u8>,
    config: &Config,
    db_pool: &Pool<DB>,
) -> RegistryResult<()> {
    if content_length != data.len() {
        error!(
            "Invalid content length, got {content_length} but data is {}",
            data.len()
        );
        return Err(RegistryError::InvalidContentLength);
    }

    // Validate the manifest
    ManifestType::parse(content_type, &data)?;

    let mut transaction = db::new_transaction(db_pool).await?;

    transaction.commit().await?;

    let digest = "".to_string();

    save_file(namespace, digest, config, data)?;

    Ok(())
}

fn save_file(
    namespace: String,
    digest: String,
    config: &Config,
    data: Vec<u8>,
) -> RegistryResult<()> {
    let path_name = format!("{}/{namespace}/images/sha256", config.storage_directory);
    let path = Path::new(path_name.as_str());
    fs::create_dir_all(path)?;

    let file_path_name = format!("{}.json", digest);
    let file_path = Path::new(file_path_name.as_str());
    let mut file = fs::File::create(path.join(file_path))?;

    file.write_all(&data)?;

    Ok(())
}

const FAT_MANIFEST_CONTENT_TYPE: &str = "application/vnd.docker.distribution.manifest.list.v2+json";
const DOCKER_IMAGE_MANIFEST_V2: &str = "application/vnd.docker.distribution.manifest.v2+json";

#[derive(Debug, Clone)]
enum ManifestType<'a> {
    FatManifest(FatManifest<'a>),
    DockerImageManifestV2(DockerImageManifestV2<'a>),
}

impl<'a> ManifestType<'a> {
    fn parse(content_type: String, data: &'a Vec<u8>) -> RegistryResult<Self> {
        let slice = data.as_slice();
        match content_type.as_str() {
            FAT_MANIFEST_CONTENT_TYPE => {
                let fat_manifest: FatManifest = serde_json::from_slice(slice)?;
                fat_manifest.validate()?;
                Ok(Self::FatManifest(fat_manifest))
            }
            DOCKER_IMAGE_MANIFEST_V2 => {
                let image_manifest: DockerImageManifestV2 = serde_json::from_slice(slice)?;
                image_manifest.validate()?;
                Ok(Self::DockerImageManifestV2(image_manifest))
            }
            _ => {
                error!("Got unsupported manifest type {content_type}");
                Err(RegistryError::UnsupportedManifestType)
            }
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct FatManifest<'a> {
    schema_version: i32,
    media_type: String,
    #[serde(borrow)]
    manifest: &'a RawValue,
}

impl<'a> FatManifest<'a> {
    fn validate(&self) -> RegistryResult<()> {
        if self.schema_version != 2 {
            return Err(RegistryError::InvalidManifestSchema(format!(
                "Expected manifest version 2, got {}",
                self.schema_version
            )));
        }

        if self.media_type.as_str() != FAT_MANIFEST_CONTENT_TYPE {
            return Err(RegistryError::InvalidManifestSchema(format!(
                "Expected media_type {FAT_MANIFEST_CONTENT_TYPE}, got {}",
                self.media_type
            )));
        }

        Ok(())
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct DockerImageManifestV2<'a> {
    schema_version: i32,
    media_type: String,
    #[serde(borrow)]
    config: &'a RawValue,
    #[serde(borrow)]
    layers: &'a RawValue,
}

impl<'a> DockerImageManifestV2<'a> {
    fn validate(&self) -> RegistryResult<()> {
        if self.schema_version != 2 {
            return Err(RegistryError::InvalidManifestSchema(format!(
                "Expected manifest version 2, got {}",
                self.schema_version
            )));
        }

        if self.media_type.as_str() != DOCKER_IMAGE_MANIFEST_V2 {
            return Err(RegistryError::InvalidManifestSchema(format!(
                "Expected media_type {DOCKER_IMAGE_MANIFEST_V2}, got {}",
                self.media_type
            )));
        }

        Ok(())
    }
}
