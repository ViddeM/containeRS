use sqlx::Transaction;

use crate::{models::manifest::Manifest, registry_error::RegistryResult};

use super::DB;

pub async fn insert(
    transaction: &mut Transaction<'_, DB>,
    repository: String,
    tag: String,
    digest: String,
) -> RegistryResult<Manifest> {
    Ok(sqlx::query_as!(
        Manifest,
        r#"
INSERT INTO manifest(repository, tag, digest)
VALUES              ($1,         $2,  $3)
RETURNING id, repository, tag, digest, created_at
        "#,
        repository,
        tag,
        digest
    )
    .fetch_one(transaction)
    .await?)
}
