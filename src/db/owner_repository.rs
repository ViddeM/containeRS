use sqlx::Transaction;

use crate::{models::owner::Owner, registry_error::RegistryResult};

use super::DB;

pub async fn insert(
    transaction: &mut Transaction<'_, DB>,
    username: &str,
) -> RegistryResult<Owner> {
    Ok(sqlx::query_as!(
        Owner,
        r#"
INSERT INTO owner(username)
VALUES           ($1)
RETURNING id, username, created_at
        "#,
        username,
    )
    .fetch_one(transaction)
    .await?)
}

pub async fn find_by_username(
    transaction: &mut Transaction<'_, DB>,
    username: &str,
) -> RegistryResult<Option<Owner>> {
    Ok(sqlx::query_as!(
        Owner,
        r#"
SELECT id, username, created_at
FROM owner
WHERE username = $1
    "#,
        username
    )
    .fetch_optional(transaction)
    .await?)
}
