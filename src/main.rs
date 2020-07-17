use std::env;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tokio::task;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect_timeout(Duration::from_secs(3))
        .connect(&env::var("DATABASE_URL").unwrap()).await?;

    let setup = ["
        CREATE TABLE demo (id int PRIMARY KEY);
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
        CREATE TRIGGER demo_forbid_truncate
            BEFORE TRUNCATE ON demo
            EXECUTE FUNCTION raise_exception('truncate is forbidden');
    "];

    let mut tx = pool.begin().await?;
    for stmt in &setup {
        sqlx::query(stmt).execute(&mut tx).await?;
    }
    tx.commit().await?;

    let mut handles = vec![];
    for _ in 0..100 {
        let pool = pool.clone();
        let handle = task::spawn(async move {
            let query = "TRUNCATE demo";
            let msg = "error returned from database: truncate is forbidden";

            let mut tx = pool.begin().await.unwrap();
            let result = sqlx::query(query).execute(&mut tx).await;
            assert_eq!(result.err().unwrap().to_string(), msg);
            dbg!("asdf");
        });
        handles.push(handle);
    }
    futures::future::join_all(handles).await;

    Ok(())
}
