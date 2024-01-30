use std::str::FromStr;

use rocket::{http::Header, State};
use sqlx::Pool;
use uuid::Uuid;

use crate::{
    api::container_spec::{
        blobs::BlobUploadHeaders, errors::UnauthorizedResponse, Auth, AuthFailure,
        DOCKER_UPLOAD_UUID_HEADER_NAME, LOCATION_HEADER_NAME, RANGE_HEADER_NAME,
    },
    config::Config,
    db::DB,
    services::upload_blob_service,
};

use super::OctetStream;

macro_rules! header {
    ($name: expr, $value: expr) => {
        Header::new($name, $value)
    };
}

macro_rules! location {
    ($name: expr, $session_id: expr) => {
        header!(
            LOCATION_HEADER_NAME,
            format!("/v2/{}/blobs/uploads/{}", $name, $session_id)
        )
    };
}

/// This flow doesn't seem to be covered by the specification?
///  1. a POST to create the session and receive an upload location.
///  2. a PATCH to upload the chunk in its entirty.
///  3. a POST to end the upload.

#[derive(Responder, Debug)]
pub struct CreateSessionResponseData<'a> {
    response: &'a str,
    location: Header<'a>,
    range: Header<'a>,
    docker_upload_uuid: Header<'a>,
}

#[derive(Responder)]
pub enum CreateSessionResponse<'a> {
    #[response(status = 202)]
    Success(CreateSessionResponseData<'a>),
    #[response(status = 401)]
    Unauthorized(UnauthorizedResponse),
    #[response(status = 500)]
    Failure(&'a str),
}

#[post("/v2/<name>/blobs/uploads")]
pub async fn post_create_session<'a>(
    auth: Result<Auth, AuthFailure>,
    name: &str,
    db_pool: &State<Pool<DB>>,
) -> CreateSessionResponse<'a> {
    let auth = match auth {
        Ok(auth) => auth,
        Err(AuthFailure::Unauthorized(resp)) => return CreateSessionResponse::Unauthorized(resp),
        Err(AuthFailure::InternalServerError(err)) => {
            error!("Unexpected auth failure {err:?}");
            return CreateSessionResponse::Failure("An unexpected error occurred");
        }
    };

    let initial_session_id =
        match upload_blob_service::create_session(db_pool, &auth.username, name).await {
            Ok(id) => id,
            Err(e) => {
                error!("Failed to create upload session, err: {e:?}");
                return CreateSessionResponse::Failure("Failed to ceate session");
            }
        };

    CreateSessionResponse::Success(CreateSessionResponseData {
        response: "Session created successfully",
        location: location!(name, initial_session_id),
        range: header!(RANGE_HEADER_NAME, "0-0"),
        docker_upload_uuid: header!(
            DOCKER_UPLOAD_UUID_HEADER_NAME,
            initial_session_id.to_string()
        ),
    })
}

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
    auth: Result<Auth, AuthFailure>,
    name: &str,
    session_id: &'a str,
    blob: OctetStream,
    config: &State<Config>,
    db_pool: &State<Pool<DB>>,
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
    upload_headers: BlobUploadHeaders,
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

        if blob.len() != upload_headers.content_length {
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
