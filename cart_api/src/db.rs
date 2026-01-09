use serde::{Deserialize,Serialize};
use sqlx::{Postgres,Pool,PgPool};
use std::env;


pub type DbPool= Pool<Postgres>; 

pub async fn init_db() -> Result<DbPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    Pool::<Postgres>::connect(&database_url).await?;
    Ok(Pool::<Postgres>::connect(&database_url).await?)
}


