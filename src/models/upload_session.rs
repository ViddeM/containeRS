use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UploadSession {
    pub id: Uuid,
    pub previous_session: Option<Uuid>,
    pub digest: Option<String>,
    pub repository: String,
    pub created_at: DateTime<Utc>,
    pub is_finished: bool,
}
