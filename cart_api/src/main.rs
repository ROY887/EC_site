use axum::{Router, routing::{get,post , put, delete}};
use dotenvy::dotenv;
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::cors::Any;
   


mod handlers;
mod models;
mod db;
//非同期処理は実行を並列して行いたいの1の実行をfutureという値を返す。furtureはすべての実行を終えたときに実行した時の実際の値を返すため、途中で実行した時の値が欲しい時はawaitを使う
#[tokio::main]
async fn main() {
    dotenv().ok();

    let db_url=env::var("DATABASE_URL").expect("database_url no set");
    println!("DB URL = {}", db_url);

    let pool = db::init_db().await.expect("DB接続に失敗しました");   //awaitとは非同期処理を行ったときに、1つのfutureから実際の実行を行うためのもの

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/cart/add", post(handlers::add_item))
        .route("/health",get(handlers::health_check))
        .route("/cart/:user_id",get(handlers::get_cart))
        .route("/cart/:user_id", delete(handlers::clear_cart))
        .route("/cart/update", put(handlers::update_item))
        .route("/cart/remove", delete(handlers::remove_item))
        .layer(cors)
        .with_state(pool);

    let addr = SocketAddr::from(([0 , 0 , 0 , 0],8085));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("server running on http://{}",addr);
    println!("cart-api running on http://{}", addr);
    println!("Available endpoints:");
    println!("  GET    /health");
    println!("  POST   /cart/add");
    println!("  GET    /cart/:user_id");
    println!("  PUT    /cart/update");
    println!("  DELETE /cart/remove");
    println!("  DELETE /cart/:user_id");

    axum::serve(listener, app)//
        .await
        .unwrap();   //unwrapはResult型などの値がOKである前提で動いているため、もしNoneやErrだった場合は、panic!マクロが呼ばれてプログラムがクラッシュする

}

