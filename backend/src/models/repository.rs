use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Repository {
    pub id: Uuid,
    pub owner: Uuid,
    pub namespace_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ViewableRepository {
    pub namespace_name: String,
    pub created_at: DateTime<Utc>,
    pub username: String,
}
