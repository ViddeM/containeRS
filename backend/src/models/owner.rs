use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Owner {
    pub id: Uuid,
    pub username: String,
    pub created_at: DateTime<Utc>,
}
