// Types pour les résultats des commandes Tauri
export interface DocumentIngestionResponse {
  document_id: string;
  document_category: string;
  chunks_created: number;
  extraction_method: string;
  source_type: string;
  processing_time_ms: number;
  confidence_score?: number;
  business_metadata?: any;
  cache_stats?: any;
}

export interface DeleteRagDocumentResponse {
  document_id: string;
  chunks_deleted: number;
  success: boolean;
}

export interface DocumentInfo {
  id: string;
  name: string;
  size: string;
  sizeBytes: number;
  type: string;
  status: string;
  date: string;
  category: string;
  pages: number;
  extracted: boolean;
  extractedAt: string;
  confidence: number;
}

export interface InjectionMetadata {
  // Informations de base
  title: string;
  description: string;
  author: string;
  category: string;
  
  // Configuration RAG
  groupId: string;
  tags: string;
  priority: string;
  language: string;
  
  // Options techniques
  forceOcr: boolean;
  chunkSize: number;
  chunkOverlap: number;
}

export interface NotificationState {
  message: string;
  type: 'success' | 'error' | 'info';
}

export type TabType = 'documents' | 'injection';