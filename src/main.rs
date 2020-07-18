use std::env;
use sqlx::Executor;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tokio::task;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let outer_pool = PgPoolOptions::new()
        .max_connections(10)
        .after_connect(|conn| Box::pin(async move {
            conn.execute("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ;").await?;
            Ok(())
        }))
        .connect_timeout(Duration::from_secs(10))
        .connect(&env::var("DATABASE_URL").unwrap()).await?;

    let setup = ["
        CREATE TABLE demo (a int PRIMARY KEY, b int, c int, d int);
    ", "
        CREATE OR REPLACE FUNCTION raise_exception() RETURNS trigger AS $$
        DECLARE
            message text;
        BEGIN
            message := TG_ARGV[0];
            RAISE EXCEPTION '%', message;
        END;
        $$ LANGUAGE plpgsql;
    ", "
        CREATE TRIGGER demo_check_update
            BEFORE UPDATE ON demo
            FOR EACH ROW
            EXECUTE FUNCTION raise_exception('cannot update');
    "];

    let mut tx = outer_pool.begin().await?;
    for stmt in &setup {
        sqlx::query(stmt).execute(&mut tx).await?;
    }
    tx.commit().await?;

    let mut handles = vec![];
    for i in 0..30 {
        let pool = outer_pool.clone();
        let handle = task::spawn(async move {
            for (column, value) in &[
                ("a", "1"),
                ("b", "2"),
                ("c", "3"),
                ("d", "4"),
            ] {
                let mut tx = pool.begin().await.unwrap();
                let query = format!("UPDATE demo SET {} = {} WHERE a = $1", column, value);
                let result = sqlx::query(&query).bind(i).execute(&mut tx).await;
                assert_eq!(result.err().expect("expected an error").to_string(), "error returned from database: cannot update");
            }
        });
        handles.push(handle);

        let pool = outer_pool.clone();
        let handle = task::spawn(async move {
            let query = "INSERT INTO demo VALUES ($1::int)";
            let mut tx = pool.begin().await.unwrap();
            sqlx::query(query).bind(i).execute(&mut tx).await.unwrap();
            tx.commit().await.unwrap();

            // Generate an error
            let query = "UPDATE demo SET a = $1 + 10000 WHERE a = $1";
            let msg = "error returned from database: cannot update";
            let mut tx = pool.begin().await.unwrap();
            let result = sqlx::query(query).bind(i).execute(&mut tx).await;
            assert_eq!(result.err().unwrap().to_string(), msg);
        });
        handles.push(handle);
    }
    futures::future::join_all(handles).await;

    Ok(())
}
