use sqlx::Pool;

use crate::{
    db::{
        self,
        manifest_repository::{self},
        DB,
    },
    registry_error::RegistryResult,
};

pub async fn get_tags(
    db_pool: &Pool<DB>,
    repository: &str,
    n: Option<usize>,
    last: Option<String>,
) -> RegistryResult<Vec<String>> {
    let mut transaction = db::new_transaction(db_pool).await?;

    let manifests = match (n, last) {
        (None, None) => {
            manifest_repository::find_all_by_repository(&mut transaction, repository).await?
        }
        (None, Some(last)) => {
            manifest_repository::find_all_by_repository_last(&mut transaction, repository, &last)
                .await?
        }
        (Some(n), None) => {
            manifest_repository::find_all_by_repository_max(&mut transaction, repository, n as i64)
                .await?
        }
        (Some(n), Some(last)) => {
            manifest_repository::find_all_by_repository_last_max(
                &mut transaction,
                repository,
                &last,
                n as i64,
            )
            .await?
        }
    };

    transaction.commit().await?;

    let tags = manifests
        .into_iter()
        .map(|m| m.tag.unwrap_or(m.digest))
        .collect::<Vec<String>>();

    Ok(tags)
}
