use sqlx::{Pool, Postgres};
use std::env;
use std::result::Result;

pub type DbPool = Pool<Postgres>;

pub async fn init_db() -> Result<DbPool, sqlx::Error> { //Resultは正しく動いた場合の出力Tと,エラーとなった時の出力Eを記入する
    let database_url = env::var("DATABASE_URL").expect("NOT FOUND DATABASE_URL"); //database_urlにenvに書いてある、DATABASE_URLを代入し、expectメソッドはerrした時メッセージを出力する
    Pool::<Postgres>::connect(&database_url).await
}


