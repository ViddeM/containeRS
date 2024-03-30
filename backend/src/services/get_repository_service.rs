use sqlx::Pool;

use crate::{
    db::{self, manifest_repository, owner_repository, repository_repository, DB},
    models::manifest::Manifest,
    registry_error::RegistryResult,
};

pub struct RepositoryInfo {
    pub name: String,
    pub owner_username: String,
    pub tags: Vec<Manifest>,
}

pub async fn get_repository(db_pool: &Pool<DB>, name: &str) -> RegistryResult<RepositoryInfo> {
    let mut transaction = db::new_transaction(db_pool).await?;

    let repository = repository_repository::find_by_name(&mut transaction, name).await?;
    let owner = owner_repository::find_by_id(&mut transaction, repository.owner).await?;

    let tags =
        manifest_repository::find_all_by_repository(&mut transaction, &repository.namespace_name)
            .await?;

    transaction.commit().await?;

    Ok(RepositoryInfo {
        name: repository.namespace_name,
        owner_username: owner.username,
        tags,
    })
}
