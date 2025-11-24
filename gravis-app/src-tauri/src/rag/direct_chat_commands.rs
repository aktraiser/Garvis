// Phase 2: Commandes Tauri pour Chat Direct - Mode Générique (MVP)
// Interface frontend/backend pour sessions temporaires

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;
use tracing::{info, warn, error, debug};

use crate::rag::{
    DocumentProcessor, TesseractProcessor, TesseractConfig, CustomE5Embedder,
    DocumentType, ChunkConfig, RagError
};
use crate::rag::core::source_spans::{SourceSpan, ExtractionMetadata};
use crate::rag::core::direct_chat::{
    DirectChatSession, DirectChatResponse, SelectionContext, OCRContent, OCRPage, 
    OCRBlock, BlockType, BoundingBox, LayoutAnalysis, DirectChatError
};
use crate::rag::core::direct_chat_manager::{DirectChatManager, ScoredChunk, SessionStats, SessionInfo};

/// État pour chat direct (ajouté au RagState principal)
#[derive(Clone)]
pub struct DirectChatState {
    pub manager: DirectChatManager,
    pub document_processor: DocumentProcessor,
}

impl DirectChatState {
    pub async fn new(embedder: std::sync::Arc<CustomE5Embedder>) -> Result<Self, RagError> {
        info!("Initializing DirectChatState for Phase 2 MVP");

        // Créer processeur OCR pour mode direct
        let ocr_processor = TesseractProcessor::new(TesseractConfig::default())
            .await
            .map_err(|e| RagError::InvalidConfig(format!("OCR processor init failed: {}", e)))?;

        // Créer processeur de documents
        let document_processor = DocumentProcessor::new(ocr_processor, embedder.clone())
            .await
            .map_err(|e| RagError::InvalidConfig(format!("DocumentProcessor init failed: {}", e)))?;

        // Créer gestionnaire avec TTL de 2 heures pour MVP
        let manager = DirectChatManager::with_ttl(embedder, 7200);

        Ok(Self {
            manager,
            document_processor,
        })
    }
}

/// Réponse de traitement de document dragué
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessDocumentResponse {
    pub session: DirectChatSession,
    pub processing_time_ms: u64,
    pub chunks_created: usize,
    pub embedded_chunks: usize,
    pub confidence_score: f64,
}

/// Paramètres de chat avec sélection optionnelle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub session_id: String,
    pub query: String,
    pub selection: Option<SelectionContext>,
    pub limit: Option<usize>,
}

/// Réponse pour URL de PDF temporaire
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempPdfUrlResponse {
    pub pdf_url: String,
    pub original_path: String,
}

/// Réponse enrichie de chat direct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub response: String,
    pub contributing_spans: Vec<SourceSpan>,
    pub confidence_score: f64,
    pub session_id: String,
    pub search_time_ms: u64,
    pub chunks_used: usize,
    pub sources_summary: Vec<SourceSummary>,
}

/// Résumé d'une source contributrice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSummary {
    pub chunk_id: String,
    pub content_preview: String,
    pub score: f32,
    pub confidence: f64,
    pub span_count: usize,
}

// === Sprint 1 Niveau 1: LLM Response Generation ===

/// Réponse LLM avec contexte formaté pour synthesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmContextResponse {
    pub session_id: String,
    pub formatted_context: String,  // Context prêt pour le LLM
    pub chunks: Vec<LlmChunkInfo>,  // Info sur chaque chunk
    pub query: String,
    pub search_time_ms: u64,
    pub has_ocr_data: bool,
}

/// Information sur un chunk pour le LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmChunkInfo {
    pub chunk_id: String,
    pub source_label: String,       // "Figure OCR - Table 2", "Document Text", etc.
    pub content: String,             // Contenu tronqué à 800 chars
    pub score: f32,
    pub confidence: f64,
    pub page: Option<u32>,
    pub figure_id: Option<String>,
    pub source_type: String,
}

// === Commandes Tauri Phase 2 ===

/// Traiter un document dragué et créer session temporaire - VERSION CANONIQUE
#[tauri::command]
pub async fn process_dropped_document(
    file_path: String,
    file_data: Vec<u8>,
    mime_type: String,
    state: State<'_, DirectChatState>,
) -> Result<ProcessDocumentResponse, String> {
    let start_time = std::time::Instant::now();
    info!("🚀 Phase 2: Processing dropped document: {} ({} bytes, {})", 
          file_path, file_data.len(), mime_type);

    // 1. Créer un fichier temporaire avec les données
    let temp_dir = std::env::temp_dir();
    let temp_file_name = format!("gravis_temp_{}", file_path);
    let temp_path = temp_dir.join(temp_file_name);
    
    // Écrire les données dans le fichier temporaire
    std::fs::write(&temp_path, file_data)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;
    
    info!("📁 Created temporary file: {:?}", temp_path);

    // 2. Traitement du document avec pipeline existant
    let chunk_config = ChunkConfig::default(); // Configuration MVP
    let temp_group_id = "direct_chat_temp";
    
    let document = state.document_processor
        .process_document(&temp_path, temp_group_id, &chunk_config)
        .await
        .map_err(|e| format!("Document processing failed: {}", e))?;

    // 3. Création du contenu OCR à partir du document traité
    let ocr_content = create_ocr_content_from_document(&document)?;
    
    // 4. Détermination du type de document (pour l'instant générique)
    let document_type = determine_document_type(&document);

    // 5. Création de la session temporaire avec chunks enrichis
    let mut enriched_chunks = document.chunks.clone();
    
    // 6. Génération des embeddings PENDANT le traitement (PR #4 Fix)
    info!("🔄 Generating embeddings for {} chunks during processing", enriched_chunks.len());
    let mut embedded_count = 0;
    
    for chunk in &mut enriched_chunks {
        if !chunk.content.trim().is_empty() 
            && !chunk.content.starts_with("EXTRACTION FAILED") {
            
            match state.manager.embedder.encode_document(&chunk.content).await {
                Ok(embedding) => {
                    chunk.embedding = Some(embedding);
                    embedded_count += 1;
                }
                Err(e) => {
                    warn!("Failed to embed chunk {} during processing: {}", chunk.id, e);
                }
            }
        }
    }
    
    info!("✅ Generated {} embeddings during processing", embedded_count);

    let session = DirectChatSession::new_legacy(
        temp_path.clone(),
        document_type,
        enriched_chunks,
        ocr_content,
    );

    let session_id = session.session_id.clone();
    let chunks_created = session.chunks.len();
    
    // 7. Stockage direct (embeddings déjà générés)
    state.manager.store_session(session.clone()).await
        .map_err(|e| format!("Failed to store session: {}", e))?;

    // 8. Récupérer la session mise à jour
    let updated_session = state.manager.get_session(&session_id).await
        .map_err(|e| format!("Failed to retrieve updated session: {}", e))?;

    let embedded_chunks = updated_session.embedded_chunks_count();
    let confidence_score = calculate_session_confidence(&updated_session);
    let processing_time = start_time.elapsed().as_millis() as u64;

    info!("✅ Created direct chat session {} with {} chunks ({} embedded) in {}ms",
          session_id, chunks_created, embedded_chunks, processing_time);

    // 9. NE PAS supprimer le fichier temporaire - conservé pour affichage PDF
    // Le fichier sera nettoyé lors du cleanup de la session via cleanup_direct_chat_session
    info!("📌 Keeping temp file for PDF display: {:?}", temp_path);

    Ok(ProcessDocumentResponse {
        session: updated_session,
        processing_time_ms: processing_time,
        chunks_created,
        embedded_chunks,
        confidence_score,
    })
}

