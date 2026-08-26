use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{db::DbPool, error::AppError, models::Product};

/// 検索クエリの最大長。
/// `ILIKE '%q%'` はインデックスが効かないため、長い文字列を投げられると
/// 全表走査の負荷を容易にかけられる。入口で弾く。
const MAX_SEARCH_LEN: usize = 64;

const PRODUCT_COLUMNS: &str =
    "id, name, description, price, stock, category, image_url, created_at";

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

/// ヘルスチェック（プロセスが生きているか）
pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "product-api"
    }))
}

/// レディネスチェック（DB に到達できるか）
///
/// 以前は match の結果を捨てていたため常に 200 を返していた。
/// StatusCode を確実に返すよう修正済み。
pub async fn db_ready_check(State(pool): State<DbPool>) -> impl IntoResponse {
    match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::error!(error = %e, "readiness check failed");
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}

/// 全商品取得
pub async fn get_products(State(pool): State<DbPool>) -> Result<Json<Vec<Product>>, AppError> {
    let sql = format!("SELECT {PRODUCT_COLUMNS} FROM products ORDER BY created_at DESC");
    let products = sqlx::query_as::<_, Product>(&sql).fetch_all(&pool).await?;
    Ok(Json(products))
}

/// 商品詳細取得
pub async fn get_product(
    State(pool): State<DbPool>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<Product>, AppError> {
    let sql = format!("SELECT {PRODUCT_COLUMNS} FROM products WHERE id = $1");
    let product = sqlx::query_as::<_, Product>(&sql)
        .bind(product_id)
        .fetch_optional(&pool)
        .await?;

    product
        .map(Json)
        .ok_or_else(|| AppError::NotFound("商品が見つかりません".to_string()))
}

/// カテゴリで商品取得
pub async fn get_products_by_category(
    State(pool): State<DbPool>,
    Path(category): Path<String>,
) -> Result<Json<Vec<Product>>, AppError> {
    if category.len() > MAX_SEARCH_LEN {
        return Err(AppError::BadRequest("カテゴリ名が長すぎます".to_string()));
    }

    let sql = format!(
        "SELECT {PRODUCT_COLUMNS} FROM products WHERE category = $1 ORDER BY created_at DESC"
    );
    let products = sqlx::query_as::<_, Product>(&sql)
        .bind(&category)
        .fetch_all(&pool)
        .await?;

    Ok(Json(products))
}

/// 商品検索（キーワード）
pub async fn search_products(
    State(pool): State<DbPool>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<Product>>, AppError> {
    let q = params.q.trim();

    if q.is_empty() {
        return Err(AppError::BadRequest("検索キーワードを入力してください".to_string()));
    }
    if q.chars().count() > MAX_SEARCH_LEN {
        return Err(AppError::BadRequest(format!(
            "検索キーワードは{MAX_SEARCH_LEN}文字以内にしてください"
        )));
    }

    // LIKE のワイルドカードをエスケープし、パターン注入を防ぐ
    let escaped = q.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
    let pattern = format!("%{escaped}%");

    let sql = format!(
        "SELECT {PRODUCT_COLUMNS} FROM products
         WHERE name ILIKE $1 OR description ILIKE $1 OR category ILIKE $1
         ORDER BY created_at DESC
         LIMIT 100"
    );
    let products = sqlx::query_as::<_, Product>(&sql)
        .bind(&pattern)
        .fetch_all(&pool)
        .await?;

    Ok(Json(products))
}
