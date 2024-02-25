use rocket::State;
use sqlx::Pool;

use crate::api::container_spec::Auth;
use crate::registry_error::RegistryError;
use crate::services::delete_blob_service;
use crate::{config::Config, db::DB};

#[derive(Responder)]
pub enum DeleteBlobResponse {
    #[response(status = 202)]
    Success(()),
    #[response(status = 404)]
    NotFound(()),
    #[response(status = 500)]
    Failure(()),
}

#[delete("/v2/<name>/blobs/<digest>")]
pub async fn delete_blob(
    db_pool: &State<Pool<DB>>,
    config: &State<Config>,
    _auth: Auth,
    name: &str,
    digest: &str,
) -> DeleteBlobResponse {
    if let Err(err) = delete_blob_service::delete_blob(db_pool, config, name, digest).await {
        match err {
            RegistryError::BlobNotFound => {
                warn!("Request to delete blob that could not be found {name} ({digest})");
                return DeleteBlobResponse::NotFound(());
            }
            err => {
                error!("Failed to delete blob, err: {err:?}");
                return DeleteBlobResponse::Failure(());
            }
        }
    }

    DeleteBlobResponse::Success(())
}
