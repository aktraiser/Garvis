// Core RAG components
pub mod embedder_manager;
pub mod ingestion_engine;
pub mod unified_cache;
pub mod qdrant_rest;
pub mod source_spans;

// Phase 2: Chat Direct modules
pub mod direct_chat;
pub mod direct_chat_manager;

#[cfg(test)]
mod source_spans_integration_test;

pub use embedder_manager::*;
pub use ingestion_engine::*;
pub use unified_cache::*;
pub use qdrant_rest::*;
pub use source_spans::*;

// Phase 2: Chat Direct exports
pub use direct_chat::*;
pub use direct_chat_manager::*;