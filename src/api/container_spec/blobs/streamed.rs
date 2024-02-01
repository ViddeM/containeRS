use std::str::FromStr;

use rocket::{http::Header, State};
use sqlx::Pool;
use uuid::Uuid;

use crate::{
    api::container_spec::{
        blobs::utils::octet_stream::OctetStream, errors::UnauthorizedResponse, Auth, AuthFailure,
        DOCKER_UPLOAD_UUID_HEADER_NAME, LOCATION_HEADER_NAME, RANGE_HEADER_NAME,
    },
    config::Config,
    db::DB,
    header, location,
    services::upload_blob_service,
};

use super::utils::content_length::ContentLength;

/// This flow doesn't seem to be covered by the specification?
/// This implementation is taken from Microsofts REST Api spec for the flow: https://learn.microsoft.com/en-us/rest/api/containerregistry/blob/start-upload?view=rest-containerregistry-2019-08-15&tabs=HTTP
///  1. a POST to create the session and receive an upload location. (Is taken care of the generic way)
///  2. a number of PATCH requests to upload the parts of the blob.
///  3. a POST to end the upload (optionally with content).

#[derive(Responder, Debug)]
pub struct UploadBlobResponseData<'a> {
    data: (),
    location: Header<'a>,
    range: Header<'a>,
    docker_upload_uuid: Header<'a>,
}

#[derive(Responder)]
pub enum UploadBlobResponse<'a> {
    #[response(status = 202)]
    Success(UploadBlobResponseData<'a>),
    #[response(status = 401)]
    Unauthorized(UnauthorizedResponse),
    #[response(status = 500)]
    Failure(&'a str),
}

#[patch("/v2/<name>/blobs/uploads/<session_id>", data = "<blob>")]
pub async fn patch_upload_blob<'a>(
    config: &State<Config>,
    db_pool: &State<Pool<DB>>,
    auth: Result<Auth, AuthFailure>,
    name: &str,
    session_id: &'a str,
    blob: OctetStream,
) -> UploadBlobResponse<'a> {
    if let Err(err) = auth {
        match err {
            AuthFailure::Unauthorized(resp) => return UploadBlobResponse::Unauthorized(resp),
            AuthFailure::InternalServerError(err) => {
                error!("Unexpected auth failure {err:?}");
                return UploadBlobResponse::Failure("An unexpected error occurred");
            }
        }
    };

    let session_id = match Uuid::from_str(session_id) {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to parse session id ({session_id}), err: {e:?}");
            return UploadBlobResponse::Failure("Invalid session ID");
        }
    };

    let new_session = match upload_blob_service::upload_blob(
        db_pool, name, session_id, config, blob.data,
    )
    .await
    {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to upload blob, err: {e:?}");
            return UploadBlobResponse::Failure("Failed to upload blob");
        }
    };

    UploadBlobResponse::Success(UploadBlobResponseData {
        data: (),
        location: location!(name, new_session.id),
        range: header!(
            RANGE_HEADER_NAME,
            format!("0-{}", new_session.starting_byte_index)
        ),
        docker_upload_uuid: header!(DOCKER_UPLOAD_UUID_HEADER_NAME, new_session.id.to_string()),
    })
}

#[derive(Responder, Debug)]
pub struct FinishBlobUploadResponseData<'a> {
    response: &'a str,
    location: Header<'a>,
}

#[derive(Responder, Debug)]
pub enum FinishBlobUploadResponse<'a> {
    #[response(status = 201)]
    Success(FinishBlobUploadResponseData<'a>),
    #[response(status = 500)]
    Failure(&'a str),
    #[response(status = 400)]
    InvalidSessionId(&'a str),
    #[response(status = 401)]
    Unauthorized(UnauthorizedResponse),
}

#[put("/v2/<name>/blobs/uploads/<session_id>?<digest>", data = "<blob>")]
pub async fn put_upload_blob<'a>(
    auth: Result<Auth, AuthFailure>,
    name: &str,
    session_id: &'a str,
    digest: &'a str,
    content_length: ContentLength,
    blob: Option<OctetStream>,
    config: &State<Config>,
    db_pool: &State<Pool<DB>>,
) -> FinishBlobUploadResponse<'a> {
    if let Err(e) = auth {
        return match e {
            AuthFailure::Unauthorized(resp) => FinishBlobUploadResponse::Unauthorized(resp),
            AuthFailure::InternalServerError(err) => {
                error!("Unexpected auth error, err: {err:?}");
                FinishBlobUploadResponse::Failure("An unexpected failure occured")
            }
        };
    }

    let session_id = match Uuid::from_str(session_id) {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to parse session id ({session_id}), err: {e:?}");
            return FinishBlobUploadResponse::InvalidSessionId(session_id);
        }
    };

    let final_session_id = if let Some(blob) = blob {
        let blob = blob.data;

        if blob.len() != content_length.length {
            return FinishBlobUploadResponse::Failure(
                "Content length doesn't match the provided blobs length",
            );
        }

        match upload_blob_service::upload_blob(db_pool, name, session_id, config, blob).await {
            Ok(s) => s,
            Err(err) => {
                error!("Failed to upload blob during finish upload, err: {err:?}");
                return FinishBlobUploadResponse::Failure("Failed to upload blob");
            }
        }
        .id
    } else {
        session_id
    };

    let blob_id = match upload_blob_service::finish_blob_upload(
        db_pool,
        config,
        name.to_string(),
        final_session_id,
        digest.to_string(),
    )
    .await
    {
        Ok(blob_id) => blob_id,
        Err(e) => {
            error!("Failed to finish blob upload, err: {e:?}");
            return FinishBlobUploadResponse::Failure("Failed to finish blob upload");
        }
    };

    FinishBlobUploadResponse::Success(FinishBlobUploadResponseData {
        response: "Blob upload finished",
        location: Header::new(LOCATION_HEADER_NAME, blob_id.to_string()),
    })
}
