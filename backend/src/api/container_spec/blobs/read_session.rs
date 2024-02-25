use rocket::{http::Header, State};
use sqlx::Pool;

use crate::api::container_spec::Auth;
use crate::db::DB;
use crate::models::upload_session::UploadSession;
use crate::registry_error::RegistryResult;
use crate::services::get_upload_session_service;
use crate::types::session_id::SessionId;
use crate::{header, location, range};

#[derive(Responder, Debug)]
pub struct GetUploadSessionResponseData<'a> {
    inner: (),
    location: Header<'a>,
    range: Header<'a>,
}

#[derive(Responder)]
pub enum GetUploadSessionResponse<'a> {
    #[response(status = 204)]
    Success(GetUploadSessionResponseData<'a>),
    #[response(status = 500)]
    Failure(&'a str),
}

#[get("/v2/<name>/blobs/uploads/<session_id>")]
pub async fn get_upload_session<'a>(
    db_pool: &State<Pool<DB>>,
    _auth: Auth,
    name: &str,
    session_id: &str,
) -> GetUploadSessionResponse<'a> {
    let latest_session = match handle_get_upload_session(db_pool, name, session_id).await {
        Ok(v) => v,
        Err(err) => {
            warn!("Failed to retrieve upload session, due to err: {err:?}");
            return GetUploadSessionResponse::Failure("Failed to retrieve upload session");
        }
    };

    GetUploadSessionResponse::Success(GetUploadSessionResponseData {
        inner: (),
        location: location!(name, latest_session.id),
        range: range!(latest_session),
    })
}

async fn handle_get_upload_session(
    db_pool: &Pool<DB>,
    name: &str,
    session_id: &str,
) -> RegistryResult<UploadSession> {
    let session_id = SessionId::parse(session_id)?;

    let session =
        get_upload_session_service::retrieve_last_upload_session(db_pool, name, session_id).await?;

    Ok(session)
}
