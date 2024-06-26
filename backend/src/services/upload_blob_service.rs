use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use sqlx::{Pool, Transaction};
use uuid::Uuid;

use crate::{
    config::Config,
    db::{
        self, blob_repository, owner_repository, repository_repository, upload_session_repository,
        DB,
    },
    models::{repository::Repository, upload_session::UploadSession},
    registry_error::{RegistryError, RegistryResult},
    types::session_id::SessionId,
};

const PG_UNIQUE_CONSTRAINT_ERROR_CODE: &str = "23505";

pub async fn create_session(
    db_pool: &Pool<DB>,
    username: &str,
    namespace: &str,
) -> RegistryResult<SessionId> {
    let mut transaction = db::new_transaction(db_pool).await?;

    let owner =
        if let Some(o) = owner_repository::find_by_username(&mut transaction, username).await? {
            o
        } else {
            owner_repository::insert(&mut transaction, username).await?
        };

    let repository =
        match repository_repository::insert(&mut transaction, &owner.id, namespace).await {
            Ok(r) => r,
            Err(RegistryError::SqlxError(err)) => {
                // Reset the transaction as the old one is cancelled.
                transaction.rollback().await?;
                transaction = db::new_transaction(db_pool).await?;
                get_repository_if_exists(err, &mut transaction, &namespace).await?
            }
            Err(e) => return Err(e),
        };

    let session =
        upload_session_repository::insert(&mut transaction, None, 0, &repository.namespace_name)
            .await?;

    transaction.commit().await?;

    Ok(session.id.into())
}

async fn get_repository_if_exists(
    err: sqlx::Error,
    transaction: &mut Transaction<'_, DB>,
    namespace: &str,
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
    namespace: &str,
    session_id: SessionId,
    config: &Config,
    blob: Vec<u8>,
    expected_start_byte: Option<usize>,
) -> RegistryResult<UploadSession> {
    let mut transaction = db::new_transaction(db_pool).await?;

    let Some(session) = upload_session_repository::find_by_repository_and_id(
        &mut transaction,
        namespace,
        session_id.clone().into(),
    )
    .await?
    else {
        return Err(RegistryError::SessionNotFound);
    };

    if session.digest.is_some() {
        return Err(RegistryError::BlobPartAlreadyUploaded);
    }

    if let Some(expected) = expected_start_byte {
        if expected != session.starting_byte_index as usize {
            warn!(
                "Received invalid range start value {expected}, expected {}",
                session.starting_byte_index
            );
            return Err(RegistryError::InvalidStartIndex);
        }
    }

    let digest = sha256::digest(blob.as_slice());

    upload_session_repository::set_digest(&mut transaction, session.id, &digest).await?;

    let Some(next_starting_byte_index) = session
        .starting_byte_index
        .checked_add_unsigned(blob.len() as u32)
    else {
        error!(
            "Failed adding together starting byte index {} with blob length {}",
            session.starting_byte_index,
            blob.len()
        );
        return Err(RegistryError::InvalidState);
    };

    let new_session = upload_session_repository::insert(
        &mut transaction,
        Some(session_id.into()),
        next_starting_byte_index,
        &session.repository,
    )
    .await?;

    save_file(config, &digest, blob)?;

    transaction.commit().await?;

    Ok(new_session)
}

