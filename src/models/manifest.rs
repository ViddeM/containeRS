use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Manifest {
    pub id: Uuid,
    pub repository: String,
    pub tag: String,
    pub digest: String,
    pub created_at: DateTime<Utc>,
}