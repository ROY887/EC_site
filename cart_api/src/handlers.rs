use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::auth::Claims;
use crate::db::DbPool;
use crate::error::AppError;
use crate::models::{AddCartItem, CartItem, RemoveCartItem, UpdateCartItem};

/// 1 商品あたりの上限数量
const MAX_QUANTITY: i32 = 99;

const CART_COLUMNS: &str = "id, user_id, product_id, quantity";

fn validate_quantity(quantity: i32) -> Result<(), AppError> {
    if quantity < 1 {
        return Err(AppError::BadRequest("数量は1以上である必要があります".to_string()));
    }
    if quantity > MAX_QUANTITY {
        return Err(AppError::BadRequest(format!(
            "数量は{MAX_QUANTITY}以下にしてください"
        )));
    }
    Ok(())
}

pub async fn health_check() -> impl IntoResponse {
    "cart-api is healthy"
}

/// レディネスチェック（DB 到達性）
pub async fn db_ready_check(State(pool): State<DbPool>) -> impl IntoResponse {
    match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::error!(error = %e, "readiness check failed");
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}

/// 認証ユーザーのカートを取得
pub async fn get_cart(
    State(pool): State<DbPool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<CartItem>>, AppError> {
    let user_id = claims.user_id()?;

    let sql = format!("SELECT {CART_COLUMNS} FROM cart_items WHERE user_id = $1 ORDER BY created_at");
    let items = sqlx::query_as::<_, CartItem>(&sql)
        .bind(user_id)
        .fetch_all(&pool)
        .await?;

    Ok(Json(items))
}

/// カートに商品を追加（既にあれば数量を加算）
pub async fn add_item(
    State(pool): State<DbPool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<AddCartItem>,
) -> Result<Json<CartItem>, AppError> {
    let user_id = claims.user_id()?;
    validate_quantity(payload.quantity)?;

    // UNIQUE(user_id, product_id) を利用した upsert。
    // 加算後も上限を超えないよう LEAST で丸める。
    let sql = format!(
        "INSERT INTO cart_items (id, user_id, product_id, quantity)
         VALUES (gen_random_uuid(), $1, $2, $3)
         ON CONFLICT (user_id, product_id)
         DO UPDATE SET quantity = LEAST(cart_items.quantity + EXCLUDED.quantity, {MAX_QUANTITY})
         RETURNING {CART_COLUMNS}"
    );

    let item = sqlx::query_as::<_, CartItem>(&sql)
        .bind(user_id)
        .bind(payload.product_id)
        .bind(payload.quantity)
        .fetch_one(&pool)
        .await?;

    Ok(Json(item))
}

/// カート内の数量を更新
pub async fn update_item(
    State(pool): State<DbPool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<UpdateCartItem>,
) -> Result<Json<CartItem>, AppError> {
    let user_id = claims.user_id()?;
    validate_quantity(payload.quantity)?;

    let sql = format!(
        "UPDATE cart_items SET quantity = $1
         WHERE user_id = $2 AND product_id = $3
         RETURNING {CART_COLUMNS}"
    );

    let updated = sqlx::query_as::<_, CartItem>(&sql)
        .bind(payload.quantity)
        .bind(user_id)
        .bind(payload.product_id)
        .fetch_optional(&pool)
        .await?;

    updated
        .map(Json)
        .ok_or_else(|| AppError::NotFound("カートに該当の商品がありません".to_string()))
}

/// カートから商品を削除
pub async fn remove_item(
    State(pool): State<DbPool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<RemoveCartItem>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = claims.user_id()?;

    let result = sqlx::query("DELETE FROM cart_items WHERE user_id = $1 AND product_id = $2")
        .bind(user_id)
        .bind(payload.product_id)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("削除する商品が見つかりませんでした".to_string()));
    }

    Ok(Json(serde_json::json!({
        "message": "商品を削除しました",
        "deleted": true
    })))
}

/// カートを全削除
pub async fn clear_cart(
    State(pool): State<DbPool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = claims.user_id()?;

    let result = sqlx::query("DELETE FROM cart_items WHERE user_id = $1")
        .bind(user_id)
        .execute(&pool)
        .await?;

    Ok(Json(serde_json::json!({
        "message": "カートをクリアしました",
        "deleted_count": result.rows_affected()
    })))
}
