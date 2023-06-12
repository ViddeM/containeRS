use std::str::FromStr;

use rocket::{http::Header, response::status, State};
use sqlx::Pool;
use uuid::Uuid;

use crate::{config::Config, db::DB, services};

const CONTENT_TYPE_HEADER_NAME: &str = "Content-Type";
const CONTENT_RANGE_HEADER_NAME: &str = "Content-Range";
const CONTENT_LENGTH_HEADER_NAME: &str = "Content-Length";
const LOCATION_HEADER_NAME: &str = "Location";
const RANGE_HEADER_NAME: &str = "Range";

#[head("/v2/<name>/blobs/<digest>")]
pub async fn head_blobs(name: &str, digest: &str) -> status::NotFound<()> {
    status::NotFound(())
}

#[derive(Responder, Debug)]
pub struct CreateSessionResponseData<'a> {
    response: &'a str,
    location: Header<'a>,
}

#[derive(Responder)]
pub enum CreateSessionResponse<'a> {
    #[response(status = 202)]
    Success(CreateSessionResponseData<'a>),
    #[response(status = 500)]
    Failure(&'a str),
}

#[post("/v2/<name>/blobs/uploads")]
pub async fn post_create_session<'a>(
    name: &str,
    db_pool: &State<Pool<DB>>,
) -> CreateSessionResponse<'a> {
    match services::upload_blob_service::create_session(db_pool, name.to_string()).await {
        Ok(session_id) => CreateSessionResponse::Success(CreateSessionResponseData {
            response: "Session created successfully",
            location: Header::new(LOCATION_HEADER_NAME, session_id.to_string()),
        }),
        Err(e) => {
            error!("Failed to create upload session, err: {e:?}");
            CreateSessionResponse::Failure("Failed to ceate session")
        }
    }
}

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
}

#[patch("/v2/<name>/blobs/uploads/<session_id>", data = "<blob>")]
pub async fn patch_upload_blob<'a>(
    name: &str,
    session_id: &'a str,
    blob: Vec<u8>,
    // The docker daemon appears to have skipped implementing these headers? Ignore for now.
    // headers: UploadBlobRequestHeaders,
    db_pool: &State<Pool<DB>>,
    config: &State<Config>,
) -> UploadBlobResponse<'a> {
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
                response: "It went well?",
                location: Header::new(LOCATION_HEADER_NAME, new_session_id.to_string()),
                range: Header::new(RANGE_HEADER_NAME, format!("0-{}", blob_length)),
            })
        }
        Err(e) => {
            error!("Failed to upload blob, err: {e:?}");
            return UploadBlobResponse::Failure("Failed to upload blob");
        }
    }
}

#[derive(Responder, Debug)]
pub enum FinishBlobUploadResponse<'a> {
    #[response(status = 500)]
    Failure(&'a str),
    #[response(status = 400)]
    InvalidSessionId(&'a str),
}

#[put("/v2/<name>/blobs/uploads/<session_id>?<digest>")]
pub async fn put_upload_blob<'a>(
    name: &str,
    session_id: &'a str,
    digest: &'a str,
    db_pool: &State<Pool<DB>>,
    config: &State<Config>,
) -> FinishBlobUploadResponse<'a> {
    println!("Finish uploading blob {name} | {session_id} | {digest}");
    let session_id = match Uuid::from_str(session_id) {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to parse session id ({session_id}), err: {e:?}");
            return FinishBlobUploadResponse::InvalidSessionId(session_id);
        }
    };

    todo!("NOT DONE");

    FinishBlobUploadResponse::Failure("ASD")
}

// TODO: This does not appear to be supported by the current docker implementation.
// #[derive(Debug, Clone)]
// pub struct UploadBlobRequestHeaders {
//     content_type: String,
//     content_start: usize,
//     content_end: usize,
//     content_length: usize,
// }

// impl UploadBlobRequestHeaders {
//     fn retrieve<'r>(request: &'r Request<'_>) -> Result<Self, String> {
//         request.headers().iter().for_each(|h| {
//             println!("\tGot header {h:?}");
//         });

//         let content_type = request
//             .headers()
//             .get_one(CONTENT_TYPE_HEADER_NAME)
//             .ok_or(format!("Missing header {CONTENT_TYPE_HEADER_NAME}"))?;

//         let content_range = request
//             .headers()
//             .get_one(CONTENT_TYPE_HEADER_NAME)
//             .ok_or(format!("Missing header {CONTENT_RANGE_HEADER_NAME}"))?;

//         let content_length = request
//             .headers()
//             .get_one(CONTENT_TYPE_HEADER_NAME)
//             .ok_or(format!("Missing header {CONTENT_LENGTH_HEADER_NAME}"))?;

//         let split: Vec<&str> = content_range.split("-").collect();
//         if split.len() != 2 {
//             return Err(format!(
//                 "Invalid format for header {CONTENT_RANGE_HEADER_NAME}"
//             ));
//         }

//         Ok(Self {
//             content_type: content_type.to_string(),
//             content_start: split[0].parse().or(Err(format!(
//                 "Invalid header format {CONTENT_RANGE_HEADER_NAME}"
//             )))?,
//             content_end: split[1].parse().or(Err(format!(
//                 "Invalid header format {CONTENT_RANGE_HEADER_NAME}"
//             )))?,
//             content_length: content_length.parse().or(Err(format!(
//                 "Invalid header format {CONTENT_LENGTH_HEADER_NAME}"
//             )))?,
//         })
//     }
// }

// #[rocket::async_trait]
// impl<'r> FromRequest<'r> for UploadBlobRequestHeaders {
//     type Error = String;

//     async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
//         match Self::retrieve(request) {
//             Ok(v) => request::Outcome::Success(v),
//             Err(e) => request::Outcome::Failure((Status::BadRequest, e)),
//         }
//     }
// }
