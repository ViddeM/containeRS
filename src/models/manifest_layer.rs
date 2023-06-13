use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ManifestLayer {
    pub manifest_id: Uuid,
    pub blob_id: Uuid,
    pub media_type: String,
    pub size: i64,
    pub created_at: DateTime<Utc>,
}
