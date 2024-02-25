use rocket::{http::Header, State};
use sqlx::Pool;

use crate::api::container_spec::{Auth, DOCKER_UPLOAD_UUID_HEADER_NAME};
use crate::range;
use crate::registry_error::RegistryError;
use crate::{
    config::Config, db::DB, header, location, models::upload_session::UploadSession,
    registry_error::RegistryResult, services::upload_blob_service, types::session_id::SessionId,
};

use super::utils::{
    content_length::ContentLength, content_range::ContentRange, octet_stream::OctetStream,
};

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
    #[response(status = 416)]
    OutOfOrder(()),
    #[response(status = 416)]
    AlreadyUploaded(()),
    #[response(status = 500)]
    Failure(&'a str),
}

#[patch("/v2/<name>/blobs/uploads/<session_id>", data = "<blob>")]
pub async fn patch_upload_blob<'a>(
    db_pool: &State<Pool<DB>>,
    config: &State<Config>,
    _auth: Auth,
    content_length: ContentLength,
    content_range: Option<ContentRange>,
    name: &str,
    session_id: &str,
    blob: OctetStream,
) -> UploadBlobResponse<'a> {
    let next_session = match handle_chunked_upload(
        db_pool,
        config,
        session_id,
        name,
        blob,
        content_length,
        content_range,
    )
    .await
    {
        Ok(next_session) => next_session,
        Err(RegistryError::BlobPartAlreadyUploaded) => {
            warn!("The request blob part has already been uploaeded!");
            return UploadBlobResponse::AlreadyUploaded(());
        }
        Err(RegistryError::InvalidStartIndex) => {
            warn!("Received invalid start index of content range");
            return UploadBlobResponse::OutOfOrder(());
        }
        Err(err) => {
            warn!("Failed to upload blob due to err {err:?}");
            return UploadBlobResponse::Failure("Failed to upload blob");
        }
    };

    UploadBlobResponse::Success(UploadBlobResponseData {
        data: (),
        location: location!(name, next_session.id),
        range: range!(next_session),
        docker_upload_uuid: header!(DOCKER_UPLOAD_UUID_HEADER_NAME, next_session.id.to_string()),
    })
}

async fn handle_chunked_upload(
    db_pool: &Pool<DB>,
    config: &Config,
    session_id: &str,
    name: &str,
    blob: OctetStream,
    content_length: ContentLength,
    content_range: Option<ContentRange>,
) -> RegistryResult<UploadSession> {
    let session_id = SessionId::parse(session_id)?;

    if let Some(content_range) = content_range.as_ref() {
        content_range.validate(&content_length)?;
    }

    content_length.validate_data_length(blob.data.len())?;

    let new_session = upload_blob_service::upload_blob(
        db_pool,
        name,
        session_id,
        config,
        blob.data,
        content_range.map(|o| o.range_start),
    )
    .await
    .map_err(|err| {
        error!("Failed to upload blob, err: {err:?}");
        err
    })?;

    Ok(new_session)
}
