use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// カート行のレスポンス表現
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CartItem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
}

/// カート追加リクエスト。
/// `user_id` は受け取らず JWT から導出するため、他人のカートは操作できない。
#[derive(Debug, Deserialize)]
pub struct AddCartItem {
    pub product_id: Uuid,
    pub quantity: i32,
}

/// 数量更新リクエスト
#[derive(Debug, Deserialize)]
pub struct UpdateCartItem {
    pub product_id: Uuid,
    pub quantity: i32,
}

/// 単品削除リクエスト
#[derive(Debug, Deserialize)]
pub struct RemoveCartItem {
    pub product_id: Uuid,
}
