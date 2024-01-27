use serde::{Deserialize, Serialize};
use sqlx::{
    types::chrono::{DateTime, Utc},
    Pool, Postgres, Transaction,
};

use crate::{
    db::{manifest_repository, new_transaction, repository_repository, DB},
    models::{manifest::Manifest, repository::Repository},
    registry_error::RegistryResult,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
    pub name: String,
    pub tags: Vec<Tag>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tag {
    pub tag: String,
    pub created_at: DateTime<Utc>,
}

impl From<Manifest> for Tag {
    fn from(value: Manifest) -> Self {
        Self {
            tag: value.tag,
            created_at: value.created_at,
        }
    }
}

pub async fn get_all_images(db_pool: &Pool<DB>) -> RegistryResult<Vec<Image>> {
    let mut transaction = new_transaction(db_pool).await?;

    let repositories = repository_repository::get_all(&mut transaction).await?;

    let mut mapped = vec![];

    for repository in repositories.into_iter() {
        mapped.push(map_image(&mut transaction, repository).await?);
    }

    transaction.commit().await?;

    Ok(mapped)
}

pub async fn get_image(db_pool: &Pool<DB>, name: &str) -> RegistryResult<Image> {
    let mut transaction = new_transaction(db_pool).await?;

    let repository = repository_repository::find_by_name(&mut transaction, name).await?;
    let mapped = map_image(&mut transaction, repository).await?;

    transaction.commit().await?;

    Ok(mapped)
}

async fn map_image(
    transaction: &mut Transaction<'_, Postgres>,
    repository: Repository,
) -> RegistryResult<Image> {
    let manifests =
        manifest_repository::find_all_by_repository(transaction, repository.namespace_name.clone())
            .await?;

    Ok(Image {
        name: repository.namespace_name,
        tags: manifests
            .into_iter()
            .map(|manifest| Tag {
                tag: manifest.tag,
                created_at: manifest.created_at,
            })
            .collect(),
    })
}
