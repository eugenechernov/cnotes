use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Note {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNote {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNote {
    pub title: Option<String>,
    pub content: Option<String>,
}

impl CreateNote {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.title.trim().is_empty() {
            return Err("Title cannot be empty");
        }
        if self.title.len() > 255 {
            return Err("Title cannot be longer than 255 characters");
        }
        if self.content.len() > 10_000 {
            return Err("Content cannot be longer than 10,000 characters");
        }
        Ok(())
    }
}

impl UpdateNote {
    pub fn validate(&self) -> Result<(), &'static str> {
        if let Some(title) = &self.title {
            if title.trim().is_empty() {
                return Err("Title cannot be empty");
            }
            if title.len() > 255 {
                return Err("Title cannot be longer than 255 characters");
            }
        }
        if let Some(content) = &self.content {
            if content.len() > 10_000 {
                return Err("Content cannot be longer than 10,000 characters");
            }
        }
        Ok(())
    }

    pub fn has_updates(&self) -> bool {
        self.title.is_some() || self.content.is_some()
    }
}