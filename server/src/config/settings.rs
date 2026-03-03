use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database_url: String,
    pub port: u16,
    pub host: String,
}

impl Settings {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load .env file if it exists
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/notes".to_string());

        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()?;

        let host = env::var("HOST")
            .unwrap_or_else(|_| "0.0.0.0".to_string());

        Ok(Settings {
            database_url,
            port,
            host,
        })
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new().expect("Failed to load settings")
    }
}