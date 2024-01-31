use std::str::FromStr;

use rocket::{
    http::{Header, Status},
    request::{self, FromRequest},
    Request, State,
};
use sqlx::Pool;
use uuid::Uuid;

use crate::{
    api::container_spec::{
        blobs::ContentLength, errors::UnauthorizedResponse, Auth, AuthFailure,
        CONTENT_RANGE_HEADER_NAME,
    },
    config::Config,
    db::DB,
};

use super::OctetStream;

struct ContentRange {
    range_start: usize,
    range_end: usize,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ContentRange {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let headers = req.headers();

        let Some(content_range) = headers.get_one(CONTENT_RANGE_HEADER_NAME) else {
            warn!("Missing content-range header");
            return request::Outcome::Error((
                Status::BadRequest,
                format!("Missing {CONTENT_RANGE_HEADER_NAME} header"),
            ));
        };

        let Some((start, end)) = content_range.split_once("-") else {
            warn!("Invalid content-range header: {content_range}");
            return request::Outcome::Error((
                Status::BadRequest,
                format!("Invalid {CONTENT_RANGE_HEADER_NAME} header"),
            ));
        };

        let start: usize = match start.parse() {
            Ok(v) => v,
            Err(err) => {
                warn!("Failed to parse content-range start, err: {err:?}");
                return request::Outcome::Error((
                    Status::BadRequest,
                    format!("Invalid {CONTENT_RANGE_HEADER_NAME} header"),
                ));
            }
        };

        let end: usize = match end.parse() {
            Ok(v) => v,
            Err(err) => {
                warn!("Failed to parse content-range end, err: {err:?}");
                return request::Outcome::Error((
                    Status::BadRequest,
                    format!("Invalid {CONTENT_RANGE_HEADER_NAME} header"),
                ));
            }
        };

        if end < start {
            warn!("Content-range end ({end}) is less than start ({start})?");
            return request::Outcome::Error((
                Status::BadRequest,
                format!("Invalid {CONTENT_RANGE_HEADER_NAME} header"),
            ));
        }

        request::Outcome::Success(ContentRange {
            range_start: start,
            range_end: end,
        })
    }
}

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

    if blob.data.len() != content_length.length {
        warn!(
            "Got invalid content_length value ({}) but data was ({})",
            content_length.length,
            blob.data.len()
        );
        return UploadBlobResponse::Failure("Content-length doesn't match provided data length");
    }

    /*
    let new_session = match upload_blob_service::upload_blob(db_pool, namespace, session_id, config, blob) {

    };
    */

    todo!("NOT DONE!");
}
