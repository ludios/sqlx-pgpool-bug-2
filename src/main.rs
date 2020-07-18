fn main() {}

#[cfg(test)]
pub(crate) mod tests {
    use std::env;
    use sqlx::postgres::{PgPoolOptions, PgPool};
    use std::time::Duration;
    use once_cell::sync::Lazy;
    use std::future::Future;
    use std::pin::Pin;
    use futures::future::{FutureExt, Shared};

    pub async fn new_pgpool(uri: &str, max_connections: u32) -> PgPool {
        PgPoolOptions::new()
            .connect_timeout(Duration::from_secs(10))
            .max_connections(max_connections)
            .connect(&uri).await.unwrap()
    }

    /// PgPool Future initialized once by the first caller
    static POOL: Lazy<Shared<Pin<Box<dyn Future<Output=PgPool> + Send>>>> = Lazy::new(|| async {
        let uri = env::var("DATABASE_URL").expect("need DATABASE_URL");
        new_pgpool(&uri, 100).await
    }.boxed().shared());

    /// Return the PgPool for tests
    async fn pool() -> PgPool {
        POOL.clone().await
    }

    #[tokio::test]
    async fn test_a() {
        let pool = pool().await;

        let mut tx = pool.begin().await.unwrap();
        sqlx::query("SELECT * FROM pg_index").execute(&mut tx).await.unwrap();
 
        // This second transaction is not always needed to see a failure
        let mut tx = pool.begin().await.unwrap();
        sqlx::query("SELECT * FROM pg_index").execute(&mut tx).await.unwrap();
    }

    #[tokio::test]
    async fn test_b() {
        let pool = pool().await;

        // You may need to adjust the number of iterations here
        for _ in 1..3 {
            let mut tx = pool.begin().await.unwrap();
            let result = sqlx::query("blah blah").execute(&mut tx).await;
            assert_eq!(result.err().expect("expected an error").to_string(), "error returned from database: syntax error at or near \"blah\"");
        }
    }
}
