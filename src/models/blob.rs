use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Blob {
    pub id: Uuid,
    pub repository: String,
    pub digest: String,
    pub created_at: DateTime<Utc>,
}
