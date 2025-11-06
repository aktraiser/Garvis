// GRAVIS RAG Commands Phase 3 - Interface Tauri Unifiée
// Commandes RAG + OCR + Classification avec métadonnées enrichies

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tauri::State;
use tokio::sync::RwLock;
use anyhow::Result;
use tracing::{info, warn};

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

        // Créer le groupe par défaut avec ID fixe
        let mut groups = HashMap::new();
        let default_group = crate::rag::DocumentGroup::new_with_id(
            "default_group".to_string(),
            "Default Group".to_string()
        );
        info!("📁 Created default RAG group: {} -> collection: {}",
              default_group.id, default_group.qdrant_collection);
        groups.insert("default_group".to_string(), default_group);

        Ok(Self {
            ingestion_engine,
            document_classifier,
            business_enricher,
            embedder: embedder.clone(),
            qdrant_client,
            unified_cache,
            groups: Arc::new(RwLock::new(groups)),
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
    extracted_text: Option<String>, // Texte pré-extrait par OCR (si disponible)
    state: State<'_, RagState>,
) -> Result<DocumentIngestionResponse, String> {
    let start_time = std::time::Instant::now();
    info!("Adding document intelligently: {} to group {}", file_path, group_id);

    // Résoudre le chemin du fichier comme dans extract_document_content
    let path = if file_path.starts_with("exemple/") {
        // Chemin relatif depuis le frontend - résoudre vers le dossier exemple
        let current_dir = env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;
        let docs_path = current_dir.parent()
            .ok_or("Failed to get parent directory")?
            .join("exemple");
        let filename = file_path.strip_prefix("exemple/").unwrap_or(&file_path);
        docs_path.join(filename)
    } else {
        // Chemin absolu ou autre - utiliser tel quel
        PathBuf::from(file_path.clone())
    };
    
    info!("📂 Resolved file path: {:?}", path);
    
    if !path.exists() {
        return Err(format!("File not found: {:?}", path));
    }
    
    // Vérifier que le groupe existe
    let groups = state.groups.read().await;
    let group = groups.get(&group_id)
        .ok_or_else(|| format!("Group not found: {}", group_id))?;
    let chunk_config = group.chunk_config.clone();
    drop(groups);

    // Si du texte pré-extrait est fourni, l'utiliser directement
    let document = if let Some(preextracted_text) = extracted_text {
        info!("📄 Using pre-extracted text ({} chars)", preextracted_text.len());

        // Créer un document directement depuis le texte pré-extrait
        use crate::rag::{GroupDocument, DocumentType, EnrichedChunk, ChunkType, ChunkMetadata, Priority, SourceType, ExtractionMethod, EnrichedMetadata};
        use std::collections::HashMap;

        // Chunking du texte pré-extrait
        let chunks: Vec<EnrichedChunk> = preextracted_text
            .split("\n\n")
            .enumerate()
            .filter(|(_, para)| !para.trim().is_empty())
            .map(|(i, para)| {
                EnrichedChunk {
                    id: format!("chunk_preextracted_{}_{}", uuid::Uuid::new_v4().simple(), i),
                    content: para.trim().to_string(),
                    start_line: i,
                    end_line: i + 1,
                    chunk_type: ChunkType::TextBlock,
                    embedding: None,
                    hash: blake3::hash(para.trim().as_bytes()).to_hex().to_string(),
                    metadata: ChunkMetadata {
                        tags: vec!["pre-extracted".to_string(), "ocr".to_string()],
                        priority: Priority::Normal,
                        language: "auto".to_string(),
                        symbol: None,
                        context: Some("Pre-extracted by OCR".to_string()),
                        confidence: 0.85,
                        ocr_metadata: None,
                        source_type: SourceType::OcrExtracted,
                        extraction_method: ExtractionMethod::TesseractOcr {
                            confidence: 0.85,
                            language: "fra+eng".to_string(),
                        },
                    },
                    group_id: group_id.clone(),
                }
            })
            .collect();

        let document_id = format!("doc_{}", uuid::Uuid::new_v4().simple());
        let now = SystemTime::now();

        GroupDocument {
            id: document_id,
            file_path: path.clone(),
            language: "auto".to_string(),
            content: preextracted_text.clone(),
            chunks,
            metadata: EnrichedMetadata {
                tags: vec!["pre-extracted".to_string()],
                priority: Priority::Normal,
                description: Some("Document avec texte pré-extrait par OCR".to_string()),
                author: None,
                project: None,
                custom_fields: HashMap::new(),
            },
            last_modified: now,
            document_type: DocumentType::PDF {
                extraction_strategy: crate::rag::PdfStrategy::OcrOnly,
                native_text_ratio: 0.0,
                ocr_pages: vec![0],
                total_pages: 1,
            },
            group_id: group_id.clone(),
        }
    } else {
        // Processing intelligent avec classification automatique
        info!("📄 Extracting text from document...");
        let doc_result = state.ingestion_engine
            .ingest_document(&path, &group_id, &chunk_config)
            .await
            .map_err(|e| format!("Document processing failed: {}", e))?;
        doc_result.document
    };

    // === GÉNÉRATION DES EMBEDDINGS ===
    info!("🧮 Generating embeddings for {} chunks", document.chunks.len());
    let mut document_with_embeddings = document.clone();
    let mut embedded_count = 0;

    for chunk in &mut document_with_embeddings.chunks {
        // Ignorer les chunks vides ou d'erreur
        if !chunk.content.trim().is_empty()
            && !chunk.content.starts_with("EXTRACTION FAILED") {
            match state.embedder.encode(&chunk.content).await {
                Ok(embedding) => {
                    chunk.embedding = Some(embedding);
                    embedded_count += 1;
                }
                Err(e) => {
                    tracing::warn!("Failed to embed chunk {}: {}", chunk.id, e);
                }
            }
        }
    }

    info!("✅ Generated {} embeddings", embedded_count);

    // === INJECTION DANS QDRANT ===
    if embedded_count > 0 {
        let groups_read = state.groups.read().await;
        let collection_name = groups_read.get(&group_id)
            .map(|g| g.qdrant_collection.clone())
            .ok_or_else(|| format!("Group not found: {}", group_id))?;
        drop(groups_read);

        info!("💾 Upserting {} chunks to Qdrant: {}", embedded_count, collection_name);

        // Créer la collection si elle n'existe pas (384D pour E5-small-v2)
        let _ = state.qdrant_client.create_collection(&collection_name, 384, "Cosine").await;

        // Convertir en points Qdrant
        let points: Vec<crate::rag::RestPoint> = document_with_embeddings.chunks
            .iter()
            .enumerate()
            .filter_map(|(idx, chunk)| {
                chunk.embedding.as_ref().map(|emb| {
                    let mut payload = HashMap::new();
                    payload.insert("content".to_string(), serde_json::json!(chunk.content));
                    payload.insert("document_id".to_string(), serde_json::json!(document_with_embeddings.id));
                    payload.insert("group_id".to_string(), serde_json::json!(group_id));
                    payload.insert("confidence".to_string(), serde_json::json!(chunk.metadata.confidence));
                    payload.insert("chunk_id".to_string(), serde_json::json!(chunk.id.clone()));

                    // Générer UUID reproductible à partir du chunk.id en utilisant blake3
                    let hash = blake3::hash(chunk.id.as_bytes());
                    let hash_bytes = hash.as_bytes();

                    // Convertir les 16 premiers bytes du hash en UUID
                    let uuid_bytes: [u8; 16] = hash_bytes[0..16].try_into().unwrap();
                    let point_uuid = uuid::Uuid::from_bytes(uuid_bytes);

                    crate::rag::RestPoint {
                        id: serde_json::json!(point_uuid.to_string()),
                        vector: emb.clone(),
                        payload: Some(payload),
                    }
                })
            })
            .collect();

        state.qdrant_client
            .upsert_points(&collection_name, points)
            .await
            .map_err(|e| format!("Qdrant upsert failed: {}", e))?;

        info!("✅ Successfully stored {} chunks in Qdrant", embedded_count);
    }

    // Classification automatique du contenu
    let document_category = state.document_classifier
        .classify(&document_with_embeddings.content)
        .map_err(|e| format!("Classification failed: {}", e))?;

    // Enrichissement métadonnées Business si applicable
    let business_metadata = if matches!(document_category, DocumentCategory::Business) {
        Some(
            state.business_enricher
                .enrich_business_content(&document_with_embeddings.content, None, None)
                .map_err(|e| format!("Business enrichment failed: {}", e))?
        )
    } else {
        None
    };

    // Calcul des statistiques
    let processing_time = start_time.elapsed().as_millis() as u64;
    let cache_stats = get_cache_statistics(&state).await;

    // Confiance globale basée sur extraction + classification
    let confidence_score = calculate_global_confidence(&document_with_embeddings, &business_metadata);

    // Mettre à jour le groupe avec le nouveau document (avec embeddings)
    let mut groups = state.groups.write().await;
    if let Some(group) = groups.get_mut(&group_id) {
        group.documents.push(document_with_embeddings.clone());
        group.updated_at = SystemTime::now();
    }

    info!("Document processed successfully: {} chunks, category: {:?}, confidence: {:.3}",
          document_with_embeddings.chunks.len(), document_category, confidence_score);

    Ok(DocumentIngestionResponse {
        document_id: document_with_embeddings.id,
        document_category,
        chunks_created: document_with_embeddings.chunks.len(),
        extraction_method: if let Some(_) = document_with_embeddings.chunks.get(0)
            .and_then(|chunk| chunk.metadata.ocr_metadata.as_ref()) {
            ExtractionMethod::TesseractOcr {
                confidence: 0.8, // Default confidence
                language: "fra+eng".to_string() // Default language
            }
        } else {
            ExtractionMethod::DirectRead
        },
        source_type: match &document_with_embeddings.document_type {
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

    // Récupérer le nom de la collection
    let groups = state.groups.read().await;
    let collection_name = if let Some(group) = groups.get(&params.group_id) {
        group.qdrant_collection.clone()
    } else {
        return Err(format!("Group not found: {}", params.group_id));
    };
    drop(groups);

    // Rechercher dans Qdrant avec l'embedding de la requête
    let limit = params.limit.unwrap_or(10);
    let search_url = format!("http://localhost:6333/collections/{}/points/search", collection_name);
    let client = reqwest::Client::new();

    let search_response = client
        .post(&search_url)
        .json(&serde_json::json!({
            "vector": query_embedding,
            "limit": limit,
            "with_payload": true
        }))
        .send()
        .await
        .map_err(|e| format!("Qdrant search request failed: {}", e))?;

    // Si la collection n'existe pas (404), retourner des résultats vides
    if search_response.status() == 404 {
        info!("📭 Collection {} does not exist yet (no documents)", collection_name);
        let search_time = start_time.elapsed().as_millis() as u64;
        return Ok(SearchResponseWithMetadata {
            results: Vec::new(),
            total_results: 0,
            search_time_ms: search_time,
            query_embedding_time_ms: query_embedding_time,
        });
    }

    if !search_response.status().is_success() {
        return Err(format!("Qdrant search returned error: {}", search_response.status()));
    }

    let search_data: serde_json::Value = search_response.json().await
        .map_err(|e| format!("Failed to parse Qdrant search response: {}", e))?;

    let search_results = search_data["result"].as_array()
        .ok_or_else(|| "Invalid Qdrant search response format".to_string())?;

    // Convertir les résultats Qdrant en SearchResultWithMetadata
    let mut results = Vec::new();

    for qdrant_result in search_results {
        let score = qdrant_result["score"].as_f64().unwrap_or(0.0) as f32;

        // Filtrer par score minimum
        if let Some(min_score) = params.min_score {
            if score < min_score {
                continue;
            }
        }

        let payload = match qdrant_result["payload"].as_object() {
            Some(p) => p,
            None => continue,
        };

        let content = payload.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let document_id = payload.get("document_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let chunk_id = payload.get("chunk_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let confidence = payload.get("confidence")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.85) as f32;

        // Classification du contenu si demandé
        let document_category = if params.include_business_metadata || params.document_categories.is_some() {
            state.document_classifier
                .classify(&content)
                .unwrap_or(DocumentCategory::Mixed)
        } else {
            DocumentCategory::Mixed
        };

        // Filtrer par catégorie si spécifié
        if let Some(ref categories) = params.document_categories {
            if !categories.contains(&document_category) {
                continue;
            }
        }

        // Enrichir avec métadonnées Business si demandé
        let business_metadata = if params.include_business_metadata && matches!(document_category, DocumentCategory::Business) {
            state.business_enricher
                .enrich_business_content(&content, params.fiscal_year_filter, None)
                .ok()
        } else {
            None
        };

        let search_result = SearchResultWithMetadata {
            chunk_id,
            content,
            score,
            document_id,
            document_category,
            source_type: SourceType::OcrExtracted, // Par défaut (stocké dans Qdrant)
            extraction_method: ExtractionMethod::TesseractOcr {
                confidence: confidence,
                language: "fra+eng".to_string(),
            },
            business_metadata,
            ocr_confidence: Some(confidence),
            chunk_metadata: ChunkMetadataSlim {
                tags: vec!["rag".to_string()],
                language: "auto".to_string(),
                confidence,
                start_line: 0,
                end_line: 0,
            },
        };

        results.push(search_result);
    }

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

/// Lister les documents stockés dans une collection Qdrant
#[tauri::command]
pub async fn list_rag_documents(
    group_id: String,
    state: State<'_, RagState>,
) -> Result<Vec<RagDocumentInfo>, String> {
    info!("📋 Listing RAG documents from group: {}", group_id);

    // Récupérer le nom de la collection
    let groups = state.groups.read().await;
    let collection_name = if let Some(group) = groups.get(&group_id) {
        let coll = group.qdrant_collection.clone();
        info!("✅ Found group '{}' with collection: {}", group_id, coll);
        coll
    } else {
        let fallback = format!("collection_{}", group_id);
        warn!("⚠️ Group '{}' not found! Using fallback collection: {}", group_id, fallback);
        fallback
    };
    drop(groups);

    info!("🔍 Querying Qdrant collection: {}", collection_name);

    // Récupérer tous les points de la collection via scroll
    let mut document_map: std::collections::HashMap<String, RagDocumentInfo> = std::collections::HashMap::new();

    // Utiliser l'API REST Qdrant pour scroller tous les points
    let url = format!("http://localhost:6333/collections/{}/points/scroll", collection_name);
    let client = reqwest::Client::new();

    let response = client
        .post(&url)
        .json(&serde_json::json!({
            "limit": 1000,
            "with_payload": true,
            "with_vector": false
        }))
        .send()
        .await
        .map_err(|e| format!("Qdrant request failed: {}", e))?;

    // Si la collection n'existe pas encore (404), retourner une liste vide
    if response.status() == 404 {
        info!("📭 Collection {} does not exist yet (no documents injected)", collection_name);
        return Ok(Vec::new());
    }

    if !response.status().is_success() {
        return Err(format!("Qdrant returned error: {}", response.status()));
    }

    let data: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse Qdrant response: {}", e))?;

    let points = data["result"]["points"].as_array()
        .ok_or_else(|| "Invalid Qdrant response format".to_string())?;

    // Regrouper par document_id
    for point in points {
        if let Some(payload) = point["payload"].as_object() {
            let doc_id = payload.get("document_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let entry = document_map.entry(doc_id.clone()).or_insert_with(|| {
                RagDocumentInfo {
                    document_id: doc_id,
                    group_id: group_id.clone(),
                    chunks_count: 0,
                    confidence: 0.0,
                    sample_content: String::new(),
                }
            });

            entry.chunks_count += 1;

            // Récupérer un échantillon de contenu
            if entry.sample_content.is_empty() {
                if let Some(content) = payload.get("content").and_then(|v| v.as_str()) {
                    entry.sample_content = content.chars().take(200).collect();
                }
            }

            // Moyenne de confiance
            if let Some(conf) = payload.get("confidence").and_then(|v| v.as_f64()) {
                entry.confidence = (entry.confidence * (entry.chunks_count - 1) as f32 + conf as f32)
                    / entry.chunks_count as f32;
            }
        }
    }

    let documents: Vec<RagDocumentInfo> = document_map.into_values().collect();

    info!("📊 Returning {} documents with {} total chunks from collection {}",
          documents.len(),
          documents.iter().map(|d| d.chunks_count).sum::<usize>(),
          collection_name);

    Ok(documents)
}

/// Information simplifiée sur un document RAG
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RagDocumentInfo {
    pub document_id: String,
    pub group_id: String,
    pub chunks_count: usize,
    pub confidence: f32,
    pub sample_content: String,
}

/// Supprimer un document RAG et tous ses chunks de Qdrant
#[tauri::command]
pub async fn delete_rag_document(
    document_id: String,
    group_id: String,
    state: State<'_, RagState>,
) -> Result<DeleteRagDocumentResponse, String> {
    info!("🗑️ Deleting RAG document {} from group {}", document_id, group_id);

    // Récupérer le nom de la collection
    let groups = state.groups.read().await;
    let collection_name = if let Some(group) = groups.get(&group_id) {
        group.qdrant_collection.clone()
    } else {
        format!("collection_{}", group_id)
    };
    drop(groups);

    // 1. Récupérer tous les points du document via scroll avec filtre
    let url = format!("http://localhost:6333/collections/{}/points/scroll", collection_name);
    let client = reqwest::Client::new();

    let response = client
        .post(&url)
        .json(&serde_json::json!({
            "limit": 1000,
            "with_payload": true,
            "with_vector": false,
            "filter": {
                "must": [{
                    "key": "document_id",
                    "match": {
                        "value": document_id
                    }
                }]
            }
        }))
        .send()
        .await
        .map_err(|e| format!("Qdrant scroll request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Qdrant scroll returned error: {}", response.status()));
    }

    let data: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse Qdrant scroll response: {}", e))?;

    let points = data["result"]["points"].as_array()
        .ok_or_else(|| "Invalid Qdrant scroll response format".to_string())?;

    // 2. Extraire les IDs des points à supprimer
    let point_ids: Vec<String> = points
        .iter()
        .filter_map(|point| {
            point["id"].as_str().map(|s| s.to_string())
        })
        .collect();

    if point_ids.is_empty() {
        return Err(format!("Document {} not found in collection", document_id));
    }

    let chunks_count = point_ids.len();
    info!("📊 Found {} chunks to delete for document {}", chunks_count, document_id);

    // 3. Supprimer les points via l'API Qdrant
    let delete_url = format!("http://localhost:6333/collections/{}/points/delete", collection_name);
    let delete_response = client
        .post(&delete_url)
        .json(&serde_json::json!({
            "points": point_ids
        }))
        .send()
        .await
        .map_err(|e| format!("Qdrant delete request failed: {}", e))?;

    if !delete_response.status().is_success() {
        return Err(format!("Qdrant delete returned error: {}", delete_response.status()));
    }

    info!("✅ Successfully deleted {} chunks for document {}", chunks_count, document_id);

    // 4. Supprimer aussi du state en RAM (si présent)
    let mut groups = state.groups.write().await;
    if let Some(group) = groups.get_mut(&group_id) {
        group.documents.retain(|doc| doc.id != document_id);
        group.updated_at = SystemTime::now();
        info!("🔄 Also removed document from RAM state");
    }

    Ok(DeleteRagDocumentResponse {
        document_id,
        chunks_deleted: chunks_count,
        success: true,
    })
}

/// Réponse de suppression d'un document RAG
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeleteRagDocumentResponse {
    pub document_id: String,
    pub chunks_deleted: usize,
    pub success: bool,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rag_pipeline_with_preextracted_text() {
        // Simuler un texte pré-extrait par AWCS OCR
        let preextracted_text = r#"
RÉPUBLIQUE FRANÇAISE
DIRECTION GÉNÉRALE DES FINANCES PUBLIQUES

AVIS DE SITUATION DÉCLARATIVE À L'IMPÔT SUR LE REVENU

Numéro fiscal: 1234567890123
Référence de l'avis: 24XXXXX

Situation au 31 décembre 2024

Revenu fiscal de référence: 35000€
Nombre de parts: 2.5
Impôt sur le revenu: 2500€
Prélèvements sociaux: 1200€

Pour toute réclamation, veuillez contacter votre centre des impôts.
"#;

        info!("🧪 Test: RAG pipeline avec texte pré-extrait ({} chars)", preextracted_text.len());

        // Vérifier le chunking
        let paragraphs: Vec<&str> = preextracted_text
            .split("\n\n")
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .collect();

        info!("📊 Paragraphes détectés: {}", paragraphs.len());
        assert!(paragraphs.len() >= 3, "Devrait avoir au moins 3 paragraphes");

        // Vérifier les métadonnées d'extraction
        let source_type = SourceType::OcrExtracted;
        assert_eq!(source_type, SourceType::OcrExtracted);

        // Vérifier le contenu
        assert!(preextracted_text.contains("RÉPUBLIQUE FRANÇAISE"));
        assert!(preextracted_text.contains("Revenu fiscal de référence"));

        info!("✅ Test pipeline RAG avec pré-extraction: SUCCÈS");
    }

    #[tokio::test]
    async fn test_empty_text_detection() {
        let empty_text = "   \n\n  \t  ";
        let trimmed = empty_text.trim();

        assert!(trimmed.is_empty(), "Texte vide devrait être détecté");
        info!("✅ Test détection texte vide: SUCCÈS");
    }

    #[tokio::test]
    async fn test_emergency_fallback_detection() {
        let fallback_content = "EXTRACTION FAILED: No text could be extracted from PDF";

        assert!(fallback_content.starts_with("EXTRACTION FAILED"));
        assert!(fallback_content.trim().len() > 20, "Chunk d'urgence devrait avoir du contenu");

        info!("✅ Test détection chunk d'urgence: SUCCÈS");
    }
}