pub async fn finish_blob_upload(
    db_pool: &Pool<DB>,
    config: &Config,
    namespace: &str,
    session_id: SessionId,
    digest: &str,
) -> RegistryResult<Uuid> {
    let mut transaction = db::new_transaction(db_pool).await?;

    let mut digests = vec![];

    let (previous_id, first_digest) =
        get_session(&mut transaction, &namespace, session_id.clone()).await?;
    if let Some(digest) = first_digest {
        digests.push(digest);
    }

    let mut curr_id = previous_id;
    while let Some(id) = curr_id {
        let (previous_id, digest) = get_session(&mut transaction, &namespace, id.clone()).await?;

        let Some(digest) = digest else {
            error!("Upload session with ID {:?} has digest set to none!", id);
            return Err(RegistryError::InvalidState);
        };

        digests.push(digest);
        curr_id = previous_id;
    }

    let mut data = vec![];
    for digest in digests.into_iter().rev() {
        let mut chunk = get_file_data(config, &digest)?;
        data.append(&mut chunk);
    }

    let received_digest = digest
        .strip_prefix("sha256:")
        .ok_or(RegistryError::UnsupportedDigest)?;
    let calculated_digest = sha256::digest(data.as_slice());
    if received_digest != calculated_digest {
        error!("Received digest {received_digest} does not match the calculated digest {calculated_digest}");
        return Err(RegistryError::InvalidDigest);
    }

    upload_session_repository::set_finished(&mut transaction, session_id.into(), namespace).await?;

    let prefixed_digest = format!("sha256:{}", calculated_digest);
    let blob = blob_repository::insert(&mut transaction, namespace, &prefixed_digest).await?;
    save_blob_file(config, &calculated_digest, data.as_slice()).map_err(|err| {
        error!("Failed to save combined data to blob file, err: {err:?}");
        err
    })?;

    transaction.commit().await?;

    Ok(blob.id)
}

fn get_blob_upload_dir(config: &Config) -> RegistryResult<PathBuf> {
    let file_name = format!("{}/uploads/blobs/sha256", config.storage_directory);
    let path = Path::new(&file_name);
    fs::create_dir_all(path)?;

    Ok(path.into())
}

fn get_blob_upload_filename(config: &Config, digest: &str) -> RegistryResult<PathBuf> {
    let dir = get_blob_upload_dir(config)?;
    let mut path = dir.join(digest);
    path.set_extension("tar.gz");

    Ok(path)
}

fn save_file(config: &Config, digest: &str, blob: Vec<u8>) -> RegistryResult<()> {
    let file_path = get_blob_upload_filename(config, digest)?;
    let mut file = fs::File::create(file_path)?;

    file.write_all(&blob)?;

    Ok(())
}

fn get_file_data(config: &Config, digest: &str) -> RegistryResult<Vec<u8>> {
    let file_path = get_blob_upload_filename(config, digest)?;
    let mut file = File::open(file_path)?;

    let mut buf = vec![];
    file.read_to_end(&mut buf)?;

    Ok(buf)
}

async fn get_session(
    transaction: &mut Transaction<'_, DB>,
    namespace: &str,
    session_id: SessionId,
) -> RegistryResult<(Option<SessionId>, Option<String>)> {
    let Some(session) = upload_session_repository::find_by_repository_and_id(
        transaction,
        namespace,
        session_id.into(),
    )
    .await?
    else {
        error!("Previous session not found!");
        return Err(RegistryError::InvalidState);
    };

    Ok((session.previous_session.map(|s| s.into()), session.digest))
}

fn get_blob_path_dir(config: &Config) -> PathBuf {
    Path::new(&config.storage_directory).join("blobs/sha256")
}

fn to_file_path(dir_path: PathBuf, digest: &str) -> PathBuf {
    let mut file_path = dir_path.join(digest);
    file_path.set_extension("tar.gz");
    file_path
}

pub fn get_blob_file_path(config: &Config, digest: &str) -> PathBuf {
    let dir = get_blob_path_dir(config);
    to_file_path(dir, digest)
}

fn save_blob_file(config: &Config, digest: &str, data: &[u8]) -> RegistryResult<()> {
    let dir_path = get_blob_path_dir(config);

    if !dir_path.exists() {
        info!("Blob dir path doesn't exist, creating at {dir_path:?}");
        fs::create_dir_all(dir_path.as_path())?;
    }

    let file_path = to_file_path(dir_path, digest);

    if file_path.exists() {
        info!("Blob already exists at path {file_path:?}");
        return Ok(());
    }

    info!("Saving blob to file {file_path:?}");
    let mut file = File::create_new(file_path)?;
    file.write_all(data)?;

    Ok(())
}
