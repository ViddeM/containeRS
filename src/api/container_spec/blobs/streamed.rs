use std::str::FromStr;

use rocket::{http::Header, State};
use sqlx::Pool;
use uuid::Uuid;

use crate::{
    api::container_spec::{
        blobs::BlobUploadHeaders, errors::UnauthorizedResponse, Auth, AuthFailure,
        CONTENT_LENGTH_HEADER_NAME, LOCATION_HEADER_NAME,
    },
    config::Config,
    db::DB,
    services::upload_blob_service,
};

/// This flow consists of two steps:
///  1. a POST to create the session and receive an upload location.
///  2. a PUT to the previously provided location with the actual data.

#[derive(Responder, Debug)]
pub struct CreateSessionResponseData<'a> {
    response: &'a str,
    location: Header<'a>,
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

    match upload_blob_service::create_session(db_pool, &auth.username, name).await {
        Ok(session_id) => CreateSessionResponse::Success(CreateSessionResponseData {
            response: "Session created successfully",
            location: Header::new(
                LOCATION_HEADER_NAME,
                format!("/v2/{name}/blobs/uploads/{session_id}"),
            ),
        }),
        Err(e) => {
            error!("Failed to create upload session, err: {e:?}");
            CreateSessionResponse::Failure("Failed to ceate session")
        }
    }
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
    blob: Vec<u8>,
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

    if blob.len() != upload_headers.content_length {
        return FinishBlobUploadResponse::Failure(
            "Content length doesn't match the provided blobs length",
        );
    }

    let session_id = match Uuid::from_str(session_id) {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to parse session id ({session_id}), err: {e:?}");
            return FinishBlobUploadResponse::InvalidSessionId(session_id);
        }
    };

    let session_id =
        match upload_blob_service::upload_blob(db_pool, name.to_string(), session_id, config, blob)
            .await
        {
            Ok(s) => s,
            Err(err) => {
                error!("Failed to upload blob during finish upload, err: {err:?}");
                return FinishBlobUploadResponse::Failure("Failed to upload blob");
            }
        };

    let blob_id = match upload_blob_service::finish_blob_upload(
        db_pool,
        name.to_string(),
        session_id,
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
