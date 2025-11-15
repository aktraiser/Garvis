// Phase 2: Chat Direct - Mode Générique (MVP)
// Structures pour session temporaire de chat avec documents droppés

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use uuid::Uuid;

use crate::rag::{
    SourceSpan, EnrichedChunk, DocumentType, ExtractionMethod, SourceType,
};

/// Session temporaire pour chat direct avec document dragué
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectChatSession {
    pub session_id: String,
    pub document_path: PathBuf,
    pub document_name: String,
    pub document_type: DocumentType,
    pub chunks: Vec<EnrichedChunk>,
    
    // DÉCOUPLAGE: Affichage vs Embedding
    pub display_content: DisplayContent,   // Pour l'affichage (PDF natif, texte original)
    pub search_content: OCRContent,       // Pour l'embedding/recherche (OCR avec spans)
    
    pub structured_data: Option<StructuredData>,
    pub embeddings: Vec<f32>,
    pub created_at: SystemTime,
    pub is_temporary: bool,
}

/// Réponse de chat direct avec spans contributeurs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectChatResponse {
    pub response: String,
    pub contributing_spans: Vec<SourceSpan>,
    pub confidence_score: f64,
    pub session_id: String,
}

/// Contexte de sélection utilisateur pour questions ciblées
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionContext {
    pub page: Option<u32>,
    pub text: Option<String>,
    pub bounding_rect: Option<BoundingBox>, // Zone rectangulaire sélectionnée
}

/// Région sélectionnée par l'utilisateur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedRegion {
    pub page: u32,
    pub text: String,
    pub rect: BoundingBox,
}

/// Contenu d'affichage (PDF natif ou texte extrait proprement)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayContent {
    pub content_type: DisplayContentType,
    pub native_text: Option<String>,        // Texte extrait nativement du PDF
    pub pdf_url: Option<String>,           // URL ou path vers le PDF original
    pub page_count: usize,
    pub extraction_quality: f64,          // Qualité de l'extraction native (0.0-1.0)
}

/// Type de contenu pour l'affichage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisplayContentType {
    PdfNative,      // PDF avec texte extractible -> afficher PDF original
    PdfScanned,     // PDF scanné -> afficher avec overlay OCR
    TextDocument,   // Document texte simple
    Image,          // Image pure
}

/// Contenu OCR structuré avec layout analysis (pour embedding/recherche uniquement)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OCRContent {
    pub pages: Vec<OCRPage>,
    pub total_confidence: f64,
    pub layout_analysis: LayoutAnalysis,
}

/// Page OCR avec blocs structurés
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OCRPage {
    pub page_number: u32,
    pub blocks: Vec<OCRBlock>, // Texte, Table, List, etc.
    pub width: f64,
    pub height: f64,
}

/// Bloc OCR avec type et positionnement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OCRBlock {
    pub page_number: u32, // 🆕 Numéro de page pour mapping précis
    pub block_type: BlockType, // Text, Table, List, Header, etc.
    pub content: String,
    pub bounding_box: BoundingBox,
    pub confidence: f64,
    pub spans: Vec<String>, // Références aux SourceSpan IDs pour ce block
}

/// Type de bloc détecté par layout analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockType {
    Text,
    Header,
    Table,
    List,
    KeyValue, // Pour "Salaire brut: 2500€"
    Amount,   // Montants monétaires
    Date,
    Figure,   // Pour graphiques, diagrammes, images avec légendes
}

/// BoundingBox pour positionnement précis (coordonnées normalisées 0.0-1.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    /// Position X normalisée (0.0-1.0) relative à la largeur de page
    pub x: f64,
    /// Position Y normalisée (0.0-1.0) relative à la hauteur de page  
    pub y: f64,
    /// Largeur normalisée (0.0-1.0) relative à la largeur de page
    pub width: f64,
    /// Hauteur normalisée (0.0-1.0) relative à la hauteur de page
    pub height: f64,
}

/// Analyse de layout pour détecter structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutAnalysis {
    pub detected_columns: usize,
    pub has_tables: bool,
    pub has_headers: bool,
    pub text_density: f64,
    pub dominant_font_size: Option<f64>,
}

/// Données structurées pour documents typés
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StructuredData {
    Payslip(PayslipData),
    Invoice(InvoiceData),
    BankStatement(BankStatementData),
    Contract(ContractData),
    Generic(serde_json::Value), // Pour docs non typés avec structure libre
}

/// Données de fiche de paie avec spans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayslipData {
    pub employee_name: String,
    pub period: String, // "2025-10"
    pub gross_salary: f64,
    pub net_salary: f64,
    pub deductions: Vec<DeductionLine>,
    pub employer_info: EmployerInfo,
    pub spans: PayslipSpans, // Liens vers les SourceSpan pour chaque champ
}

