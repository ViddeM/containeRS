use std::path::Path;

use rocket::fs::NamedFile;
use sqlx::Pool;

use crate::{
    config::Config,
    db::{self, blob_repository, DB},
    models::blob::Blob,
    registry_error::RegistryResult,
};

pub async fn find_blob_by_digest(
    db_pool: &Pool<DB>,
    config: &Config,
    namespace: String,
    digest: String,
) -> RegistryResult<Option<(Blob, NamedFile)>> {
    let mut transaction = db::new_transaction(db_pool).await?;

    let blob = blob_repository::find_by_repository_and_digest(&mut transaction, namespace, &digest)
        .await?;

    transaction.commit().await?;

    let blob = if let Some(b) = blob {
        b
    } else {
        return Ok(None);
    };

    let file = blob_file(config, blob.digest.clone()).await?;

    Ok(Some((blob, file)))
}

async fn blob_file(config: &Config, digest: String) -> RegistryResult<NamedFile> {
    let digest = if let Some(d) = digest.strip_prefix("sha256:") {
        d.to_string()
    } else {
        digest
    };

    let file_path = format!(
        "{}/blobs/sha256/{}.tar.gz",
        config.storage_directory, digest,
    );
    info!("Looking for blob at path {file_path}");
    let blob_path = Path::new(file_path.as_str());
    let file = NamedFile::open(blob_path).await?;

    Ok(file)
}
