use axum::{Router, 
    routing::get, routing::post, 
    routing::delete, routing::put, 
};
use std::net::SocketAddr;
use dotenvy::dotenv;
use tower_http::cors::CorsLayer;
use tower_http::cors::Any;
use std::env;


mod handlers;
mod models;
mod db;
mod auth;

// 開発環境　本番環境によって読み込む.envファイルを選択
fn load_env() {
    let app_env =  env::var("APP_ENV")
        .unwrap_or_else(|_| "development".to_string());
    match app_env.as_str() {
        "production" => {
            dotenvy::from_filename(".env.production").ok();
        }
        _  => {
            dotenvy::dotenv().ok();
        }
    }
} 



#[tokio::main]
async fn main() {

    load_env();

    let database_url = env::var("DATABASE_URL").expect("NOT FOUND DATABASE_URL"); 

    let pool = db::init_db(&database_url).await.expect("DB 接続に失敗しました");

    let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);

    //ルータインスタンスを作成、
    let app = Router::new()
        .route("/health", get(handlers::health_check))//apiエンドポイント(URL)を作成
        .route("/ready", get(handlers::db_ready_check))
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

