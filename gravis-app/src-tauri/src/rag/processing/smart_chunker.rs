// GRAVIS RAG - Smart Chunker Section-Aware
// Chunking intelligent avec détection de sections et tailles optimisées

use regex::Regex;
use anyhow::Result;
use tracing::{debug, info};

use crate::rag::{EnrichedChunk, ChunkType, ChunkMetadata, SourceType, ExtractionMethod, Priority, LigatureCleaner};

/// Configuration pour le chunking intelligent
#[derive(Debug, Clone)]
pub struct SmartChunkConfig {
    /// Taille cible en tokens (600-900 recommandé)
    pub target_tokens: usize,
    /// Overlap en pourcentage (10-15% recommandé)  
    pub overlap_percent: f32,
    /// Taille minimum pour créer un chunk
    pub min_tokens: usize,
    /// Taille maximum avant split forcé
    pub max_tokens: usize,
    /// Facteur de conversion approximatif chars → tokens
    pub chars_per_token: f32,
    /// Overlap dynamique target ratio pour optimisation (optionnel)
    pub overlap_target_ratio: Option<f32>,
    /// MMR lambda pour équilibre relevance/diversité
    pub mmr_lambda: f32,
    /// Nombre maximum de documents de contexte final
    pub max_context_docs: usize,
}

impl Default for SmartChunkConfig {
    fn default() -> Self {
        Self::academic_optimized()
    }
}

impl SmartChunkConfig {
    /// Configuration optimisée pour documents académiques (Phase 2 validée)
    pub fn academic_optimized() -> Self {
        Self {
            target_tokens: 500,        // Plus petit pour plus de chunks
            overlap_percent: 0.15,     // 15% overlap
            min_tokens: 150,           // Minimum réduit (600 chars)
            max_tokens: 800,           // Maximum plus agressif (3200 chars)
            chars_per_token: 4.0,      // ~4.0 chars/token optimisé pour académique
            overlap_target_ratio: Some(0.15), // Target ratio pour overlap dynamique
            mmr_lambda: 0.5,           // Équilibre relevance/diversité
            max_context_docs: 5,       // Top-5 final après MMR
        }
    }

    /// Configuration adaptative pour documents Business (Phase 3A)
    pub fn business_optimized() -> Self {
        Self {
            target_tokens: 500,           // Sections business plus longues
            overlap_percent: 0.15,        // Contexte financier important
            min_tokens: 200,              // Minimum plus élevé pour contexte business
            max_tokens: 1000,             // Maximum plus généreux pour rapports
            chars_per_token: 4.0,         // Même ratio que académique
            overlap_target_ratio: Some(0.15), // Target optimisé Phase 2
            mmr_lambda: 0.6,              // Plus de relevance pour business
            max_context_docs: 6,          // Plus de contexte pour analyse
        }
    }

    /// Configuration pour documents Legal (Phase 3B - à implémenter)
    pub fn legal_optimized() -> Self {
        Self {
            target_tokens: 600,           // Clauses plus longues
            overlap_percent: 0.30,        // Overlap élevé pour références
            min_tokens: 250,              // Clauses substantielles
            max_tokens: 1200,             // Clauses longues autorisées
            chars_per_token: 4.0,
            overlap_target_ratio: Some(0.30),
            mmr_lambda: 0.4,              // Plus de diversité pour legal
            max_context_docs: 8,          // Beaucoup de contexte pour legal
        }
    }

    /// Configuration pour documents Technical (Phase 3C - à implémenter)
    pub fn technical_optimized() -> Self {
        Self {
            target_tokens: 450,           // Chunks plus courts pour précision
            overlap_percent: 0.18,        // Overlap modéré
            min_tokens: 180,              // Minimum standard
            max_tokens: 900,              // Maximum standard
            chars_per_token: 4.0,
            overlap_target_ratio: Some(0.18),
            mmr_lambda: 0.5,              // Équilibré
            max_context_docs: 5,          // Standard
        }
    }

    /// Configuration universelle balanced (Mixed documents)
    pub fn mixed_universal() -> Self {
        Self {
            target_tokens: 500,           // Taille standard
            overlap_percent: 0.18,        // Overlap modéré
            min_tokens: 180,              // Minimum standard
            max_tokens: 900,              // Maximum standard
            chars_per_token: 4.0,
            overlap_target_ratio: Some(0.18),
            mmr_lambda: 0.5,              // Équilibré
            max_context_docs: 5,          // Standard
        }
    }
}

