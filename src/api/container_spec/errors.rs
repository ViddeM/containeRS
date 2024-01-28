use rocket::{http::Header, serde::json::Json};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
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
    pub fn unauthorized_response() -> ContainerSpecError {
        ContainerSpecError {
            code: OCIError::Unauthorized,
            message: "access to the requested resource is not authorized".to_string(),
            detail: "Unable to authorize client, please follow indicated authorization steps before proceeding".to_string(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct ContainerSpecErrorResponse {
    errors: Vec<ContainerSpecError>,
}

#[derive(Serialize, Debug)]
pub struct ContainerSpecError {
    code: OCIError,
    message: String,
    detail: String,
}

#[derive(Responder)]
#[response(status = 401, content_type = "json")]
pub struct UnauthorizedResponse {
    inner: Json<ContainerSpecErrorResponse>,
    www_authenticate: Header<'static>,
}

#[catch(401)]
pub fn container_spec_unauthorized() -> UnauthorizedResponse {
    UnauthorizedResponse {
        inner: Json(ContainerSpecErrorResponse {
            errors: vec![OCIError::unauthorized_response()],
        }),
        www_authenticate: Header::new(
            "www-authenticate",
            r#"Bearer realm="http://localhost:8000/api/oauth/token",service="dockerboyo",scope="""#,
        ),
    }
}
