use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::{Claims, create_jwt, hash_password, verify_password};
use crate::db::DbPool;
use crate::error::AppError;
use crate::models::User;

const MIN_PASSWORD_LEN: usize = 8;
const MAX_PASSWORD_LEN: usize = 128;
const MAX_USERNAME_LEN: usize = 50;
const MAX_EMAIL_LEN: usize = 254;

#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub user: User,
    pub message: String,
    pub token: String,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
}

/// 最低限のメールアドレス形式チェック。
/// 完全な RFC 準拠は目的とせず、明らかな誤入力を弾く。
fn validate_email(email: &str) -> Result<(), AppError> {
    let ok = email.len() <= MAX_EMAIL_LEN
        && email.matches('@').count() == 1
        && !email.starts_with('@')
        && !email.ends_with('@')
        && email.split('@').nth(1).is_some_and(|d| d.contains('.') && !d.starts_with('.'));

    if ok {
        Ok(())
    } else {
        Err(AppError::BadRequest(
            "メールアドレスの形式が正しくありません".to_string(),
        ))
    }
}

fn validate_password(password: &str) -> Result<(), AppError> {
    let len = password.chars().count();
    if len < MIN_PASSWORD_LEN {
        return Err(AppError::BadRequest(format!(
            "パスワードは{MIN_PASSWORD_LEN}文字以上にしてください"
        )));
    }
    if len > MAX_PASSWORD_LEN {
        return Err(AppError::BadRequest(format!(
            "パスワードは{MAX_PASSWORD_LEN}文字以内にしてください"
        )));
    }
    Ok(())
}

fn validate_username(username: &str) -> Result<(), AppError> {
    let len = username.trim().chars().count();
    if len == 0 {
        return Err(AppError::BadRequest("ユーザー名を入力してください".to_string()));
    }
    if len > MAX_USERNAME_LEN {
        return Err(AppError::BadRequest(format!(
            "ユーザー名は{MAX_USERNAME_LEN}文字以内にしてください"
        )));
    }
    Ok(())
}

/// 認証済みユーザーが対象リソースの持ち主か確認する。
fn ensure_self(claims: &Claims, user_id: Uuid) -> Result<(), AppError> {
    if claims.sub == user_id.to_string() {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "他のユーザーの情報は操作できません".to_string(),
        ))
    }
}

pub async fn health_check() -> &'static str {
    "user-api is healthy"
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

/// ユーザー登録
pub async fn create_user(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), AppError> {
    validate_username(&payload.username)?;
    validate_email(&payload.email)?;
    validate_password(&payload.password)?;

    let email = payload.email.trim().to_lowercase();

    let existing = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash FROM users WHERE email = $1",
    )
    .bind(&email)
    .fetch_optional(&pool)
    .await?;

    if existing.is_some() {
        return Err(AppError::Conflict(
            "このメールアドレスはすでに登録されています".to_string(),
        ));
    }

    let password_hash = hash_password(&payload.password)?;

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (id, username, email, password_hash)
         VALUES ($1, $2, $3, $4)
         RETURNING id, username, email, password_hash",
    )
    .bind(Uuid::new_v4())
    .bind(payload.username.trim())
    .bind(&email)
    .bind(&password_hash)
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(user)))
}

/// ログイン
pub async fn login_user(
    State(pool): State<DbPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let email = payload.email.trim().to_lowercase();

    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash FROM users WHERE email = $1",
    )
    .bind(&email)
    .fetch_optional(&pool)
    .await?;

    // ユーザーが存在しない場合とパスワード誤りで応答を変えない（列挙攻撃対策）
    let invalid = || AppError::Unauthorized("メールアドレスまたはパスワードが違います".to_string());

    let user = user.ok_or_else(invalid)?;

    if !verify_password(&payload.password, &user.password_hash)? {
        return Err(invalid());
    }

    let token = create_jwt(&user.id.to_string())
        .map_err(|e| AppError::Internal(format!("JWT の発行に失敗しました: {e}")))?;

    Ok(Json(LoginResponse {
        user,
        message: "ログインに成功しました".to_string(),
        token,
    }))
}

/// ユーザー情報取得（本人のみ）
pub async fn get_user(
    State(pool): State<DbPool>,
    Extension(claims): Extension<Claims>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<User>, AppError> {
    ensure_self(&claims, user_id)?;

    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(&pool)
    .await?;

    user.map(Json)
        .ok_or_else(|| AppError::NotFound("ユーザーが見つかりません".to_string()))
}

/// ユーザー情報更新（本人のみ）
pub async fn update_user(
    State(pool): State<DbPool>,
    Extension(claims): Extension<Claims>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<User>, AppError> {
    ensure_self(&claims, user_id)?;

    let existing = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("ユーザーが見つかりません".to_string()))?;

    let new_username = match payload.username {
        Some(u) => {
            validate_username(&u)?;
            u.trim().to_string()
        }
        None => existing.username,
    };

    let new_email = match payload.email {
        Some(e) => {
            validate_email(&e)?;
            e.trim().to_lowercase()
        }
        None => existing.email,
    };

    let updated = sqlx::query_as::<_, User>(
        "UPDATE users SET username = $1, email = $2
         WHERE id = $3
         RETURNING id, username, email, password_hash",
    )
    .bind(&new_username)
    .bind(&new_email)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db) if db.constraint().is_some() => {
            AppError::Conflict("このメールアドレスはすでに使われています".to_string())
        }
        _ => AppError::from(e),
    })?;

    Ok(Json(updated))
}

/// ユーザー削除（本人のみ）
pub async fn delete_user(
    State(pool): State<DbPool>,
    Extension(claims): Extension<Claims>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    ensure_self(&claims, user_id)?;

    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("ユーザーが見つかりません".to_string()));
    }

    Ok(Json(serde_json::json!({
        "message": "ユーザーを削除しました",
        "deleted": true
    })))
}
