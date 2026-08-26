use axum::{Router, routing::get};
use dotenvy::dotenv;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod db;
mod error;
mod handlers;
mod models;

//リッスンポートは環境変数 PORT を唯一の情報源とする。
//コードと k8s マニフェストのポートがずれる事故を防ぐ。
fn app_port() -> u16 {
    std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8084)
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

    let pool = db::init_db().await.expect("DB接続に失敗しました");

    // 本番はフロントと同一オリジン（Gateway が /api を同じホストに割り当てる）ため
    // CORS は不要。ローカル開発時のみ許可する。
    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/ready", get(handlers::db_ready_check))
        .route("/products", get(handlers::get_products))
        .route("/products/search", get(handlers::search_products))
        .route("/products/category/:category", get(handlers::get_products_by_category))
        .route("/products/:id", get(handlers::get_product))
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
    tracing::info!(%addr, "product-api listening");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
