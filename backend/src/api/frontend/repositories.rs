use rocket::{serde::json::Json, State};
use serde::Serialize;
use sqlx::{
    types::chrono::{DateTime, Utc},
    Pool,
};

use crate::{
    db::DB, models::repository::ViewableRepository, services::get_all_repositories_service,
};

#[derive(Responder, Debug)]
pub enum GetRepositoriesResponse {
    #[response(status = 200)]
    Success(Json<GetRepositoriesResponseData>),
    #[response(status = 500)]
    Failure(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct GetRepositoriesResponseData {
    repositories: Vec<Repository>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
    name: String,
    author: String,
    last_modified: DateTime<Utc>,
}

impl From<ViewableRepository> for Repository {
    fn from(value: ViewableRepository) -> Self {
        Self {
            name: value.namespace_name,
            author: value.username,
            last_modified: value.created_at,
        }
    }
}

#[get("/repositories")]
pub async fn get_all_repositories(db_pool: &State<Pool<DB>>) -> GetRepositoriesResponse {
    let repos = match get_all_repositories_service::get_all_repositories(db_pool).await {
        Ok(repos) => repos,
        Err(err) => {
            error!("Failed to retrieve all repositories, err: {err:?}");
            return GetRepositoriesResponse::Failure("Failed to retrieve repositories".to_string());
        }
    };

    GetRepositoriesResponse::Success(Json(GetRepositoriesResponseData {
        repositories: repos
            .into_iter()
            .map(|r| r.into())
            .collect::<Vec<Repository>>(),
    }))
}
