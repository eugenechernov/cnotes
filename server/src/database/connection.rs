use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database connection error: {0}")]
    Connection(#[from] sqlx::Error),
    #[error("Database migration error: {0}")]
    Migration(String),
    #[error("Query execution error: {0}")]
    Query(String),
}

pub async fn create_connection_pool(database_url: &str) -> Result<PgPool, DatabaseError> {
    tracing::info!("Creating database connection pool");
    
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await
        .map_err(DatabaseError::Connection)?;

    tracing::info!("Database connection pool created successfully");
    Ok(pool)
}

pub async fn test_connection(pool: &PgPool) -> Result<(), DatabaseError> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(DatabaseError::Connection)?;
    Ok(())
}