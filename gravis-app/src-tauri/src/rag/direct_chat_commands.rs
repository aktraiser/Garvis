// Phase 2: Commandes Tauri pour Chat Direct - Mode Générique (MVP)
// Interface frontend/backend pour sessions temporaires

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;
use tracing::{info, warn, error};

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

    let session = DirectChatSession::new(
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

    // 9. Nettoyer le fichier temporaire
    if let Err(e) = std::fs::remove_file(&temp_path) {
        warn!("Failed to clean up temp file {:?}: {}", temp_path, e);
    }

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

/// Supprimer session temporaire
#[tauri::command]
pub async fn cleanup_direct_chat_session(
    session_id: String,
    state: State<'_, DirectChatState>,
) -> Result<(), String> {
    info!("🗑️ Cleaning up direct chat session: {}", session_id);

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

/// Créer contenu OCR à partir du document traité (MVP - structure basique)
fn create_ocr_content_from_document(
    document: &crate::rag::GroupDocument
) -> Result<OCRContent, String> {
    // Pour MVP: créer structure OCR basique à partir des chunks
    let mut blocks = Vec::new();
    
    for (idx, chunk) in document.chunks.iter().enumerate() {
        // Déterminer type de bloc basique
        let block_type = match chunk.chunk_type {
            crate::rag::ChunkType::Function => BlockType::Text,
            crate::rag::ChunkType::Class => BlockType::Header,
            crate::rag::ChunkType::Module => BlockType::Header,
            crate::rag::ChunkType::TextBlock => BlockType::Text,
            crate::rag::ChunkType::Comment => BlockType::Text,
        };

        // Position estimée (MVP - layout simple)
        let y_position = idx as f64 * 100.0; // Espacement vertical
        let bounding_box = BoundingBox {
            x: 10.0,
            y: y_position,
            width: 580.0, // Largeur standard A4
            height: 80.0,  // Hauteur estimée
        };

        // Confidence depuis les métadonnées OCR si disponible
        let confidence = chunk.metadata.ocr_metadata
            .as_ref()
            .map(|_ocr| 0.95) // Default confidence since OcrMetadata doesn't have confidence field
            .unwrap_or(chunk.metadata.confidence as f64);

        let block = OCRBlock {
            block_type,
            content: chunk.content.clone(),
            bounding_box,
            confidence,
            spans: chunk.source_spans
                .as_ref()
                .cloned()
                .unwrap_or_default(),
        };

        blocks.push(block);
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
    let ocr_factor = session.ocr_content.total_confidence;
    
    // Facteur embeddings
    let embedding_factor = if session.embedded_chunks_count() > 0 {
        session.embedded_chunks_count() as f64 / session.chunks.len() as f64
    } else {
        0.0
    };

    // Moyenne pondérée
    (avg_chunk_confidence * 0.4 + ocr_factor * 0.3 + embedding_factor * 0.3)
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