/// Chatter avec un document via session temporaire
#[tauri::command]
pub async fn chat_with_dropped_document(
    request: ChatRequest,
    state: State<'_, DirectChatState>,
) -> Result<ChatResponse, String> {
    let start_time = std::time::Instant::now();
    info!("💬 Chat request for session {}: '{}'", request.session_id, request.query);

    // 1. Recherche sémantique dans la session
    let scored_chunks = state.manager
        .search_in_session(
            &request.session_id,
            &request.query,
            request.selection,
            request.limit,
        )
        .await
        .map_err(|e| format!("Search failed: {}", e))?;

    if scored_chunks.is_empty() {
        warn!("No relevant chunks found for query: {}", request.query);
        return Ok(ChatResponse {
            response: "Je n'ai pas trouvé d'informations pertinentes pour répondre à votre question dans ce document.".to_string(),
            contributing_spans: vec![],
            confidence_score: 0.0,
            session_id: request.session_id,
            search_time_ms: start_time.elapsed().as_millis() as u64,
            chunks_used: 0,
            sources_summary: vec![],
        });
    }

    // 2. Génération de la réponse contextuelle
    let response = generate_contextual_response(&scored_chunks, &request.query)?;
    
    // 3. Extraction des spans contributeurs
    let contributing_spans = extract_contributing_spans(&scored_chunks);
    
    // 4. Calcul de la confidence globale
    let confidence_score = calculate_response_confidence(&scored_chunks);
    
    // 5. Création du résumé des sources
    let sources_summary = create_sources_summary(&scored_chunks);

    let search_time = start_time.elapsed().as_millis() as u64;

    info!("✅ Generated response from {} chunks in {}ms (confidence: {:.2})",
          scored_chunks.len(), search_time, confidence_score);

    Ok(ChatResponse {
        response,
        contributing_spans,
        confidence_score,
        session_id: request.session_id,
        search_time_ms: search_time,
        chunks_used: scored_chunks.len(),
        sources_summary,
    })
}

/// Sprint 1 Niveau 1: Chat avec contexte formaté pour LLM synthesis
#[tauri::command]
pub async fn chat_with_llm_context(
    request: ChatRequest,
    state: State<'_, DirectChatState>,
) -> Result<LlmContextResponse, String> {
    let start_time = std::time::Instant::now();
    info!("🤖 LLM Context Chat - session: {}, query: '{}'",
          request.session_id, request.query);

    // 1. Recherche RAG classique (réutilise le pipeline existant)
    // Fetch top-20 pour avoir un pool élargi, puis reranking + filtres
    let scored_chunks = state.manager
        .search_in_session(
            &request.session_id,
            &request.query,
            request.selection,
            Some(20),  // Pool de 20 chunks avant reranking (élargi pour mieux capturer objectifs stratégiques)
        )
        .await
        .map_err(|e| format!("Search failed: {}", e))?;

    if scored_chunks.is_empty() {
        warn!("No relevant chunks found for LLM context");
        return Ok(LlmContextResponse {
            session_id: request.session_id,
            formatted_context: String::new(),
            chunks: vec![],
            query: request.query,
            search_time_ms: start_time.elapsed().as_millis() as u64,
            has_ocr_data: false,
        });
    }

    // ========== MODE SIMPLE vs COMPLEXE ==========
    // AUDIT 22 NOV 2024: Test A/B pour valider utilité des composants
    //
    // MODE SIMPLE (baseline): RAG vanilla → top-10 → LLM
    // MODE COMPLEXE: RAG → query-aware rerank → filtres 3-pass → top-7 → LLM
    //
    // Configuration: Set ENABLE_QUERY_RERANKING=false pour mode SIMPLE
    const ENABLE_QUERY_RERANKING: bool = false;  // ⚠️ DÉSACTIVÉ pour test baseline

    let mut scored_chunks = if ENABLE_QUERY_RERANKING {
        // MODE COMPLEXE: Query-aware reranking (Sprint 1 Niveau 1.5)
        use crate::rag::search::QueryAwareReranker;
        let reranker = QueryAwareReranker::default();

        let reranked_items: Vec<(ScoredChunk, f32)> = scored_chunks
            .into_iter()
            .map(|sc| {
                let score = sc.score;
                (sc, score)
            })
            .collect();

        let reranked = reranker.rerank(
            &request.query,
            reranked_items,
            |sc: &ScoredChunk| sc.chunk.content.as_str(),
        );

        let result: Vec<ScoredChunk> = reranked
            .into_iter()
            .map(|(sc, new_score)| {
                let mut sc = sc;
                sc.score = new_score;
                sc
            })
            .take(10)
            .collect();

        debug!("🔄 MODE COMPLEXE: Query-aware reranking 20 → 10, top: {:.3}",
               result.first().map(|sc| sc.score).unwrap_or(0.0));
        result
    } else {
        // MODE SIMPLE: Prendre top-10 directement, pas de reranking
        let result: Vec<ScoredChunk> = scored_chunks.into_iter().take(10).collect();
        debug!("✅ MODE SIMPLE: Top-10 direct (no reranking), top: {:.3}",
               result.first().map(|sc| sc.score).unwrap_or(0.0));
        result
    };

    // ========== SECTION PRIOR + CONTAMINATION FILTER ==========
    // AUDIT 22 NOV: Section prior simple (~50 lignes) remplace filtres 3-pass (~300 lignes)
    use crate::rag::search::SectionPriorReranker;

    let items: Vec<(ScoredChunk, f32)> = scored_chunks
        .into_iter()
        .map(|sc| (sc.clone(), sc.score))
        .collect();

    let reranked = SectionPriorReranker::rerank_and_filter(
        items,
        |sc: &ScoredChunk| sc.chunk.content.as_str(),
        |sc: &ScoredChunk| {
            use crate::rag::ChunkSource;
            match sc.chunk.chunk_source {
                ChunkSource::FigureCaption => "Figure Caption",
                ChunkSource::Table => "Table",
                _ => "Document Text",
            }
        },
    );

    let filtered_chunks: Vec<ScoredChunk> = reranked
        .into_iter()
        .map(|(mut sc, new_score)| {
            sc.score = new_score;
            sc
        })
        .take(10)  // TEST A/B: Temporarily back to top-10 to debug "16x compressor" recall issue
        .collect();

    debug!("✅ Section Prior: {} chunks (top-10 TEST), top: {:.3}",
           filtered_chunks.len(), filtered_chunks.first().map(|sc| sc.score).unwrap_or(0.0));

    if filtered_chunks.is_empty() {
        warn!("All chunks filtered out by section prior");
        return Ok(LlmContextResponse {
            session_id: request.session_id,
            formatted_context: String::new(),
            chunks: vec![],
            query: request.query,
            search_time_ms: start_time.elapsed().as_millis() as u64,
            has_ocr_data: false,
        });
    }

    // 3. Construction du contexte formaté pour le LLM
    let (formatted_context, chunk_infos, has_ocr) = build_llm_context(&filtered_chunks);

    let search_time = start_time.elapsed().as_millis() as u64;

    info!("✅ Built LLM context from {} chunks in {}ms (OCR: {})",
          chunk_infos.len(), search_time, has_ocr);

    Ok(LlmContextResponse {
        session_id: request.session_id,
        formatted_context,
        chunks: chunk_infos,
        query: request.query,
        search_time_ms: search_time,
        has_ocr_data: has_ocr,
    })
}

/// Obtenir informations sur session temporaire
#[tauri::command]
pub async fn get_direct_chat_session(
    session_id: String,
    state: State<'_, DirectChatState>,
) -> Result<DirectChatSession, String> {
    info!("📋 Getting direct chat session: {}", session_id);

    state.manager
        .get_session(&session_id)
        .await
        .map_err(|e| format!("Session retrieval failed: {}", e))
}

/// Supprimer session temporaire et nettoyer fichier PDF associé
#[tauri::command]
pub async fn cleanup_direct_chat_session(
    session_id: String,
    state: State<'_, DirectChatState>,
) -> Result<(), String> {
    info!("🗑️ Cleaning up direct chat session: {}", session_id);

    // 1. Récupérer la session avant suppression pour obtenir le path du fichier
    if let Ok(session) = state.manager.get_session(&session_id).await {
        let temp_path = &session.document_path;

        // 2. Supprimer le fichier temporaire si c'est un PDF
        if temp_path.exists() && temp_path.to_string_lossy().contains("gravis_temp_") {
            info!("🗑️ Removing temporary file: {:?}", temp_path);
            if let Err(e) = std::fs::remove_file(temp_path) {
                warn!("Failed to remove temp file {:?}: {}", temp_path, e);
            } else {
                info!("✅ Temporary file removed successfully");
            }
        }
    }

    // 3. Supprimer la session de la mémoire
    state.manager
        .remove_session(&session_id)
        .await
        .map_err(|e| format!("Session cleanup failed: {}", e))
}