/// Métadonnées enrichies pour chunks avec sections
#[derive(Debug, Clone)]
pub struct ChunkSection {
    pub section_title: Option<String>,
    pub section_level: u8,              // 1=titre principal, 2=sous-section, etc.
    pub section_number: Option<String>,  // "2.1", "3.2.1", etc.
    pub page_number: Option<u32>,
    pub char_range: (usize, usize),     // Position dans le document original
}

/// Résultat d'un chunking intelligent
#[derive(Debug)]
pub struct SmartChunkResult {
    pub chunks: Vec<EnrichedChunk>,
    pub sections_detected: Vec<String>,
    pub total_chars: usize,
    pub avg_chunk_size: f32,
    pub processing_time_ms: u64,
}

/// Chunker intelligent avec détection de sections
pub struct SmartChunker {
    config: SmartChunkConfig,
    section_regex: Regex,
    sentence_regex: Regex,
    ligature_cleaner: LigatureCleaner,
}

impl SmartChunker {
    /// Crée un nouveau chunker intelligent (générique - utilise patterns académiques)
    pub fn new(config: SmartChunkConfig) -> Result<Self> {
        Self::new_academic(config)
    }

    /// Crée un chunker optimisé pour documents académiques
    pub fn new_academic(config: SmartChunkConfig) -> Result<Self> {
        // Regex pour détecter les vrais titres de sections (exclut figures/légendes)
        let section_regex = Regex::new(
            r"(?m)^(?:\s*(\d+(?:\.\d+)*)\s*\.?\s*)?(Abstract|Introduction|Related\s+Works?|Methods?|Methodology|Approach|Experiments?|Evaluation|Results?|Discussion|Conclusion|Future\s+Work|Limitations|Acknowledgments?|References|Appendix|Background)\s*$"
        )?;
        
        // Regex pour split par phrases avec boundary penalty réduit (pas de lookahead)
        let sentence_regex = Regex::new(r"[.!?]\s+[A-Z]|\n\s*\n|;\s+[A-Z]")?;
        
        Ok(Self {
            config,
            section_regex,
            sentence_regex,
            ligature_cleaner: LigatureCleaner::default(),
        })
    }

    /// Crée un chunker optimisé pour documents Business (Phase 3A)
    pub fn new_business(config: SmartChunkConfig) -> Result<Self> {
        // Regex pour sections Business selon feuille de route
        let section_regex = Regex::new(
            r"(?m)^(?:\s*(\d+(?:\.\d+)*)\s*\.?\s*)?(Executive\s+Summary|Financial\s+(?:Performance|Highlights)|Business\s+Overview|Risk\s+Factors|Management\s+Discussion|Market\s+Analysis|Financial\s+Results|Key\s+Metrics|Shareholder|Governance|Sustainability)\s*$"
        )?;
        
        // Regex pour split par phrases (même que académique)
        let sentence_regex = Regex::new(r"[.!?]\s+[A-Z]|\n\s*\n|;\s+[A-Z]")?;
        
        Ok(Self {
            config,
            section_regex,
            sentence_regex,
            ligature_cleaner: LigatureCleaner::default(),
        })
    }

    /// Crée un chunker optimisé pour documents Legal (Phase 3B - à implémenter)
    pub fn new_legal(config: SmartChunkConfig) -> Result<Self> {
        // Patterns pour clauses légales
        let section_regex = Regex::new(
            r"(?m)^(?:\s*(\d+(?:\.\d+)*)\s*\.?\s*)?(Article|Clause|Section|Whereas|Therefore|Party|Obligation|Liability|Termination|Definitions|Representations|Warranties)\s*$"
        )?;
        
        let sentence_regex = Regex::new(r"[.!?]\s+[A-Z]|\n\s*\n|;\s+[A-Z]")?;
        
        Ok(Self {
            config,
            section_regex,
            sentence_regex,
            ligature_cleaner: LigatureCleaner::default(),
        })
    }

    /// Crée un chunker optimisé pour documents Technical (Phase 3C - à implémenter)
    pub fn new_technical(config: SmartChunkConfig) -> Result<Self> {
        // Patterns pour docs techniques
        let section_regex = Regex::new(
            r"(?m)^(?:\s*(\d+(?:\.\d+)*)\s*\.?\s*)?(Specification|Implementation|Algorithm|API|Interface|Requirements|Architecture|Design|Testing|Deployment)\s*$"
        )?;
        
        let sentence_regex = Regex::new(r"[.!?]\s+[A-Z]|\n\s*\n|;\s+[A-Z]")?;
        
        Ok(Self {
            config,
            section_regex,
            sentence_regex,
            ligature_cleaner: LigatureCleaner::default(),
        })
    }

