use ::serde::Deserialize;
use serde_json::value::RawValue;

use crate::registry_error::{RegistryError, RegistryResult};

const FAT_MANIFEST_CONTENT_TYPE: &str = "application/vnd.docker.distribution.manifest.list.v2+json";
const DOCKER_IMAGE_MANIFEST_V2: &str = "application/vnd.docker.distribution.manifest.v2+json";
const CONTAINER_CONFIG_JSON: &str = "application/vnd.docker.container.image.v1+json";
const LAYER_TAR_GZIP: &str = "application/vnd.docker.image.rootfs.diff.tar.gzip";

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FatManifest<'a> {
    pub schema_version: i32,
    pub media_type: String,
    #[serde(borrow)]
    manifest: &'a RawValue, // TODO: Implement
}

impl<'a> FatManifest<'a> {
    pub fn validate(&self) -> RegistryResult<()> {
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
pub struct DockerImageManifestV2 {
    pub schema_version: i32,
    pub media_type: String,
    pub config: ManifestConfig,
    pub layers: Vec<LayerManifest>,
}

impl DockerImageManifestV2 {
    pub fn parse(content_type: String, data: Vec<u8>) -> RegistryResult<Self> {
        let slice = data.as_slice();
        match content_type.as_str() {
            DOCKER_IMAGE_MANIFEST_V2 => {
                let image_manifest: Self = serde_json::from_slice(slice)?;
                image_manifest.validate()?;
                Ok(image_manifest)
            }
            _ => {
                error!("Got unsupported manifest type {content_type}");
                Err(RegistryError::UnsupportedManifestType)
            }
        }
    }

    pub fn validate(&self) -> RegistryResult<()> {
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

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ManifestConfig {
    pub media_type: String,
    pub size: i64,
    pub digest: String,
}

impl ManifestConfig {
    pub fn validate(&self) -> RegistryResult<()> {
        if self.media_type.as_str() != CONTAINER_CONFIG_JSON {
            error!(
                "Expected manifest config type to have media type {}",
                self.media_type
            );
            return Err(RegistryError::InvalidManifestSchema(format!(
                "Expected media_type {CONTAINER_CONFIG_JSON}, got {}",
                self.media_type
            )));
        }

        // TODO: Validate size whenever docker actually uses it...

        Ok(())
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LayerManifest {
    pub media_type: String,
    pub size: i64,
    pub digest: String,
}

impl LayerManifest {
    pub fn validate(&self) -> RegistryResult<()> {
        if self.media_type != LAYER_TAR_GZIP {
            error!(
                "Expected manifest config type to have media type {}",
                self.media_type
            );
            return Err(RegistryError::InvalidManifestSchema(format!(
                "Expected media_type {LAYER_TAR_GZIP}, got {}",
                self.media_type
            )));
        }

        // TODO: Validate size...

        Ok(())
    }
}
