use docker_api::{
    models::ImageBuildChunk,
    opts::{ContainerCreateOpts, ContainerFilter, ContainerListOpts, PullOpts},
    Docker,
};
use rocket::{futures::StreamExt, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use sqlx::Pool;
use uuid::Uuid;

use crate::{
    api::INTERNAL_SERVER_ERROR,
    config::Config,
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

#[derive(Serialize, Deserialize)]
pub struct RunImageRequest {
    name: String,
    tag: String,
}

#[derive(Serialize, Deserialize)]
pub struct RunImageResponseData {
    name: String,
}

#[derive(Responder)]
pub enum RunImageResponse {
    #[response(status = 200)]
    Sucess(Json<RunImageResponseData>),
    #[response(status = 500)]
    Failure(String),
}

#[post("/images", data = "<body>")]
pub async fn run_image(
    body: Json<RunImageRequest>,
    docker: &State<Docker>,
    config: &State<Config>,
) -> RunImageResponse {
    let images = docker.images();

    let image_name = format!("{}/{}:{}", config.registry_url, body.name, body.tag);

    let mut stream = images.pull(
        &PullOpts::builder()
            .image(&image_name)
            .tag(body.tag.clone())
            .build(),
    );

    while let Some(pull_result) = stream.next().await {
        match pull_result {
            Ok(ImageBuildChunk::Digest { aux }) => println!("Image pull DIGEST aux: {aux:?}"),
            Ok(ImageBuildChunk::Update { stream }) => {
                println!("Image pull UPDATE stream: {stream}")
            }
            Ok(ImageBuildChunk::PullStatus {
                status,
                id: _,
                progress: _,
                progress_detail: _,
            }) => {
                println!("Image pull PULL STATUS status: {status}")
            }
            Ok(ImageBuildChunk::Error {
                error,
                error_detail,
            }) => println!("Image pull ERROR '{error}', details: '{error_detail:?}'"),
            Err(e) => {
                eprintln!("Err: {e:?}");
                return RunImageResponse::Failure(String::from(
                    "Failed to pull image, maybe it doesn't exist?",
                ));
            }
        }
    }

    // We should now have the image locally
    let containers = docker.containers();

    let id = Uuid::new_v4(); // Random ID to keep it unique
    let container_name = format!("GAME_CONTAINER_{}_{}__{id}", body.name, body.tag);
    let container = match containers
        .create(
            &ContainerCreateOpts::builder()
                .image(&image_name)
                .name(container_name.clone())
                .build(),
        )
        .await
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to create container, err: {e:?}");
            return RunImageResponse::Failure(String::from(
                "Failed to create container from image :(",
            ));
        }
    };

    if let Err(e) = container.start().await {
        eprintln!("Failed to start container, err: {e:?}");
    }

    RunImageResponse::Sucess(Json(RunImageResponseData {
        name: container_name,
    }))
}

#[derive(Serialize, Deserialize)]
pub struct GetContainerResponseData {
    status: String,
}

#[derive(Responder)]
pub enum GetContainerResponse {
    #[response(status = 200)]
    Success(Json<GetContainerResponseData>),
    #[response(status = 404)]
    NotFound(()),
    #[response(status = 500)]
    Failure(String),
}

#[get("/images/status/<name>")]
pub async fn get_container_status(name: String, docker: &State<Docker>) -> GetContainerResponse {
    let containers = docker.containers();

    let filters = vec![ContainerFilter::Name(name)];
    let containers = match containers
        .list(
            &ContainerListOpts::builder()
                .filter(filters.into_iter())
                .build(),
        )
        .await
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to retrieve containers, err: {e:?}");
            return GetContainerResponse::Failure(String::from("Failed to retrieve containers"));
        }
    };

    let container = match containers.len() {
        0 => return GetContainerResponse::NotFound(()),
        1 => containers.first().unwrap(), // We have ensured that there is at least 1 container
        len => {
            println!("Multiple containers ({len}) found with the same name? Using the first one");
            containers.first().unwrap() // We have ensured that there is at least 1 container
        }
    };

    GetContainerResponse::Success(Json(GetContainerResponseData {
        status: container.state.clone().unwrap_or_else(|| {
            eprintln!("Failed to retrieve container state?");
            String::from("Unknown")
        }),
    }))
}