/// Obtenir statistiques des sessions directes
#[tauri::command]
pub async fn get_direct_chat_stats(
    state: State<'_, DirectChatState>,
) -> Result<SessionStats, String> {
    Ok(state.manager.get_stats().await)
}

/// Lister toutes les sessions directes actives
#[tauri::command]
pub async fn list_direct_chat_sessions(
    state: State<'_, DirectChatState>,
) -> Result<Vec<SessionInfo>, String> {
    Ok(state.manager.list_sessions().await)
}

/// Nettoyer sessions expirées (maintenance)
#[tauri::command]
pub async fn cleanup_expired_sessions(
    state: State<'_, DirectChatState>,
) -> Result<usize, String> {
    let cleaned_count = state.manager.cleanup_expired_sessions().await;
    info!("🧹 Cleaned {} expired direct chat sessions", cleaned_count);
    Ok(cleaned_count)
}

/// Récupérer le PDF associé à une session (pour affichage dans PdfSemanticOverlay)
#[tauri::command]
pub async fn get_pdf_for_session(
    session_id: String,
    state: State<'_, DirectChatState>,
) -> Result<Vec<u8>, String> {
    info!("📄 Fetching PDF for session: {}", session_id);

    // 1. Récupérer la session
    let session = state.manager
        .get_session(&session_id)
        .await
        .map_err(|e| format!("Session not found: {}", e))?;

    // 2. Vérifier que c'est bien un PDF
    let path = &session.document_path;
    if !path.exists() {
        return Err(format!("PDF file not found: {:?}", path));
    }

    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| format!("Invalid file extension for: {:?}", path))?;

    if extension.to_lowercase() != "pdf" {
        return Err(format!("File is not a PDF: {:?}", path));
    }

    // 3. Lire le fichier PDF en bytes
    let pdf_bytes = std::fs::read(path)
        .map_err(|e| format!("Failed to read PDF file: {}", e))?;

    info!("✅ PDF loaded: {} bytes", pdf_bytes.len());
    Ok(pdf_bytes)
}

// === Fonctions utilitaires ===

/// Résoudre le chemin du fichier (compatible avec architecture existante)
fn resolve_file_path(file_path: &str) -> Result<PathBuf, String> {
    let path = if file_path.starts_with("exemple/") {
        // Chemin relatif depuis le frontend - résoudre vers le dossier exemple
        let current_dir = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;
        let docs_path = current_dir.parent()
            .ok_or("Failed to get parent directory")?
            .join("exemple");
        let filename = file_path.strip_prefix("exemple/").unwrap_or(file_path);
        docs_path.join(filename)
    } else {
        // Chemin absolu ou autre - utiliser tel quel
        PathBuf::from(file_path)
    };

    Ok(path)
}

/// Détection intelligente du type de contenu pour un bloc de lignes
fn detect_content_type(lines: &[&str], start_idx: usize) -> (BlockType, usize) {
    if start_idx >= lines.len() {
        return (BlockType::Text, 1);
    }

    let current_line = lines[start_idx].trim();
    
    // 1. Détecter Figure/Chart (graphiques avec données)
    if let Some(lines_consumed) = detect_figure_content(lines, start_idx) {
        return (BlockType::Figure, lines_consumed);
    }
    
    // 2. Détecter Tables structurées
    if let Some(lines_consumed) = detect_table_content(lines, start_idx) {
        return (BlockType::Table, lines_consumed);
    }
    
    // 3. Détecter Table des matières / Listes numérotées
    if let Some(lines_consumed) = detect_toc_or_list(lines, start_idx) {
        return (BlockType::List, lines_consumed);
    }
    
    // 4. Détecter Headers
    if is_likely_header(current_line) {
        return (BlockType::Header, 1);
    }
    
    // 5. Détecter Key-Value pairs
    if is_key_value_pair(current_line) {
        return (BlockType::KeyValue, 1);
    }
    
    // 6. Détecter montants/dates
    if is_amount_or_date(current_line) {
        return (BlockType::Amount, 1);
    }
    
    // Par défaut: Text
    (BlockType::Text, 1)
}

/// Détecter contenu de figure/graphique avec données numériques
fn detect_figure_content(lines: &[&str], start_idx: usize) -> Option<usize> {
    let mut lines_consumed = 0;
    let mut has_figure_indicators = false;
    let mut has_numerical_data = false;
    let mut has_percentage_data = false;

    for i in start_idx..std::cmp::min(start_idx + 10, lines.len()) {
        let line = lines[i].trim();
        
        if line.is_empty() {
            lines_consumed += 1;
            continue;
        }

        // Indicateurs de figure/graphique
        if line.contains("Figure") || line.contains("Chart") || line.contains("Graph") 
            || line.contains("Compression") || line.contains("Performance") {
            has_figure_indicators = true;
        }

        // Données numériques avec axes/échelles
        if line.contains("600-700") || line.contains("0%") || line.contains("100%") 
            || line.contains("Vision Tokens") || line.contains("Edit Distance") {
            has_numerical_data = true;
        }

        // Pourcentages multiples (données de graphique)
        let percentage_count = line.matches('%').count();
        if percentage_count >= 2 {
            has_percentage_data = true;
        }

        lines_consumed += 1;

        // Stop si on trouve une ligne clairement non-figure
        if line.len() > 200 && !line.contains("%") && !line.contains("Figure") {
            break;
        }
    }

    if (has_figure_indicators && has_numerical_data) || has_percentage_data {
        Some(lines_consumed)
    } else {
        None
    }
}

/// Détecter contenu tabulaire avec colonnes alignées
fn detect_table_content(lines: &[&str], start_idx: usize) -> Option<usize> {
    let mut lines_consumed = 0;
    let mut consistent_columns = 0;
    let mut total_potential_rows = 0;

    for i in start_idx..std::cmp::min(start_idx + 8, lines.len()) {
        let line = lines[i].trim();
        
        if line.is_empty() {
            lines_consumed += 1;
            continue;
        }

        // Détecter séparateurs tabulaires
        let separators = ["\t", "  ", " | ", "|"];
        let mut max_columns = 0;
        
        for sep in &separators {
            let columns = line.split(sep).filter(|s| !s.trim().is_empty()).count();
            max_columns = max_columns.max(columns);
        }

        if max_columns >= 2 {
            if consistent_columns == 0 {
                consistent_columns = max_columns;
            }
            
            // Vérifier cohérence des colonnes
            if max_columns >= consistent_columns / 2 {
                total_potential_rows += 1;
            }
        }

        lines_consumed += 1;

        // Stop si trop de lignes sans structure tabulaire
        if total_potential_rows == 0 && lines_consumed > 3 {
            break;
        }
    }

    // Au moins 2 lignes avec structure cohérente = table
    if total_potential_rows >= 2 {
        Some(lines_consumed)
    } else {
        None
    }
}

/// Détecter table des matières ou listes numérotées
fn detect_toc_or_list(lines: &[&str], start_idx: usize) -> Option<usize> {
    let mut lines_consumed = 0;
    let mut numbered_lines = 0;
    let mut toc_pattern_lines = 0;

    for i in start_idx..std::cmp::min(start_idx + 6, lines.len()) {
        let line = lines[i].trim();
        
        if line.is_empty() {
            lines_consumed += 1;
            continue;
        }

        // Patterns de TOC
        if line.matches('.').count() >= 3 {  // "3.2.1 Architecture..."
            toc_pattern_lines += 1;
        }

        // Listes numérotées ou bullet points
        if line.starts_with("1 ") || line.starts_with("2 ") || line.starts_with("3 ") 
            || line.starts_with("• ") || line.starts_with("- ") 
            || line.starts_with("1.") || line.starts_with("2.") {
            numbered_lines += 1;
        }

        // TOC sections avec numéros
        if line.contains("Introduction") || line.contains("Methodology") || line.contains("Discussion") 
            || line.contains("Conclusion") || line.contains("Related Works") {
            toc_pattern_lines += 1;
        }

        lines_consumed += 1;
    }

    if numbered_lines >= 2 || toc_pattern_lines >= 2 {
        Some(lines_consumed)
    } else {
        None
    }
}

