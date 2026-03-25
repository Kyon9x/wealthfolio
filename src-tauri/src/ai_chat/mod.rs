//! AI chat module for Tauri.
//!
//! This module provides database-backed chat functionality for the AI assistant.

pub mod repository;

// Re-export the repository with a shorter type alias for convenience
pub use repository::AiChatRepository as ChatRepository;
