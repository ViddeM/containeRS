use ::serde::Deserialize;
use rocket::http::ContentType;
use serde_json::value::RawValue;

use crate::registry_error::{RegistryError, RegistryResult};

pub const APPLICATION_CONTENT_TYPE_TOP: &str = "application";

const DOCKER_IMAGE_MANIFEST_V2_CONTENT_TYPE: &str = "vnd.docker.distribution.manifest.v2+json";

const FAT_MANIFEST_CONTENT_TYPE_DOCKER: &str =
    "application/vnd.docker.distribution.manifest.list.v2+json";
const FAT_MANIFEST_CONTENT_TYPE: &str = "application/vnd.oci.image.index.v1+json";
const SUPPORTED_FAT_MANIFEST_TYPES: [&str; 2] =
    [FAT_MANIFEST_CONTENT_TYPE, FAT_MANIFEST_CONTENT_TYPE_DOCKER];

const IMAGE_MANIFEST_DOCKER: &str = "application/vnd.docker.distribution.manifest.v2+json";
const IMAGE_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
const SUPPORTED_IMAGE_MANIFEST_TYPES: [&str; 2] = [IMAGE_MANIFEST, IMAGE_MANIFEST_DOCKER];

const CONTAINER_CONFIG_DOCKER: &str = "application/vnd.docker.container.image.v1+json";
const CONTAINER_CONFIG: &str = "application/vnd.oci.image.config.v1+json";
const SUPPORTED_CONTAINER_CONFIG_TYPES: [&str; 2] = [CONTAINER_CONFIG, CONTAINER_CONFIG_DOCKER];

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
    pub config: ManifestConfig,
    pub layers: Vec<LayerManifest>,
    pub media_type: Option<String>,
}

impl DockerImageManifestV2 {
    pub fn parse(content_type: &ContentType, data: Vec<u8>) -> RegistryResult<Self> {
        let slice = data.as_slice();

        if !SUPPORTED_IMAGE_MANIFEST_TYPES.contains(&content_type.to_string().as_str()) {
            error!("Got unsupported manifest type {content_type}");
            return Err(RegistryError::UnsupportedManifestType);
        }

        let image_manifest: Self = serde_json::from_slice(slice)?;
        image_manifest.validate(content_type)?;
        Ok(image_manifest)
    }

    pub fn validate(&self, content_type: &ContentType) -> RegistryResult<()> {
        if self.schema_version != 2 {
            return Err(RegistryError::InvalidManifestSchema(format!(
                "Expected manifest version 2, got {}",
                self.schema_version
            )));
        }

        if let Some(ct) = self.media_type.as_ref() {
            if ct != &content_type.to_string() {
                error!(
                    "Manifest media type ({ct}) does not match the provided content type ({})!",
                    content_type.to_string()
                );
                return Err(RegistryError::InvalidManifestSchema(format!(
                    "Manifest media type does not match content type"
                )));
            }

            let Some(media_type) = ct.strip_prefix(&format!("{APPLICATION_CONTENT_TYPE_TOP}/"))
            else {
                error!("Invalid media type for DockerImageManifestV2 '{ct}'",);
                return Err(RegistryError::InvalidManifestSchema(format!(
                    "Invalid media type for image manifest"
                )));
            };
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
        let Some(media_type) = self
            .media_type
            .strip_prefix(&format!("{APPLICATION_CONTENT_TYPE_TOP}/"))
        else {
            error!(
                "Unexpected manifest config media type '{}'",
                self.media_type
            );
            return Err(RegistryError::InvalidManifestSchema(format!(
                "Got unsupported config format {}",
                self.media_type
            )));
        };

        if !SUPPORTED_CONTAINER_CONFIG_TYPES.contains(&media_type) {
            error!(
                "Expected manifest config type to have media type {}",
                self.media_type
            );
            return Err(RegistryError::InvalidManifestSchema(format!(
                "Got unsupported config format {}",
                self.media_type
            )));
        }

        // TODO: Validate size whenever docker actually uses it...

        Ok(())
    }
}

const LAYER_TAR_GZIP: &str = "vnd.docker.image.rootfs.diff.tar.gzip";
const OCI_LAYER_TAR: &str = "vnd.oci.image.layer.v1.tar";
const OCI_LAYER_TAR_GZIP: &str = "vnd.oci.image.layer.v1.tar+gzip";
const OCI_LAYER_NONDISTRIBUTABLE_TAR: &str = "vnd.oci.image.layer.nondistributable.v1.tar";
const OCI_LAYER_NONDISTRIBUTABLE_TAR_GZIP: &str =
    "vnd.oci.image.layer.nondistributable.v1.tar+gzip";

const SUPPORTED_LAYER_TYPES: [&str; 5] = [
    LAYER_TAR_GZIP,
    OCI_LAYER_TAR,
    OCI_LAYER_TAR_GZIP,
    OCI_LAYER_NONDISTRIBUTABLE_TAR,
    OCI_LAYER_NONDISTRIBUTABLE_TAR_GZIP,
];

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LayerManifest {
    pub media_type: String,
    pub size: i64,
    pub digest: String,
}

impl LayerManifest {
    pub fn validate(&self) -> RegistryResult<()> {
        let Some(media_type) = self
            .media_type
            .strip_prefix(&format!("{APPLICATION_CONTENT_TYPE_TOP}/"))
        else {
            error!("Invalid layer manifest media type '{}'", self.media_type);
            return Err(RegistryError::InvalidManifestSchema(format!(
                "Invalid media type for schema, expected it to start with `application/`"
            )));
        };

        if !SUPPORTED_LAYER_TYPES.contains(&media_type) {
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
