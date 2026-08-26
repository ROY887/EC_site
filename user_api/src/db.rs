use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::time::Duration;

pub type DbPool = Pool<Postgres>;

pub async fn init_db(database_url: &str) -> Result<DbPool, sqlx::Error> {
    // コネクション数の制限をかける
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await?;

    Ok(pool)
}
