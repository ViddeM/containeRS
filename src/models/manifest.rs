use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Manifest {
    pub id: Uuid,
    pub repository: String,
    pub tag: Option<String>,
    pub blob_id: Uuid,
    pub digest: String,
    pub content_type_top: String,
    pub content_type_sub: String,
    pub created_at: DateTime<Utc>,
}
