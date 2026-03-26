//! AI Chat database repository for Tauri.
//!
//! Implements ChatRepositoryTrait from wealthvn-ai using SQLite via Diesel.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use wealthvn_ai::{
    ChatMessage, ChatMessageContent, ChatMessagePart, ChatMessageRole,
    ChatRepositoryResult, ChatRepositoryTrait, ChatThread, ChatThreadConfig, ListThreadsRequest,
    ThreadPage, CHAT_MAX_CONTENT_SIZE_BYTES,
};
use wealthvn_ai::AiError;

use wealthvn_core::db::{get_connection, WriteHandle};
use wealthvn_core::errors::{DatabaseError, Error as CoreError};

// Import schema tables
use wealthvn_core::schema::{ai_messages, ai_thread_tags, ai_threads};

// ============================================================================
// Database Models (inline to avoid extra module file issues)
// ============================================================================

#[derive(Debug, Clone, Queryable, Identifiable, Insertable, AsChangeset, Selectable)]
#[diesel(table_name = ai_threads)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
struct AiThreadDB {
    pub id: String,
    pub title: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub config_snapshot: Option<String>,
    pub is_pinned: i32,
}

#[derive(Debug, Clone, Queryable, Identifiable, Insertable, AsChangeset, Selectable)]
#[diesel(table_name = ai_messages)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
struct AiMessageDB {
    pub id: String,
    pub thread_id: String,
    pub role: String,
    pub content_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Queryable, Identifiable, Insertable)]
#[diesel(table_name = ai_thread_tags)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
struct AiThreadTagDB {
    pub id: String,
    pub thread_id: String,
    pub tag: String,
    pub created_at: String,
}

// ============================================================================
// Content JSON Schema
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MessageContent {
    schema_version: u32,
    parts: Vec<MessagePart>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    truncated: bool,
}

impl MessageContent {
    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    fn to_json_with_limit(&self, max_bytes: usize) -> Result<String, serde_json::Error> {
        let json = self.to_json()?;
        if json.len() <= max_bytes {
            return Ok(json);
        }
        let mut truncated = self.clone();
        truncated.truncated = true;
        truncated.truncate_large_payloads(max_bytes);
        truncated.to_json()
    }

