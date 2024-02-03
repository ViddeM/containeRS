use sqlx::Pool;

use crate::{
    db::{self, upload_session_repository, DB},
    models::upload_session::UploadSession,
    registry_error::{RegistryError, RegistryResult},
    types::session_id::SessionId,
};

pub async fn retrieve_last_upload_session(
    db_pool: &Pool<DB>,
    namespace: &str,
    session_id: SessionId,
) -> RegistryResult<UploadSession> {
    let mut transaction = db::new_transaction(db_pool).await?;

    let previous = upload_session_repository::find_by_repository_and_id(
        &mut transaction,
        namespace,
        session_id.clone().into(),
    )
    .await?;

    let Some(mut previous) = previous else {
        warn!("Tried to retrieve non-existant upload session ({session_id})");
        return Err(RegistryError::SessionNotFound);
    };

    while let Some(next) =
        upload_session_repository::try_find_by_previous_id(&mut transaction, namespace, previous.id)
            .await?
    {
        previous = next;
    }

    if previous.is_finished {
        warn!("Session has already been finished ({session_id})");
        return Err(RegistryError::SessionNotFound);
    }

    if previous.digest.is_some() {
        error!(
            "Invalid state: last session in unfinished upload has been uploaded! ({session_id})"
        );
        return Err(RegistryError::InvalidState);
    }

    Ok(previous)
}
