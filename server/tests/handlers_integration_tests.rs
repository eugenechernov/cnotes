/*
INTEGRATION TESTS FOR NOTES HANDLERS

These tests require a PostgreSQL database to be running.

Setup Requirements:
1. PostgreSQL running on localhost:5432
2. Database 'test_notes_db' created: CREATE DATABASE test_notes_db;
3. User 'postgres' with password 'password' (or set TEST_DATABASE_URL)

To run these tests:
    cargo test --test handlers_integration_tests

To skip these tests and run only unit tests:
    cargo test --lib
*/

use axum::extract::{Path, State};
use axum::Json as JsonExtract;
use chrono::Utc;
use notes_server::handlers::notes::{create_note, delete_note, get_all_notes, get_note_by_id, update_note};
use notes_server::models::{CreateNote, UpdateNote};
use sqlx::{PgPool, Row};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{
        runners::SyncRunner,
        Container as TestContainer,
        Image as TestImage
    }
};

// A small wrapper that ensures the underlying sync `Container` is dropped
// on a dedicated OS thread. Dropping the sync container may call into
// blocking APIs (and `block_on`) which must not run inside an existing
// Tokio runtime.
struct DetachedContainer<T: TestImage + 'static>(Option<TestContainer<T>>);

impl<T: TestImage + 'static> DetachedContainer<T> {
    fn new(c: TestContainer<T>) -> Self {
        DetachedContainer(Some(c))
    }
}

impl<T: TestImage + 'static> Drop for DetachedContainer<T> {
    fn drop(&mut self) {
        if let Some(container) = self.0.take() {
            // Move the container into a new thread so its Drop happens
            // outside the current async runtime.
            std::thread::spawn(move || drop(container));
        }
    }
}

// Helper function to create test database pool
async fn create_test_pool() -> Result<(DetachedContainer<Postgres>, PgPool), Box<dyn std::error::Error + Send + Sync>> {
    // Start Postgres container in a blocking thread to avoid creating
    // a new Tokio runtime inside the existing async runtime.
    // Start container and query mapped port inside the blocking thread
    let (container, port) = tokio::task::spawn_blocking(|| -> Result<(TestContainer<Postgres>, u16), Box<dyn std::error::Error + Send + Sync>> {
        let c = Postgres::default()
            .with_db_name("test_db")
            .with_user("test")
            .with_password("test")
            .start()
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        let p = c.get_host_port_ipv4(5432).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        Ok((c, p))
    })
    .await??;

    let url = format!("postgres://test:test@127.0.0.1:{}/test_db", port);

    // Wait for database to be ready with an exponential backoff and timeout
    let mut attempts: u32 = 0;
    let pool = loop {
        match PgPool::connect(&url).await {
            Ok(pool) => break pool,
            Err(err) => {
                attempts += 1;
                if attempts > 60 {
                    return Err(format!("Timed out waiting for Postgres to be ready: {}", err).into());
                }
                // backoff: increase sleep up to 1s
                let backoff_ms = std::cmp::min(1000, 50 * attempts as i32) as u64;
                tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;
            }
        }
    };

    // Run migrations (fails fast if migrations are broken)
    sqlx::migrate!("./migrations").run(&pool).await.map_err(|e| {
        format!("Failed to run migrations against {}: {}", url, e)
    })?;

    // Wrap the sync container so its Drop runs on a separate thread
    Ok((DetachedContainer::new(container), pool))
}

// Helper function to insert test note
async fn insert_test_note(pool: &PgPool, title: &str, content: &str) -> i32 {
    let now = Utc::now();
    let row = sqlx::query(
        "INSERT INTO notes (title, content, created_at, updated_at) 
         VALUES ($1, $2, $3, $4) 
         RETURNING id"
    )
    .bind(title)
    .bind(content)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await
    .expect("Failed to insert test note");

    row.get::<i32, _>("id")
}

