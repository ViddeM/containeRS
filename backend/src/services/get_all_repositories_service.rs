use sqlx::Pool;

use crate::{
    db::{self, repository_repository, DB},
    models::repository::ViewableRepository,
    registry_error::RegistryResult,
};

pub async fn get_all_repositories(db_pool: &Pool<DB>) -> RegistryResult<Vec<ViewableRepository>> {
    let mut transaction = db::new_transaction(db_pool).await?;

    let repositories = repository_repository::find_all_with_owners(&mut transaction).await?;

    transaction.commit().await?;

    Ok(repositories)
}
