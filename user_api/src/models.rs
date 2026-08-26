use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// DB 行に対応する内部表現。
/// `password_hash` は `skip_serializing` によりレスポンスへ出力されない。
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
}
