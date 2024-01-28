use sqlx::Transaction;
use uuid::Uuid;

use crate::{models::repository::Repository, registry_error::RegistryResult};

use super::DB;

pub async fn insert(
    transaction: &mut Transaction<'_, DB>,
    owner: &Uuid,
    namespace: &str,
) -> RegistryResult<Repository> {
    Ok(sqlx::query_as!(
        Repository,
        r#"
INSERT INTO repository(owner, namespace_name)
VALUES                ($1,    $2)
RETURNING id, owner, namespace_name, created_at
        "#,
        owner,
        namespace
    )
    .fetch_one(transaction)
    .await?)
}

pub async fn find_by_name(
    transaction: &mut Transaction<'_, DB>,
    namespace: &str,
) -> RegistryResult<Repository> {
    Ok(sqlx::query_as!(
        Repository,
        r#"
SELECT id, owner, namespace_name, created_at
FROM repository
WHERE namespace_name = $1
        "#,
        namespace
    )
    .fetch_one(transaction)
    .await?)
}

pub async fn get_all(transaction: &mut Transaction<'_, DB>) -> RegistryResult<Vec<Repository>> {
    Ok(sqlx::query_as!(
        Repository,
        r#"
SELECT id, owner, namespace_name, created_at
FROM repository
        "#
    )
    .fetch_all(transaction)
    .await?)
}
