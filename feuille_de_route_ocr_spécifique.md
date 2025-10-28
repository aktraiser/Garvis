# 🚀 GRAVIS Universal RAG Pipeline - Feuille de Route Production

**Version:** 1.1  
**Date:** 27 octobre 2025  
**Statut:** Phase 2 Validée → Phase 3 Planning + Optimisations Prod  
**Dernière révision:** Ajout optimisations haute valeur / faible effort

  ---
  📋 Executive Summary

  Le pipeline RAG Phase 2 "production-grade" est validé avec succès sur les documents académiques avec
   tous les garde-fous CI (4/4). Cette feuille de route définit l'évolution vers un système Universal 
  RAG capable de traiter n'importe quel type de PDF avec une qualité production.

  🎯 Objectifs Phase 3

  Évoluer du pipeline actuel (optimisé papiers académiques) vers un système universel supportant :

  | Type Document | Exemple                              | Priorité |
  |---------------|--------------------------------------|----------|
  | Business      | Rapports annuels, slides corporate   | 🥇 P1    |
  | Legal         | Contrats, règlements, jurisprudence  | 🥈 P2    |
  | Technical     | Manuels, docs avec figures & schémas | 🥉 P3    |
  | Scanned/Mixed | PDFs OCR, documents archivés         | 📋 P4    |

  ---
  ✅ État Actuel - Phase 2 "Production-Grade"

  🏆 Résultats Validés

  Pipeline "Academic-Optimized" complet :
  - ✅ 47 chunks pour 70k chars → P50: 1520 chars (≈380 tokens/chunk)
  - ✅ Boundary penalty: 0.043 (9x meilleur que limite 0.35)
  - ✅ Overlap optimisé: 21.6% (cible 12-22%)
  - ✅ Search quality: 0.490 (> seuil 0.48)
  - ✅ Performance: 12.70ms/chunk
  - ✅ Garde-fous CI: 4/4 validés automatiquement

  🔧 Architecture Technique Solide

  PDF → TesseractOCR → SmartChunker → CustomE5(384D) → Qdrant(HNSW) → MMR → Results

  Composants validés :
  - Singleton CustomE5 Embedder - Évite double initialisation
  - Smart Section-Aware Chunker - Détection regex académique
  - Qdrant HNSW optimisé - m=32, ef_construct=256, ef_search=128
  - MMR Re-ranking - Diversité avec λ=0.5, top-5 final
  - Ligature Cleaner - Nettoyage Unicode avec sampling
  - CI Health Guards - Protection automatique contre régressions

  ---
  🚀 Phase 3 - Universal RAG Implementation

  📅 Timeline Globale (6 semaines)

  | Semaine | Focus                  | Delivery                             |
  |---------|------------------------|--------------------------------------|
  | 1       | Document Classifier    | Module Rust de détection automatique |
  | 2       | Adaptive Chunking      | Configs par type + tests             |
  | 3       | Layout Metadata        | Navigation UI + bbox                 |
  | 4       | Multimodal Basics      | Détection tableaux                   |
  | 5       | Benchmarks Multi-types | Scoreboard CI complet                |
  | 6       | RAG Explorer UI        | Interface visualisation              |

  💰 Budget & Observabilité Production

  📊 Budget-Aware Processing

  /// Contrôle des coûts avec budgets configurables par opération
  pub struct BudgetManager {
      embedding_budget: TokenBudget,     // Limite tokens embedding/jour
      search_budget: QueryBudget,       // Limite queries/heure  
      storage_budget: StorageBudget,    // Limite stockage Qdrant
      processing_budget: ProcessingBudget, // Limite CPU/RAM
  }

  #[derive(Debug, Clone)]
  pub struct TokenBudget {
      daily_limit: usize,        // Max tokens embeddable/jour
      current_usage: usize,      // Usage actuel
      priority_reserve: usize,   // Tokens réservés priority HIGH
      reset_time: SystemTime,    // Reset quotidien
  }

  impl BudgetManager {
      pub fn can_process_document(&self, doc_size: usize, priority: Priority) -> bool {
          let estimated_tokens = doc_size / 4; // Rough estimate
          
          match priority {
              Priority::High => true, // Always allow high priority
              Priority::Normal => {
                  self.embedding_budget.current_usage + estimated_tokens 
                      <= self.embedding_budget.daily_limit - self.embedding_budget.priority_reserve
              },
              Priority::Low => {
                  self.embedding_budget.current_usage + estimated_tokens 
                      <= self.embedding_budget.daily_limit * 8 / 10 // 80% pour low priority
              }
          }
      }
      
      pub fn track_usage(&mut self, tokens_used: usize, operation_type: OperationType) {
          match operation_type {
              OperationType::Embedding => {
                  self.embedding_budget.current_usage += tokens_used;
              },
              OperationType::Search => {
                  self.search_budget.current_usage += 1;
              },
          }
      }
  }

  🔍 Observabilité Production

  /// Monitoring complet pour debugging et optimisation continue
  pub struct ObservabilityEngine {
      metrics_collector: MetricsCollector,
      performance_tracer: PerformanceTracer,
      health_monitor: HealthMonitor,
      alert_manager: AlertManager,
  }

  #[derive(Debug, Clone)]
  pub struct PerformanceMetrics {
      // === Métriques Chunking ===
      pub chunking_latency_p50: f32,    // ms
      pub chunking_latency_p95: f32,    // ms
      pub chunks_per_second: f32,
      pub boundary_penalty_avg: f32,
      pub overlap_efficiency_avg: f32,
      
      // === Métriques Search ===
      pub search_latency_p50: f32,      // ms
      pub search_latency_p95: f32,      // ms
      pub search_recall_at_5: f32,
      pub search_recall_at_10: f32,
      pub mmr_diversity_score: f32,
      
      // === Métriques Resources ===
      pub memory_usage_mb: f32,
      pub cpu_usage_percent: f32,
      pub qdrant_disk_usage_mb: f32,
      pub embedding_cache_hit_rate: f32,
  }

  impl ObservabilityEngine {
      pub fn start_operation_trace(&self, operation: &str) -> OperationSpan {
          OperationSpan {
              operation_id: uuid::Uuid::new_v4().to_string(),
              operation_type: operation.to_string(),
              start_time: SystemTime::now(),
              metadata: HashMap::new(),
          }
      }
      
      pub fn record_chunking_metrics(&mut self, chunk_result: &SmartChunkResult) {
          self.metrics_collector.record_chunking_latency(chunk_result.processing_time_ms);
          self.metrics_collector.record_boundary_penalty(chunk_result.boundary_penalty);
          self.metrics_collector.record_overlap_efficiency(chunk_result.overlap_percentage);
      }
      
      pub fn record_search_metrics(&mut self, search_result: &SearchResult, latency_ms: f32) {
          self.metrics_collector.record_search_latency(latency_ms);
          self.metrics_collector.record_search_quality(search_result.recall_score);
      }
      
      pub fn check_health_alerts(&self) -> Vec<HealthAlert> {
          let mut alerts = Vec::new();
          
          // Alert si boundary penalty dégrade
          if self.metrics_collector.boundary_penalty_avg > 0.4 {
              alerts.push(HealthAlert::BoundaryPenaltyHigh);
          }
          
          // Alert si search recall chute
          if self.metrics_collector.search_recall_avg < 0.75 {
              alerts.push(HealthAlert::SearchRecallLow);
          }
          
          // Alert si latency explose  
          if self.metrics_collector.search_latency_p95 > 1000.0 {
              alerts.push(HealthAlert::SearchLatencyHigh);
          }
          
          alerts
      }
  }

  #[derive(Debug)]
  pub enum HealthAlert {
      BoundaryPenaltyHigh,
      SearchRecallLow,
      SearchLatencyHigh,
      MemoryUsageHigh,
      EmbeddingCacheMissHigh,
  }

  ---
  🥇 Phase 3A - Business Documents (Priorité 1)

  🎯 Pourquoi Business First ?

  Impact économique maximal :
  - Marché énorme : toutes entreprises = rapports annuels
  - ROI immédiat : analyse financière, benchmarking, due diligence
  - Complexité technique optimale (plus simple que Legal, moins multimodal que Technical)

  🔧 Implémentation Business Classifier

  // Module: src/rag/document_classifier.rs
  pub enum DocumentType {
      Academic,
      Business,
      Legal,
      Technical,
      Mixed,
  }

  pub struct BusinessSignals {
      pub executive_summary: bool,
      pub financial_metrics: Vec<String>,
      pub company_identifiers: Vec<String>,
      pub fiscal_year: Option<i32>,
  }

  impl DocumentClassifier {
      pub fn detect_business_confidence(content: &str) -> f32 {
          let signals = [
              ("Executive Summary", 0.3),
              ("Annual Report", 0.4),
              ("Financial Performance", 0.3),
              ("Revenue|EBITDA|Shareholders", 0.4),
              ("Management Discussion", 0.2),
          ];
          // Algorithme de scoring
      }
  }

  📊 Business Chunking Strategy

  // Configuration adaptative Business
  let business_config = SmartChunkConfig {
      target_tokens: 500,           // Sections business plus longues
      overlap_percent: 0.15,        // Contexte financier important
      overlap_target_ratio: Some(0.15),
      mmr_lambda: 0.6,             // Plus de relevance pour business
      max_context_docs: 6,         // Plus de contexte pour analyse
      section_patterns: r"(Executive Summary|Financial Highlights|Business Overview|Risk 
  Factors|Management Discussion|Market Analysis)",
      boundary_penalty_weight: 0.4, // Moins strict que academic
  };

  📈 Tables-First Business Processing

  /// Stratégie spécialisée pour documents business riches en tableaux financiers
  pub struct BusinessTablesProcessor {
      table_detector: TableDetector,
      financial_extractor: FinancialKPIExtractor,
      layout_analyzer: LayoutAnalyzer,
  }

  #[derive(Debug, Clone)]
  pub struct TableDetectionConfig {
      pub min_rows: usize,           // Minimum 3 lignes pour être considéré table
      pub min_cols: usize,           // Minimum 2 colonnes
      pub financial_keywords: Vec<String>, // ["Revenue", "EBITDA", "Assets", "Liabilities"]
      pub confidence_threshold: f32, // 0.8 minimum pour extraction
  }

  impl BusinessTablesProcessor {
      pub fn process_business_document(&self, content: &str) -> Result<BusinessProcessResult> {
          // 1. Détection prioritaire des tableaux
          let tables = self.table_detector.extract_tables(content)?;
          
          // 2. Classification des tableaux par type business
          let financial_tables = tables.into_iter()
              .filter_map(|table| self.classify_financial_table(table))
              .collect::<Vec<_>>();
              
          // 3. Extraction structurée des KPIs
          let extracted_kpis = financial_tables.iter()
              .flat_map(|table| self.financial_extractor.extract_kpis(table))
              .collect::<Vec<_>>();
              
          // 4. Chunking adaptatif : Tables = chunks prioritaires
          let table_chunks = self.create_table_chunks(&financial_tables)?;
          let text_chunks = self.create_contextual_text_chunks(content, &financial_tables)?;
          
          Ok(BusinessProcessResult {
              table_chunks,
              text_chunks,
              extracted_kpis,
              financial_summary: self.generate_financial_summary(&extracted_kpis),
          })
      }
      
      fn classify_financial_table(&self, table: ExtractedTable) -> Option<FinancialTable> {
          let financial_score = table.headers.iter()
              .map(|header| self.calculate_financial_score(header))
              .sum::<f32>() / table.headers.len() as f32;
              
          if financial_score > 0.6 {
              Some(FinancialTable {
                  table_type: self.detect_table_type(&table),
                  confidence: financial_score,
                  period_detected: self.extract_period(&table),
                  currency_detected: self.extract_currency(&table),
                  raw_table: table,
              })
          } else {
              None
          }
      }
      
      fn create_table_chunks(&self, tables: &[FinancialTable]) -> Result<Vec<EnrichedChunk>> {
          tables.iter().map(|table| {
              let chunk_content = format!(
                  "{}\n\n{}\n\nKPIs: {}",
                  table.generate_description(),
                  table.serialize_structured_data(),
                  table.extract_key_metrics().join(", ")
              );
              
              EnrichedChunk {
                  id: format!("table_{}", uuid::Uuid::new_v4().simple()),
                  content: chunk_content,
                  chunk_type: ChunkType::FinancialTable, // Nouveau type
                  importance_score: 0.9, // Tables = haute priorité
                  metadata: ChunkMetadata {
                      table_metadata: Some(table.metadata.clone()),
                      source_type: SourceType::StructuredData,
                      extraction_method: ExtractionMethod::TableExtraction,
                      ..Default::default()
                  },
                  ..Default::default()
              }
          }).collect()
      }
  }

  #[derive(Debug, Clone)]
  pub struct FinancialTable {
      pub table_type: FinancialTableType,
      pub confidence: f32,
      pub period_detected: Option<String>,   // "2023", "Q3 2023"
      pub currency_detected: Option<String>, // "USD", "EUR"
      pub raw_table: ExtractedTable,
  }

  #[derive(Debug, Clone)]
  pub enum FinancialTableType {
      IncomeStatement,    // P&L
      BalanceSheet,       // Bilan
      CashFlow,          // Cash Flow Statement
      KeyMetrics,        // KPIs Summary
      Comparative,       // Multi-period comparison
  }

  🏢 Business Metadata Enrichment

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct BusinessMetadata {
      pub fiscal_year: Option<i32>,
      pub company_name: Option<String>,
      pub sector: Option<String>,
      pub financial_kpis: Vec<FinancialKPI>,
      pub section_type: BusinessSection,
      pub page_number: Option<u32>,
      pub bbox: Option<BoundingBox>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub enum BusinessSection {
      ExecutiveSummary,
      FinancialHighlights,
      BusinessOverview,
      RiskFactors,
      MarketAnalysis,
      Governance,
      Sustainability,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct FinancialKPI {
      pub name: String,        // "Revenue", "EBITDA", "Net Income"
      pub value: f64,
      pub currency: String,
      pub period: String,      // "2023", "Q3 2023"
      pub growth_rate: Option<f32>,
  }

  📈 Dataset Business d'Entraînement

  50 rapports annuels stratifiés :
  - 20 Fortune 500 (2022-2023) : Apple, Microsoft, Amazon, Tesla...
  - 10 CAC40 français : LVMH, L'Oréal, Sanofi, Total...
  - 10 Tech Giants : Google, Meta, Netflix, Spotify...
  - 10 Secteurs variés : Banking, Healthcare, Manufacturing, Energy

  Sources :
  - SEC EDGAR filings (10-K forms)
  - Sites corporate officiels
  - Annual reports publics

  🧪 Business Health Checks

  let business_health_checks = vec![
      (financial_entities_detected >= 5, "financial_kpis_extraction"),
      (business_sections_coverage >= 0.8, "business_sections_detected"),
      (chunk_financial_coherence >= 0.7, "business_context_preservation"),
      (search_recall_financial_queries >= 0.85, "business_search_quality"),
      (company_name_detection_rate >= 0.9, "company_identification"),
  ];

  ---
  🥈 Phase 3B - Legal Documents (Priorité 2)

  ⚖️ Spécificités Legal

  Défis techniques :
  - Clauses numérotées complexes (Art. 1.2.3.a)
  - Références croisées fréquentes
  - Jargon juridique spécialisé
  - Structure hiérarchique stricte

  🔧 Legal Chunking Adaptatif

  let legal_config = SmartChunkConfig {
      target_tokens: 600,           // Clauses plus longues
      overlap_percent: 0.30,        // Overlap élevé pour références
      section_patterns:
  r"(Article|Clause|Section|Whereas|Therefore|Party|Obligation|Liability|Termination)",
      boundary_penalty_weight: 0.2, // Très strict sur boundaries
  };

  📋 Legal Metadata

  pub struct LegalMetadata {
      pub contract_type: Option<ContractType>,
      pub parties: Vec<String>,
      pub jurisdiction: Option<String>,
      pub effective_date: Option<chrono::DateTime<Utc>>,
      pub clause_hierarchy: Vec<String>, // ["Art. 1", "1.2", "1.2.a"]
  }

  ---
  🥉 Phase 3C - Technical Documents (Priorité 3)

  🔧 Spécificités Technical

  Défis multimodaux :
  - Figures et schémas critiques
  - Tableaux de spécifications
  - Code snippets intégrés
  - Références vers annexes

  🖼️ Layout-Aware Processing

  pub struct TechnicalMetadata {
      pub figures: Vec<FigureReference>,
      pub tables: Vec<TableExtract>,
      pub code_blocks: Vec<CodeSnippet>,
      pub specifications: Vec<TechnicalSpec>,
  }

  pub struct FigureReference {
      pub figure_id: String,
      pub caption: String,
      pub bbox: BoundingBox,
      pub page: u32,
  }

  ---
  📋 Phase 3D - Scanned/Mixed (Priorité 4)

  📄 OCR Enhancement

  Améliorations nécessaires :
  - Preprocessing avancé (deskew, denoise)
  - Confidence scoring par caractère
  - Correction automatique post-OCR
  - Gestion documents multi-langues

  ---
  🏗️ Architecture Technique Universelle

  🔄 Pipeline Unifié Hybride

  // Pipeline universel avec classification automatique + hybrid search
  pub async fn process_universal_document(
      pdf_path: &Path,
      group_id: &str,
  ) -> Result<UniversalRAGResult> {

      // 1. Extraction initiale
      let raw_content = extract_pdf_content(pdf_path).await?;

      // 2. Classification automatique
      let doc_type = DocumentClassifier::classify(&raw_content)?;

      // 3. Configuration adaptative
      let config = AdaptiveConfig::for_document_type(doc_type);

      // 4. Chunking spécialisé avec IDs stables
      let chunks = AdaptiveChunker::chunk_with_config(&raw_content, &config)?;

      // 5. Metadata enrichment + CommonMeta
      let enriched_chunks = MetadataEnricher::enrich_by_type(chunks, doc_type)?;

      // 6. Indexation hybride (Vector + BM25)
      let result = HybridRAGPipeline::index_and_search(enriched_chunks).await?;

      Ok(result)
  }

  🔍 Hybrid Search Architecture

  /// Combinaison optimale Vector + BM25 pour qualité maximale
  pub struct HybridSearchEngine {
      pub vector_engine: CustomE5Embedder,
      pub bm25_engine: BM25Index,
      pub fusion_strategy: FusionStrategy,
      pub query_router: QueryRouter,
  }

  pub enum FusionStrategy {
      RRF,           // Reciprocal Rank Fusion (recommandé)
      LinearCombine, // Weighted combination
      Adaptive,      // Dynamic based on query type
  }

  impl HybridSearchEngine {
      pub async fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>> {
          // 1. Route query pour stratégie optimale
          let query_intent = self.query_router.analyze_intent(query)?;
          
          // 2. Search parallèle Vector + BM25
          let (vector_results, bm25_results) = tokio::join!(
              self.vector_engine.search(query, top_k * 2),
              self.bm25_engine.search(query, top_k * 2)
          );
          
          // 3. Fusion intelligente basée sur l'intent
          let fused_results = match query_intent {
              QueryIntent::Factual => self.fusion_strategy.combine_factual(vector_results, bm25_results),
              QueryIntent::Conceptual => self.fusion_strategy.combine_conceptual(vector_results, bm25_results),
              QueryIntent::Mixed => self.fusion_strategy.combine_balanced(vector_results, bm25_results),
          };
          
          // 4. MMR re-ranking final
          let reranked = MMRReranker::new(0.5).rerank(fused_results, top_k)?;
          
          Ok(reranked)
      }
  }

  📊 Configuration Adaptative

  pub struct AdaptiveConfig {
      pub chunking: SmartChunkConfig,
      pub metadata: MetadataConfig,
      pub search: SearchConfig,
      pub validation: HealthCheckConfig,
  }

  impl AdaptiveConfig {
      pub fn for_document_type(doc_type: DocumentType) -> Self {
          match doc_type {
              DocumentType::Academic => Self::academic_optimized(),
              DocumentType::Business => Self::business_optimized(),
              DocumentType::Legal => Self::legal_optimized(),
              DocumentType::Technical => Self::technical_optimized(),
              DocumentType::Mixed => Self::balanced_universal(),
          }
      }
  }

  🔗 CommonMeta + Stable Chunk IDs

  /// Métadonnées universelles pour interopérabilité et cache stable
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct CommonMeta {
      // === IDs Stables (invariants aux re-indexations) ===
      pub chunk_id: String,          // ID stable basé sur contenu + position
      pub document_id: String,       // ID stable du document source
      pub source_path: String,       // Chemin original du document
      
      // === Localisation Stable ===
      pub semantic_location: SemanticLocation,
      pub physical_location: PhysicalLocation,
      
      // === Versioning & Cache ===
      pub content_hash: String,      // Hash blake3 pour invalidation cache
      pub index_version: String,     // Version du pipeline d'indexation
      pub last_updated: SystemTime,
      
      // === Classification Universelle ===
      pub document_type: DocumentType,
      pub section_type: Option<SectionType>,
      pub importance_score: f32,     // 0.0-1.0 pour ranking
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct SemanticLocation {
      pub section_hierarchy: Vec<String>, // ["Executive Summary", "Financial Highlights"]
      pub section_confidence: f32,        // Confiance détection section
      pub relative_position: f32,         // Position 0.0-1.0 dans le document
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct PhysicalLocation {
      pub page_number: Option<u32>,
      pub bbox: Option<BoundingBox>,      // Coordonnées PDF si disponible
      pub line_range: (usize, usize),    // Lignes source dans le texte
      pub char_range: (usize, usize),    // Position caractères absolue
  }

  /// Générateur d'IDs stables pour chunk persistence
  pub struct StableIdGenerator;

  impl StableIdGenerator {
      /// Génère un ID stable pour un chunk (invariant aux re-indexations)
      pub fn generate_chunk_id(
          document_path: &str,
          content: &str,
          position: usize,
          section_context: &str
      ) -> String {
          let context = format!("{}:{}:{}", document_path, position, section_context);
          let content_sample = if content.len() > 200 {
              &content[..200]
          } else {
              content
          };
          let combined = format!("{}:{}", context, content_sample);
          let hash = blake3::hash(combined.as_bytes());
          format!("chunk_{}", hash.to_hex()[..16].to_string())
      }
      
      /// Génère un ID stable pour un document
      pub fn generate_document_id(file_path: &str, creation_time: SystemTime) -> String {
          let timestamp = creation_time
              .duration_since(SystemTime::UNIX_EPOCH)
              .unwrap_or_default()
              .as_secs();
          let combined = format!("{}:{}", file_path, timestamp);
          let hash = blake3::hash(combined.as_bytes());
          format!("doc_{}", hash.to_hex()[..12].to_string())
      }
  }

  🧠 Query Router & Intent Detection

  /// Routeur intelligent pour optimiser la stratégie de recherche par type de query
  pub struct QueryRouter {
      intent_classifier: IntentClassifier,
      document_type_weights: HashMap<DocumentType, f32>,
      fusion_strategy_map: HashMap<QueryIntent, FusionStrategy>,
  }

  #[derive(Debug, Clone, Copy)]
  pub enum QueryIntent {
      Factual,      // "What is the revenue for 2023?" → BM25 dominant
      Conceptual,   // "Explain the business strategy" → Vector dominant  
      Mixed,        // "Compare financial performance" → Balanced fusion
      Navigation,   // "Show me all financial sections" → Metadata search
  }

  pub struct IntentClassifier;

  impl IntentClassifier {
      pub fn analyze_intent(&self, query: &str) -> Result<QueryIntent> {
          let query_lower = query.to_lowercase();
          
          // Patterns pour classification d'intent rapide
          let factual_patterns = ["what is", "how much", "when did", "who is", "where is"];
          let conceptual_patterns = ["explain", "describe", "analyze", "why", "how does"];
          let navigation_patterns = ["show me", "list all", "find sections", "navigate to"];
          
          let factual_score = factual_patterns.iter()
              .map(|&p| if query_lower.contains(p) { 1.0 } else { 0.0 })
              .sum::<f32>();
              
          let conceptual_score = conceptual_patterns.iter()
              .map(|&p| if query_lower.contains(p) { 1.0 } else { 0.0 })
              .sum::<f32>();
              
          let navigation_score = navigation_patterns.iter()
              .map(|&p| if query_lower.contains(p) { 1.0 } else { 0.0 })
              .sum::<f32>();
          
          if navigation_score > 0.0 {
              Ok(QueryIntent::Navigation)
          } else if factual_score > conceptual_score {
              Ok(QueryIntent::Factual)
          } else if conceptual_score > factual_score {
              Ok(QueryIntent::Conceptual)
          } else {
              Ok(QueryIntent::Mixed)
          }
      }
  }

  impl QueryRouter {
      pub fn new() -> Self {
          let mut fusion_map = HashMap::new();
          fusion_map.insert(QueryIntent::Factual, FusionStrategy::RRF); // BM25 + Vector équilibré
          fusion_map.insert(QueryIntent::Conceptual, FusionStrategy::LinearCombine); // Vector dominant
          fusion_map.insert(QueryIntent::Mixed, FusionStrategy::Adaptive);
          fusion_map.insert(QueryIntent::Navigation, FusionStrategy::RRF);
          
          Self {
              intent_classifier: IntentClassifier,
              document_type_weights: HashMap::new(),
              fusion_strategy_map: fusion_map,
          }
      }
      
      pub fn route_query(&self, query: &str, context: &SearchContext) -> SearchStrategy {
          let intent = self.intent_classifier.analyze_intent(query).unwrap_or(QueryIntent::Mixed);
          let fusion_strategy = self.fusion_strategy_map.get(&intent)
              .unwrap_or(&FusionStrategy::RRF)
              .clone();
              
          SearchStrategy {
              intent,
              fusion_strategy,
              vector_weight: self.calculate_vector_weight(intent, context),
              bm25_weight: self.calculate_bm25_weight(intent, context),
              rerank_aggressive: matches!(intent, QueryIntent::Conceptual),
          }
      }
      
      fn calculate_vector_weight(&self, intent: QueryIntent, context: &SearchContext) -> f32 {
          match intent {
              QueryIntent::Factual => 0.3,    // BM25 dominant pour facts
              QueryIntent::Conceptual => 0.8, // Vector dominant pour concepts
              QueryIntent::Mixed => 0.5,      // Équilibré
              QueryIntent::Navigation => 0.2, // Metadata dominant
          }
      }
      
      fn calculate_bm25_weight(&self, intent: QueryIntent, context: &SearchContext) -> f32 {
          1.0 - self.calculate_vector_weight(intent, context)
      }
  }

  #[derive(Debug)]
  pub struct SearchStrategy {
      pub intent: QueryIntent,
      pub fusion_strategy: FusionStrategy,
      pub vector_weight: f32,
      pub bm25_weight: f32,
      pub rerank_aggressive: bool,
  }

  #[derive(Debug)]
  pub struct SearchContext {
      pub document_types: Vec<DocumentType>,
      pub user_preferences: Option<UserPreferences>,
      pub query_history: Vec<String>,
  }

  ---
  🧪 Validation & Benchmarking

  📈 Métriques Universelles

  pub struct UniversalMetrics {
      // Métriques par type de document
      pub academic_performance: DocumentTypeMetrics,
      pub business_performance: DocumentTypeMetrics,
      pub legal_performance: DocumentTypeMetrics,
      pub technical_performance: DocumentTypeMetrics,

      // Métriques globales
      pub overall_recall: f32,
      pub processing_speed: f32,      // chunks/second
      pub classification_accuracy: f32,
      pub cross_type_coherence: f32,
      
      // === Nouvelles métriques Phase 3 ===
      pub hybrid_search_performance: HybridSearchMetrics,
      pub query_routing_accuracy: f32,
      pub budget_efficiency: BudgetMetrics,
      pub table_extraction_accuracy: f32,
      pub stable_id_consistency: f32,
  }

  pub struct DocumentTypeMetrics {
      pub boundary_penalty: f32,
      pub overlap_efficiency: f32,
      pub search_recall_at_10: f32,
      pub mmr_diversity_score: f32,
      pub metadata_extraction_rate: f32,
      
      // === Nouvelles métriques spécialisées ===
      pub table_detection_rate: f32,      // Pour Business documents
      pub section_classification_acc: f32, // Pour tous types
      pub chunk_id_stability: f32,        // Stabilité des IDs
  }

  /// Métriques spécifiques à la recherche hybride
  #[derive(Debug, Clone)]
  pub struct HybridSearchMetrics {
      pub vector_search_recall: f32,      // Performance pure vector
      pub bm25_search_recall: f32,        // Performance pure BM25  
      pub fusion_improvement: f32,        // Gain hybride vs meilleur individuel
      pub query_intent_accuracy: f32,     // Précision classification intent
      pub rrf_effectiveness: f32,         // Efficacité Reciprocal Rank Fusion
  }

  /// Métriques de budget et coûts
  #[derive(Debug, Clone)]
  pub struct BudgetMetrics {
      pub cost_per_processed_token: f32,  // Coût moyen par token
      pub embedding_cache_hit_rate: f32,  // Taux de cache hit embeddings
      pub budget_compliance_rate: f32,    // % requêtes respectant budget
      pub resource_utilization: f32,      // Utilisation CPU/RAM moyenne
  }

  🎯 KPIs de Validation

  ### Métriques Core (Phase 2 validées)
  | Métrique           | Academic | Business | Legal    | Technical | Target Global |
  |--------------------|----------|----------|----------|-----------|---------------|
  | Boundary Penalty   | ≤0.15    | ≤0.25    | ≤0.10    | ≤0.20     | ≤0.18         |
  | Search Recall@10   | ≥0.85    | ≥0.80    | ≥0.75    | ≥0.78     | ≥0.80         |
  | Processing Speed   | ≥80 ch/s | ≥70 ch/s | ≥60 ch/s | ≥65 ch/s  | ≥70 ch/s      |
  | Classification Acc | N/A      | ≥0.90    | ≥0.85    | ≥0.88     | ≥0.88         |

  ### Nouvelles Métriques Phase 3 (Production++)
  | Métrique                 | Target | Description                           |
  |--------------------------|--------|---------------------------------------|
  | Hybrid Search Improvement| ≥15%   | Gain fusion vs meilleur individuel    |
  | Query Intent Accuracy    | ≥0.90  | Précision classification intent       |
  | Table Detection Rate     | ≥0.85  | Pour docs Business avec tableaux      |
  | Chunk ID Stability       | ≥0.95  | Invariance re-indexation             |
  | Budget Compliance        | ≥0.98  | % requêtes respectant limites budget  |
  | Cache Hit Rate           | ≥0.80  | Embeddings cache efficiency          |
  | RRF Effectiveness        | ≥1.10  | Ratio gain RRF vs linear combination |

  ---
  🔄 Migration Strategy

  📋 Phase de Transition

  1. Rétro-compatibilité : Pipeline académique actuel préservé
  2. Feature flags : Activation graduelle des nouvelles fonctionnalités
  3. A/B testing : Comparaison performance ancien vs nouveau
  4. Rollback plan : Retour rapide en cas de régression

  🔧 Code Integration

  // Feature flags pour migration douce
  #[cfg(feature = "universal-rag")]
  pub use universal_pipeline::*;

  #[cfg(not(feature = "universal-rag"))]
  pub use academic_pipeline::*; // Fallback vers Phase 2

  ---
  💰 Business Value & ROI

  🎯 Valeur Marchande

  Positionnement concurrentiel :
  - LlamaIndex : Bon mais Python-only, pas d'adaptation par type
  - Haystack : Complexe, manque de spécialisation document
  - Azure Cognitive Search : Cloud-only, coûteux à scale
  - GRAVIS Universal RAG : Rust-native, adaptatif, on-premise

  📊 Avantages Différenciants

  1. Performance Rust : 3-5x plus rapide que Python
  2. Adaptive Chunking : Qualité supérieure par type de document
  3. Production-Ready : Garde-fous CI, monitoring intégré
  4. Cost-Effective : Déploiement on-premise, pas de vendor lock-in
  5. Extensible : Architecture modulaire pour nouveaux types

  💵 Estimation ROI

  Coût développement Phase 3 : ~6 semaines dev
  Valeur marchande estimée : Document processing SaaS = $50k-200k ARR
  ROI première année : 300-800%

  🧾 Commercial - Devis & Factures (Conformité FR)

  ### Modèles Templates

  **🔢 Numérotation Standard**
  - Devis : `DEV-{{YYYY}}-{{NNN}}` (ex. DEV-2025-003)
  - Factures : `FAC-{{YYYY}}-{{NNN}}`
  - Avoirs : `AV-{{YYYY}}-{{NNN}}`

  **📋 Lignes Produits GRAVIS**
  ```json
  {
    "products": [
      { "code": "GRAVIS-IMPL", "designation": "Implémentation on-prem GRAVIS Universal RAG", "type": "forfait" },
      { "code": "GRAVIS-LIC-ANNUAL", "designation": "Licence annuelle Universal RAG", "unit": "par site" },
      { "code": "GRAVIS-MCO-OR", "designation": "Support & MCO (SLA Or)", "unit": "mensuel" },
      { "code": "GRAVIS-EMBED", "designation": "Run embeddings supplémentaires", "unit": "par 1M tokens" },
      { "code": "GRAVIS-DEV", "designation": "Développement spécifique", "unit": "TJM" }
    ]
  }
  ```

  **💼 Structure de Données (JSON)**
  ```json
  {
    "document_type": "quote|invoice|credit_note",
    "number": "FAC-2025-001",
    "date": "2025-10-27",
    "due_date": "2025-11-26",
    "currency": "EUR",
    "client": {
      "name": "Acme SA",
      "contact": "Jane Doe",
      "address": "10 rue Exemple, 75000 Paris",
      "siren": "123456789",
      "vat_id": "FRXX123456789"
    },
    "supplier": {
      "name": "GRAVIS AI",
      "address": "42 avenue Rust, 69000 Lyon", 
      "siren": "987654321",
      "rcs": "Lyon B 987 654 321",
      "vat_id": "FRYY987654321",
      "iban": "FR76....",
      "bic": "AGRIFRPPXXX"
    },
    "lines": [
      { "designation": "Licence GRAVIS Universal RAG - 12 mois", "qty": 1, "unit_ht": 25000.0 },
      { "designation": "Support & MCO (SLA Or)", "qty": 12, "unit_ht": 800.0 }
    ],
    "vat_rate": 20.0,
    "totals": { "subtotal_ht": 34600.0, "vat": 6920.0, "total_ttc": 41520.0 },
    "terms": {
      "payment": "Virement 30 jours fin de mois",
      "late_fees": "Taux légal x3 + indemnité forfaitaire 40€",
      "notes": "Merci de rappeler FAC-2025-001 en référence."
    },
    "refs": { "quote_number": "DEV-2025-005", "original_invoice": null }
  }
  ```

  **✅ Checklist Conformité FR**
  - ✅ Numéro unique continu
  - ✅ Identité & SIREN/TVA vendeur + client  
  - ✅ Date facture + échéance
  - ✅ Détail HT, TVA (20% standard), TTC
  - ✅ Conditions de paiement, pénalités, indemnité 40€
  - ✅ RCS / Ville d'immatriculation
  - ✅ Avoirs référencent la facture d'origine

  **🧰 Génération & Intégration**
  - Render : Markdown → PDF (WeasyPrint/wkhtmltopdf)
  - Signature : QR code avec URL de vérification
  - Paiement : Stripe/Checkout + virement IBAN
  - Stockage : Qdrant pour recherche documents commerciaux

  ---
  🚀 Next Steps

  📋 Actions Immédiates

  1. ✅ Validation roadmap avec équipe
  2. 📊 Setup dataset Business (Fortune 500 + CAC40)
  3. 🔧 Architecture DocumentClassifier module
  4. 🧪 Proof of concept Business chunking
  5. 📈 Benchmark baseline performance actuelle

  🎯 Première Milestone (Semaine 1)

  - DocumentClassifier module complet
  - Dataset Business (20 rapports) préparé
  - Tests unitaires classification
  - Benchmark performance vs Phase 2
  - Documentation technique mise à jour

  ---
  📝 Conclusion

  La Phase 3 Universal RAG représente une évolution naturelle du pipeline production actuel vers un
  système industriel polyvalent. L'approche Business-first garantit un ROI rapide tout en construisant
   les fondations pour l'universalité complète.

  Le code Phase 2 reste la base solide - on ajoute uniquement les couches d'adaptation nécessaires
  pour l'universalité, sans régresser sur les performances académiques validées.

  ---
  Document préparé par : Claude (GRAVIS AI)Review requise : Équipe technique + Product OwnerProchaine 
  révision : Fin Semaine 1 Phase 3A