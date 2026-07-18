use serde::{Deserialize,Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize,Deserialize, sqlx::FromRow)]
pub struct User{
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