    /// Calcul dynamique de l'overlap basé sur P50 tokens observé
    fn calculate_dynamic_overlap(&self, p50_tokens: usize) -> f32 {
        // Formule: overlap = clamp(round(0.15 * p50_tokens), 20, 64) / p50_tokens
        let overlap_tokens = ((0.15 * p50_tokens as f32).round() as usize)
            .clamp(20, 64);
        overlap_tokens as f32 / p50_tokens as f32
    }

    /// Chunking principal avec détection de sections
    pub fn chunk_document(
        &mut self,
        content: &str,
        source_type: SourceType,
        extraction_method: &ExtractionMethod,
        group_id: &str,
    ) -> Result<SmartChunkResult> {
        let start_time = std::time::Instant::now();
        
        debug!("Smart chunking {} chars with section detection", content.len());
        
        // 0. Nettoyage des ligatures avec logging
        let cleaned_content = self.ligature_cleaner.clean_and_log(content, "document_chunking");
        
        // 1. Détection des sections
        let sections = self.detect_sections(&cleaned_content);
        info!("Detected {} sections", sections.len());
        
        // 2. Split en segments par section
        let section_segments = self.split_by_sections(&cleaned_content, &sections);
        
        // 3. Chunking intelligent de chaque segment
        let mut all_chunks = Vec::new();
        let mut chunk_index = 0;
        
        for (section_info, section_content) in section_segments {
            let section_chunks = self.chunk_section(
                &section_content,
                &section_info,
                &mut chunk_index,
                source_type.clone(),
                extraction_method,
                group_id,
            )?;
            
            all_chunks.extend(section_chunks);
        }
        
        let processing_time = start_time.elapsed();
        let avg_chunk_size = if !all_chunks.is_empty() {
            all_chunks.iter().map(|c| c.content.len()).sum::<usize>() as f32 / all_chunks.len() as f32
        } else {
            0.0
        };
        
        // Générer le résumé des ligatures traitées
        self.ligature_cleaner.log_summary();
        
        Ok(SmartChunkResult {
            chunks: all_chunks,
            sections_detected: sections.iter().map(|s| s.section_title.clone().unwrap_or_default()).collect(),
            total_chars: content.len(),
            avg_chunk_size,
            processing_time_ms: processing_time.as_millis() as u64,
        })
    }

    /// Détecte les sections dans le contenu
    fn detect_sections(&self, content: &str) -> Vec<ChunkSection> {
        let mut sections = Vec::new();
        
        for cap in self.section_regex.captures_iter(content) {
            let full_match = cap.get(0).unwrap();
            let section_number = cap.get(1).map(|m| m.as_str().to_string());
            let section_title = cap.get(2).map(|m| m.as_str().to_string());
            
            let section_level = if section_number.is_some() {
                section_number.as_ref().unwrap().matches('.').count() as u8 + 1
            } else {
                1
            };
            
            if let Some(title) = section_title {
                // Filtrer les faux titres (figures, captions, légendes)
                if !self.is_false_heading(&title) {
                    sections.push(ChunkSection {
                        section_title: Some(title),
                        section_level,
                        section_number,
                        page_number: None,
                        char_range: (full_match.start(), full_match.end()),
                    });
                }
            }
        }
        
        debug!("Detected {} valid sections: {:?}", sections.len(), sections.iter().map(|s| &s.section_title).collect::<Vec<_>>());
        sections
    }
    
    /// Vérifie si un titre est probablement une figure/légende (faux titre)
    fn is_false_heading(&self, title: &str) -> bool {
        let title_lower = title.to_lowercase();
        
        // Patterns de faux titres
        title_lower.contains("figure") ||
        title_lower.contains("table") ||
        title_lower.contains("image") ||
        title_lower.contains("result") ||
        title_lower.contains("input") ||
        title_lower.contains("output") ||
        title_lower.contains("clear") ||
        title_lower.contains("blurry") ||
        title_lower.contains("crystal") ||
        title_lower.contains("gundam") ||
        title_lower.contains("large") ||
        title_lower.contains("small") ||
        title_lower.contains("tiny") ||
        title_lower.contains("vision") ||
        title_lower.contains("text token") ||
        title_lower.contains("memory") ||
        title_lower.contains("pipeline") ||
        title_lower.len() < 3 ||  // Titres trop courts
        title_lower.chars().filter(|&c| c.is_alphabetic()).count() < 3  // Trop peu de lettres
    }

