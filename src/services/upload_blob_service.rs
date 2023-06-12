use std::{fs, io::Write, path::Path};

use sha256::try_digest;
use sqlx::{Pool, Transaction};
use uuid::Uuid;

use crate::{
    config::Config,
    db::{self, blob_repository, repository_repository, upload_session_repository, DB},
    models::repository::Repository,
    registry_error::{RegistryError, RegistryResult},
};

const PG_UNIQUE_CONSTRAINT_ERROR_CODE: &str = "23505";

pub async fn create_session(db_pool: &Pool<DB>, namespace: String) -> RegistryResult<Uuid> {
    let mut transaction = db::new_transaction(db_pool).await?;

    let repository = match repository_repository::insert(&mut transaction, namespace.clone()).await
    {
        Ok(r) => r,
        Err(RegistryError::SqlxError(err)) => {
            // Reset the transaction as the old one is cancelled.
            transaction.rollback().await?;
            transaction = db::new_transaction(db_pool).await?;
            get_repository_if_exists(err, &mut transaction, namespace).await?
        }
        Err(e) => return Err(e),
    };

    let session =
        upload_session_repository::insert(&mut transaction, None, repository.namespace_name)
            .await?;

    transaction.commit().await?;

    Ok(session.id)
}

async fn get_repository_if_exists(
    err: sqlx::Error,
    transaction: &mut Transaction<'_, DB>,
    namespace: String,
) -> RegistryResult<Repository> {
    if let Some(db_err) = err.as_database_error() {
        if let Some(code) = db_err.code() {
            if code.to_string().as_str() == PG_UNIQUE_CONSTRAINT_ERROR_CODE {
                // The repository already exists, let's get it!
                return Ok(repository_repository::find_by_name(transaction, namespace).await?);
            }
        }
    }

    return Err(RegistryError::SqlxError(err));
}

pub async fn upload_blob(
    db_pool: &Pool<DB>,
    namespace: String,
    session_id: Uuid,
    config: &Config,
    blob: Vec<u8>,
) -> RegistryResult<Uuid> {
    let mut transaction = db::new_transaction(db_pool).await?;

    let session = match upload_session_repository::find_by_repository_and_id(
        &mut transaction,
        namespace.clone(),
        session_id,
    )
    .await?
    {
        Some(us) => us,
        None => return Err(RegistryError::SessionNotFound),
    };

    let new_session =
        upload_session_repository::insert(&mut transaction, Some(session_id), session.repository)
            .await?;

    save_file(namespace, session_id, config, blob)?;

    transaction.commit().await?;

    Ok(new_session.id)
}

fn save_file(
    namespace: String,
    session_id: Uuid,
    config: &Config,
    blob: Vec<u8>,
) -> RegistryResult<()> {
    let path_name = format!("{}/{namespace}/blobs", config.storage_directory);
    let path = Path::new(path_name.as_str());
    fs::create_dir_all(path)?;

    let file_path_name = format!("{}.tar.gz", session_id.to_string());
    let file_path = Path::new(file_path_name.as_str());
    let mut file = fs::File::create(path.join(file_path))?;

    file.write_all(&blob)?;

    Ok(())
}
pub async fn finish_blob_upload(
    db_pool: &Pool<DB>,
    namespace: String,
    session_id: Uuid,
    digest: String,
    config: &Config,
) -> RegistryResult<Uuid> {
    let mut transaction = db::new_transaction(db_pool).await?;

    let session = upload_session_repository::find_by_repository_and_id(
        &mut transaction,
        namespace.clone(),
        session_id,
    )
    .await?
    .ok_or(RegistryError::SessionNotFound)?;

    let file_session = upload_session_repository::find_by_repository_and_id(
        &mut transaction,
        namespace.clone(),
        session.previous_session.unwrap(),
    )
    .await?
    .ok_or_else(|| {
        error!("Previous session not found!");
        RegistryError::InvalidState
    })?;

    validate_digest(namespace, file_session.id, config, digest.clone())?;

    upload_session_repository::set_finished(
        &mut transaction,
        session.id,
        session.repository.clone(),
    )
    .await?;

    let blob = blob_repository::insert(&mut transaction, session.repository, digest).await?;

    transaction.commit().await?;

    Ok(blob.id)
}

fn validate_digest(
    namespace: String,
    session_id: Uuid,
    config: &Config,
    digest: String,
) -> RegistryResult<()> {
    let digest = digest
        .strip_prefix("sha256:")
        .ok_or(RegistryError::UnsupportedDigest)?;

    let file_name = format!(
        "{}/{namespace}/blobs/{}.tar.gz",
        config.storage_directory, session_id
    );
    let file_path = Path::new(&file_name);
    let file_hash = try_digest(file_path)?;

    if digest != &file_hash {
        println!("Digest missmatch '{digest}' != '{file_hash}'");
        return Err(RegistryError::InvalidDigest);
    }

    Ok(())
}
