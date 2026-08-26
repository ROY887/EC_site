use axum::{
    extract::Request,
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use bcrypt::{DEFAULT_COST, hash, verify};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// user_id
    pub sub: String,
    /// 有効期限（UNIX 秒）
    pub exp: usize,
}

/// JWT の有効期限
const JWT_EXPIRATION_HOURS: u64 = 24;

static JWT_SECRET: OnceLock<String> = OnceLock::new();

/// JWT 署名鍵を取得する。
///
/// 以前は未設定時に固定文字列へフォールバックしていたため、マニフェスト側で
/// 値が空になっていても気付けず、空の鍵で署名される危険があった。
/// 本番では起動時に必ず失敗させる。
pub fn get_jwt_secret() -> &'static str {
    JWT_SECRET.get_or_init(|| {
        let secret = std::env::var("JWT_SECRET").unwrap_or_default();
        let is_production = std::env::var("APP_ENV").as_deref() == Ok("production");

        if secret.len() < 32 {
            if is_production {
                panic!(
                    "JWT_SECRET が未設定か短すぎます（32文字以上必要）。\
                     Secret が正しくマウントされているか確認してください。"
                );
            }
            tracing::warn!(
                "JWT_SECRET が未設定です。開発用の既定値を使用します（本番では起動に失敗します）"
            );
            return "dev-only-insecure-secret-please-override-me".to_string();
        }
        secret
    })
}

pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, bcrypt::BcryptError> {
    verify(password, password_hash)
}

pub fn create_jwt(user_id: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("システム時刻が不正です")
        .as_secs()
        + (JWT_EXPIRATION_HOURS * 3600);

    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(get_jwt_secret().as_bytes()),
    )
}

pub fn verify_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(get_jwt_secret().as_bytes()),
        &Validation::new(Algorithm::HS256),
    )?;
    Ok(token_data.claims)
}

/// `Authorization: Bearer <token>` を検証し、Claims をリクエスト拡張に載せる。
///
/// これを通したルートのハンドラは `Extension<Claims>` で認証済みユーザーを取得できる。
pub async fn require_auth(mut req: Request, next: Next) -> Result<Response, AppError> {
    let token = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized("認証トークンが必要です".to_string()))?;

    let claims = verify_jwt(token)
        .map_err(|_| AppError::Unauthorized("認証トークンが無効です".to_string()))?;

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}
