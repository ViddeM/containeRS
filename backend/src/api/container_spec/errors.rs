use rocket::{http::Header, serde::json::Json};
use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OCIError {
    BlobUnknown,
    BlobUploadInvalid,
    BlobUploadUnknown,
    DigestInvalid,
    ManifestBlobUnknown,
    ManifestInvalid,
    ManifestUnverified,
    NameInvalid,
    NameUnknown,
    SizeInvalid,
    TagInvalid,
    Unauthorized,
    Denied,
    Unsupported,
}

impl OCIError {
    pub fn to_response(self) -> ContainerSpecError {
        let (message, detail) = match self {
            OCIError::BlobUnknown => todo!(),
            OCIError::BlobUploadInvalid => todo!(),
            OCIError::BlobUploadUnknown => todo!(),
            OCIError::DigestInvalid => todo!(),
            OCIError::ManifestBlobUnknown => todo!(),
            OCIError::ManifestInvalid => todo!(),
            OCIError::ManifestUnverified => todo!(),
            OCIError::NameInvalid => todo!(),
            OCIError::NameUnknown => todo!(),
            OCIError::SizeInvalid => todo!(),
            OCIError::TagInvalid => todo!(),
            OCIError::Unauthorized => ("access to the requested resource is not authorized", "Unable to authorize client, please follow indicated authorization steps before proceeding"),
            OCIError::Denied => todo!(),
            OCIError::Unsupported => todo!(),
        };

        ContainerSpecError {
            code: self,
            message: message.to_string(),
            detail: detail.to_string(),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct ContainerSpecErrorResponse {
    errors: Vec<ContainerSpecError>,
}

#[derive(Serialize, Debug, Clone)]
pub struct ContainerSpecError {
    code: OCIError,
    message: String,
    detail: String,
}

#[derive(Responder, Debug, Clone)]
#[response(status = 401, content_type = "json")]
pub struct UnauthorizedResponse {
    inner: Json<ContainerSpecErrorResponse>,
    www_authenticate: Header<'static>,
}

impl UnauthorizedResponse {
    pub fn new(config: &Config) -> Self {
        Self {
            inner: Json(ContainerSpecErrorResponse {
                errors: vec![OCIError::Unauthorized.to_response()],
            }),
            www_authenticate: Header::new(
                "www-authenticate",
                format!(
                    r#"Bearer realm="{}", service="{}", scope=""#,
                    config.accounts_rs_auth_endpoint, config.auth_service
                ),
            ),
        }
    }
}
