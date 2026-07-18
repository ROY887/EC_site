use jsonwebtoken::{encode,decode,Algorithm,DecodingKey,EncodingKey,Validation,Header,} ;
use serde::{Serialize, Deserialize};
use chrono::{Duration, Utc};
use sha2::{Sha256,Digest};
use std::time::{SystemTime, UNIX_EPOCH};
use std::env;
use bcrypt::{hash, DEFAULT_COST,verify};


#[derive(Debug, Serialize, Deserialize)]
pub struct Claims{
    pub sub: String, //user_id
    pub exp: usize, // 有効期限
    
}



//jwtトークンの有効期限
const JWT_EXPIRATION_HOURS: u64 = 24;



pub fn get_jwt_secret() -> String {
    std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "dev-secret-key".to_string())
}

pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, bcrypt::BcryptError> {
    verify(password, password_hash)
}

pub fn create_jwt(user_id: &str) -> Result<String, jsonwebtoken::errors::Error> {
    //トークンの有効期限を計算する
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time different")
        .as_secs() + (JWT_EXPIRATION_HOURS * 3600);

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

//jwtトークンの確認
pub fn verify_jwt(token: &str) -> Result<Claims,jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(get_jwt_secret().as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}


