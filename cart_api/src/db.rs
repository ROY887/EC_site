use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::env;
use std::time::Duration;

pub type DbPool = Pool<Postgres>;

pub async fn init_db() -> Result<DbPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");

    // レプリカ数 × max_connections が PostgreSQL の上限を超えないよう制限する
    PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await
}
