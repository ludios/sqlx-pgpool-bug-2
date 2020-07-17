use std::env;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // Create a connection pool
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&env::var("DATABASE_URL").unwrap()).await?;

    let query = "blah blah"; // SELECT 1 works fine

    let mut tx = pool.begin().await?;
    let result = sqlx::query(query).execute(&mut tx).await;
    assert_eq!(result.err().unwrap().to_string(), "error returned from database: syntax error at or near \"blah\"");
    drop(tx);

    let mut tx = pool.begin().await?;
    let result = sqlx::query(query).execute(&mut tx).await;
    assert_eq!(result.err().unwrap().to_string(), "error returned from database: syntax error at or near \"blah\"");
    drop(tx);

    let mut tx = pool.begin().await?;
    let result = sqlx::query(query).execute(&mut tx).await;
    assert_eq!(result.err().unwrap().to_string(), "error returned from database: syntax error at or near \"blah\"");
    drop(tx);

    Ok(())
}
