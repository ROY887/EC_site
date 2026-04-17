use sqlx::{Pool,Postgres};
use dotenvy::dotenv;
use std::env;

pub type DbPool = Pool<Postgres>;

pub async fn init_db() -> Result<DbPool, sqlx::Error> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    Pool::<Postgres>::connect (&database_url).await
}

