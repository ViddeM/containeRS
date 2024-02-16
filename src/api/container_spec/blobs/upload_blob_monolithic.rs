use rocket::{http::Header, State};
use sqlx::Pool;

use crate::{
    api::container_spec::{Auth, LOCATION_HEADER_NAME},
    config::Config,
    db::DB,
    header,
    registry_error::RegistryResult,
    services::upload_blob_service,
};

use super::utils::octet_stream::OctetStream;

#[derive(Responder, Debug)]
pub struct MonolithicUploadResponseData<'a> {
    response: &'a str,
    location: Header<'a>,
}

#[derive(Responder)]
pub enum MonolithicUploadResponse<'a> {
    #[response(status = 201)]
    Success(MonolithicUploadResponseData<'a>),
    #[response(status = 500)]
    Failure(&'a str),
}

#[post("/v2/<name>/blobs/uploads?<digest>", data = "<blob>")]
pub async fn post_monolithic_upload<'a>(
    db_pool: &State<Pool<DB>>,
    config: &State<Config>,
    auth: Auth,
    name: &str,
    blob: OctetStream,
    digest: &str,
) -> MonolithicUploadResponse<'a> {
    if let Err(err) = upload_blob(db_pool, config, auth, name, blob, digest).await {
        warn!("Failed to monolithicly upload blob due to error: {err:?}");
        return MonolithicUploadResponse::Failure("Failed to upload blob");
    };

    MonolithicUploadResponse::Success(MonolithicUploadResponseData {
        response: "Blob upload successful",
        location: header!(LOCATION_HEADER_NAME, format!("/v2/{name}/blobs/{digest}")),
    })
}

async fn upload_blob(
    db_pool: &Pool<DB>,
    config: &Config,
    auth: Auth,
    name: &str,
    blob: OctetStream,
    digest: &str,
) -> RegistryResult<()> {
    let session_id = upload_blob_service::create_session(db_pool, &auth.username, name)
        .await
        .map_err(|err| {
            error!("Failed to initialize monolithic blob upload, err {err:?}");
            err
        })?;
    let upload_session =
        upload_blob_service::upload_blob(db_pool, name, session_id, config, blob.data, None)
            .await
            .map_err(|err| {
                error!("Failed to monolithicly upload blob, err: {err:?}");
                err
            })?;
    let _blob_id = upload_blob_service::finish_blob_upload(
        db_pool,
        config,
        name,
        upload_session.id.into(),
        digest,
    )
    .await
    .map_err(|err| {
        error!("Failed to finalize monolithic blob upload, err: {err:?}");
        err
    })?;

    Ok(())
}
