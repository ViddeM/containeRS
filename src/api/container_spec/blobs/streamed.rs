use rocket::{http::Header, State};
use sqlx::Pool;
use uuid::Uuid;

use crate::{
    api::container_spec::{
        blobs::utils::octet_stream::OctetStream, Auth, DOCKER_UPLOAD_UUID_HEADER_NAME,
        LOCATION_HEADER_NAME, RANGE_HEADER_NAME,
    },
    config::Config,
    db::DB,
    header, location,
    models::upload_session::UploadSession,
    registry_error::RegistryResult,
    services::upload_blob_service,
    types::session_id::SessionId,
};

use super::utils::content_length::ContentLength;

/// This flow doesn't seem to be covered by the specification?
/// This implementation is taken from Microsofts REST Api spec for the flow: https://learn.microsoft.com/en-us/rest/api/containerregistry/blob/start-upload?view=rest-containerregistry-2019-08-15&tabs=HTTP
///  1. a POST to create the session and receive an upload location. (Is taken care of the generic way)
///  2. a number of PATCH requests to upload the parts of the blob.
///  3. a POST to end the upload (optionally with content).

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
    #[response(status = 500)]
    Failure(&'a str),
}

#[patch("/v2/<name>/blobs/uploads/<session_id>", data = "<blob>", rank = 2)]
pub async fn patch_upload_blob<'a>(
    config: &State<Config>,
    db_pool: &State<Pool<DB>>,
    _auth: Auth,
    name: &str,
    session_id: &'a str,
    blob: OctetStream,
) -> UploadBlobResponse<'a> {
    let next_session = match upload_blob_part(db_pool, config, session_id, name, blob).await {
        Ok(session) => session,
        Err(err) => {
            warn!("Failed to upload blob part due to err {err:?}");
            return UploadBlobResponse::Failure("Failed to upload blob");
        }
    };

    UploadBlobResponse::Success(UploadBlobResponseData {
        data: (),
        location: location!(name, next_session.id),
        range: header!(
            RANGE_HEADER_NAME,
            format!("0-{}", next_session.starting_byte_index - 1)
        ),
        docker_upload_uuid: header!(DOCKER_UPLOAD_UUID_HEADER_NAME, next_session.id.to_string()),
    })
}

async fn upload_blob_part(
    db_pool: &Pool<DB>,
    config: &Config,
    session_id: &str,
    name: &str,
    blob: OctetStream,
) -> RegistryResult<UploadSession> {
    let session_id = SessionId::parse(session_id)?;

    let new_session =
        upload_blob_service::upload_blob(db_pool, name, session_id, config, blob.data, None)
            .await
            .map_err(|err| {
                error!("Failed to upload blob, err: {err:?}");
                err
            })?;

    Ok(new_session)
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
}

#[put("/v2/<name>/blobs/uploads/<session_id>?<digest>", data = "<blob>")]
pub async fn put_upload_blob<'a>(
    _auth: Auth,
    name: &str,
    session_id: &'a str,
    digest: &'a str,
    content_length: ContentLength,
    blob: Option<OctetStream>,
    config: &State<Config>,
    db_pool: &State<Pool<DB>>,
) -> FinishBlobUploadResponse<'a> {
    let blob_id = match finalize_blob_upload(
        db_pool,
        config,
        content_length,
        session_id,
        name,
        blob,
        digest,
    )
    .await
    {
        Ok(id) => id,
        Err(err) => {
            warn!("Failed to finalize blob upload due to error: {err:?}");
            return FinishBlobUploadResponse::Failure("Failed to finalize blob upload");
        }
    };

    FinishBlobUploadResponse::Success(FinishBlobUploadResponseData {
        response: "Blob upload finished",
        location: Header::new(LOCATION_HEADER_NAME, blob_id.to_string()),
    })
}

async fn finalize_blob_upload(
    db_pool: &Pool<DB>,
    config: &Config,
    content_length: ContentLength,
    session_id: &str,
    name: &str,
    blob: Option<OctetStream>,
    digest: &str,
) -> RegistryResult<Uuid> {
    let session_id = SessionId::parse(session_id)?;

    let final_session_id = if let Some(blob) = blob {
        let blob = blob.data;

        content_length.validate_blob_length(blob.len())?;

        upload_blob_service::upload_blob(db_pool, name, session_id, config, blob, None)
            .await
            .map_err(|err| {
                error!("Failed to upload final blob section, err {err:?}");
                err
            })?
            .id
            .into()
    } else {
        session_id
    };

    let blob_id = upload_blob_service::finish_blob_upload(
        db_pool,
        config,
        name.to_string(),
        final_session_id,
        digest.to_string(),
    )
    .await
    .map_err(|err| {
        error!("Failed to convert blob parts to finalized blob, err {err:?}");
        err
    })?;

    Ok(blob_id)
}