/// Vérifier si une ligne est probablement un header (version améliorée)
fn is_likely_header(line: &str) -> bool {
    let line = line.trim();

    // Headers typiques: courts, majuscules, ou numérotés
    let is_short = line.len() < 80;
    let has_many_caps = line.chars().filter(|c| c.is_uppercase()).count() as f32 / line.len().max(1) as f32 > 0.5;
    let starts_with_number = line.chars().next().map(|c| c.is_numeric()).unwrap_or(false);
    let is_numbered_section = line.starts_with("1 ") || line.starts_with("2 ") ||
                              line.starts_with("3 ") || line.starts_with("4 ") ||
                              line.starts_with("1.") || line.starts_with("2.") ||
                              line.starts_with("3.") || line.starts_with("4.");

    // Headers spécifiques aux papers scientifiques
    let is_academic_header = line == "Abstract" || line == "Introduction" || line == "Methodology" 
                            || line == "Evaluation" || line == "Discussion" || line == "Conclusion"
                            || line == "Related Works" || line == "Contents";

    (is_short && has_many_caps) || is_numbered_section || is_academic_header
}

/// Détecter paires clé-valeur
fn is_key_value_pair(line: &str) -> bool {
    line.contains(": ") && line.split(": ").count() == 2 && line.len() < 150
}

/// Détecter montants/dates
fn is_amount_or_date(line: &str) -> bool {
    // Montants monétaires
    let has_currency = line.contains("$") || line.contains("€") || line.contains("£");
    
    // Dates
    let has_date_pattern = line.contains("2024") || line.contains("2025") 
                          || line.contains("/") && line.chars().filter(|c| c.is_numeric()).count() >= 4;

    (has_currency || has_date_pattern) && line.len() < 100
}

/// Structurer le contenu d'une figure avec données numériques
fn structure_figure_content(lines: &[&str]) -> String {
    let mut structured = String::new();
    let content = lines.join(" ");

    // 1. Détecter le titre de la figure
    if let Some(title_line) = lines.iter().find(|line| line.contains("Figure") || line.contains("Chart")) {
        structured.push_str("📊 **");
        structured.push_str(title_line.trim());
        structured.push_str("**\n\n");
    }

    // 2. Extraire les données numériques
    let mut _data_points: Vec<String> = Vec::new();
    let mut performance_metrics = Vec::new();

    for line in lines {
        // Données avec pourcentages (ex: "96.5% 93.8% 83.8%")
        if line.matches('%').count() >= 2 {
            let percentages: Vec<&str> = line.split_whitespace()
                .filter(|s| s.contains('%'))
                .collect();
            if !percentages.is_empty() {
                structured.push_str("📈 **Performance Data:**\n");
                for (i, perc) in percentages.iter().enumerate() {
                    structured.push_str(&format!("  • Point {}: {}\n", i + 1, perc));
                }
                structured.push('\n');
            }
        }

        // Données de compression/tokens (ex: "600-700 700-800")
        if line.contains("-") && line.chars().filter(|c| c.is_numeric()).count() >= 4 {
            let ranges: Vec<&str> = line.split_whitespace()
                .filter(|s| s.contains('-') && s.chars().any(|c| c.is_numeric()))
                .collect();
            if !ranges.is_empty() {
                structured.push_str("📏 **Data Ranges:**\n");
                for range in ranges {
                    structured.push_str(&format!("  • {}\n", range));
                }
                structured.push('\n');
            }
        }

        // Vision tokens / métriques de performance
        if line.contains("Vision Tokens") || line.contains("Edit Distance") || line.contains("Compression") {
            performance_metrics.push(*line);
        }
    }

    // 3. Ajouter métriques de performance
    if !performance_metrics.is_empty() {
        structured.push_str("⚡ **Performance Metrics:**\n");
        for metric in performance_metrics {
            structured.push_str(&format!("  • {}\n", metric.trim()));
        }
        structured.push('\n');
    }

    // 4. Si pas de structure détectée, afficher le contenu brut organisé
    if structured.is_empty() || structured.len() < 50 {
        structured.clear();
        structured.push_str("📊 **Figure/Chart Data**\n\n");
        structured.push_str(&content);
    }

    structured
}

/// Structurer le contenu tabulaire
fn structure_table_content(lines: &[&str]) -> String {
    let mut structured = String::new();
    structured.push_str("📋 **Table Data**\n\n");

    let mut is_first_row = true;

    for line in lines {
        let line = line.trim();
        if line.is_empty() { continue; }

        // Essayer différents séparateurs
        let separators = ["\t", "  ", " | ", "|"];
        let mut best_columns = Vec::new();
        let mut max_columns = 0;

        for sep in &separators {
            let columns: Vec<&str> = line.split(sep)
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            
            if columns.len() > max_columns {
                max_columns = columns.len();
                best_columns = columns;
            }
        }

        if best_columns.len() >= 2 {
            // Format en table markdown
            if is_first_row {
                structured.push_str("| ");
                structured.push_str(&best_columns.join(" | "));
                structured.push_str(" |\n");
                
                // Ligne de séparation
                structured.push_str("|");
                for _ in 0..best_columns.len() {
                    structured.push_str("---|");
                }
                structured.push('\n');
                is_first_row = false;
            } else {
                structured.push_str("| ");
                structured.push_str(&best_columns.join(" | "));
                structured.push_str(" |\n");
            }
        } else {
            // Ligne simple si pas assez de colonnes
            structured.push_str(&format!("• {}\n", line));
        }
    }

    structured
}

/// Structurer les listes et tables des matières
fn structure_list_content(lines: &[&str]) -> String {
    let mut structured = String::new();
    
    // Détecter le type de liste
    let has_toc_patterns = lines.iter().any(|line| 
        line.matches('.').count() >= 3 || 
        line.contains("Introduction") || 
        line.contains("Methodology")
    );

    if has_toc_patterns {
        structured.push_str("📚 **Table of Contents**\n\n");
    } else {
        structured.push_str("📝 **Structured List**\n\n");
    }

    for line in lines {
        let line = line.trim();
        if line.is_empty() { continue; }

        // Formater selon le type de ligne
        if line.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
            // Ligne numérotée
            structured.push_str(&format!("  {}\n", line));
        } else if line.starts_with("• ") || line.starts_with("- ") {
            // Bullet point
            structured.push_str(&format!("  {}\n", line));
        } else {
            // Autre type de liste
            structured.push_str(&format!("  • {}\n", line));
        }
    }

    structured
}

/// NOUVELLE FONCTION - Reconstruction intelligente des blocs selon les best practices 2024
fn reconstruct_block_content(
    lines: &[&str], 
    start_idx: usize, 
    block_type: BlockType, 
    initial_lines_consumed: usize
) -> (String, usize) {
    
    match block_type {
        BlockType::Text => reconstruct_paragraph_block(lines, start_idx),
        BlockType::Table => reconstruct_table_block(lines, start_idx, initial_lines_consumed),
        BlockType::Figure => reconstruct_figure_block(lines, start_idx, initial_lines_consumed),
        BlockType::List => reconstruct_list_block(lines, start_idx, initial_lines_consumed),
        _ => {
            // Pour Header, KeyValue, Amount - retour simple
            let line = lines.get(start_idx).unwrap_or(&"").trim();
            (line.to_string(), 1)
        }
    }
}

/// Reconstruction de paragraphes - Regroupement intelligent basé sur la proximité sémantique
fn reconstruct_paragraph_block(lines: &[&str], start_idx: usize) -> (String, usize) {
    let mut paragraph_lines = Vec::new();
    let mut lines_consumed = 0;
    
    for i in start_idx..lines.len() {
        let line = lines[i].trim();
        
        if line.is_empty() {
            lines_consumed += 1;
            break; // Fin du paragraphe sur ligne vide
        }
        
        // Détecter fin de paragraphe
        if i > start_idx && is_paragraph_break(lines, i) {
            break;
        }
        
        paragraph_lines.push(line);
        lines_consumed += 1;
        
        // Limite de sécurité pour éviter les paragraphes trop longs
        if paragraph_lines.len() >= 20 {
            break;
        }
    }
    
    // Joindre avec espaces, gérer la ponctuation intelligemment
    let content = join_paragraph_lines(&paragraph_lines);
    (content, lines_consumed)
}

