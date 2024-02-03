use rocket::{http::Header, State};
use sqlx::Pool;

use crate::{
    api::container_spec::{
        blobs::utils::{
            content_length::ContentLength, content_range::ContentRange, octet_stream::OctetStream,
        },
        Auth, RANGE_HEADER_NAME,
    },
    config::Config,
    db::DB,
    header, location,
    models::upload_session::UploadSession,
    registry_error::RegistryResult,
    services::upload_blob_service,
    types::session_id::SessionId,
};

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
    #[response(status = 500)]
    Failure(&'a str),
}

#[patch("/v2/<name>/blobs/uploads/<session_id>", data = "<blob>")]
pub async fn patch_upload_blob<'a>(
    config: &State<Config>,
    db_pool: &State<Pool<DB>>,
    _auth: Auth,
    content_range: ContentRange,
    content_length: ContentLength,
    name: &str,
    session_id: &'a str,
    blob: OctetStream,
) -> UploadBlobResponse<'a> {
    let next_session = match handle_chunked_upload(
        db_pool,
        config,
        session_id,
        blob,
        content_length,
        content_range,
        name,
    )
    .await
    {
        Ok(next_session) => next_session,
        Err(err) => {
            warn!("Failed to upload blob due to err {err:?}");
            return UploadBlobResponse::Failure("Failed to upload blob");
        }
    };

    UploadBlobResponse::Success(UploadBlobResponseData {
        data: (),
        location: location!(name, next_session.id),
        range: header!(
            RANGE_HEADER_NAME,
            format!("0-{}", next_session.starting_byte_index)
        ),
    })
}

async fn handle_chunked_upload(
    db_pool: &Pool<DB>,
    config: &Config,
    session_id: &str,
    blob: OctetStream,
    content_length: ContentLength,
    content_range: ContentRange,
    name: &str,
) -> RegistryResult<UploadSession> {
    let session_id = SessionId::parse(session_id)?;

    content_range.validate(&content_length)?;

    content_length.validate_blob_length(blob.data.len())?;

    let new_session = upload_blob_service::upload_blob(
        db_pool,
        name,
        session_id,
        config,
        blob.data,
        Some(content_range.range_start),
    )
    .await
    .map_err(|err| {
        error!("Failed to upload blob, err: {err:?}");
        err
    })?;

    Ok(new_session)
}
