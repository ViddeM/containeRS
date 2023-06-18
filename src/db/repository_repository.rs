use sqlx::Transaction;

use crate::{models::repository::Repository, registry_error::RegistryResult};

use super::DB;

pub async fn insert(
    transaction: &mut Transaction<'_, DB>,
    namespace: String,
) -> RegistryResult<Repository> {
    Ok(sqlx::query_as!(
        Repository,
        r#"
INSERT INTO repository(namespace_name)
VALUES                ($1)
RETURNING id, namespace_name, created_at
        "#,
        namespace
    )
    .fetch_one(transaction)
    .await?)
}

pub async fn find_by_name(
    transaction: &mut Transaction<'_, DB>,
    namespace: String,
) -> RegistryResult<Repository> {
    Ok(sqlx::query_as!(
        Repository,
        r#"
SELECT id, namespace_name, created_at
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
SELECT id, namespace_name, created_at
FROM repository
        "#
    )
    .fetch_all(transaction)
    .await?)
}