/// Détection intelligente des fins de paragraphe
fn is_paragraph_break(lines: &[&str], idx: usize) -> bool {
    let current = lines[idx].trim();
    let previous = if idx > 0 { lines[idx - 1].trim() } else { "" };
    
    // Nouvelle section numérotée
    if current.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
        if current.contains('.') && current.len() < 50 {
            return true;
        }
    }
    
    // Nouveau header détecté
    if is_likely_header(current) {
        return true;
    }
    
    // Figure/Table markers
    if current.contains("Figure") || current.contains("Table") || current.contains("Chart") {
        return true;
    }
    
    // Fin de phrase + Nouvelle phrase avec majuscule
    if previous.ends_with('.') || previous.ends_with(':') {
        if current.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
            // Mais pas si c'est une continuation logique
            if !current.starts_with("The") && !current.starts_with("This") && !current.starts_with("We") {
                return current.len() > 30; // Nouvelle phrase substantielle
            }
        }
    }
    
    false
}

/// Jointure intelligente des lignes de paragraphe avec gestion ponctuation
fn join_paragraph_lines(lines: &[&str]) -> String {
    if lines.is_empty() {
        return String::new();
    }
    
    let mut result = String::new();
    
    for (i, line) in lines.iter().enumerate() {
        if i == 0 {
            result.push_str(line);
        } else {
            // Gestion intelligente des espaces selon la ponctuation
            let prev_line = lines[i - 1];
            let needs_space = !prev_line.ends_with('-') && !line.starts_with('-');
            
            if needs_space {
                result.push(' ');
            }
            
            // Supprimer tiret de césure si présent
            if prev_line.ends_with('-') && !line.starts_with('-') {
                result.pop(); // Enlever le tiret
            }
            
            result.push_str(line);
        }
    }
    
    result
}

/// Reconstruction de blocs table - Garder la structure spatiale
fn reconstruct_table_block(lines: &[&str], start_idx: usize, initial_consumed: usize) -> (String, usize) {
    let mut table_lines = Vec::new();
    let mut lines_consumed = 0;
    
    // Collecter toutes les lignes qui semblent tabulaires
    for i in start_idx..std::cmp::min(start_idx + initial_consumed + 10, lines.len()) {
        let line = lines[i].trim();
        
        if line.is_empty() {
            lines_consumed += 1;
            if table_lines.len() > 0 {
                table_lines.push(""); // Garder les lignes vides dans les tables
            }
            continue;
        }
        
        // Stop si nouvelle section détectée
        if is_likely_header(line) || line.contains("Figure") {
            break;
        }
        
        table_lines.push(line);
        lines_consumed += 1;
    }
    
    // Joindre avec retours à la ligne pour préserver la structure
    let content = table_lines.join("\n");
    (content, lines_consumed)
}

/// Reconstruction de blocs figure - Regrouper données numériques cohérentes  
fn reconstruct_figure_block(lines: &[&str], start_idx: usize, initial_consumed: usize) -> (String, usize) {
    let mut figure_lines = Vec::new();
    let mut lines_consumed = 0;
    let mut in_data_section = false;
    
    for i in start_idx..std::cmp::min(start_idx + initial_consumed + 15, lines.len()) {
        let line = lines[i].trim();
        
        if line.is_empty() {
            lines_consumed += 1;
            continue;
        }
        
        // Stop conditions
        if i > start_idx && is_likely_header(line) {
            break;
        }
        
        // Détecter sections de données
        let has_percentages = line.matches('%').count() >= 1;
        let has_numbers = line.chars().filter(|c| c.is_numeric()).count() >= 3;
        let has_ranges = line.contains('-') && has_numbers;
        
        if has_percentages || has_ranges || line.contains("Vision Tokens") || line.contains("Figure") {
            in_data_section = true;
            figure_lines.push(line);
            lines_consumed += 1;
        } else if in_data_section && line.len() > 100 {
            // Long texte = fin de la figure
            break;
        } else if in_data_section || line.contains("Compression") || line.contains("Performance") {
            figure_lines.push(line);
            lines_consumed += 1;
        } else {
            break;
        }
    }
    
    let content = figure_lines.join(" ");
    (content, lines_consumed)
}

/// Reconstruction de listes - Préserver structure hiérarchique
fn reconstruct_list_block(lines: &[&str], start_idx: usize, initial_consumed: usize) -> (String, usize) {
    let mut list_lines = Vec::new();
    let mut lines_consumed = 0;
    
    for i in start_idx..std::cmp::min(start_idx + initial_consumed + 8, lines.len()) {
        let line = lines[i].trim();
        
        if line.is_empty() {
            lines_consumed += 1;
            continue;
        }
        
        // Stop si plus de structure de liste
        if !line.chars().next().map(|c| c.is_numeric() || c == '•' || c == '-').unwrap_or(false) {
            if !line.contains('.') || line.len() > 100 {
                break;
            }
        }
        
        list_lines.push(line);
        lines_consumed += 1;
    }
    
    let content = list_lines.join("\n");
    (content, lines_consumed)
}

// ============================================================================
// NATIVE OCR BLOCKS - PR #4 Phase 3
// ============================================================================

/// Structure pour blocs OCR natifs provenant de l'extraction initiale
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NativeOCRBlock {  // 🆕 pub pour utilisation depuis document_processor
    pub page_number: u32,
    pub block_type: String,   // "header", "paragraph", "table", "figure", etc.
    pub text: String,
    pub bbox: NativeBBox,
    pub confidence: f64,
    #[serde(default)]  // Pour compatibilité ascendante
    pub page_width: Option<f64>,
    #[serde(default)]
    pub page_height: Option<f64>,
}

/// Bounding box native (pixels absolus)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NativeBBox {  // 🆕 pub pour utilisation depuis document_processor
    pub x: f64,      // Position X en pixels
    pub y: f64,      // Position Y en pixels
    pub width: f64,  // Largeur en pixels
    pub height: f64, // Hauteur en pixels
}

/// Parser les blocs OCR natifs depuis metadata JSON
fn parse_native_ocr_blocks(raw_ocr: &serde_json::Value) -> Result<OCRContent, String> {
    // On attend: { "blocks": [...] } ou directement [...]
    let blocks_json = if raw_ocr.get("blocks").is_some() {
        &raw_ocr["blocks"]
    } else {
        raw_ocr
    };

    let native_blocks: Vec<NativeOCRBlock> = serde_json::from_value(blocks_json.clone())
        .map_err(|e| format!("Erreur de parsing des blocs OCR natifs: {}", e))?;

    if native_blocks.is_empty() {
        return Err("Aucun bloc OCR natif trouvé".into());
    }

    // Grouper par page
    use std::collections::HashMap;
    let mut pages_map: HashMap<u32, (Vec<OCRBlock>, f64, f64)> = HashMap::new();

    for nb in native_blocks {
        let block_type = map_block_type_from_str(&nb.block_type);

        // On garde les coordonnées en pixels pour l'instant
        // Le frontend fera la normalisation
        let ocr_block = OCRBlock {
            page_number: nb.page_number,
            block_type,
            content: nb.text,
            bounding_box: BoundingBox {
                x: nb.bbox.x,
                y: nb.bbox.y,
                width: nb.bbox.width,
                height: nb.bbox.height,
            },
            confidence: nb.confidence,
            spans: Vec::new(), // On les peuplera plus tard
        };

        // Stocker les blocs + dimensions de page
        // Utiliser dimensions du bloc si disponibles, sinon A4 par défaut
        let (width, height) = (
            nb.page_width.unwrap_or(595.0),
            nb.page_height.unwrap_or(842.0)
        );

        let entry = pages_map.entry(nb.page_number)
            .or_insert_with(|| (Vec::new(), width, height));

        // Mettre à jour les dimensions si on a de vraies valeurs
        if nb.page_width.is_some() && nb.page_height.is_some() {
            entry.1 = width;
            entry.2 = height;
        }

        entry.0.push(ocr_block);
    }

    // Construire les OCRPage
    let mut pages: Vec<OCRPage> = pages_map
        .into_iter()
        .map(|(page_number, (blocks, width, height))| {
            debug!("📄 Page {}: {}x{} with {} blocks", page_number, width, height, blocks.len());
            OCRPage {
                page_number,
                width,
                height,
                blocks,
            }
        })
        .collect();

    pages.sort_by_key(|p| p.page_number);

    // Calculer confidence moyenne
    let total_conf = if !pages.is_empty() {
        let mut sum = 0.0;
        let mut count = 0;
        for p in &pages {
            for b in &p.blocks {
                sum += b.confidence;
                count += 1;
            }
        }
        if count > 0 { sum / (count as f64) } else { 0.0 }
    } else {
        0.0
    };

    // Analyse de layout basique
    let all_blocks: Vec<_> = pages.iter().flat_map(|p| &p.blocks).collect();
    let layout_analysis = LayoutAnalysis {
        detected_columns: 1,
        has_tables: all_blocks.iter().any(|b| matches!(b.block_type, BlockType::Table)),
        has_headers: all_blocks.iter().any(|b| matches!(b.block_type, BlockType::Header)),
        text_density: 0.7,
        dominant_font_size: Some(12.0),
    };

    info!("✅ Parsed {} pages with {} total blocks from native OCR",
          pages.len(),
          all_blocks.len());

    Ok(OCRContent {
        pages,
        total_confidence: total_conf,
        layout_analysis,
    })
}

