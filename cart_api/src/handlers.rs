use axum::{extract::{Path, State}, Json, response::IntoResponse};
use crate::{models::CartItem, db::DbPool};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use axum::debug_handler;

#[derive(Serialize, Debug, Deserialize, sqlx::FromRow)]
pub struct CartItemResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
}

#[derive(Deserialize)]
pub struct UpdateCartItem {
    pub user_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
}

#[derive(Deserialize)]
pub struct RemoveCartItem {
    pub user_id: Uuid,
    pub product_id: Uuid,
}

pub async fn health_check() -> &'static str {
    "cart-api is running"
}

/// カートに商品を追加
#[debug_handler]
pub async fn add_item(
    State(pool): State<DbPool>,
    Json(payload): Json<CartItem>
) -> Result<impl IntoResponse, String> {
    // 既に同じ商品がカートにある場合は数量を増やす
    let existing = sqlx::query_as::<_, CartItemResponse>(
        "SELECT id, user_id, product_id, quantity FROM cart_items 
         WHERE user_id = $1 AND product_id = $2"
    )
    .bind(&payload.user_id)
    .bind(&payload.product_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(item) = existing {
        // 既存アイテムの数量を増やす
        let updated = sqlx::query_as::<_, CartItemResponse>(
            "UPDATE cart_items SET quantity = quantity + $1 
             WHERE user_id = $2 AND product_id = $3 
             RETURNING id, user_id, product_id, quantity"
        )
        .bind(&payload.quantity)
        .bind(&payload.user_id)
        .bind(&payload.product_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
        
        Ok(Json(updated))
    } else {
        // 新規追加
        let cart = sqlx::query_as::<_, CartItemResponse>(
            "INSERT INTO cart_items (id, user_id, product_id, quantity) 
             VALUES ($1, $2, $3, $4) 
             RETURNING id, user_id, product_id, quantity"
        )
        .bind(&Uuid::new_v4())
        .bind(&payload.user_id)
        .bind(&payload.product_id)
        .bind(&payload.quantity)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
        
        Ok(Json(cart))
    }
}

/// ユーザーのカート全体を取得
#[debug_handler]
pub async fn get_cart(
    State(pool): State<DbPool>,
    Path(user_id): Path<Uuid>
) -> Result<impl IntoResponse, String> {
    let items = sqlx::query_as::<_, CartItemResponse>(
        "SELECT id, user_id, product_id, quantity FROM cart_items WHERE user_id = $1"
    )
    .bind(&user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Json(items))
}

/// カート内の商品数量を更新


#[debug_handler]
pub async fn update_item(
    State(pool): State<DbPool>,
    Json(payload): Json<UpdateCartItem>
) -> Result<impl IntoResponse, String> {
    if payload.quantity <= 0 {
        return Err("数量は1以上である必要があります".to_string());
    }

    let updated = sqlx::query_as::<_, CartItemResponse>(
        "UPDATE cart_items SET quantity = $1 
         WHERE user_id = $2 AND product_id = $3 
         RETURNING id, user_id, product_id, quantity"
    )
    .bind(&payload.quantity)
    .bind(&payload.user_id)
    .bind(&payload.product_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Json(updated))
}


/// カートから商品を削除
#[debug_handler]
pub async fn remove_item(
    State(pool): State<DbPool>,
    Json(payload): Json<RemoveCartItem>
) -> Result<impl IntoResponse, String> {
    let result = sqlx::query(
        "DELETE FROM cart_items WHERE user_id = $1 AND product_id = $2"
    )
    .bind(&payload.user_id)
    .bind(&payload.product_id)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    if result.rows_affected() == 0 {
        return Err("削除する商品が見つかりませんでした".to_string());
    }

    Ok(Json(serde_json::json!({
        "message": "商品を削除しました",
        "deleted": true
    })))
}

/// ユーザーのカートを全削除
#[debug_handler]
pub async fn clear_cart(
    State(pool): State<DbPool>,
    Path(user_id): Path<Uuid>
) -> Result<impl IntoResponse, String> {
    let result = sqlx::query(
        "DELETE FROM cart_items WHERE user_id = $1"
    )
    .bind(&user_id)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Json(serde_json::json!({
        "message": "カートをクリアしました",
        "deleted_count": result.rows_affected()
    })))
}
