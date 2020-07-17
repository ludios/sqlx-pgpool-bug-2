use std::env;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect_timeout(Duration::from_secs(3))
        .connect(&env::var("DATABASE_URL").unwrap()).await?;

    let query = "blah blah"; // SELECT 1 works fine if you remove the assert_eq! below

    let mut tx = pool.begin().await?;
    let result = sqlx::query(query).execute(&mut tx).await;
    assert_eq!(result.err().unwrap().to_string(), "error returned from database: syntax error at or near \"blah\"");
    drop(tx);

    let mut tx = pool.begin().await?;
    let result = sqlx::query(query).execute(&mut tx).await;
    assert_eq!(result.err().unwrap().to_string(), "error returned from database: syntax error at or near \"blah\"");
    drop(tx);

    println!("hanging for 3 seconds now");
    let mut tx = pool.begin().await?;
    println!("not reached");
    let result = sqlx::query(query).execute(&mut tx).await;
    assert_eq!(result.err().unwrap().to_string(), "error returned from database: syntax error at or near \"blah\"");
    drop(tx);

    Ok(())
}