/// Mapper string vers BlockType
fn map_block_type_from_str(s: &str) -> BlockType {
    match s.to_lowercase().as_str() {
        "header" | "title" | "heading" => BlockType::Header,
        "paragraph" | "text" => BlockType::Text,
        "table" => BlockType::Table,
        "figure" | "image" => BlockType::Figure,
        "list" | "bullet_list" | "numbered_list" => BlockType::List,
        "keyvalue" | "key_value" | "field" => BlockType::KeyValue,
        "amount" | "money" => BlockType::Amount,
        "date" => BlockType::Date,
        _ => BlockType::Text,
    }
}

/// Créer contenu OCR à partir du document traité avec analyse de structure
/// VERSION REFACTORISÉE PR #4 - Utilise blocs natifs quand disponibles
fn create_ocr_content_from_document(
    document: &crate::rag::GroupDocument
) -> Result<OCRContent, String> {
    // 1️⃣ Priorité: Blocs OCR natifs dans metadata.custom_fields
    if let Some(raw_ocr_str) = document.metadata.custom_fields.get("ocr_blocks") {
        info!("🎯 Using native OCR blocks from metadata");
        // Parser le JSON depuis le string
        match serde_json::from_str::<serde_json::Value>(raw_ocr_str) {
            Ok(raw_ocr) => {
                match parse_native_ocr_blocks(&raw_ocr) {
                    Ok(ocr_content) => return Ok(ocr_content),
                    Err(e) => {
                        warn!("⚠️ Failed to parse native OCR blocks: {}, falling back to synthetic", e);
                    }
                }
            },
            Err(e) => {
                warn!("⚠️ Failed to parse OCR blocks JSON: {}, falling back to synthetic", e);
            }
        }
    }

    // 2️⃣ Fallback: Reconstruction synthétique (ancienne méthode)
    warn!("⚠️ No native OCR blocks found, using synthetic reconstruction (1 page only)");
    create_synthetic_ocr_content(document)
}

/// Créer contenu OCR synthétique (ancien système - fallback uniquement)
fn create_synthetic_ocr_content(
    document: &crate::rag::GroupDocument
) -> Result<OCRContent, String> {
    let mut blocks = Vec::new();

    // First, add any OCR blocks from PDF extraction (figures/images)
    blocks.extend(document.ocr_blocks.clone());

    // Analyser le contenu complet pour détecter la structure
    let content_lines: Vec<&str> = document.content.lines().collect();
    let mut current_y = if !document.ocr_blocks.is_empty() {
        document.ocr_blocks.len() as f64 * 150.0  // Espace après les figures
    } else {
        40.0
    };

    let mut i = 0;
    while i < content_lines.len() {
        let line = content_lines[i].trim();

        if line.is_empty() {
            i += 1;
            current_y += 20.0; // Espacement pour ligne vide
            continue;
        }

        // 🎯 NOUVELLE DÉTECTION INTELLIGENTE AVEC LOOK-AHEAD
        let (block_type, lines_consumed) = detect_content_type(&content_lines, i);
        
        // 📝 RECONSTRUCTION INTELLIGENTE DES BLOCS SELON LE TYPE
        let (block_content, actual_lines_consumed) = reconstruct_block_content(&content_lines, i, block_type.clone(), lines_consumed);

        if block_content.trim().is_empty() {
            i += 1;
            continue;
        }

        // 📊 Créer le contenu structuré selon le type
        let (final_content, confidence, height_multiplier) = match block_type {
            BlockType::Figure => {
                // Structurer les données de figure
                let structured_content = structure_figure_content(&[&block_content]);
                (structured_content, 0.85, 2.5)
            },
            BlockType::Table => {
                // Structurer les données tabulaires 
                let structured_content = structure_table_content(&[&block_content]);
                (structured_content, 0.88, 1.8)
            },
            BlockType::List => {
                // Structurer TOC/liste numérotée
                let structured_content = structure_list_content(&[&block_content]);
                (structured_content, 0.92, 1.5)
            },
            BlockType::Header => {
                (block_content.clone(), 0.95, 1.2)
            },
            BlockType::KeyValue => {
                (block_content.clone(), 0.90, 1.0)
            },
            BlockType::Amount => {
                (block_content.clone(), 0.93, 1.0)
            },
            _ => {
                // Text par défaut
                (block_content.clone(), 0.90, 1.0)
            }
        };

        // Calculer dimensions et position
        let line_count = final_content.lines().count().max(1);
        let base_height = (line_count as f64 * 20.0).max(30.0);
        let calculated_height = base_height * height_multiplier;
        
        // Ajuster largeur selon le type
        let width = match block_type {
            BlockType::Figure | BlockType::Table => 580.0, // Pleine largeur
            BlockType::Header => 580.0,
            _ => 560.0
        };

        let block = OCRBlock {
            page_number: 1, // Fallback synthétique = toujours page 1
            block_type: block_type.clone(),
            content: final_content,
            bounding_box: BoundingBox {
                x: 10.0,
                y: current_y,
                width,
                height: calculated_height,
            },
            confidence,
            spans: Vec::new(),
        };

        blocks.push(block);
        
        // Espacement adaptatif selon le type
        let spacing = match block_type {
            BlockType::Figure => 60.0, // Plus d'espace après les figures
            BlockType::Header => 40.0,
            BlockType::Table => 50.0,
            _ => 30.0
        };
        
        current_y += calculated_height + spacing;
        i += actual_lines_consumed.max(1); // Avancer du nombre de lignes RÉELLEMENT consommées
    }

    // Page unique pour MVP
    let page = OCRPage {
        page_number: 1,
        blocks: blocks.clone(),
        width: 595.0,  // A4 width in points
        height: 842.0, // A4 height in points
    };

    // Analyse de layout basique
    let layout_analysis = LayoutAnalysis {
        detected_columns: 1,
        has_tables: blocks.iter().any(|b| matches!(b.block_type, BlockType::Table)),
        has_headers: blocks.iter().any(|b| matches!(b.block_type, BlockType::Header)),
        text_density: 0.7, // Estimation
        dominant_font_size: Some(12.0),
    };

    // Confidence moyenne
    let total_confidence = if !blocks.is_empty() {
        blocks.iter().map(|b| b.confidence).sum::<f64>() / blocks.len() as f64
    } else {
        0.0
    };

    Ok(OCRContent {
        pages: vec![page],
        total_confidence,
        layout_analysis,
    })
}

/// Déterminer type de document (MVP - générique pour l'instant)
fn determine_document_type(document: &crate::rag::GroupDocument) -> DocumentType {
    // Pour MVP: utiliser le type existant du document
    document.document_type.clone()
}

