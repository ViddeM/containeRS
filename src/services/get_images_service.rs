use serde::{Deserialize, Serialize};
use sqlx::Pool;

use crate::{
    db::{manifest_repository, new_transaction, repository_repository, DB},
    registry_error::RegistryResult,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Repository {
    name: String,
    tags: Vec<String>,
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
            tags: manifests.into_iter().map(|manifest| manifest.tag).collect(),
        });
    }

    transaction.commit().await?;

    Ok(mapped)
}
