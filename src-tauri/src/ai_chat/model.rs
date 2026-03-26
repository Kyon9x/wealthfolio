//! AI Chat domain models.
//!
//! Provides types for chat threads and messages with structured
//! content parts representing the full agent loop.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A chat thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatThread {
    pub id: String,
    pub title: Option<String>,
    #[serde(default)]
    pub is_pinned: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ChatThread {
    /// Create a new thread with generated UUID.
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title: None,
            is_pinned: false,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// A chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub thread_id: String,
    pub role: String,
    pub content: ChatMessageContent,
    pub created_at: DateTime<Utc>,
}

/// Structured message content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageContent {
    pub parts: Vec<ChatMessagePart>,
}

/// Individual message parts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ChatMessagePart {
    #[serde(rename_all = "camelCase")]
    Text { content: String },
}
