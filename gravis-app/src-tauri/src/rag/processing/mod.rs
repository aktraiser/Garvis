// Document processing and classification
pub mod document_processor;
pub mod document_classifier;
pub mod smart_chunker;
pub mod business_metadata;
pub mod span_aware_chunker;
// Phase 3: Vision-Aware RAG
pub mod figure_detector;
pub mod figure_ocr;
pub mod figure_chunk_builder;

pub use document_processor::*;
pub use document_classifier::*;
pub use smart_chunker::*;
pub use business_metadata::*;
pub use span_aware_chunker::*;
pub use figure_detector::*;
pub use figure_ocr::*;
pub use figure_chunk_builder::*;