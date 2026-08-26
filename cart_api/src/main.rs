use axum::{
    Router, middleware,
    routing::{delete, get, post, put},
};
use dotenvy::dotenv;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod db;
mod error;
mod handlers;
mod models;

fn app_port() -> u16 {
    std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8085)
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    init_tracing();

    // JWT_SECRET の検証を起動時に済ませる
    let _ = auth::get_jwt_secret();

    let pool = db::init_db().await.expect("DB接続に失敗しました");

    // カート操作は全て認証必須。対象ユーザーは JWT から導出するため
    // リクエストで user_id を指定することはできない。
    let protected = Router::new()
        .route("/cart", get(handlers::get_cart))
        .route("/cart", delete(handlers::clear_cart))
        .route("/cart/add", post(handlers::add_item))
        .route("/cart/update", put(handlers::update_item))
        .route("/cart/remove", delete(handlers::remove_item))
        .layer(middleware::from_fn(auth::require_auth));

    let public = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/ready", get(handlers::db_ready_check));

    let app = public
        .merge(protected)
        .layer(TraceLayer::new_for_http())
        .with_state(pool);

    let app = match std::env::var("APP_ENV").as_deref() {
        Ok("production") => app,
        _ => app.layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        ),
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], app_port()));
    tracing::info!(%addr, "cart-api listening");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
