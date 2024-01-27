use serde::{Deserialize, Serialize};
use sqlx::{
    types::chrono::{DateTime, Utc},
    Pool,
};

use crate::{
    db::{manifest_repository, new_transaction, repository_repository, DB},
    models::manifest::Manifest,
    registry_error::RegistryResult,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Repository {
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

pub async fn get_all_images(db_pool: &Pool<DB>) -> RegistryResult<Vec<Repository>> {
    let mut transaction = new_transaction(db_pool).await?;

    let repositories = repository_repository::get_all(&mut transaction).await?;

    let mut mapped = vec![];

    for repository in repositories.into_iter() {
        let manifests = manifest_repository::find_all_by_repository(
            &mut transaction,
            repository.namespace_name.clone(),
        )
        .await?;
        mapped.push(Repository {
            name: repository.namespace_name,
            tags: manifests
                .into_iter()
                .map(|manifest| Tag {
                    tag: manifest.tag,
                    created_at: manifest.created_at,
                })
                .collect(),
        });
    }

    transaction.commit().await?;

    Ok(mapped)
}
