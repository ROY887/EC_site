use axum::{extract::Request, http::header::AUTHORIZATION, middleware::Next, response::Response};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use uuid::Uuid;

use crate::error::AppError;

/// user-apiが発行したJWTのクレーム。
/// cart-apiは検証のみ行い、発行はしない。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

impl Claims {
    /// `sub` を UUID として解釈する。
    pub fn user_id(&self) -> Result<Uuid, AppError> {
        Uuid::parse_str(&self.sub)
            .map_err(|_| AppError::Unauthorized("トークンのユーザーIDが不正です".to_string()))
    }
}

static JWT_SECRET: OnceLock<String> = OnceLock::new();

//user-api と同じ鍵を共有する必要がある(同じSecretをマウントする)。
pub fn get_jwt_secret() -> &'static str {
    JWT_SECRET.get_or_init(|| {
        let secret = std::env::var("JWT_SECRET").unwrap_or_default();
        let is_production = std::env::var("APP_ENV").as_deref() == Ok("production");

        if secret.len() < 32 {
            if is_production {
                panic!(
                    "JWT_SECRET が未設定か短すぎます（32文字以上必要）。\
                     user-api と同一の値を設定してください。"
                );
            }
            tracing::warn!("JWT_SECRET が未設定です。開発用の既定値を使用します");
            return "dev-only-insecure-secret-please-override-me".to_string();
        }
        secret
    })
}

pub fn verify_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(get_jwt_secret().as_bytes()),
        &Validation::new(Algorithm::HS256),
    )?;
    Ok(data.claims)
}

//カート系ルート全体に適用する認証を作成した。
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
