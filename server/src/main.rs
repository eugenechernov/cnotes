use axum::{
    routing::{get, post, put, delete},
    Router,
    serve::Serve,
};
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod database;
mod handlers;
mod models;

use config::Settings;
use database::create_connection_pool;
use handlers::notes::{
    create_note, 
    get_all_notes, 
    get_note_by_id, 
    update_note, 
    delete_note
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "notes_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let settings = Settings::new()?;
    
    // Create database connection pool
    let pool = create_connection_pool(&settings.database_url).await?;
    
    // Run database migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/notes", get(get_all_notes).post(create_note))
        .route(
            "/api/notes/{:id}",
            get(get_note_by_id)
                .put(update_note)
                .delete(delete_note),
        )
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], settings.port));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}