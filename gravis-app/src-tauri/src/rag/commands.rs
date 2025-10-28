// GRAVIS RAG Commands Phase 3 - Interface Tauri Unifiée
// Commandes RAG + OCR + Classification avec métadonnées enrichies

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tauri::State;
use tokio::sync::RwLock;
use anyhow::Result;
use tracing::info;

use crate::rag::{
    DocumentGroup, DocumentCategory, BusinessMetadata, SourceType, ExtractionMethod,
    DocumentProcessor, IngestionEngine, DocumentClassifier, BusinessMetadataEnricher,
    UnifiedCache, QdrantRestClient, CustomE5Embedder, CustomE5Config, QdrantRestConfig,
    OcrCache, CacheConfig, TesseractConfig, GroupDocument, RagError
};

/// État unifié RAG Phase 3 avec OCR et Classification
#[derive(Clone)]
pub struct RagState {
    pub ingestion_engine: Arc<IngestionEngine>,
    pub document_classifier: Arc<DocumentClassifier>,
    pub business_enricher: Arc<BusinessMetadataEnricher>,
    pub embedder: Arc<CustomE5Embedder>,
    pub qdrant_client: Arc<QdrantRestClient>,
    pub unified_cache: Arc<UnifiedCache>,
    pub groups: Arc<RwLock<HashMap<String, DocumentGroup>>>,
}

