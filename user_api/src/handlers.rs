use axum::{extract::{State, Path}, Json};
use crate::db::DbPool;
use crate::models::User;
use crate::auth::{verify_jwt, hash_password,create_jwt, verify_password};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
//  use argon2::{
//    password_hash::{rand_core::OsRng, SaltString, PasswordHasher},
//    Argon2,
// };
use axum::{response::IntoResponse,http::StatusCode};


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
    token: Option<String>, // jwtトークン
}



#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
}

// ヘルスチェック
pub async fn health_check() -> &'static str {
    "user-api is healthy"
}

// k8sのlifycycleのためのエンドポイント
#[axum::debug_handler]
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

// ユーザー登録
#[axum::debug_handler]
pub async fn create_user(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateUser> 
) -> Result<Json<User>, String> {
    //パスワードのハッシュ化
    let hash = hash_password(&payload.password)
        .map_err(|e| e.to_string())?;
    // メールアドレスの重複チェック
    let existing = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash FROM users WHERE email = $1"
    )
    .bind(&payload.email)
    .bind(&hash)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;
    
    if existing.is_some() {
        return Err("このemailはすでに登録されています".to_string());
    }

    // ユーザー作成
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (id, username, email, password_hash) 
         VALUES ($1, $2, $3, $4) 
         RETURNING id, username, email, password_hash"
    )
    .bind(Uuid::new_v4())
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&hash)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Json(user))
}

// ログイン
#[axum::debug_handler]
pub async fn login_user(
    State(pool): State<DbPool>,
    Json(payload): Json<LoginRequest> 
) -> Result<Json<LoginResponse>, String> {
   
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash 
         FROM users 
         WHERE email = $1"
    )

    .bind(&payload.email)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    match user {
        Some(user) => {
            // パスワード検証
            let confirm = verify_password(&payload.password,&user.password_hash)
                    .map_err(|e| e.to_string())?;
            // jwt発行
            if confirm {
                let token =  create_jwt(&user.id.to_string())
                    .map_err(|e| e.to_string())?;
             
            Ok(Json(LoginResponse {
                user,
                message: "ログインに成功しました".to_string(),
                token: Some(token),
            }))

            } else {
                Err("メールアドレスまたはパスワードが違います".to_string())
            }
    }
    None => {
        Err("メールアドレスまたはパスワードを入力してください".to_string())
        }
    }   
}
// ユーザー情報更新
#[axum::debug_handler]
pub async fn update_user(
    State(pool): State<DbPool>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>
) -> Result<Json<User>, String> {
    // 既存ユーザーを取得
    let existing = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let existing = existing.ok_or("ユーザーが見つかりません".to_string())?;
    
    // 更新値を決定（Noneの場合は既存値を使用）
    let new_username = payload.username.unwrap_or(existing.username);
    let new_email = payload.email.unwrap_or(existing.email);

    // ユーザー情報を更新
    let updated = sqlx::query_as::<_, User>(
        "UPDATE users SET username = $1, email = $2 
         WHERE id = $3 
         RETURNING id, username, email, password_hash"
    )
    .bind(&new_username)
    .bind(&new_email)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Json(updated))
}

// ユーザー削除
#[axum::debug_handler]
pub async fn delete_user(
    State(pool): State<DbPool>,
    Path(user_id): Path<Uuid>
) -> Result<Json<serde_json::Value>, String> {
    let result = sqlx::query(
        "DELETE FROM users WHERE id = $1"
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    if result.rows_affected() == 0 {
        return Err("ユーザーが見つかりません".to_string());
    }

    Ok(Json(serde_json::json!({
        "message": "ユーザーを削除しました",
        "deleted": true
    })))
}

// ユーザー情報取得
#[axum::debug_handler]
pub async fn get_user(
    State(pool): State<DbPool>,
    Path(user_id): Path<Uuid>
) -> Result<Json<User>, String> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    match user {
        Some(user) => Ok(Json(user)),
        None => Err("ユーザーが見つかりません".to_string()),
    }
}

// 全ユーザー取得
#[axum::debug_handler]
pub async fn get_all_user(
    State(pool): State<DbPool>
) -> Result<Json<Vec<User>>, String> {
    let users = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash FROM users"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Json(users))


}