    fn truncate_large_payloads(&mut self, target_bytes: usize) {
        let overhead = 100;
        let available = target_bytes.saturating_sub(overhead);
        let part_count = self.parts.len().max(1);
        let per_part_budget = available / part_count;

        for part in &mut self.parts {
            match part {
                MessagePart::ToolCall { arguments, .. } => {
                    let json = serde_json::to_string(arguments).unwrap_or_default();
                    if json.len() > per_part_budget {
                        *arguments = serde_json::json!({
                            "_truncated": true,
                            "_originalSize": json.len()
                        });
                    }
                }
                MessagePart::ToolResult { data, meta, .. } => {
                    let json = serde_json::to_string(data).unwrap_or_default();
                    if json.len() > per_part_budget {
                        meta.insert("_truncated".to_string(), serde_json::json!(true));
                        meta.insert("_originalSize".to_string(), serde_json::json!(json.len()));
                        *data = serde_json::Value::Null;
                    }
                }
                MessagePart::Text { content } => {
                    if content.len() > per_part_budget {
                        content.truncate(per_part_budget.saturating_sub(20));
                        content.push_str("... [truncated]");
                    }
                }
                MessagePart::Reasoning { content } => {
                    if content.len() > per_part_budget {
                        content.truncate(per_part_budget.saturating_sub(20));
                        content.push_str("... [truncated]");
                    }
                }
                MessagePart::Error { message, .. } => {
                    if message.len() > per_part_budget {
                        message.truncate(per_part_budget.saturating_sub(20));
                        message.push_str("... [truncated]");
                    }
                }
                MessagePart::System { .. } => {}
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum MessagePart {
    #[serde(rename_all = "camelCase")]
    System { content: String },
    #[serde(rename_all = "camelCase")]
    Text { content: String },
    #[serde(rename_all = "camelCase")]
    Reasoning { content: String },
    #[serde(rename_all = "camelCase")]
    ToolCall {
        tool_call_id: String,
        name: String,
        arguments: serde_json::Value,
    },
    #[serde(rename_all = "camelCase")]
    ToolResult {
        tool_call_id: String,
        success: bool,
        data: serde_json::Value,
        #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
        meta: std::collections::HashMap<String, serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Error { code: String, message: String },
}

// ============================================================================
// Repository Implementation
// ============================================================================

pub struct AiChatRepository {
    pool: Arc<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::sqlite::SqliteConnection>>>,
    writer: WriteHandle,
}

impl AiChatRepository {
    pub fn new(
        pool: Arc<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::sqlite::SqliteConnection>>>,
        writer: WriteHandle,
    ) -> Self {
        Self { pool, writer }
    }
}

#[async_trait]
impl ChatRepositoryTrait for AiChatRepository {
    async fn create_thread(&self, thread: ChatThread) -> ChatRepositoryResult<ChatThread> {
        let thread_db = thread_to_db(&thread);
        let thread_id = thread_db.id.clone();

        let result = self
            .writer
            .exec(move |conn| -> Result<ChatThread, CoreError> {
                diesel::insert_into(ai_threads::table)
                    .values(&thread_db)
                    .execute(conn)
                    .map_err(|e| CoreError::Database(DatabaseError::Internal(e.to_string())))?;

                let db = ai_threads::table
                    .find(&thread_id)
                    .first::<AiThreadDB>(conn)
                    .map_err(|e| CoreError::Database(DatabaseError::Internal(e.to_string())))?;

                Ok(db_to_thread(&db))
            })
            .await
            .map_err(|e| AiError::Core(e))?;

        Ok(result)
    }

    fn get_thread(&self, thread_id: &str) -> ChatRepositoryResult<Option<ChatThread>> {
        let mut conn = get_connection(&self.pool)
            .map_err(|e| AiError::Core(CoreError::Database(DatabaseError::Internal(e.to_string()))))?;

        let result = ai_threads::table
            .find(thread_id)
            .first::<AiThreadDB>(&mut conn)
            .optional()
            .map_err(|e| AiError::Core(CoreError::Database(DatabaseError::Internal(e.to_string()))))?;

        Ok(result.map(|db| db_to_thread(&db)))
    }

    fn list_threads(&self, limit: i64, offset: i64) -> ChatRepositoryResult<Vec<ChatThread>> {
        let mut conn = get_connection(&self.pool)
            .map_err(|e| AiError::Core(CoreError::Database(DatabaseError::Internal(e.to_string()))))?;

        let threads_db = ai_threads::table
            .order((ai_threads::is_pinned.desc(), ai_threads::updated_at.desc()))
            .limit(limit)
            .offset(offset)
            .load::<AiThreadDB>(&mut conn)
            .map_err(|e| AiError::Core(CoreError::Database(DatabaseError::Internal(e.to_string()))))?;

        let mut threads: Vec<ChatThread> = Vec::with_capacity(threads_db.len());
        for db in threads_db {
            let mut thread = db_to_thread(&db);
            thread.tags = ai_thread_tags::table
                .filter(ai_thread_tags::thread_id.eq(&db.id))
                .select(ai_thread_tags::tag)
                .load::<String>(&mut conn)
                .unwrap_or_default();
            threads.push(thread);
        }
        Ok(threads)
    }

    fn list_threads_paginated(
        &self,
        request: &ListThreadsRequest,
    ) -> ChatRepositoryResult<ThreadPage> {
        let mut conn = get_connection(&self.pool)
            .map_err(|e| AiError::Core(CoreError::Database(DatabaseError::Internal(e.to_string()))))?;

        let limit = request.limit.unwrap_or(20).min(100) as i64;
        let mut query = ai_threads::table.into_boxed();

        if let Some(search) = request
            .search
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            let search_pattern = format!("%{}%", search);
            query = query.filter(ai_threads::title.like(search_pattern));
        }

        if let Some(cursor) = &request.cursor {
            let (cursor_pinned, cursor_updated_at, cursor_id) = parse_cursor(cursor)?;
            query = query.filter(
                ai_threads::is_pinned
                    .lt(cursor_pinned)
                    .or(ai_threads::is_pinned
                        .eq(cursor_pinned)
                        .and(ai_threads::updated_at.lt(cursor_updated_at.clone())))
                    .or(ai_threads::is_pinned
                        .eq(cursor_pinned)
                        .and(ai_threads::updated_at.eq(cursor_updated_at))
                        .and(ai_threads::id.lt(cursor_id))),
            );
        }

        query = query.order((
            ai_threads::is_pinned.desc(),
            ai_threads::updated_at.desc(),
            ai_threads::id.desc(),
        ));

        let threads_db = query
            .limit(limit + 1)
            .load::<AiThreadDB>(&mut conn)
            .map_err(|e| AiError::Core(CoreError::Database(DatabaseError::Internal(e.to_string()))))?;

        let has_more = threads_db.len() > limit as usize;
        let threads_db: Vec<_> = threads_db.into_iter().take(limit as usize).collect();

        let mut threads: Vec<ChatThread> = Vec::with_capacity(threads_db.len());
        for db in &threads_db {
            let mut thread = db_to_thread(db);
            thread.tags = ai_thread_tags::table
                .filter(ai_thread_tags::thread_id.eq(&db.id))
                .select(ai_thread_tags::tag)
                .load::<String>(&mut conn)
                .unwrap_or_default();
            threads.push(thread);
        }

        let next_cursor = if has_more {
            threads_db
                .last()
                .map(|t| encode_cursor(t.is_pinned, &t.updated_at, &t.id))
        } else {
            None
        };

        Ok(ThreadPage {
            threads,
            next_cursor,
            has_more,
        })
    }

    async fn update_thread(&self, thread: ChatThread) -> ChatRepositoryResult<ChatThread> {
        let thread_id = thread.id.clone();
        let title = thread.title.clone();
        let is_pinned: i32 = if thread.is_pinned { 1 } else { 0 };
        let config_snapshot = thread
            .config
            .as_ref()
            .and_then(|c| serde_json::to_string(c).ok());
        let updated_at = Utc::now().to_rfc3339();

        let result = self.writer
            .exec(move |conn| -> Result<ChatThread, CoreError> {
                diesel::update(ai_threads::table.find(&thread_id))
                    .set((
                        ai_threads::title.eq(&title),
                        ai_threads::is_pinned.eq(&is_pinned),
                        ai_threads::config_snapshot.eq(&config_snapshot),
                        ai_threads::updated_at.eq(&updated_at),
                    ))
                    .execute(conn)
                    .map_err(|e| CoreError::Database(DatabaseError::Internal(e.to_string())))?;

                let db = ai_threads::table
                    .find(&thread_id)
                    .first::<AiThreadDB>(conn)
                    .map_err(|e| CoreError::Database(DatabaseError::Internal(e.to_string())))?;

                Ok(db_to_thread(&db))
            })
            .await
            .map_err(|e| AiError::Core(e))?;

        Ok(result)
    }

    async fn delete_thread(&self, thread_id: &str) -> ChatRepositoryResult<()> {
        let thread_id = thread_id.to_string();
        self.writer
            .exec(move |conn| -> Result<(), CoreError> {
                diesel::delete(ai_threads::table.find(&thread_id))
                    .execute(conn)
                    .map_err(|e| CoreError::Database(DatabaseError::Internal(e.to_string())))?;
                Ok(())
            })
            .await
            .map_err(|e| AiError::Core(e))?;

        Ok(())
    }

    async fn create_message(&self, message: ChatMessage) -> ChatRepositoryResult<ChatMessage> {
        let message_db = message_to_db(&message)?;
        let message_id = message_db.id.clone();
        let thread_id = message_db.thread_id.clone();

        let result = self.writer
            .exec(move |conn| -> Result<ChatMessage, CoreError> {
                diesel::insert_into(ai_messages::table)
                    .values(&message_db)
                    .execute(conn)
                    .map_err(|e| CoreError::Database(DatabaseError::Internal(e.to_string())))?;

                diesel::update(ai_threads::table.find(&thread_id))
                    .set(ai_threads::updated_at.eq(chrono::Utc::now().to_rfc3339()))
                    .execute(conn)
                    .map_err(|e| CoreError::Database(DatabaseError::Internal(e.to_string())))?;

                let db = ai_messages::table
                    .find(&message_id)
                    .first::<AiMessageDB>(conn)
                    .map_err(|e| CoreError::Database(DatabaseError::Internal(e.to_string())))?;

                db_to_message(&db).map_err(|e| CoreError::Database(DatabaseError::Internal(e.to_string())))
            })
            .await
            .map_err(|e| AiError::Core(e))?;

        Ok(result)
    }

    fn get_message(&self, message_id: &str) -> ChatRepositoryResult<Option<ChatMessage>> {
        let mut conn = get_connection(&self.pool)
            .map_err(|e| AiError::Core(CoreError::Database(DatabaseError::Internal(e.to_string()))))?;

        let result = ai_messages::table
            .find(message_id)
            .first::<AiMessageDB>(&mut conn)
            .optional()
            .map_err(|e| AiError::Core(CoreError::Database(DatabaseError::Internal(e.to_string()))))?;

        match result {
            Some(db) => Ok(Some(db_to_message(&db)?)),
            None => Ok(None),
        }
    }

    fn get_messages_by_thread(&self, thread_id: &str) -> ChatRepositoryResult<Vec<ChatMessage>> {
        let mut conn = get_connection(&self.pool)
            .map_err(|e| AiError::Core(CoreError::Database(DatabaseError::Internal(e.to_string()))))?;

        let messages_db = ai_messages::table
            .filter(ai_messages::thread_id.eq(thread_id))
            .order(ai_messages::created_at.asc())
            .load::<AiMessageDB>(&mut conn)
            .map_err(|e| AiError::Core(CoreError::Database(DatabaseError::Internal(e.to_string()))))?;

        messages_db
            .iter()
            .map(db_to_message)
            .collect::<ChatRepositoryResult<Vec<_>>>()
    }

    async fn update_message(&self, message: ChatMessage) -> ChatRepositoryResult<ChatMessage> {
        let message_id = message.id.clone();
        let content_json = convert_content_to_json(&message.content)?;

        let result = self.writer
            .exec(move |conn| -> Result<ChatMessage, CoreError> {
                diesel::update(ai_messages::table.find(&message_id))
                    .set(ai_messages::content_json.eq(&content_json))
                    .execute(conn)
                    .map_err(|e| CoreError::Database(DatabaseError::Internal(e.to_string())))?;

                let db = ai_messages::table
                    .find(&message_id)
                    .first::<AiMessageDB>(conn)
                    .map_err(|e| CoreError::Database(DatabaseError::Internal(e.to_string())))?;

                db_to_message(&db).map_err(|e| CoreError::Database(DatabaseError::Internal(e.to_string())))
            })
            .await
            .map_err(|e| AiError::Core(e))?;

        Ok(result)
    }

    async fn add_tag(&self, thread_id: &str, tag: &str) -> ChatRepositoryResult<()> {
        let tag_db = AiThreadTagDB {
            id: uuid::Uuid::new_v4().to_string(),
            thread_id: thread_id.to_string(),
            tag: tag.to_string(),
            created_at: Utc::now().to_rfc3339(),
        };

        self.writer
            .exec(move |conn| -> Result<(), CoreError> {
                diesel::insert_into(ai_thread_tags::table)
                    .values(&tag_db)
                    .on_conflict((ai_thread_tags::thread_id, ai_thread_tags::tag))
                    .do_nothing()
                    .execute(conn)
                    .map_err(|e| CoreError::Database(DatabaseError::Internal(e.to_string())))?;
                Ok(())
            })
            .await
            .map_err(|e| AiError::Core(e))?;

        Ok(())
    }

    async fn remove_tag(&self, thread_id: &str, tag: &str) -> ChatRepositoryResult<()> {
        let thread_id = thread_id.to_string();
        let tag = tag.to_string();

        self.writer
            .exec(move |conn| -> Result<(), CoreError> {
                diesel::delete(
                    ai_thread_tags::table
                        .filter(ai_thread_tags::thread_id.eq(&thread_id))
                        .filter(ai_thread_tags::tag.eq(&tag)),
                )
                .execute(conn)
                .map_err(|e| CoreError::Database(DatabaseError::Internal(e.to_string())))?;
                Ok(())
            })
            .await
            .map_err(|e| AiError::Core(e))?;

        Ok(())
    }

    fn get_tags(&self, thread_id: &str) -> ChatRepositoryResult<Vec<String>> {
        let mut conn = get_connection(&self.pool)
            .map_err(|e| AiError::Core(CoreError::Database(DatabaseError::Internal(e.to_string()))))?;

        ai_thread_tags::table
            .filter(ai_thread_tags::thread_id.eq(thread_id))
            .select(ai_thread_tags::tag)
            .load::<String>(&mut conn)
            .map_err(|e| AiError::Core(CoreError::Database(DatabaseError::Internal(e.to_string()))))
    }
}

// ============================================================================
// Cursor Helpers
// ============================================================================

fn parse_cursor(cursor: &str) -> ChatRepositoryResult<(i32, String, String)> {
    let parts: Vec<&str> = cursor.splitn(3, ':').collect();
    if parts.len() != 3 {
        return Err(AiError::InvalidCursor(format!(
            "Expected format 'is_pinned:updated_at:id', got '{}'",
            cursor
        )));
    }
    let is_pinned: i32 = parts[0]
        .parse()
        .map_err(|_| AiError::InvalidCursor(format!("Invalid is_pinned value: {}", parts[0])))?;
    Ok((is_pinned, parts[1].to_string(), parts[2].to_string()))
}

fn encode_cursor(is_pinned: i32, updated_at: &str, id: &str) -> String {
    format!("{}:{}:{}", is_pinned, updated_at, id)
}

// ============================================================================
// Conversion Functions
// ============================================================================

fn thread_to_db(thread: &ChatThread) -> AiThreadDB {
    let config_snapshot = thread
        .config
        .as_ref()
        .and_then(|c| serde_json::to_string(c).ok());
    AiThreadDB {
        id: thread.id.clone(),
        title: thread.title.clone(),
        created_at: thread.created_at.to_rfc3339(),
        updated_at: thread.updated_at.to_rfc3339(),
        config_snapshot,
        is_pinned: if thread.is_pinned { 1 } else { 0 },
    }
}

fn db_to_thread(db: &AiThreadDB) -> ChatThread {
    let config = db
        .config_snapshot
        .as_ref()
        .and_then(|json| serde_json::from_str::<ChatThreadConfig>(json).ok());
    ChatThread {
        id: db.id.clone(),
        title: db.title.clone(),
        is_pinned: db.is_pinned != 0,
        tags: Vec::new(),
        config,
        created_at: DateTime::parse_from_rfc3339(&db.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        updated_at: DateTime::parse_from_rfc3339(&db.updated_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
    }
}

fn message_to_db(msg: &ChatMessage) -> ChatRepositoryResult<AiMessageDB> {
    let content_json = convert_content_to_json(&msg.content)?;
    Ok(AiMessageDB {
        id: msg.id.clone(),
        thread_id: msg.thread_id.clone(),
        role: msg.role.to_string(),
        content_json,
        created_at: msg.created_at.to_rfc3339(),
    })
}

fn db_to_message(db: &AiMessageDB) -> ChatRepositoryResult<ChatMessage> {
    let content = convert_json_to_content(&db.content_json)?;
    let role = db
        .role
        .parse::<ChatMessageRole>()
        .map_err(AiError::InvalidInput)?;
    Ok(ChatMessage {
        id: db.id.clone(),
        thread_id: db.thread_id.clone(),
        role,
        content,
        created_at: DateTime::parse_from_rfc3339(&db.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
    })
}

fn convert_content_to_json(content: &ChatMessageContent) -> ChatRepositoryResult<String> {
    let storage_parts: Vec<MessagePart> = content
        .parts
        .iter()
        .map(|p| match p {
            ChatMessagePart::System { content } => MessagePart::System {
                content: content.clone(),
            },
            ChatMessagePart::Text { content } => MessagePart::Text {
                content: content.clone(),
            },
            ChatMessagePart::Reasoning { content } => MessagePart::Reasoning {
                content: content.clone(),
            },
            ChatMessagePart::ToolCall {
                tool_call_id,
                name,
                arguments,
            } => MessagePart::ToolCall {
                tool_call_id: tool_call_id.clone(),
                name: name.clone(),
                arguments: arguments.clone(),
            },
            ChatMessagePart::ToolResult {
                tool_call_id,
                success,
                data,
                meta,
                error,
            } => MessagePart::ToolResult {
                tool_call_id: tool_call_id.clone(),
                success: *success,
                data: data.clone(),
                meta: meta.clone(),
                error: error.clone(),
            },
            ChatMessagePart::Error { code, message } => MessagePart::Error {
                code: code.clone(),
                message: message.clone(),
            },
        })
        .collect();

    let storage_content = MessageContent {
        schema_version: content.schema_version,
        parts: storage_parts,
        truncated: content.truncated,
    };

    storage_content
        .to_json_with_limit(CHAT_MAX_CONTENT_SIZE_BYTES)
        .map_err(|e| AiError::InvalidInput(e.to_string()))
}

fn convert_json_to_content(json: &str) -> ChatRepositoryResult<ChatMessageContent> {
    let storage_content =
        MessageContent::from_json(json).map_err(|e| AiError::InvalidInput(e.to_string()))?;

    let core_parts: Vec<ChatMessagePart> = storage_content
        .parts
        .into_iter()
        .map(|p| match p {
            MessagePart::System { content } => ChatMessagePart::System { content },
            MessagePart::Text { content } => ChatMessagePart::Text { content },
            MessagePart::Reasoning { content } => ChatMessagePart::Reasoning { content },
            MessagePart::ToolCall {
                tool_call_id,
                name,
                arguments,
            } => ChatMessagePart::ToolCall {
                tool_call_id,
                name,
                arguments,
            },
            MessagePart::ToolResult {
                tool_call_id,
                success,
                data,
                meta,
                error,
            } => ChatMessagePart::ToolResult {
                tool_call_id,
                success,
                data,
                meta,
                error,
            },
            MessagePart::Error { code, message } => ChatMessagePart::Error { code, message },
        })
        .collect();

    Ok(ChatMessageContent {
        schema_version: storage_content.schema_version,
        parts: core_parts,
        truncated: storage_content.truncated,
    })
}
