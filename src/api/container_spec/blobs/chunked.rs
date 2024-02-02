use std::str::FromStr;

use rocket::{http::Header, State};
use sqlx::Pool;
use uuid::Uuid;

use crate::{
    api::container_spec::{
        blobs::utils::{
            content_length::ContentLength, content_range::ContentRange, octet_stream::OctetStream,
        },
        errors::UnauthorizedResponse,
        Auth, AuthFailure,
    },
    check_auth,
    config::Config,
    db::DB,
    services::upload_blob_service::{self, upload_blob},
};

use super::utils::content_length;

#[derive(Responder, Debug)]
pub struct UploadBlobResponseData<'a> {
    data: (),
    location: Header<'a>,
    range: Header<'a>,
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

#[patch("/v2/<name>/blobs/chunked/uploads/<session_id>", data = "<blob>")]
pub async fn patch_upload_blob<'a>(
    config: &State<Config>,
    db_pool: &State<Pool<DB>>,
    auth: Result<Auth, AuthFailure>,
    content_range: ContentRange,
    content_length: ContentLength,
    name: &str,
    session_id: &'a str,
    blob: OctetStream,
) -> UploadBlobResponse<'a> {
    check_auth!(auth, UploadBlobResponse);

    let session_id = match Uuid::from_str(session_id) {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to parse session id ({session_id}), err: {e:?}");
            return UploadBlobResponse::Failure("Invalid session ID");
        }
    };

    // if content_length.length != content_range.expected_range() {}

    if blob.data.len() != content_length.length {
        warn!(
            "Got invalid content_length value ({}) but data was ({})",
            content_length.length,
            blob.data.len()
        );
        return UploadBlobResponse::Failure("Content-length doesn't match provided data length");
    }

    let new_session = match upload_blob_service::upload_blob(
        db_pool, name, session_id, config, blob.data,
    )
    .await
    {
        Ok(session) => session,
        Err(err) => {
            // error!("Failed to upload ")
            todo!("NOT IMPLEMENTED")
        }
    };

    todo!("NOT DONE!");
}
