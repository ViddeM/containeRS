use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UploadSession {
    pub id: Uuid,
    pub repository: String,
    pub created_at: DateTime<Utc>,
}