/// Données de facture avec spans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceData {
    pub invoice_number: String,
    pub date: chrono::NaiveDate,
    pub supplier: CompanyInfo,
    pub client: CompanyInfo,
    pub items: Vec<InvoiceItem>,
    pub total_ht: f64,
    pub total_ttc: f64,
    pub spans: InvoiceSpans,
}

/// Données de relevé bancaire avec spans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankStatementData {
    pub account_number: String,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub opening_balance: f64,
    pub closing_balance: f64,
    pub transactions: Vec<Transaction>,
    pub spans: BankStatementSpans,
}

/// Données de contrat placeholder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractData {
    pub contract_type: String,
    pub parties: Vec<String>,
    pub effective_date: Option<chrono::NaiveDate>,
    pub spans: HashMap<String, String>,
}

// Structures de liens spans pour traçabilité
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayslipSpans {
    pub employee_name_span: Option<String>, // SourceSpan.id
    pub gross_salary_span: Option<String>,
    pub net_salary_span: Option<String>,
    pub deduction_spans: HashMap<String, String>, // deduction_id -> span_id
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceSpans {
    pub invoice_number_span: Option<String>,
    pub total_ht_span: Option<String>,
    pub total_ttc_span: Option<String>,
    pub item_spans: HashMap<String, ItemSpans>, // item_id -> spans
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankStatementSpans {
    pub account_number_span: Option<String>,
    pub balance_spans: HashMap<String, String>, // "opening"/"closing" -> span_id
    pub transaction_spans: HashMap<String, String>, // transaction_id -> span_id
}

// Supporting structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeductionLine {
    pub id: String,
    pub label: String,
    pub amount: f64,
    pub type_deduction: DeductionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeductionType {
    Social,
    Tax,
    Insurance,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployerInfo {
    pub name: String,
    pub address: Option<String>,
    pub siret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyInfo {
    pub name: String,
    pub address: String,
    pub vat_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceItem {
    pub id: String,
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemSpans {
    pub description_span: Option<String>,
    pub quantity_span: Option<String>,
    pub price_span: Option<String>,
    pub total_span: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub date: chrono::NaiveDate,
    pub description: String,
    pub amount: f64,
    pub balance_after: f64,
}

impl DirectChatSession {
    /// Créer nouvelle session temporaire avec découplage affichage/embedding
    pub fn new(
        document_path: PathBuf,
        document_type: DocumentType,
        chunks: Vec<EnrichedChunk>,
        display_content: DisplayContent,
        search_content: OCRContent,
    ) -> Self {
        let document_name = document_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();

        Self {
            session_id: Uuid::new_v4().to_string(),
            document_path,
            document_name,
            document_type,
            chunks,
            display_content,
            search_content,
            structured_data: None,
            embeddings: vec![],
            created_at: SystemTime::now(),
            is_temporary: true,
        }
    }

    /// Créer session avec contenu d'affichage PDF natif (nouveau constructeur recommandé)
    pub fn new_with_native_pdf(
        document_path: PathBuf,
        document_type: DocumentType,
        chunks: Vec<EnrichedChunk>,
        native_text: String,
        extraction_quality: f64,
        ocr_content: OCRContent,
    ) -> Self {
        let display_content = DisplayContent {
            content_type: DisplayContentType::PdfNative,
            native_text: Some(native_text),
            pdf_url: Some(document_path.to_string_lossy().to_string()),
            page_count: 1, // TODO: detecter nombre de pages
            extraction_quality,
        };

        Self::new(document_path, document_type, chunks, display_content, ocr_content)
    }

    /// Constructeur temporaire pour compatibilité (ancien format)
    pub fn new_legacy(
        document_path: PathBuf,
        document_type: DocumentType,
        chunks: Vec<EnrichedChunk>,
        ocr_content: OCRContent,
    ) -> Self {
        // Créer un DisplayContent de fallback
        let display_content = DisplayContent {
            content_type: DisplayContentType::PdfScanned, // Par défaut OCR
            native_text: None,
            pdf_url: Some(format!("file://{}", document_path.to_string_lossy())),
            page_count: ocr_content.pages.len(),
            extraction_quality: ocr_content.total_confidence,
        };

        Self::new(document_path, document_type, chunks, display_content, ocr_content)
    }

    /// Ajouter données structurées après extraction
    pub fn with_structured_data(mut self, data: StructuredData) -> Self {
        self.structured_data = Some(data);
        self
    }

    /// Vérifier si session a expiré (TTL par défaut: 1 heure)
    pub fn is_expired(&self, ttl_seconds: u64) -> bool {
        if let Ok(elapsed) = self.created_at.elapsed() {
            elapsed.as_secs() > ttl_seconds
        } else {
            true // Si erreur système, considérer comme expiré
        }
    }

    /// Obtenir nombre de chunks avec embeddings
    pub fn embedded_chunks_count(&self) -> usize {
        self.chunks
            .iter()
            .filter(|chunk| chunk.embedding.is_some())
            .count()
    }
}

impl OCRContent {
    /// Créer contenu OCR vide pour tests/fallback
    pub fn empty() -> Self {
        Self {
            pages: vec![],
            total_confidence: 0.0,
            layout_analysis: LayoutAnalysis::default(),
        }
    }

    /// Obtenir tout le texte concaténé
    pub fn get_full_text(&self) -> String {
        self.pages
            .iter()
            .flat_map(|page| &page.blocks)
            .map(|block| block.content.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for LayoutAnalysis {
    fn default() -> Self {
        Self {
            detected_columns: 1,
            has_tables: false,
            has_headers: false,
            text_density: 0.5,
            dominant_font_size: None,
        }
    }
}

/// Erreurs spécifiques au chat direct
#[derive(Debug, thiserror::Error)]
pub enum DirectChatError {
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    
    #[error("Session expired: {0}")]
    SessionExpired(String),
    
    #[error("Document processing failed: {0}")]
    ProcessingFailed(String),
    
    #[error("OCR extraction failed: {0}")]
    OcrFailed(String),
    
    #[error("Invalid selection context: {0}")]
    InvalidSelection(String),
    
    #[error("Embedding generation failed: {0}")]
    EmbeddingFailed(String),
}

pub type DirectChatResult<T> = Result<T, DirectChatError>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag::{ChunkType, ChunkMetadata, Priority, EnrichedMetadata};
    use std::collections::HashMap;

    #[test]
    fn test_direct_chat_session_creation() {
        let path = PathBuf::from("/test/document.pdf");
        let doc_type = DocumentType::PDF {
            extraction_strategy: crate::rag::PdfStrategy::OcrOnly,
            native_text_ratio: 0.0,
            ocr_pages: vec![1],
            total_pages: 1,
        };
        
        let chunks = vec![EnrichedChunk {
            id: "test_chunk".to_string(),
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
                confidence: 0.9,
                ocr_metadata: None,
                source_type: SourceType::OcrExtracted,
                extraction_method: ExtractionMethod::TesseractOcr {
                    confidence: 0.9,
                    language: "fra".to_string(),
                },
            },
            group_id: "temp_group".to_string(),
            source_spans: None,
        }];
        
        let ocr_content = OCRContent::empty();
        
        let session = DirectChatSession::new(path, doc_type, chunks, ocr_content);
        
        assert!(!session.session_id.is_empty());
        assert_eq!(session.document_name, "document.pdf");
        assert!(session.is_temporary);
        assert_eq!(session.chunks.len(), 1);
        assert!(!session.is_expired(3600)); // 1 heure
    }

    #[test]
    fn test_session_expiration() {
        let mut session = DirectChatSession::new(
            PathBuf::from("/test.pdf"),
            DocumentType::PlainText,
            vec![],
            OCRContent::empty(),
        );
        
        // Simuler une session créée il y a 2 heures
        session.created_at = SystemTime::now()
            .checked_sub(std::time::Duration::from_secs(7200))
            .unwrap();
        
        assert!(session.is_expired(3600)); // TTL de 1 heure
        assert!(!session.is_expired(10800)); // TTL de 3 heures
    }

    #[test]
    fn test_structured_data_creation() {
        let payslip_data = PayslipData {
            employee_name: "Jean Dupont".to_string(),
            period: "2025-01".to_string(),
            gross_salary: 3500.0,
            net_salary: 2600.0,
            deductions: vec![],
            employer_info: EmployerInfo {
                name: "ACME Corp".to_string(),
                address: None,
                siret: None,
            },
            spans: PayslipSpans {
                employee_name_span: Some("span_1".to_string()),
                gross_salary_span: Some("span_2".to_string()),
                net_salary_span: Some("span_3".to_string()),
                deduction_spans: HashMap::new(),
            },
        };
        
        let structured_data = StructuredData::Payslip(payslip_data);
        
        match structured_data {
            StructuredData::Payslip(data) => {
                assert_eq!(data.employee_name, "Jean Dupont");
                assert_eq!(data.gross_salary, 3500.0);
                assert!(data.spans.employee_name_span.is_some());
            }
            _ => panic!("Should be payslip data"),
        }
    }
}