    /// Split le contenu par sections détectées avec merge des sections trop petites
    fn split_by_sections(&self, content: &str, sections: &[ChunkSection]) -> Vec<(ChunkSection, String)> {
        let mut segments = Vec::new();
        
        if sections.is_empty() {
            // Pas de sections détectées, traiter comme un seul segment
            segments.push((
                ChunkSection {
                    section_title: Some("Document".to_string()),
                    section_level: 1,
                    section_number: None,
                    page_number: None,
                    char_range: (0, content.len()),
                },
                content.to_string(),
            ));
            return segments;
        }
        
        let min_section_chars = (self.config.min_tokens as f32 * self.config.chars_per_token * 3.0) as usize; // ~2400 chars
        let mut pending_merge: Option<(ChunkSection, String)> = None;
        
        for (i, section) in sections.iter().enumerate() {
            let start = section.char_range.0;
            let end = if i + 1 < sections.len() {
                sections[i + 1].char_range.0
            } else {
                content.len()
            };
            
            if start < content.len() && end <= content.len() && start < end {
                let section_content = content[start..end].to_string();
                let current_segment = (section.clone(), section_content);
                
                // Si section trop petite, la merger avec la précédente ou suivante
                if current_segment.1.len() < min_section_chars {
                    if let Some(mut prev_segment) = pending_merge.take() {
                        // Merger avec la section précédente
                        prev_segment.1.push_str(&current_segment.1);
                        let merged_title = format!("{} + {}", 
                                                 prev_segment.0.section_title.as_deref().unwrap_or("Unknown"),
                                                 current_segment.0.section_title.as_deref().unwrap_or("Unknown"));
                        prev_segment.0.section_title = Some(merged_title);
                        segments.push(prev_segment);
                        debug!("Merged small section: {} chars", current_segment.1.len());
                    } else {
                        // Garder en attente pour merger avec la suivante
                        pending_merge = Some(current_segment);
                    }
                } else {
                    // Section assez grande, finaliser any pending merge
                    if let Some(prev_segment) = pending_merge.take() {
                        segments.push(prev_segment);
                    }
                    segments.push(current_segment);
                }
            }
        }
        
        // Finaliser dernière section en attente
        if let Some(prev_segment) = pending_merge {
            segments.push(prev_segment);
        }
        
        debug!("Created {} segments after merging small sections", segments.len());
        segments
    }