impl RagState {
    pub async fn new() -> Result<Self, RagError> {
        info!("Initializing RAG State Phase 3 with Universal Pipeline");

        // Initialiser les composants
        let embedder = Arc::new(
            CustomE5Embedder::new(CustomE5Config::default())
                .await
                .map_err(|e| RagError::InvalidConfig(format!("Embedder init failed: {}", e)))?
        );

        let qdrant_client = Arc::new(
            QdrantRestClient::new(QdrantRestConfig::default())
                .map_err(|e| RagError::InvalidConfig(format!("Qdrant init failed: {}", e)))?
        );

        let ocr_cache = OcrCache::new(CacheConfig::default()).await
            .map_err(|e| RagError::InvalidConfig(format!("OCR cache init failed: {}", e)))?;
        let unified_cache = Arc::new(UnifiedCache::new(ocr_cache));
        
        // Composants Phase 3A
        let document_classifier = Arc::new(DocumentClassifier::new());
        let business_enricher = Arc::new(BusinessMetadataEnricher::new());

        // Document processor avec OCR
        let ocr_processor = crate::rag::ocr::TesseractProcessor::new(TesseractConfig::default())
            .await
            .map_err(|e| RagError::InvalidConfig(format!("OCR init failed: {}", e)))?;
        
        let document_processor = DocumentProcessor::new(ocr_processor, embedder.clone())
            .await
            .map_err(|e| RagError::InvalidConfig(format!("DocumentProcessor init failed: {}", e)))?;

        let ingestion_engine = Arc::new(
            IngestionEngine::new(document_processor)
        );

        Ok(Self {
            ingestion_engine,
            document_classifier,
            business_enricher,
            embedder: embedder.clone(),
            qdrant_client,
            unified_cache,
            groups: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}

/// Réponse d'ingestion de document avec métadonnées enrichies
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DocumentIngestionResponse {
    pub document_id: String,
    pub document_category: DocumentCategory,
    pub chunks_created: usize,
    pub extraction_method: ExtractionMethod,
    pub source_type: SourceType,
    pub processing_time_ms: u64,
    pub business_metadata: Option<BusinessMetadata>,
    pub cache_stats: CacheStats,
    pub confidence_score: f32,
}

/// Statistiques de cache
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CacheStats {
    pub ocr_cache_hits: u64,
    pub embedding_cache_hits: u64,
    pub document_cache_hits: u64,
    pub total_hits: u64,
    pub hit_ratio: f32,
}

/// Réponse de recherche avec métadonnées enrichies
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResponseWithMetadata {
    pub results: Vec<SearchResultWithMetadata>,
    pub total_results: usize,
    pub search_time_ms: u64,
    pub query_embedding_time_ms: u64,
}

/// Résultat de recherche individuel avec métadonnées
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResultWithMetadata {
    pub chunk_id: String,
    pub content: String,
    pub score: f32,
    pub document_id: String,
    pub document_category: DocumentCategory,
    pub source_type: SourceType,
    pub extraction_method: ExtractionMethod,
    pub business_metadata: Option<BusinessMetadata>,
    pub ocr_confidence: Option<f32>,
    pub chunk_metadata: ChunkMetadataSlim,
}

/// Métadonnées de chunk simplifiées pour l'API
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChunkMetadataSlim {
    pub tags: Vec<String>,
    pub language: String,
    pub confidence: f32,
    pub start_line: usize,
    pub end_line: usize,
}

/// Paramètres de recherche avancée
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AdvancedSearchParams {
    pub query: String,
    pub group_id: String,
    pub limit: Option<usize>,
    pub min_score: Option<f32>,
    pub document_categories: Option<Vec<DocumentCategory>>,
    pub source_types: Option<Vec<SourceType>>,
    pub min_ocr_confidence: Option<f32>,
    pub include_business_metadata: bool,
    pub fiscal_year_filter: Option<i32>,
}

// === Commandes Tauri Phase 3 ===

/// Ajouter un document avec classification automatique et extraction intelligente
#[tauri::command]
pub async fn add_document_intelligent(
    file_path: String,
    group_id: String,
    _force_ocr: Option<bool>,
    state: State<'_, RagState>,
) -> Result<DocumentIngestionResponse, String> {
    let start_time = std::time::Instant::now();
    info!("Adding document intelligently: {} to group {}", file_path, group_id);

    let path = PathBuf::from(file_path);
    
    // Vérifier que le groupe existe
    let groups = state.groups.read().await;
    let group = groups.get(&group_id)
        .ok_or_else(|| format!("Group not found: {}", group_id))?;
    let chunk_config = group.chunk_config.clone();
    drop(groups);

    // Processing intelligent avec classification automatique
    let document = state.ingestion_engine
        .ingest_document(&path, &group_id, &chunk_config)
        .await
        .map_err(|e| format!("Document processing failed: {}", e))?;

    // Classification automatique du contenu
    let document_category = state.document_classifier
        .classify(&document.document.content)
        .map_err(|e| format!("Classification failed: {}", e))?;

    // Enrichissement métadonnées Business si applicable
    let business_metadata = if matches!(document_category, DocumentCategory::Business) {
        Some(
            state.business_enricher
                .enrich_business_content(&document.document.content, None, None)
                .map_err(|e| format!("Business enrichment failed: {}", e))?
        )
    } else {
        None
    };

    // Calcul des statistiques
    let processing_time = start_time.elapsed().as_millis() as u64;
    let cache_stats = get_cache_statistics(&state).await;
    
    // Confiance globale basée sur extraction + classification
    let confidence_score = calculate_global_confidence(&document.document, &business_metadata);

    // Mettre à jour le groupe avec le nouveau document
    let mut groups = state.groups.write().await;
    if let Some(group) = groups.get_mut(&group_id) {
        group.documents.push(document.document.clone());
        group.updated_at = SystemTime::now();
    }

    info!("Document processed successfully: {} chunks, category: {:?}, confidence: {:.3}", 
          document.document.chunks.len(), document_category, confidence_score);

    Ok(DocumentIngestionResponse {
        document_id: document.document.id,
        document_category,
        chunks_created: document.document.chunks.len(),
        extraction_method: if let Some(_) = document.document.chunks.get(0)
            .and_then(|chunk| chunk.metadata.ocr_metadata.as_ref()) {
            ExtractionMethod::TesseractOcr { 
                confidence: 0.8, // Default confidence
                language: "fra+eng".to_string() // Default language
            }
        } else {
            ExtractionMethod::DirectRead
        },
        source_type: match &document.document.document_type {
            crate::rag::DocumentType::PDF { extraction_strategy, .. } => {
                match extraction_strategy {
                    crate::rag::PdfStrategy::NativeOnly => SourceType::NativeText,
                    crate::rag::PdfStrategy::OcrOnly => SourceType::OcrExtracted,
                    crate::rag::PdfStrategy::HybridIntelligent => SourceType::HybridPdfNative,
                }
            },
            crate::rag::DocumentType::Image { .. } => SourceType::OcrExtracted,
            _ => SourceType::NativeText,
        },
        processing_time_ms: processing_time,
        business_metadata,
        cache_stats,
        confidence_score,
    })
}

/// Recherche avancée avec filtres de métadonnées
#[tauri::command]
pub async fn search_with_metadata(
    params: AdvancedSearchParams,
    state: State<'_, RagState>,
) -> Result<SearchResponseWithMetadata, String> {
    let start_time = std::time::Instant::now();
    info!("Advanced search with metadata: '{}' in group {}", params.query, params.group_id);

    // Générer embedding de la requête
    let embedding_start = std::time::Instant::now();
    let query_embedding = state.embedder
        .encode(&params.query)
        .await
        .map_err(|e| format!("Query embedding failed: {}", e))?;
    let query_embedding_time = embedding_start.elapsed().as_millis() as u64;

    // Récupérer le groupe
    let groups = state.groups.read().await;
    let group = groups.get(&params.group_id)
        .ok_or_else(|| format!("Group not found: {}", params.group_id))?;

    // Filtrer et scorer les chunks selon les critères
    let mut results = Vec::new();
    
    for document in &group.documents {
        // Filtrer par catégorie si spécifié
        if let Some(ref categories) = params.document_categories {
            let doc_category = state.document_classifier
                .classify(&document.content)
                .unwrap_or(DocumentCategory::Mixed);
            
            if !categories.contains(&doc_category) {
                continue;
            }
        }

        // Filtrer par source type si spécifié
        if let Some(ref source_types) = params.source_types {
            let doc_source_type = match &document.document_type {
                crate::rag::DocumentType::PDF { extraction_strategy, .. } => {
                    match extraction_strategy {
                        crate::rag::PdfStrategy::NativeOnly => SourceType::NativeText,
                        crate::rag::PdfStrategy::OcrOnly => SourceType::OcrExtracted,
                        crate::rag::PdfStrategy::HybridIntelligent => SourceType::HybridPdfNative,
                    }
                },
                _ => SourceType::NativeText,
            };
            
            if !source_types.contains(&doc_source_type) {
                continue;
            }
        }

        // Traiter les chunks du document
        for chunk in &document.chunks {
            if let Some(ref embedding) = chunk.embedding {
                let score = cosine_similarity(&query_embedding, embedding);
                
                // Filtrer par score minimum
                if let Some(min_score) = params.min_score {
                    if score < min_score {
                        continue;
                    }
                }

                // TODO: Filtrer par confiance OCR quand disponible dans OcrMetadata
                // let _min_ocr_conf = params.min_ocr_confidence;

                // Enrichir avec métadonnées Business si demandé
                let business_metadata = if params.include_business_metadata {
                    let doc_category = state.document_classifier
                        .classify(&chunk.content)
                        .unwrap_or(DocumentCategory::Mixed);
                    
                    if matches!(doc_category, DocumentCategory::Business) {
                        state.business_enricher
                            .enrich_business_content(&chunk.content, params.fiscal_year_filter, None)
                            .ok()
                    } else {
                        None
                    }
                } else {
                    None
                };

                let result = SearchResultWithMetadata {
                    chunk_id: chunk.id.clone(),
                    content: chunk.content.clone(),
                    score,
                    document_id: document.id.clone(),
                    document_category: state.document_classifier
                        .classify(&document.content)
                        .unwrap_or(DocumentCategory::Mixed),
                    source_type: match &document.document_type {
                        crate::rag::DocumentType::PDF { extraction_strategy, .. } => {
                            match extraction_strategy {
                                crate::rag::PdfStrategy::NativeOnly => SourceType::NativeText,
                                crate::rag::PdfStrategy::OcrOnly => SourceType::OcrExtracted,
                                crate::rag::PdfStrategy::HybridIntelligent => SourceType::HybridPdfNative,
                            }
                        },
                        _ => SourceType::NativeText,
                    },
                    extraction_method: if chunk.metadata.ocr_metadata.is_some() {
                        ExtractionMethod::TesseractOcr { 
                            confidence: 0.8, // Default confidence
                            language: "fra+eng".to_string() // Default language
                        }
                    } else {
                        ExtractionMethod::DirectRead
                    },
                    business_metadata,
                    ocr_confidence: if chunk.metadata.ocr_metadata.is_some() {
                        Some(0.8)
                    } else {
                        None
                    },
                    chunk_metadata: ChunkMetadataSlim {
                        tags: chunk.metadata.tags.clone(),
                        language: chunk.metadata.language.clone(),
                        confidence: chunk.metadata.confidence,
                        start_line: chunk.start_line,
                        end_line: chunk.end_line,
                    },
                };

                results.push(result);
            }
        }
    }

    // Trier par score et limiter
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    
    let limit = params.limit.unwrap_or(10);
    results.truncate(limit);

    let search_time = start_time.elapsed().as_millis() as u64;

    info!("Search completed: {} results in {}ms", results.len(), search_time);

    Ok(SearchResponseWithMetadata {
        total_results: results.len(),
        results,
        search_time_ms: search_time,
        query_embedding_time_ms: query_embedding_time,
    })
}

/// Obtenir les métadonnées enrichies d'un document
#[tauri::command]
pub async fn get_document_metadata(
    document_id: String,
    group_id: String,
    state: State<'_, RagState>,
) -> Result<DocumentMetadataResponse, String> {
    info!("Getting metadata for document {} in group {}", document_id, group_id);

    let groups = state.groups.read().await;
    let group = groups.get(&group_id)
        .ok_or_else(|| format!("Group not found: {}", group_id))?;

    let document = group.documents
        .iter()
        .find(|doc| doc.id == document_id)
        .ok_or_else(|| format!("Document not found: {}", document_id))?;

    // Classification et enrichissement
    let document_category = state.document_classifier
        .classify(&document.content)
        .map_err(|e| format!("Classification failed: {}", e))?;

    let business_metadata = if matches!(document_category, DocumentCategory::Business) {
        Some(
            state.business_enricher
                .enrich_business_content(&document.content, None, None)
                .map_err(|e| format!("Business enrichment failed: {}", e))?
        )
    } else {
        None
    };

    Ok(DocumentMetadataResponse {
        document_id: document.id.clone(),
        document_category,
        chunks_count: document.chunks.len(),
        total_characters: document.content.len(),
        language: document.language.clone(),
        business_metadata,
        processing_metadata: document.metadata.clone(),
    })
}

/// Réponse métadonnées document
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DocumentMetadataResponse {
    pub document_id: String,
    pub document_category: DocumentCategory,
    pub chunks_count: usize,
    pub total_characters: usize,
    pub language: String,
    pub business_metadata: Option<BusinessMetadata>,
    pub processing_metadata: crate::rag::EnrichedMetadata,
}

// === Fonctions utilitaires ===

async fn get_cache_statistics(_state: &RagState) -> CacheStats {
    // TODO: Implémenter vraies statistiques depuis UnifiedCache
    CacheStats {
        ocr_cache_hits: 0,
        embedding_cache_hits: 0,
        document_cache_hits: 0,
        total_hits: 0,
        hit_ratio: 0.0,
    }
}

fn calculate_global_confidence(document: &GroupDocument, business_metadata: &Option<BusinessMetadata>) -> f32 {
    let mut confidence = 0.0;
    let mut factors = 0;

    // Confiance OCR moyenne (temporaire sans métadonnées OCR)
    let has_ocr = document.chunks.iter()
        .any(|chunk| chunk.metadata.ocr_metadata.is_some());
    
    if has_ocr {
        confidence += 0.8; // Default OCR confidence
        factors += 1;
    } else {
        confidence += 1.0; // Texte natif = confiance maximale
        factors += 1;
    }

    // Confiance métadonnées Business
    if let Some(ref business) = business_metadata {
        confidence += business.confidence_score;
        factors += 1;
    }

    // Confiance chunks moyenne
    if !document.chunks.is_empty() {
        let avg_chunk_confidence: f32 = document.chunks
            .iter()
            .map(|chunk| chunk.metadata.confidence)
            .sum::<f32>() / document.chunks.len() as f32;
        confidence += avg_chunk_confidence;
        factors += 1;
    }

    if factors > 0 {
        confidence / factors as f32
    } else {
        0.0
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}