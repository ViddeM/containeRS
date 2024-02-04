use rocket::{http::Header, State};
use sqlx::Pool;
use uuid::Uuid;

use crate::api::container_spec::Auth;
use crate::{
    api::container_spec::{blobs::utils::octet_stream::OctetStream, LOCATION_HEADER_NAME},
    config::Config,
    db::DB,
    registry_error::RegistryResult,
    services::upload_blob_service,
    types::session_id::SessionId,
};

use super::utils::content_length::ContentLength;

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

        content_length.validate_data_length(blob.len())?;

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

    let blob_id =
        upload_blob_service::finish_blob_upload(db_pool, config, name, final_session_id, digest)
            .await
            .map_err(|err| {
                error!("Failed to convert blob parts to finalized blob, err {err:?}");
                err
            })?;

    Ok(blob_id)
}
