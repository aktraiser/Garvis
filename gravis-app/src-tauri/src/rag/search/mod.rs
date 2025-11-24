// Search and retrieval components
pub mod search_optimizer;
pub mod mmr_reranker;
pub mod custom_e5;
pub mod enhanced_bm25;
pub mod scoring_engine;
pub mod numerical_reranker;
pub mod query_aware_reranker;  // Sprint 1 Niveau 1.5: Query-aware reranking
pub mod section_prior;  // AUDIT 22 NOV: Section prior simple et générique

pub use search_optimizer::*;
pub use mmr_reranker::*;
pub use custom_e5::*;
pub use enhanced_bm25::*;
pub use scoring_engine::*;
pub use numerical_reranker::*;
pub use query_aware_reranker::*;
pub use section_prior::*;