    /// Chunk une section spécifique avec taille optimisée
    fn chunk_section(
        &self,
        content: &str,
        section_info: &ChunkSection,
        chunk_index: &mut usize,
        source_type: SourceType,
        extraction_method: &ExtractionMethod,
        group_id: &str,
    ) -> Result<Vec<EnrichedChunk>> {
        let target_chars = (self.config.target_tokens as f32 * self.config.chars_per_token) as usize;
        
        // Calcul overlap dynamique si configuré
        let overlap_ratio = if let Some(target_ratio) = self.config.overlap_target_ratio {
            let p50_tokens = self.config.target_tokens; // Approximation du P50
            self.calculate_dynamic_overlap(p50_tokens).min(target_ratio)
        } else {
            self.config.overlap_percent
        };
        
        let overlap_chars = (target_chars as f32 * overlap_ratio) as usize;
        let max_chars = (self.config.max_tokens as f32 * self.config.chars_per_token) as usize;
        
        let mut chunks = Vec::new();
        
        // Si la section est petite, la traiter comme un seul chunk
        if content.len() <= target_chars {
            if content.trim().len() >= (self.config.min_tokens as f32 * self.config.chars_per_token) as usize {
                let chunk = self.create_enriched_chunk(
                    content.trim(),
                    *chunk_index,
                    section_info,
                    source_type.clone(),
                    extraction_method,
                    group_id,
                )?;
                chunks.push(chunk);
                *chunk_index += 1;
            }
            return Ok(chunks);
        }
        
        // Split par phrases pour préserver la cohérence
        let sentences: Vec<&str> = self.sentence_regex.split(content).collect();
        let mut current_chunk = String::new();
        let mut current_size = 0;
        
        for sentence in sentences {
            let sentence = sentence.trim();
            if sentence.is_empty() {
                continue;
            }
            
            let sentence_size = sentence.len();
            
            // Si ajouter cette phrase dépasse la taille cible
            if current_size + sentence_size > target_chars && !current_chunk.is_empty() {
                // Créer le chunk actuel
                let chunk = self.create_enriched_chunk(
                    &current_chunk,
                    *chunk_index,
                    section_info,
                    source_type.clone(),
                    extraction_method,
                    group_id,
                )?;
                chunks.push(chunk);
                *chunk_index += 1;
                
                // Commencer nouveau chunk avec overlap - safe Unicode slicing
                let overlap_content = if current_chunk.chars().count() > overlap_chars {
                    let skip_chars = current_chunk.chars().count().saturating_sub(overlap_chars);
                    current_chunk.chars().skip(skip_chars).collect::<String>()
                } else {
                    current_chunk.clone()
                };
                
                current_chunk = format!("{} {}", overlap_content, sentence);
                current_size = current_chunk.len();
            } else {
                // Ajouter à ce chunk
                if !current_chunk.is_empty() {
                    current_chunk.push(' ');
                }
                current_chunk.push_str(sentence);
                current_size += sentence_size + 1;
            }
            
            // Split forcé si trop grand
            if current_size > max_chars {
                let chunk = self.create_enriched_chunk(
                    &current_chunk,
                    *chunk_index,
                    section_info,
                    source_type.clone(),
                    extraction_method,
                    group_id,
                )?;
                chunks.push(chunk);
                *chunk_index += 1;
                current_chunk.clear();
                current_size = 0;
            }
        }
        
        // Finaliser le dernier chunk
        if !current_chunk.trim().is_empty() {
            let chunk = self.create_enriched_chunk(
                &current_chunk,
                *chunk_index,
                section_info,
                source_type.clone(),
                extraction_method,
                group_id,
            )?;
            chunks.push(chunk);
            *chunk_index += 1;
        }
        
        debug!("Section '{}' → {} chunks", 
               section_info.section_title.as_deref().unwrap_or("Unknown"), 
               chunks.len());
        
        Ok(chunks)
    }

    /// Crée un chunk enrichi avec métadonnées de section
    fn create_enriched_chunk(
        &self,
        content: &str,
        index: usize,
        section_info: &ChunkSection,
        source_type: SourceType,
        extraction_method: &ExtractionMethod,
        group_id: &str,
    ) -> Result<EnrichedChunk> {
        let confidence = match source_type {
            SourceType::NativeText => 1.0,
            SourceType::OcrExtracted => 0.8,
            _ => 0.9,
        };
        
        let mut chunk = EnrichedChunk {
            id: format!("chunk_smart_{}_{}", uuid::Uuid::new_v4().simple(), index),
            content: content.to_string(),
            start_line: index,
            end_line: index + 1,
            chunk_type: ChunkType::TextBlock,
            embedding: None,
            hash: String::new(),
            metadata: ChunkMetadata {
                tags: vec![
                    format!("section:{}", section_info.section_title.as_deref().unwrap_or("unknown")),
                    format!("level:{}", section_info.section_level),
                ],
                priority: if confidence > 0.9 { Priority::High } else { Priority::Normal },
                language: "fra".to_string(),
                symbol: None,
                context: Some(format!(
                    "Section: {} (Level {})", 
                    section_info.section_title.as_deref().unwrap_or("Unknown"),
                    section_info.section_level
                )),
                confidence,
                ocr_metadata: None,
                source_type,
                extraction_method: extraction_method.clone(),
            },
            group_id: group_id.to_string(),
            source_spans: None,
        };

        chunk.generate_hash();
        Ok(chunk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag::ExtractionMethod;

    #[test]
    fn test_section_detection() {
        let chunker = SmartChunker::new(SmartChunkConfig::default()).unwrap();
        
        let content = r#"
1. Introduction

This is the introduction section.

2. Method

This describes our method.

2.1 Data Processing

Details about data processing.

3. Results

Our experimental results.
"#;
        
        let sections = chunker.detect_sections(content);
        assert_eq!(sections.len(), 4);
        assert_eq!(sections[0].section_title, Some("Introduction".to_string()));
        assert_eq!(sections[1].section_title, Some("Method".to_string()));
        assert_eq!(sections[2].section_title, Some("Data Processing".to_string()));
        assert_eq!(sections[3].section_title, Some("Results".to_string()));
    }
}