/// Générer réponse contextuelle à partir des chunks pertinents - VERSION AMÉLIORÉE PR #4
/// Synthétise l'information au lieu de lister les chunks bruts
fn generate_contextual_response(
    scored_chunks: &[ScoredChunk],
    query: &str,
) -> Result<String, String> {
    if scored_chunks.is_empty() {
        return Ok("Aucune information pertinente trouvée.".to_string());
    }

    // Filtrer les chunks pertinents (score > 0.3)
    let top_chunks: Vec<&ScoredChunk> = scored_chunks
        .iter()
        .take(5) // Top 5 chunks pour une meilleure couverture
        .filter(|chunk| chunk.score > 0.3)
        .collect();

    if top_chunks.is_empty() {
        return Ok("Les informations trouvées ne semblent pas suffisamment pertinentes pour répondre à votre question.".to_string());
    }

    // Déterminer le type de question pour adapter la réponse
    let query_lower = query.to_lowercase();
    let is_summary_request = query_lower.contains("résume") || query_lower.contains("résumé")
        || query_lower.contains("synthèse") || query_lower.contains("overview");
    let is_explanation_request = query_lower.contains("explique") || query_lower.contains("comment")
        || query_lower.contains("pourquoi") || query_lower.contains("qu'est-ce");
    let is_list_request = query_lower.contains("quels") || query_lower.contains("quelles")
        || query_lower.contains("liste") || query_lower.contains("énumère");

    // Construire une réponse synthétisée
    let mut response = String::new();

    if is_summary_request {
        // Pour les demandes de résumé : synthèse narrative
        response.push_str("**Résumé du document :**\n\n");

        // Regrouper les informations clés
        let combined_content: String = top_chunks
            .iter()
            .map(|sc| sc.chunk.content.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        // Extraire les points clés (phrases importantes)
        let key_points = extract_key_sentences(&combined_content, 4);

        if !key_points.is_empty() {
            for (idx, point) in key_points.iter().enumerate() {
                response.push_str(&format!("{}. {}\n\n", idx + 1, point.trim()));
            }
        } else {
            // Fallback : utiliser les chunks directement mais de manière condensée
            for (idx, chunk) in top_chunks.iter().take(3).enumerate() {
                let preview = condense_text(&chunk.chunk.content, 200);
                response.push_str(&format!("• {}\n\n", preview));
            }
        }

    } else if is_explanation_request {
        // Pour les explications : réponse structurée avec contexte
        response.push_str("**Explication :**\n\n");

        // Prendre le chunk le plus pertinent comme réponse principale
        if let Some(best_chunk) = top_chunks.first() {
            let main_content = condense_text(&best_chunk.chunk.content, 400);
            response.push_str(&format!("{}\n\n", main_content));
        }

        // Ajouter contexte additionnel si pertinent
        if top_chunks.len() > 1 {
            response.push_str("**Informations complémentaires :**\n\n");
            for chunk in top_chunks.iter().skip(1).take(2) {
                let additional = condense_text(&chunk.chunk.content, 200);
                response.push_str(&format!("• {}\n\n", additional));
            }
        }

    } else if is_list_request {
        // Pour les listes : format bullet points
        response.push_str("**Éléments identifiés :**\n\n");

        for (idx, chunk) in top_chunks.iter().enumerate() {
            let item = condense_text(&chunk.chunk.content, 250);
            response.push_str(&format!("{}. {}\n\n", idx + 1, item));
        }

    } else {
        // Réponse générique : synthèse intelligente
        response.push_str("**Réponse basée sur le document :**\n\n");

        // Chunk principal
        if let Some(best_chunk) = top_chunks.first() {
            let main_answer = condense_text(&best_chunk.chunk.content, 350);
            response.push_str(&format!("{}\n\n", main_answer));
        }

        // Informations additionnelles si score > 0.5
        let highly_relevant: Vec<_> = top_chunks.iter()
            .skip(1)
            .filter(|sc| sc.score > 0.5)
            .take(2)
            .collect();

        if !highly_relevant.is_empty() {
            response.push_str("**Détails supplémentaires :**\n\n");
            for chunk in highly_relevant {
                let detail = condense_text(&chunk.chunk.content, 200);
                response.push_str(&format!("• {}\n\n", detail));
            }
        }
    }

    // Footer avec métadonnées
    let avg_score = top_chunks.iter().map(|sc| sc.score).sum::<f32>() / top_chunks.len() as f32;
    let confidence_level = if avg_score > 0.7 {
        "haute"
    } else if avg_score > 0.5 {
        "moyenne"
    } else {
        "modérée"
    };

    response.push_str(&format!("\n*Réponse générée à partir de {} sections du document (confiance: {})*",
                              top_chunks.len(), confidence_level));

    Ok(response)
}

/// Extraire les phrases clés d'un texte (phrases complètes)
fn extract_key_sentences(text: &str, max_sentences: usize) -> Vec<String> {
    let sentences: Vec<String> = text
        .split(|c| c == '.' || c == '!' || c == '?')
        .filter(|s| !s.trim().is_empty() && s.len() > 30) // Filtrer phrases trop courtes
        .take(max_sentences)
        .map(|s| s.trim().to_string())
        .collect();

    sentences
}

/// Condenser un texte à une longueur maximale en préservant les phrases complètes
fn condense_text(text: &str, max_chars: usize) -> String {
    let trimmed = text.trim();

    if trimmed.len() <= max_chars {
        return trimmed.to_string();
    }

    // Trouver la dernière phrase complète avant max_chars
    let truncated = &trimmed[..max_chars];

    // Chercher le dernier point/virgule/espace pour couper proprement
    if let Some(last_period) = truncated.rfind('.') {
        if last_period > max_chars / 2 { // Garder au moins 50% du texte
            return format!("{}.", &trimmed[..last_period]);
        }
    }

    // Sinon, chercher le dernier espace
    if let Some(last_space) = truncated.rfind(' ') {
        return format!("{}...", &trimmed[..last_space]);
    }

    // Fallback: couper directement
    format!("{}...", truncated)
}

/// Extraire spans contributeurs des chunks scorés - VERSION AMÉLIORÉE PR #4
/// Génère des SourceSpan avec bbox synthétiques pour le surlignage visuel
fn extract_contributing_spans(scored_chunks: &[ScoredChunk]) -> Vec<SourceSpan> {
    let mut all_spans = Vec::new();

    for (chunk_idx, scored_chunk) in scored_chunks.iter().enumerate() {
        // Note: source_spans dans EnrichedChunk contient des IDs (Vec<String>), pas des SourceSpan
        // Pour le moment, on génère toujours des spans synthétiques avec bbox pour visualisation

        // 1. Hash du contenu pour traçabilité
        let content_hash = blake3::hash(scored_chunk.chunk.content.as_bytes()).to_hex().to_string();

        // 2. Générer bbox synthétique basé sur la position du chunk
        // Position verticale: espacer les chunks de 120px (hauteur moyenne + marge)
        let y_position = (chunk_idx as f32) * 120.0 + 50.0; // 50px de marge top

        // Calculer hauteur approximative basée sur le contenu
        // Estimation: 80 caractères par ligne, hauteur ligne = 14px
        let estimated_lines = (scored_chunk.chunk.content.len() as f32 / 80.0).ceil();
        let estimated_height = (estimated_lines * 14.0).min(100.0); // Max 100px

        let synthetic_bbox = Some(crate::rag::core::source_spans::BoundingBox {
            page: Some(1), // Page 1 par défaut pour MVP
            x: 50.0,  // Marge gauche standard
            y: y_position,
            width: 500.0, // Largeur standard pour A4 (595px - marges)
            height: estimated_height,
            rotation: None, // Pas de rotation
            coordinate_system: crate::rag::core::source_spans::CoordinateSystem::PdfPoints,
        });

        // 3. Créer le SourceSpan synthétique enrichi
        let synthetic_span = SourceSpan {
            span_id: format!("synthetic_chunk_{}", scored_chunk.chunk.id),
            document_id: "direct_chat_temp".to_string(),
            document_path: std::path::PathBuf::from("temp_document"),
            char_start: 0,
            char_end: scored_chunk.chunk.content.len(),
            line_start: scored_chunk.chunk.start_line,
            line_end: scored_chunk.chunk.end_line,
            bbox: synthetic_bbox, // ✅ BBOX SYNTHÉTIQUE AJOUTÉ
            original_content: scored_chunk.chunk.content.clone(),
            extraction_metadata: ExtractionMetadata {
                method: scored_chunk.chunk.metadata.extraction_method.clone(),
                confidence: scored_chunk.chunk.metadata.confidence,
                language: Some(scored_chunk.chunk.metadata.language.clone()),
                method_specific: {
                    let mut map = std::collections::HashMap::new();
                    map.insert("chunk_type".to_string(), serde_json::Value::String(format!("{:?}", scored_chunk.chunk.chunk_type)));
                    map.insert("relevance_score".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(scored_chunk.score as f64).unwrap_or(serde_json::Number::from(0))));
                    map.insert("is_synthetic".to_string(), serde_json::Value::Bool(true));
                    map
                },
                content_hash,
            },
            created_at: std::time::SystemTime::now(),
        };

        all_spans.push(synthetic_span);
    }

    all_spans
}

