use rocket::{http::Header, response::status, State};
use sqlx::Pool;

use crate::{db::DB, services};

#[head("/v2/<name>/blobs/<digest>")]
pub async fn head_blobs(name: &str, digest: &str) -> status::NotFound<()> {
    status::NotFound(())
}

#[derive(Responder)]
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
            location: Header::new("location", session_id.to_string()),
        }),
        Err(e) => {
            error!("Failed to create upload session, err: {e:?}");
            CreateSessionResponse::Failure("Failed to ceate session")
        }
    }
}
