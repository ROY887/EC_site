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

// 最低限のメールアドレス形式チェック。
// 完全なRFC準拠は目的とせず、明らかな誤入力を弾く。
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




// 認証済みユーザーが対象リソースの持ち主か確認する。
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

// レディネスチェック（DB到達性）
pub async fn db_ready_check(State(pool): State<DbPool>) -> impl IntoResponse {
    match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::error!(error = %e, "readiness check failed");
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}

// ユーザー登録
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

// ログイン
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

// ユーザー情報取得（本人のみ）
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

// ユーザー情報更新（本人のみ）
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

// ユーザー削除（本人のみ）
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

// 実行方法:
//   make up   # docker compose の postgres を起動しておく
//   DATABASE_URL=postgres://ecuser:devpassword@localhost:5432/ec_db cargo test

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

   

    // テスト用ユーザーを1人作る。
    async fn setup_user(pool: &PgPool, username: &str, email: &str, password: &str) -> User {
        let (_, Json(user)) = create_user(
            State(pool.clone()),
            Json(CreateUser {
                username: username.to_string(),
                email: email.to_string(),
                password: password.to_string(),
            }),
        )
        .await
        .expect("テスト用ユーザーの作成に失敗しました");
        user
    }

    // 指定ユーザーとして認証済みの Claims を作る。
    // exp はミドルウェアを通さないため検証されない。
    fn claims_for(user_id: Uuid) -> Claims {
        Claims {
            sub: user_id.to_string(),
            exp: 0,
        }
    }

    // 失敗を期待する呼び出しから AppError を取り出す。
    // LoginResponse など Debug を実装していない型があり unwrap_err が使えないため。
    fn expect_err<T>(result: Result<T, AppError>) -> AppError {
        match result {
            Err(e) => e,
            Ok(_) => panic!("エラーを期待しましたが成功しました"),
        }
    }

    // validate_password test

    #[test]
    fn short_password_check() {
        assert!(matches!(
            validate_password(&"a".repeat(7)),
            Err(AppError::BadRequest(_))
        ));
    }

    // 境界値テスト
    #[test]
    fn just_password_check() {
        assert!(validate_password(&"a".repeat(8)).is_ok());
        assert!(validate_password(&"a".repeat(128)).is_ok());
    }

    #[test]
    fn excessive_password_check() {
        assert!(matches!(
            validate_password(&"a".repeat(129)),
            Err(AppError::BadRequest(_))
        ));
    }

    // validate_email test

    #[test]
    fn err_email_check() {
        for bad in ["@aa.com", "user@", "aaa@@aa.com", "user", "a@a", "a@.com"] {
            assert!(validate_email(bad).is_err(), "{bad}が通ってしまった");
        }
    }

    #[test]
    fn ok_email_check() {
        for ok in ["example@example.com", "a.a+a@sudo.co.jp"] {
            assert!(validate_email(ok).is_ok(), "{ok}");
        }
    }

    // validate_username test

    #[test]
    fn err_username_check() {
        let too_long = "a".repeat(MAX_USERNAME_LEN + 1);
        for bad in ["", "   ", too_long.as_str()] {
            assert!(
                matches!(validate_username(bad), Err(AppError::BadRequest(_))),
                "{bad:?}が通ってしまった"
            );
        }
    }

    #[test]
    fn ok_username_check() {
        assert!(validate_username("a").is_ok());
        assert!(validate_username(&"a".repeat(MAX_USERNAME_LEN)).is_ok());
    }

    // ensure_self test
    #[test]
    fn ensure_self_allows_owner() {
        let id = Uuid::new_v4();
        assert!(ensure_self(&claims_for(id), id).is_ok());
    }

    #[test]
    fn ensure_self_rejects_other_user() {
        let me = Uuid::new_v4();
        let other = Uuid::new_v4();
        assert!(matches!(
            ensure_self(&claims_for(me), other),
            Err(AppError::Forbidden(_))
        ));
    }

    #[test]
    fn ensure_self_rejects_malformed_sub() {
        // sub が UUID として不正でも素通りしないこと
        let claims = Claims {
            sub: "not-uuid".to_string(),
            exp: 0,
        };
        assert!(matches!(
            ensure_self(&claims, Uuid::new_v4()),
            Err(AppError::Forbidden(_))
        ));
    }

    // create_user

    #[sqlx::test]
    async fn create_user_returns_created(pool: PgPool) {
        let (status, Json(user)) = create_user(
            State(pool),
            Json(CreateUser {
                username: "テスト".to_string(),
                email: "new@example.com".to_string(),
                password: "password123".to_string(),
            }),
        )
        .await
        .unwrap();

        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(user.email, "new@example.com");
    }

    // models.rs の #[serde(skip_serializing)] を守る。
    // あの 1 行を消してもコンパイルは通るため、テストでしか検出できない。
    #[sqlx::test]
    async fn create_user_response_hides_password_hash(pool: PgPool) {
        let user = setup_user(&pool, "テスト太郎", "hide@example.com", "password123").await;
        let json = serde_json::to_string(&user).unwrap();

        assert!(
            !json.contains("password_hash"),
            "レスポンスに password_hash が含まれている: {json}"
        );
        assert!(
            !json.contains("$2b$"),
            "bcrypt ハッシュが漏れている: {json}"
        );
    }

    #[sqlx::test]
    async fn create_user_rejects_duplicate_email(pool: PgPool) {
        setup_user(&pool, "先客", "dup@example.com", "password123").await;

        let err = expect_err(
            create_user(
                State(pool),
                Json(CreateUser {
                    username: "後".to_string(),
                    email: "dup@example.com".to_string(),
                    password: "password456".to_string(),
                }),
            )
            .await,
        );

        assert!(matches!(err, AppError::Conflict(_)));
    }

    

    // 大文字小文字を区別すると同一人物が二重登録できてしまう。
    #[sqlx::test]
    async fn create_user_treats_email_case_insensitively(pool: PgPool) {
        setup_user(&pool, "先客", "DUP@Example.COM", "password123").await;

        let err = expect_err(
            create_user(
                State(pool),
                Json(CreateUser {
                    username: "あとから".to_string(),
                    email: "dup@example.com".to_string(),
                    password: "password456".to_string(),
                }),
            )
            .await,
        );

        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[sqlx::test]
    async fn create_user_trims_username(pool: PgPool) {
        let user = setup_user(&pool, "  空白あり  ", "trim@example.com", "password123").await;
        assert_eq!(user.username, "空白あり");
    }

    // パスワードが平文で保存されていないこと。
    #[sqlx::test]
    async fn create_user_stores_hashed_password(pool: PgPool) {
        setup_user(&pool, "テスト太郎", "hash@example.com", "password123").await;

        let stored: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE email = $1")
            .bind("hash@example.com")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_ne!(stored, "password123", "パスワードが平文で保存されている");
        assert!(verify_password("password123", &stored).unwrap());
    }

    // login_user

    #[sqlx::test]
    async fn login_returns_token_for_the_user(pool: PgPool) {
        let user = setup_user(&pool, "テスト太郎", "login@example.com", "password123").await;

        let Json(res) = login_user(
            State(pool),
            Json(LoginRequest {
                email: "login@example.com".to_string(),
                password: "password123".to_string(),
            }),
        )
        .await
        .unwrap();

        let claims = crate::auth::verify_jwt(&res.token).expect("発行したトークンが検証できない");
        assert_eq!(claims.sub, user.id.to_string());
    }

    // 列挙攻撃対策。「ユーザーが存在しない」と「パスワードが違う」で
    // 応答が変わると、メールアドレスの登録有無を外部から判定できてしまう。
    // handlers.rs のコメントで宣言されているだけの約束なので、ここで固定する。
    #[sqlx::test]
    async fn login_does_not_reveal_whether_email_exists(pool: PgPool) {
        setup_user(&pool, "テスト", "test@example.com", "password123").await;

        let wrong_password = expect_err(
            login_user(
                State(pool.clone()),
                Json(LoginRequest {
                    email: "exists@example.com".to_string(),
                    password: "wrongpassword".to_string(),
                }),
            )
            .await,
        );

        let no_such_user = expect_err(
            login_user(
                State(pool),
                Json(LoginRequest {
                    email: "missing@example.com".to_string(),
                    password: "password123".to_string(),
                }),
            )
            .await,
        );

        let message = |e: &AppError| match e {
            AppError::Unauthorized(m) => m.clone(),
            other => panic!("Unauthorized を期待しましたが {other:?} でした"),
        };

        assert_eq!(
            message(&wrong_password),
            message(&no_such_user),
            "エラー内容が異なるとメールアドレスの存在を判別できてしまう"
        );
    }

    #[sqlx::test]
    async fn login_normalizes_email(pool: PgPool) {
        setup_user(&pool, "テスト", "norm@example.com", "password123").await;

        let result = login_user(
            State(pool),
            Json(LoginRequest {
                email: "  NORM@Example.COM  ".to_string(),
                password: "password123".to_string(),
            }),
        )
        .await;

        assert!(result.is_ok(), "大文字と前後の空白を正規化できていない");
    }

    #[sqlx::test]
    async fn login_response_hides_password_hash(pool: PgPool) {
        setup_user(&pool, "テスト", "login@example.com", "password123").await;

        let Json(res) = login_user(
            State(pool),
            Json(LoginRequest {
                email: "login@example.com".to_string(),
                password: "password123".to_string(),
            }),
        )
        .await
        .unwrap();

        let json = serde_json::to_string(&res).unwrap();
        assert!(
            !json.contains("password_hash") && !json.contains("$2b$"),
            "ログイン応答にハッシュが漏れている: {json}"
        );
    }

    // get_user

    #[sqlx::test]
    async fn get_user_returns_own_record(pool: PgPool) {
        let user = setup_user(&pool, "テスト太郎", "get@example.com", "password123").await;

        let Json(fetched) = get_user(
            State(pool),
            Extension(claims_for(user.id)),
            Path(user.id),
        )
        .await
        .unwrap();

        assert_eq!(fetched.id, user.id);
        assert_eq!(fetched.email, "get@example.com");
    }

    // ユーザーを 1 人も作らずに実行する。
    // Forbidden が返れば、DB に到達する前に ensure_self が弾いている。
    #[sqlx::test]
    async fn get_user_rejects_other_users_id(pool: PgPool) {
        let err = expect_err(
            get_user(
                State(pool),
                Extension(claims_for(Uuid::new_v4())),
                Path(Uuid::new_v4()),
            )
            .await,
        );

        assert!(matches!(err, AppError::Forbidden(_)));
    }

    #[sqlx::test]
    async fn get_user_returns_not_found_for_missing_record(pool: PgPool) {
        let id = Uuid::new_v4();

        let err = expect_err(get_user(State(pool), Extension(claims_for(id)), Path(id)).await);

        assert!(matches!(err, AppError::NotFound(_)));
    }

    // update_user

    #[sqlx::test]
    async fn update_user_changes_only_username(pool: PgPool) {
        let user = setup_user(&pool, "旧ユーザー名", "keep@example.com", "password123").await;

        let Json(updated) = update_user(
            State(pool),
            Extension(claims_for(user.id)),
            Path(user.id),
            Json(UpdateUserRequest {
                username: Some("新ユーザー名".to_string()),
                email: None,
            }),
        )
        .await
        .unwrap();

        assert_eq!(updated.username, "新ユーザー名");
        assert_eq!(updated.email, "keep@example.com", "email が巻き込まれている");
    }

    #[sqlx::test]
    async fn update_user_changes_only_email(pool: PgPool) {
        let user = setup_user(&pool, "変わらない名前", "before@example.com", "password123").await;

        let Json(updated) = update_user(
            State(pool),
            Extension(claims_for(user.id)),
            Path(user.id),
            Json(UpdateUserRequest {
                username: None,
                email: Some("after@example.com".to_string()),
            }),
        )
        .await
        .unwrap();

        assert_eq!(updated.email, "after@example.com");
        assert_eq!(updated.username, "変わらない名前", "username が巻き込まれている");
    }

    #[sqlx::test]
    async fn update_user_rejects_other_users_id(pool: PgPool) {
        let victim = setup_user(&pool, "被害者", "victim@example.com", "password123").await;
        let attacker = Uuid::new_v4();

        let err = expect_err(
            update_user(
                State(pool),
                Extension(claims_for(attacker)),
                Path(victim.id),
                Json(UpdateUserRequest {
                    username: Some("乗っ取り".to_string()),
                    email: None,
                }),
            )
            .await,
        );

        assert!(matches!(err, AppError::Forbidden(_)));
    }

    // UNIQUE 制約違反を AppError::Conflict へ変換している箇所を守る。
    // sqlx のバージョンを上げてエラーの形が変わると 409 が 500 に化けるが、
    // このテストが無いと気付けない。
    #[sqlx::test]
    async fn update_user_rejects_email_taken_by_another(pool: PgPool) {
        let a = setup_user(&pool, "Aさん", "a@example.com", "password123").await;
        setup_user(&pool, "Bさん", "b@example.com", "password123").await;

        let err = expect_err(
            update_user(
                State(pool),
                Extension(claims_for(a.id)),
                Path(a.id),
                Json(UpdateUserRequest {
                    username: None,
                    email: Some("b@example.com".to_string()),
                }),
            )
            .await,
        );

        assert!(
            matches!(err, AppError::Conflict(_)),
            "UNIQUE 制約違反が Conflict に変換されていない: {err:?}"
        );
    }

    // delete_user test

    #[sqlx::test]
    async fn delete_user_removes_own_record(pool: PgPool) {
        let user = setup_user(&pool, "消える人", "del@example.com", "password123").await;

        let _ = delete_user(
            State(pool.clone()),
            Extension(claims_for(user.id)),
            Path(user.id),
        )
        .await
        .unwrap();

        let err = expect_err(
            get_user(
                State(pool),
                Extension(claims_for(user.id)),
                Path(user.id),
            )
            .await,
        );

        assert!(matches!(err, AppError::NotFound(_)), "削除後も取得できている");
    }

    #[sqlx::test]
    async fn delete_user_rejects_other_users_id(pool: PgPool) {
        let victim = setup_user(&pool, "被害者", "victim2@example.com", "password123").await;

        let err = expect_err(
            delete_user(
                State(pool),
                Extension(claims_for(Uuid::new_v4())),
                Path(victim.id),
            )
            .await,
        );

        assert!(matches!(err, AppError::Forbidden(_)));
    }

    #[sqlx::test]
    async fn delete_user_returns_not_found_when_already_deleted(pool: PgPool) {
        let id = Uuid::new_v4();

        let err = expect_err(delete_user(State(pool), Extension(claims_for(id)), Path(id)).await);

        assert!(matches!(err, AppError::NotFound(_)));
    }
}
