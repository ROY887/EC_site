use axum::{Json, extract::{State, Path}};
use serde_json::json;
use serde::Deserialize;
use uuid::Uuid;
use crate::{models::Product, db::DbPool};
use axum::extract::Query;
use rust_decimal::Decimal;

#[derive(Deserialize)]
pub struct CreateProduct {
    pub name: String,
    pub description: Option<String>,
    pub price: Decimal,
    pub stock: i32,
    pub category: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateProduct {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<Decimal>,
    pub stock: Option<i32>,
    pub category: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

/// ヘルスチェック
pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "product-api"
    }))
}

#[debug_handler]
pub async fn db_ready_check(
    State(pool): State<DbPool>
) -> impl IntoResponse {
    match sqlx::query("SELECT 1")
    .execute(&pool)
    .await {
        Ok(_) => {
            StatusCode::OK
        }
        Err(e) => {
            eprintln!("Error log:{}",e);
            StatusCode::SERVICE_UNAVAILABLE
        }
    };

}

/// 全商品取得
#[axum::debug_handler]
pub async fn get_products(
    State(pool): State<DbPool>
) -> Result<Json<Vec<Product>>, String> {
    let products = sqlx::query_as::<_, Product>(
        "SELECT id, name, description, price, stock, category, image_url, created_at 
         FROM products 
         ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Json(products))
}

/// 商品詳細取得
#[axum::debug_handler]
pub async fn get_product(
    State(pool): State<DbPool>,
    Path(product_id): Path<Uuid>
) -> Result<Json<Product>, String> {
    let product = sqlx::query_as::<_, Product>(
        "SELECT id, name, description, price, stock, category, image_url, created_at 
         FROM products 
         WHERE id = $1"
    )
    .bind(product_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    match product {
        Some(product) => Ok(Json(product)),
        None => Err("商品が見つかりません".to_string()),
    }
}

/// 商品作成
#[axum::debug_handler]
pub async fn create_product(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateProduct>
) -> Result<Json<Product>, String> {
    if payload.price.is_sign_negative() {
        return Err("価格は0以上である必要があります".to_string());
    }
    if payload.stock < 0 {
        return Err("在庫数は0以上である必要があります".to_string());
    }

    let product = sqlx::query_as::<_, Product>(
        "INSERT INTO products (id, name, description, price, stock, category, image_url) 
         VALUES ($1, $2, $3, $4, $5, $6, $7) 
         RETURNING id, name, description, price, stock, category, image_url, created_at"
    )
    .bind(Uuid::new_v4())
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.price)
    .bind(&payload.stock)
    .bind(&payload.category)
    .bind(&payload.image_url)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Json(product))
}

/// 商品更新
#[axum::debug_handler]
pub async fn update_product(
    State(pool): State<DbPool>,
    Path(product_id): Path<Uuid>,
    Json(payload): Json<UpdateProduct>
) -> Result<Json<Product>, String> {
    // 既存商品を取得
    let existing = sqlx::query_as::<_, Product>(
        "SELECT id, name, description, price, stock, category, image_url, created_at 
         FROM products 
         WHERE id = $1"
    )
    .bind(product_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let existing = existing.ok_or("商品が見つかりません".to_string())?;

    // 更新値を決定
    let new_name = payload.name.unwrap_or(existing.name);
    let new_description = payload.description.or(existing.description);
    let new_price = payload.price.unwrap_or(existing.price);
    let new_stock = payload.stock.unwrap_or(existing.stock);
    let new_category = payload.category.or(existing.category);
    let new_image_url = payload.image_url.or(existing.image_url);

    if new_price.is_sign_negative() {
        return Err("価格は0以上である必要があります".to_string());
    }
    if new_stock < 0 {
        return Err("在庫数は0以上である必要があります".to_string());
    }

    let updated = sqlx::query_as::<_, Product>(
        "UPDATE products 
         SET name = $1, description = $2, price = $3, stock = $4, category = $5, image_url = $6 
         WHERE id = $7 
         RETURNING id, name, description, price, stock, category, image_url, created_at"
    )
    .bind(&new_name)
    .bind(&new_description)
    .bind(&new_price)
    .bind(&new_stock)
    .bind(&new_category)
    .bind(&new_image_url)
    .bind(product_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Json(updated))
}

/// 商品削除
#[axum::debug_handler]
pub async fn delete_product(
    State(pool): State<DbPool>,
    Path(product_id): Path<Uuid>
) -> Result<Json<serde_json::Value>, String> {
    let result = sqlx::query(
        "DELETE FROM products WHERE id = $1"
    )
    .bind(product_id)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    if result.rows_affected() == 0 {
        return Err("商品が見つかりません".to_string());
    }

    Ok(Json(json!({
        "message": "商品を削除しました",
        "deleted": true
    })))
}

/// カテゴリで商品検索
#[axum::debug_handler]
pub async fn get_products_by_category(
    State(pool): State<DbPool>,
    Path(category): Path<String>
) -> Result<Json<Vec<Product>>, String> {
    let products = sqlx::query_as::<_, Product>(
        "SELECT id, name, description, price, stock, category, image_url, created_at 
         FROM products 
         WHERE category = $1 
         ORDER BY created_at DESC"
    )
    .bind(&category)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Json(products))
}



/// 商品検索（キーワード）
#[axum::debug_handler]
pub async fn search_products(
    State(pool): State<DbPool>,
    Query(params): Query<SearchQuery>
) -> Result<Json<Vec<Product>>, String> {
    let search_pattern = format!("%{}%", params.q);
    
    let products = sqlx::query_as::<_, Product>(
        "SELECT id, name, description, price, stock, category, image_url, created_at 
         FROM products 
         WHERE name ILIKE $1 OR description ILIKE $1 OR category ILIKE $1 
         ORDER BY created_at DESC"
    )
    .bind(&search_pattern)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Json(products))
}

