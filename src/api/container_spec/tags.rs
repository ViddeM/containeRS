use rocket::{serde::json::Json, State};
use serde::Serialize;
use sqlx::Pool;

use crate::{db::DB, services::get_tags_service};

#[derive(Debug, Clone, Serialize)]
pub struct TagsResponseData {
    name: String,
    tags: Vec<String>,
}

#[derive(Responder)]
pub enum TagsResponse {
    #[response(status = 200)]
    Success(Json<TagsResponseData>),
    #[response(status = 500)]
    Failure(()),
}

#[get("/v2/<name>/tags/list?<n>&<last>")]
pub async fn get_tags(
    db_pool: &State<Pool<DB>>,
    name: &str,
    n: Option<usize>,
    last: Option<String>,
) -> TagsResponse {
    let tags = match get_tags_service::get_tags(db_pool, name, n, last).await {
        Ok(tags) => tags,
        Err(err) => {
            error!("Failed to retrieve tags, err: {err:?}");
            return TagsResponse::Failure(());
        }
    };

    TagsResponse::Success(Json(TagsResponseData {
        name: name.to_string(),
        tags,
    }))
}
