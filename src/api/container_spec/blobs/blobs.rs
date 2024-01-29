use std::str::FromStr;

use rocket::{
    http::{Header, Status},
    request::{self, FromRequest},
    Request, State,
};
use sqlx::Pool;
use uuid::Uuid;

use crate::{
    api::container_spec::{errors::UnauthorizedResponse, Auth, AuthFailure},
    config::Config,
    db::DB,
    services::{self, upload_blob_service},
};

/*
#[derive(Responder, Debug)]
pub struct UploadBlobResponseData<'a> {
    response: &'a str,
    location: Header<'a>,
    range: Header<'a>,
}

#[derive(Responder, Debug)]
pub enum UploadBlobResponse<'a> {
    #[response(status = 202)]
    Success(UploadBlobResponseData<'a>),
    #[response(status = 500)]
    Failure(&'a str),
    #[response(status = 400)]
    InvalidSessionId(&'a str),
    #[response(status = 401)]
    Unauthorized(UnauthorizedResponse),
}

#[patch("/v2/<name>/blobs/uploads/<session_id>", data = "<blob>")]
pub async fn patch_upload_blob<'a>(
    auth: Result<Auth, AuthFailure>,
    name: &str,
    session_id: &'a str,
    blob: Vec<u8>,
    db_pool: &State<Pool<DB>>,
    config: &State<Config>,
) -> UploadBlobResponse<'a> {
    if let Err(e) = auth {
        return match e {
            AuthFailure::Unauthorized(resp) => UploadBlobResponse::Unauthorized(resp),
            AuthFailure::InternalServerError(err) => {
                error!("Unexpected auth error, err {err:?}");
                return UploadBlobResponse::Failure("An unexpected error occurred");
            }
        };
    };

    // Validate the session ID
    let session_id = match Uuid::from_str(session_id) {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to parse session id ({session_id}), err: {e:?}");
            return UploadBlobResponse::InvalidSessionId(session_id);
        }
    };

    let blob_length = blob.len();
    match services::upload_blob_service::upload_blob(
        db_pool,
        name.to_string(),
        session_id,
        config,
        blob,
    )
    .await
    {
        Ok(new_session_id) => {
            return UploadBlobResponse::Success(UploadBlobResponseData {
                response: "",
                location: Header::new(
                    LOCATION_HEADER_NAME,
                    format!("/v2/{name}/blobs/uploads/{new_session_id}"),
                ),
                range: Header::new(RANGE_HEADER_NAME, format!("0-{}", blob_length)),
            })
        }
        Err(e) => {
            error!("Failed to upload blob, err: {e:?}");
            return UploadBlobResponse::Failure("Failed to upload blob");
        }
    }
}
*/
