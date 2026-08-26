use axum::{
    Router, middleware,
    routing::{delete, get, post, put},
};
use std::env;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod db;
mod error;
mod handlers;
mod models;

/// 開発環境・本番環境で読み込む .env を切り替える
fn load_env() {
    match env::var("APP_ENV").as_deref() {
        Ok("production") => {
            dotenvy::from_filename(".env.production").ok();
        }
        _ => {
            dotenvy::dotenv().ok();
        }
    }
}

fn app_port() -> u16 {
    env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8081)
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}

#[tokio::main]
async fn main() {
    load_env();
    init_tracing();

    // JWT_SECRET の検証を起動時に済ませる（本番で未設定ならここで落ちる）
    let _ = auth::get_jwt_secret();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL が設定されていません");
    let pool = db::init_db(&database_url)
        .await
        .expect("DB 接続に失敗しました");

    // 認証が必要なルート
    let protected = Router::new()
        .route("/users/:id", get(handlers::get_user))
        .route("/users/:id", put(handlers::update_user))
        .route("/users/:id", delete(handlers::delete_user))
        .layer(middleware::from_fn(auth::require_auth));

    // 認証不要のルート
    let public = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/ready", get(handlers::db_ready_check))
        .route("/users", post(handlers::create_user))
        .route("/login", post(handlers::login_user));

    let app = public
        .merge(protected)
        .layer(TraceLayer::new_for_http())
        .with_state(pool);

    // 本番はフロントと同一オリジンのため CORS 不要。開発時のみ許可する。
    let app = match env::var("APP_ENV").as_deref() {
        Ok("production") => app,
        _ => app.layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        ),
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], app_port()));
    tracing::info!(%addr, "user-api listening");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
