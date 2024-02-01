use rocket::{http::Header, State};
use sqlx::Pool;

use crate::{
    api::container_spec::{
        blobs::utils::content_length::ContentLength, errors::UnauthorizedResponse, Auth,
        AuthFailure, DOCKER_UPLOAD_UUID_HEADER_NAME, RANGE_HEADER_NAME,
    },
    db::DB,
    header, location,
    services::upload_blob_service,
};

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
    db_pool: &State<Pool<DB>>,
    auth: Result<Auth, AuthFailure>,
    content_length: Option<ContentLength>,
    name: &str,
) -> CreateSessionResponse<'a> {
    let auth = match auth {
        Ok(auth) => auth,
        Err(AuthFailure::Unauthorized(resp)) => return CreateSessionResponse::Unauthorized(resp),
        Err(AuthFailure::InternalServerError(err)) => {
            error!("Unexpected auth failure {err:?}");
            return CreateSessionResponse::Failure("An unexpected error occurred");
        }
    };

    let is_chunked_flow = if let Some(length) = content_length {
        if length.length != 0 {
            return CreateSessionResponse::Failure("Content length of first request must be 0");
        }
        true
    } else {
        false
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
        location: location!(name, initial_session_id, is_chunked_flow),
        range: header!(RANGE_HEADER_NAME, "0-0"),
        docker_upload_uuid: header!(
            DOCKER_UPLOAD_UUID_HEADER_NAME,
            initial_session_id.to_string()
        ),
    })
}
