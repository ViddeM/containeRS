#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Sqlx error")]
    SqlxError(#[from] sqlx::Error),
    #[error("Rocket error")]
    RocketError(#[from] rocket::Error),
    #[error("Session with was not found")]
    SessionNotFound,
}

pub type RegistryResult<T> = Result<T, RegistryError>;