/// Calculer confidence de la réponse
fn calculate_response_confidence(scored_chunks: &[ScoredChunk]) -> f64 {
    if scored_chunks.is_empty() {
        return 0.0;
    }

    // Moyenne pondérée: score de similarité * confidence du chunk
    let weighted_sum: f64 = scored_chunks
        .iter()
        .map(|sc| sc.score as f64 * sc.chunk.metadata.confidence as f64)
        .sum();
    
    let total_weight: f64 = scored_chunks
        .iter()
        .map(|sc| sc.score as f64)
        .sum();

    if total_weight > 0.0 {
        weighted_sum / total_weight
    } else {
        0.0
    }
}

/// Créer résumé des sources contributives - VERSION AMÉLIORÉE PR #4
fn create_sources_summary(scored_chunks: &[ScoredChunk]) -> Vec<SourceSummary> {
    scored_chunks
        .iter()
        .take(5) // Top 5 sources
        .map(|sc| {
            let preview = if sc.chunk.content.len() > 200 {
                format!("{}...", sc.chunk.content.chars().take(200).collect::<String>())
            } else {
                sc.chunk.content.clone()
            };

            // Compter les spans existants ou synthétiques
            let span_count = sc.chunk.source_spans
                .as_ref()
                .map(|spans| spans.len())
                .unwrap_or(1); // ✅ Au moins 1 span synthétique sera généré

            SourceSummary {
                chunk_id: sc.chunk.id.clone(),
                content_preview: preview,
                score: sc.score,
                confidence: sc.chunk.metadata.confidence as f64,
                span_count,
            }
        })
        .collect()
}

/// Sprint 1 Niveau 1: Construire contexte formaté pour LLM synthesis
/// Retourne: (formatted_context, chunk_infos, has_ocr_data)
fn build_llm_context(scored_chunks: &[ScoredChunk]) -> (String, Vec<LlmChunkInfo>, bool) {
    use crate::rag::ChunkSource;

    let mut context_parts = Vec::new();
    let mut chunk_infos = Vec::new();
    let mut has_ocr_data = false;

    for (i, scored_chunk) in scored_chunks.iter().enumerate() {
        let chunk = &scored_chunk.chunk;

        // Déterminer le label de source
        let source_label = match chunk.chunk_source {
            ChunkSource::FigureCaption => {
                format!("Figure Caption - {}",
                    chunk.figure_id.as_deref().unwrap_or("Unknown"))
            },
            ChunkSource::FigureRegionText => {
                has_ocr_data = true;
                format!("Figure OCR - {}",
                    chunk.figure_id.as_deref().unwrap_or("Unknown"))
            },
            ChunkSource::Table => "Table".to_string(),
            ChunkSource::BodyText => "Document Text".to_string(),
            _ => "Content".to_string(),
        };

        // SIMPLIFICATION 23 Nov: Tronquer à 500 chars au lieu de 800 pour réduire latence LLM
        let truncated_content: String = chunk.content
            .chars()
            .take(500)
            .collect();

        // Formater pour le contexte LLM
        let context_block = format!(
            "### Source {} - {} (Page {}, Confidence: {:.0}%)\n{}\n",
            i + 1,
            source_label,
            chunk.start_line, // TODO: Utiliser page_index réel si disponible
            chunk.metadata.confidence * 100.0,
            truncated_content
        );
        context_parts.push(context_block);

        // Créer info du chunk
        chunk_infos.push(LlmChunkInfo {
            chunk_id: chunk.id.clone(),
            source_label,
            content: truncated_content,
            score: scored_chunk.score,
            confidence: chunk.metadata.confidence as f64,
            page: None, // TODO: Extraire vrai page_index
            figure_id: chunk.figure_id.clone(),
            source_type: format!("{:?}", chunk.chunk_source),
        });
    }

    // Joindre avec séparateurs
    let formatted_context = context_parts.join("\n---\n\n");

    (formatted_context, chunk_infos, has_ocr_data)
}

/// Calculer confidence globale d'une session
fn calculate_session_confidence(session: &DirectChatSession) -> f64 {
    if session.chunks.is_empty() {
        return 0.0;
    }

    let avg_chunk_confidence: f64 = session.chunks
        .iter()
        .map(|chunk| chunk.metadata.confidence as f64)
        .sum::<f64>() / session.chunks.len() as f64;

    // Facteur OCR
    let ocr_factor = session.search_content.total_confidence;
    
    // Facteur embeddings
    let embedding_factor = if session.embedded_chunks_count() > 0 {
        session.embedded_chunks_count() as f64 / session.chunks.len() as f64
    } else {
        0.0
    };

    // Moyenne pondérée
    avg_chunk_confidence * 0.4 + ocr_factor * 0.3 + embedding_factor * 0.3
}

/// Convertir un fichier temporaire en URL accessible pour affichage PDF
#[tauri::command]
pub async fn get_temp_pdf_url(
    file_path: String,
) -> Result<TempPdfUrlResponse, String> {
    info!("🔗 Converting temp file to accessible URL: {}", file_path);
    
    let path = std::path::Path::new(&file_path);
    
    // Vérifier que le fichier existe
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }
    
    // Pour l'instant, utiliser convertFileSrc concept de Tauri
    // Note: Il faudra peut-être ajuster selon la config Tauri
    let pdf_url = format!("file://{}", path.to_string_lossy());
    
    info!("✅ Generated PDF URL: {} → {}", file_path, pdf_url);
    
    Ok(TempPdfUrlResponse {
        pdf_url,
        original_path: file_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_path_resolution() {
        let exemple_path = "exemple/test.pdf";
        let result = resolve_file_path(exemple_path);
        
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("exemple"));
        assert!(path.to_string_lossy().ends_with("test.pdf"));
    }

    #[test]
    fn test_confidence_calculation() {
        let scored_chunks = vec![
            ScoredChunk {
                chunk: create_test_chunk("chunk1", 0.9),
                score: 0.8,
            },
            ScoredChunk {
                chunk: create_test_chunk("chunk2", 0.7),
                score: 0.6,
            },
        ];

        let confidence = calculate_response_confidence(&scored_chunks);
        assert!(confidence > 0.0 && confidence <= 1.0);
    }

    fn create_test_chunk(id: &str, confidence: f32) -> crate::rag::EnrichedChunk {
        use crate::rag::{ChunkType, ChunkMetadata, Priority, SourceType, ExtractionMethod};
        
        crate::rag::EnrichedChunk {
            id: id.to_string(),
            content: "test content".to_string(),
            start_line: 1,
            end_line: 1,
            chunk_type: ChunkType::TextBlock,
            embedding: None,
            hash: "test_hash".to_string(),
            metadata: ChunkMetadata {
                tags: vec!["test".to_string()],
                priority: Priority::Normal,
                language: "fr".to_string(),
                symbol: None,
                context: None,
                confidence,
                ocr_metadata: None,
                source_type: SourceType::NativeText,
                extraction_method: ExtractionMethod::DirectRead,
            },
            group_id: "test_group".to_string(),
            source_spans: None,
        }
    }
}