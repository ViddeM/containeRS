use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
use sqlx::Pool;

use crate::{
    api::INTERNAL_SERVER_ERROR,
    db::DB,
    services::get_images_service::{self, Repository},
};

use super::ErrorResponse;

#[derive(Serialize, Deserialize, Debug)]
pub struct ImagesResponse {
    repositories: Vec<Repository>,
}

#[derive(Responder, Debug)]
pub enum GetImagesResponse {
    #[response(status = 200)]
    Success(Json<ImagesResponse>),
    #[response(status = 500)]
    Failure(Json<ErrorResponse>),
}

#[get("/images")]
pub async fn get_images(db_pool: &State<Pool<DB>>) -> GetImagesResponse {
    match get_images_service::get_all_images(db_pool).await {
        Ok(repositories) => GetImagesResponse::Success(Json(ImagesResponse { repositories })),
        Err(e) => {
            error!("Failed to get images, err: {e:?}");
            GetImagesResponse::Failure(Json(ErrorResponse {
                error: INTERNAL_SERVER_ERROR.to_string(),
            }))
        }
    }
}
