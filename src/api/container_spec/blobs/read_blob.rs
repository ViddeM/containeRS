use rocket::{
    fs::NamedFile,
    http::{ContentType, Header},
    State,
};
use sqlx::Pool;

use crate::{
    api::container_spec::DOCKER_CONTENT_DIGEST_HEADER_NAME, config::Config, db::DB,
    services::get_blob_service,
};

#[derive(Responder)]
pub struct GetBlobResponseData<'a> {
    file: NamedFile,
    content_type: ContentType,
    digest: Header<'a>,
}

#[derive(Responder)]
pub enum GetBlobResponse<'a> {
    #[response(status = 200)]
    Found(GetBlobResponseData<'a>),
    #[response(status = 404)]
    NotFound(()),
    #[response(status = 500)]
    Err(String),
}

#[get("/v2/<name>/blobs/<digest>")]
pub async fn get_blob<'a>(
    name: &str,
    digest: &str,
    db_pool: &State<Pool<DB>>,
    config: &State<Config>,
) -> GetBlobResponse<'a> {
    match get_blob_service::find_blob_by_digest(
        db_pool,
        config,
        name.to_string(),
        digest.to_string(),
    )
    .await
    {
        Ok(Some((blob, file))) => {
            info!("Blob exists {}", blob.digest);
            GetBlobResponse::Found(GetBlobResponseData {
                file,
                content_type: ContentType::GZIP,
                digest: Header::new(DOCKER_CONTENT_DIGEST_HEADER_NAME, blob.digest),
            })
        }
        Ok(None) => {
            info!("Blob does not exist {digest}");
            GetBlobResponse::NotFound(())
        }
        Err(e) => {
            error!("Failed to find blob, err: {e:?}");
            GetBlobResponse::Err("Something went wrong whilst looking for blob".to_string())
        }
    }
}
