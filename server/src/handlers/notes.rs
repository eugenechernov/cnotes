use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Json as JsonExtract,
};
use chrono::Utc;
use serde_json::{json, Value};

use crate::database::Pool;
use crate::models::{CreateNote, Note, UpdateNote};

// Response types
type ApiResponse = Result<Json<Value>, (StatusCode, Json<Value>)>;

fn success_response(data: Value) -> ApiResponse {
    Ok(Json(json!({
        "success": true,
        "data": data,
        "error": null
    })))
}

fn error_response(status: StatusCode, code: &str, message: &str) -> ApiResponse {
    Err((
        status,
        Json(json!({
            "success": false,
            "data": null,
            "error": {
                "code": code,
                "message": message
            }
        })),
    ))
}

pub async fn get_all_notes(State(pool): State<Pool>) -> ApiResponse {
    // TODO: Add pagination support
    
    let notes = sqlx::query_as::<_, Note>(
        "SELECT id, title, content, created_at, updated_at 
         FROM notes 
         ORDER BY created_at DESC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch notes: {:?}", e);
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DATABASE_ERROR",
            "Failed to fetch notes",
        )
    })?;

    success_response(json!({ "notes": notes }))
}

pub async fn get_note_by_id(
    State(pool): State<Pool>,
    Path(id): Path<i32>,
) -> ApiResponse {
    let note = sqlx::query_as::<_, Note>(
        "SELECT id, title, content, created_at, updated_at 
         FROM notes 
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch note {}: {:?}", id, e);
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DATABASE_ERROR",
            "Failed to fetch note",
        )
    })?;

    match note {
        Some(note) => success_response(json!({ "note": note })),
        None => error_response(
            StatusCode::NOT_FOUND,
            "NOT_FOUND",
            &format!("Note with id {} not found", id),
        ),
    }
}

pub async fn create_note(
    State(pool): State<Pool>,
    JsonExtract(payload): JsonExtract<CreateNote>,
) -> ApiResponse {
    // Validate input
    if let Err(msg) = payload.validate() {
        return error_response(StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg);
    }

    let now = Utc::now();
    let note = sqlx::query_as::<_, Note>(
        "INSERT INTO notes (title, content, created_at, updated_at) 
         VALUES ($1, $2, $3, $4) 
         RETURNING id, title, content, created_at, updated_at",
    )
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(now)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create note: {:?}", e);
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DATABASE_ERROR",
            "Failed to create note",
        )
    })?;

    success_response(json!({ "note": note }))
}

pub async fn update_note(
    State(pool): State<Pool>,
    Path(id): Path<i32>,
    JsonExtract(payload): JsonExtract<UpdateNote>,
) -> ApiResponse {
    // Validate input
    if let Err(msg) = payload.validate() {
        return error_response(StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg);
    }

    if !payload.has_updates() {
        return error_response(
            StatusCode::BAD_REQUEST,
            "NO_UPDATES",
            "No fields provided for update",
        );
    }

    // Check if note exists
    let existing_note = sqlx::query_as::<_, Note>(
        "SELECT id, title, content, created_at, updated_at 
         FROM notes 
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to check note existence {}: {:?}", id, e);
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DATABASE_ERROR",
            "Failed to update note",
        )
    })?;

    let existing_note = match existing_note {
        Some(note) => note,
        None => {
            return error_response(
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                &format!("Note with id {} not found", id),
            )
        }
    };

    // Prepare update values
    let title = payload.title.unwrap_or(existing_note.title);
    let content = payload.content.unwrap_or(existing_note.content);
    let now = Utc::now();

    let note = sqlx::query_as::<_, Note>(
        "UPDATE notes 
         SET title = $1, content = $2, updated_at = $3 
         WHERE id = $4 
         RETURNING id, title, content, created_at, updated_at",
    )
    .bind(&title)
    .bind(&content)
    .bind(now)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update note {}: {:?}", id, e);
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DATABASE_ERROR",
            "Failed to update note",
        )
    })?;

    success_response(json!({ "note": note }))
}

pub async fn delete_note(State(pool): State<Pool>, Path(id): Path<i32>) -> ApiResponse {
    let result = sqlx::query("DELETE FROM notes WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete note {}: {:?}", id, e);
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                "Failed to delete note",
            )
        })?;

    if result.rows_affected() == 0 {
        return error_response(
            StatusCode::NOT_FOUND,
            "NOT_FOUND",
            &format!("Note with id {} not found", id),
        );
    }

    success_response(json!({ "message": "Note deleted successfully" }))
}