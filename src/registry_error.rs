use std::io;

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Sqlx error")]
    SqlxError(#[from] sqlx::Error),
    #[error("Rocket error")]
    RocketError(#[from] rocket::Error),
    #[error("IO Error")]
    IOError(#[from] io::Error),
    #[error("Session with was not found")]
    SessionNotFound,
    #[error("Invalid content length")]
    InvalidContentLength,
    #[error("Invalid state")]
    InvalidState,
    #[error("Unsupported digest algorithm")]
    UnsupportedDigest,
    #[error("Unsupported manifest type")]
    UnsupportedManifestType,
    #[error("Invalid digest")]
    InvalidDigest,
    #[error("Invalid manifest schema")]
    InvalidManifestSchema(String),
    #[error("Serde json error")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Invalid content range")]
    InvalidContentRange,
}

pub type RegistryResult<T> = Result<T, RegistryError>;
