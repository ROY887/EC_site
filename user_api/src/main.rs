use axum::{Router, routing::get, routing::post, routing::delete, routing::put};
use std::net::SocketAddr;
use dotenvy::dotenv;
use tower_http::cors::CorsLayer;
use tower_http::cors::Any;
use std::env;


mod handlers;
mod models;
mod db;



 
#[tokio::main]
async fn main() {
    dotenv().ok();

    let pool = db::init_db().await.expect("DB 接続に失敗しました");

    let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);

    //ルータインスタンスを作成、
    let app = Router::new()
        .route("/health", get(handlers::health_check))//apiエンドポイント(URL)を作成
        .route("/users", post(handlers::create_user))
        .route("/login", post(handlers::login_user))
        .route("/users", get(handlers::get_all_user))
        .route("/users/:id", get(handlers::get_user))
        .route("/users/:id", put(handlers::update_user))
        .route("/users/:id", delete(handlers::delete_user))
        .layer(cors)
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
    println!("user-api running on http://{}", addr);
    println!("Available endpoints:");
    println!("  GET    /health");
    println!("  POST   /users (signup)");
    println!("  POST   /login");
    println!("  GET    /users");
    println!("  GET    /users/:id");
    println!("  PUT    /users/:id");
    println!("  DELETE /users/:id");


    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

