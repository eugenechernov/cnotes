pub mod connection;

pub use connection::{create_connection_pool, DatabaseError};

use sqlx::PgPool;

pub type Pool = PgPool;