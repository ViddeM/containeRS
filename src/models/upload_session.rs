use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UploadSession {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub created_at: DateTime<Utc>,
}
