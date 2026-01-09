

use axum::{Router, routing::{get, post, put, delete}};
use std::net::SocketAddr;
use dotenvy::dotenv;
use tower_http::cors::CorsLayer;
use tower_http::cors::Any;

mod handlers;
mod models;
mod db;
// mod lib; を削除（警告の原因）

#[tokio::main]
async fn main() {
    dotenv().ok();

    let pool = db::init_db().await.expect("DB接続に失敗しました");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/products", get(handlers::get_products))
        .route("/products", post(handlers::create_product))
        .route("/products/:id", get(handlers::get_product))
        .route("/products/:id", put(handlers::update_product))
        .route("/products/:id", delete(handlers::delete_product))
        .route("/products/category/:category", get(handlers::get_products_by_category))
        .route("/products/search", get(handlers::search_products))
        .layer(cors)
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8084));
    
    println!("product-api running on http://{}", addr);
    println!("Available endpoints:");
    println!("  GET    /health");
    println!("  GET    /products");
    println!("  POST   /products");
    println!("  GET    /products/:id");
    println!("  PUT    /products/:id");
    println!("  DELETE /products/:id");
    println!("  GET    /products/category/:category");
    println!("  GET    /products/search?q=keyword");

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

