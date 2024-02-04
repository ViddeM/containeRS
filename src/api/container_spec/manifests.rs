use rocket::{
    fs::NamedFile,
    http::{ContentType, Header},
    State,
};
use sqlx::Pool;
use uuid::Uuid;

use crate::{
    config::Config,
    db::DB,
    registry_error::{RegistryError, RegistryResult},
    services::{self, delete_manifest_service, get_manifest_service},
};

use super::{
    blobs::utils::content_length::ContentLength, Auth, DOCKER_CONTENT_DIGEST_HEADER_NAME,
    LOCATION_HEADER_NAME,
};

#[derive(Responder, Debug)]
pub struct GetManifestResponseData<'a> {
    file: NamedFile,
    content_type: ContentType,
    docker_digest: Header<'a>,
}

#[derive(Responder, Debug)]
pub enum GetManifestResponse<'a> {
    #[response(status = 200)]
    Success(GetManifestResponseData<'a>),
    #[response(status = 404)]
    FileNotFound(&'a str),
    #[response(status = 500)]
    Failure(&'a str),
}

#[get("/v2/<name>/manifests/<reference>")]
pub async fn get_manifest<'a>(
    name: &str,
    reference: &str,
    db_pool: &State<Pool<DB>>,
    config: &State<Config>,
) -> GetManifestResponse<'a> {
    match get_manifest_service::find_manifest(db_pool, name, reference, config).await {
        Ok(Some(manifest_info)) => {
            info!("Manifest found for {name}/{reference}");
            GetManifestResponse::Success(GetManifestResponseData {
                file: manifest_info.named_file,
                content_type: ContentType::new(
                    manifest_info.manifest.content_type_top,
                    manifest_info.manifest.content_type_sub,
                ),
                docker_digest: Header::new(
                    DOCKER_CONTENT_DIGEST_HEADER_NAME,
                    manifest_info.manifest.digest,
                ),
            })
        }
        Ok(None) => {
            warn!("Failed to find manifest {name}/{reference}");
            GetManifestResponse::FileNotFound("File not found")
        }
        Err(e) => {
            error!("Failed to get manifest, err: {e:?}");
            GetManifestResponse::Failure("An error occurred")
        }
    }
}

#[derive(Responder, Debug)]
pub struct PutManifestResponseData<'a> {
    response: &'a str,
    location: Header<'a>,
    docker_content_digest: Header<'a>,
}

#[derive(Responder, Debug)]
pub enum PutManifestResponse<'a> {
    #[response(status = 201)]
    Success(PutManifestResponseData<'a>),
    #[response(status = 400)]
    BadRequest(String),
    #[response(status = 500)]
    Failure(&'a str),
}

#[put("/v2/<name>/manifests/<reference>", data = "<data>")]
pub async fn put_manifest<'a>(
    db_pool: &State<Pool<DB>>,
    config: &State<Config>,
    _auth: Auth,
    name: &str,
    reference: &str,
    content_length: ContentLength,
    content_type: &ContentType,
    data: Vec<u8>,
) -> PutManifestResponse<'a> {
    match upload_manifest(
        db_pool,
        config,
        name,
        reference,
        content_type,
        content_length,
        data,
    )
    .await
    {
        Ok((manifest_id, digest)) => PutManifestResponse::Success(PutManifestResponseData {
            response: "Upload manifest successful",
            location: Header::new(LOCATION_HEADER_NAME, format!("/{manifest_id}")),
            docker_content_digest: Header::new(DOCKER_CONTENT_DIGEST_HEADER_NAME, digest),
        }),
        Err(e) => {
            error!("Failed to upload manifest {e:?}");
            PutManifestResponse::Failure("Failed to upload manifest")
        }
    }
}

async fn upload_manifest(
    db_pool: &Pool<DB>,
    config: &Config,
    name: &str,
    reference: &str,
    manifest_type: &ContentType,
    content_length: ContentLength,
    data: Vec<u8>,
) -> RegistryResult<(Uuid, String)> {
    content_length.validate_data_length(data.len())?;

    let (id, digest) = services::upload_manifest_service::upload_manifest(
        db_pool,
        config,
        name,
        reference,
        manifest_type,
        data,
    )
    .await?;

    Ok((id, digest))
}

#[derive(Responder)]
pub enum DeleteManifestResponse {
    #[response(status = 202)]
    Success(()),
    #[response(status = 404)]
    NotFound(()),
    #[response(status = 500)]
    Failure(()),
}

#[delete("/v2/<name>/manifests/<digest>")]
pub async fn delete_manifest(
    db_pool: &State<Pool<DB>>,
    config: &State<Config>,
    _auth: Auth,
    name: &str,
    digest: &str,
) -> DeleteManifestResponse {
    if let Err(err) = delete_manifest_service::delete_manifest(db_pool, config, name, digest).await
    {
        match err {
            RegistryError::ManifestNotFound => {
                return DeleteManifestResponse::NotFound(());
            }
            err => {
                error!("Failed to delete manifest, err: {err:?}");
                return DeleteManifestResponse::Failure(());
            }
        }
    }

    DeleteManifestResponse::Success(())
}