#[tokio::test]
async fn test_get_all_notes_empty() {
    let (_container, pool) = create_test_pool().await.unwrap();
    let response = get_all_notes(State(pool)).await;
    
    assert!(response.is_ok());
    let json_response = response.unwrap();
    let value = json_response.0;
    
    assert_eq!(value["success"], true);
    assert_eq!(value["data"]["notes"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_get_all_notes_with_data() {
    let (_container, pool) = create_test_pool().await.unwrap();
    
    // Insert test notes
    insert_test_note(&pool, "Test Note 1", "Content 1").await;
    insert_test_note(&pool, "Test Note 2", "Content 2").await;
    
    let response = get_all_notes(State(pool)).await;
    
    assert!(response.is_ok());
    let json_response = response.unwrap();
    let value = json_response.0;
    
    assert_eq!(value["success"], true);
    let notes = value["data"]["notes"].as_array().unwrap();
    assert_eq!(notes.len(), 2);
    
    // Notes should be ordered by created_at DESC
    assert_eq!(notes[0]["title"], "Test Note 2");
    assert_eq!(notes[1]["title"], "Test Note 1");
}

#[tokio::test]
async fn test_get_note_by_id_success() {
    let (_container, pool) = create_test_pool().await.unwrap();
    let note_id = insert_test_note(&pool, "Test Note", "Test Content").await;
    
    let response = get_note_by_id(State(pool), Path(note_id)).await;
    
    assert!(response.is_ok());
    let json_response = response.unwrap();
    let value = json_response.0;
    
    assert_eq!(value["success"], true);
    assert_eq!(value["data"]["note"]["id"], note_id);
    assert_eq!(value["data"]["note"]["title"], "Test Note");
    assert_eq!(value["data"]["note"]["content"], "Test Content");
}

#[tokio::test]
async fn test_get_note_by_id_not_found() {
    let (_container, pool) = create_test_pool().await.unwrap();
    
    let response = get_note_by_id(State(pool), Path(999)).await;
    
    assert!(response.is_err());
    let (status, json_response) = response.unwrap_err();
    let value = json_response.0;
    
    assert_eq!(status, axum::http::StatusCode::NOT_FOUND);
    assert_eq!(value["success"], false);
    assert_eq!(value["error"]["code"], "NOT_FOUND");
}

#[tokio::test]
async fn test_create_note_success() {
    let (_container, pool) = create_test_pool().await.unwrap();
    let payload = CreateNote {
        title: "New Note".to_string(),
        content: "New Content".to_string(),
    };
    
    let response = create_note(State(pool), JsonExtract(payload)).await;
    
    assert!(response.is_ok());
    let json_response = response.unwrap();
    let value = json_response.0;
    
    assert_eq!(value["success"], true);
    assert_eq!(value["data"]["note"]["title"], "New Note");
    assert_eq!(value["data"]["note"]["content"], "New Content");
    assert!(value["data"]["note"]["id"].as_i64().unwrap() > 0);
}

#[tokio::test]
async fn test_create_note_validation_error() {
    let (_container, pool) = create_test_pool().await.unwrap();
    let payload = CreateNote {
        title: "".to_string(), // Empty title should fail validation
        content: "Content".to_string(),
    };
    
    let response = create_note(State(pool), JsonExtract(payload)).await;
    
    assert!(response.is_err());
    let (status, json_response) = response.unwrap_err();
    let value = json_response.0;
    
    assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(value["success"], false);
    assert_eq!(value["error"]["code"], "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_create_note_title_too_long() {
    let (_container, pool) = create_test_pool().await.unwrap();
    let long_title = "a".repeat(256); // Exceeds 255 character limit
    let payload = CreateNote {
        title: long_title,
        content: "Content".to_string(),
    };
    
    let response = create_note(State(pool), JsonExtract(payload)).await;
    
    assert!(response.is_err());
    let (status, json_response) = response.unwrap_err();
    let value = json_response.0;
    
    assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(value["error"]["code"], "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_update_note_success() {
    let (_container, pool) = create_test_pool().await.unwrap();
    let note_id = insert_test_note(&pool, "Original Title", "Original Content").await;
    
    let payload = UpdateNote {
        title: Some("Updated Title".to_string()),
        content: Some("Updated Content".to_string()),
    };
    
    let response = update_note(State(pool), Path(note_id), JsonExtract(payload)).await;
    
    assert!(response.is_ok());
    let json_response = response.unwrap();
    let value = json_response.0;
    
    assert_eq!(value["success"], true);
    assert_eq!(value["data"]["note"]["title"], "Updated Title");
    assert_eq!(value["data"]["note"]["content"], "Updated Content");
    assert_eq!(value["data"]["note"]["id"], note_id);
}

#[tokio::test]
async fn test_update_note_partial() {
    let (_container, pool) = create_test_pool().await.unwrap();
    let note_id = insert_test_note(&pool, "Original Title", "Original Content").await;
    
    let payload = UpdateNote {
        title: Some("Updated Title".to_string()),
        content: None, // Only update title
    };
    
    let response = update_note(State(pool), Path(note_id), JsonExtract(payload)).await;
    
    assert!(response.is_ok());
    let json_response = response.unwrap();
    let value = json_response.0;
    
    assert_eq!(value["success"], true);
    assert_eq!(value["data"]["note"]["title"], "Updated Title");
    assert_eq!(value["data"]["note"]["content"], "Original Content"); // Should remain unchanged
}

#[tokio::test]
async fn test_update_note_not_found() {
    let (_container, pool) = create_test_pool().await.unwrap();
    
    let payload = UpdateNote {
        title: Some("Updated Title".to_string()),
        content: None,
    };
    
    let response = update_note(State(pool), Path(999), JsonExtract(payload)).await;
    
    assert!(response.is_err());
    let (status, json_response) = response.unwrap_err();
    let value = json_response.0;
    
    assert_eq!(status, axum::http::StatusCode::NOT_FOUND);
    assert_eq!(value["error"]["code"], "NOT_FOUND");
}

#[tokio::test]
async fn test_update_note_no_updates() {
    let (_container, pool) = create_test_pool().await.unwrap();
    let note_id = insert_test_note(&pool, "Title", "Content").await;
    
    let payload = UpdateNote {
        title: None,
        content: None,
    };
    
    let response = update_note(State(pool), Path(note_id), JsonExtract(payload)).await;
    
    assert!(response.is_err());
    let (status, json_response) = response.unwrap_err();
    let value = json_response.0;
    
    assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(value["error"]["code"], "NO_UPDATES");
}

#[tokio::test]
async fn test_update_note_validation_error() {
    let (_container, pool) = create_test_pool().await.unwrap();
    let note_id = insert_test_note(&pool, "Title", "Content").await;
    
    let payload = UpdateNote {
        title: Some("".to_string()), // Empty title should fail validation
        content: None,
    };
    
    let response = update_note(State(pool), Path(note_id), JsonExtract(payload)).await;
    
    assert!(response.is_err());
    let (status, json_response) = response.unwrap_err();
    let value = json_response.0;
    
    assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(value["error"]["code"], "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_delete_note_success() {
    let (_container, pool) = create_test_pool().await.unwrap();
    let note_id = insert_test_note(&pool, "Test Note", "Test Content").await;
    
    let response = delete_note(State(pool), Path(note_id)).await;
    
    assert!(response.is_ok());
    let json_response = response.unwrap();
    let value = json_response.0;
    
    assert_eq!(value["success"], true);
    assert_eq!(value["data"]["message"], "Note deleted successfully");
}

#[tokio::test]
async fn test_delete_note_not_found() {
    let (_container, pool) = create_test_pool().await.unwrap();
    
    let response = delete_note(State(pool), Path(999)).await;
    
    assert!(response.is_err());
    let (status, json_response) = response.unwrap_err();
    let value = json_response.0;
    
    assert_eq!(status, axum::http::StatusCode::NOT_FOUND);
    assert_eq!(value["success"], false);
    assert_eq!(value["error"]["code"], "NOT_FOUND");
}
