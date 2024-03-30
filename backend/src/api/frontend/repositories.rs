use rocket::{serde::json::Json, State};
use serde::Serialize;
use sqlx::{
    types::chrono::{DateTime, Utc},
    Pool,
};

use crate::{
    db::DB,
    models::repository::ViewableRepository,
    services::{
        get_all_repositories_service,
        get_repository_service::{self, RepositoryInfo},
    },
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

#[derive(Responder, Debug)]
pub enum GetRepositoryResponse {
    #[response(status = 200)]
    Success(Json<GetRepositoryResponseData>),
    #[response(status = 404)]
    RepositoryNotFound(String),
    #[response(status = 500)]
    Failure(String),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRepositoryResponseData {
    name: String,
    author: String,
    tags: Vec<Tag>,
}

impl From<RepositoryInfo> for GetRepositoryResponseData {
    fn from(value: RepositoryInfo) -> Self {
        Self {
            name: value.name,
            author: value.owner_username,
            tags: value
                .tags
                .into_iter()
                .map(|manifest| Tag {
                    name: manifest.tag.unwrap_or("latest".to_string()),
                    created_at: manifest.created_at,
                })
                .collect::<Vec<Tag>>(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    name: String,
    created_at: DateTime<Utc>,
}

#[get("/repositories/<repository>")]
pub async fn get_repository(db_pool: &State<Pool<DB>>, repository: &str) -> GetRepositoryResponse {
    let repository = match get_repository_service::get_repository(db_pool, repository).await {
        Ok(data) => data,
        Err(err) => {
            // TODO: Handle not found
            warn!("Failed to retrieve repository, err: {err:?}");
            return GetRepositoryResponse::Failure("Failed to retrieve repository".to_string());
        }
    };

    return GetRepositoryResponse::Success(Json(repository.into()));
}
