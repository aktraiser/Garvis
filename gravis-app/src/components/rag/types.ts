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

export type ChunkProfile = 'precise' | 'balanced' | 'large';

export interface ChunkProfileConfig {
  name: string;
  icon: string;
  chunkSize: number;
  chunkOverlap: number;
  description: string;
  details: string;
  expectedChunks: string;
  bestFor: string[];
}

export const CHUNK_PROFILES: Record<ChunkProfile, ChunkProfileConfig> = {
  precise: {
    name: 'Précision Maximale',
    icon: '🎯',
    chunkSize: 256,
    chunkOverlap: 32,
    description: 'Plus de chunks, meilleure précision pour les détails',
    details: 'Idéal pour trouver des informations très spécifiques',
    expectedChunks: '~40-50 chunks',
    bestFor: ['Questions précises', 'Documents techniques', 'Recherche de détails']
  },
  balanced: {
    name: 'Équilibré',
    icon: '⭐',
    chunkSize: 384,
    chunkOverlap: 48,
    description: 'Configuration optimale pour E5-small-v2 (recommandé)',
    details: 'Meilleur compromis qualité/performance',
    expectedChunks: '~25-30 chunks',
    bestFor: ['Usage général', 'Mix questions larges/précises', 'Meilleure performance']
  },
  large: {
    name: 'Contexte Large',
    icon: '📚',
    chunkSize: 512,
    chunkOverlap: 64,
    description: 'Moins de chunks, meilleur pour les questions générales',
    details: 'Plus rapide à indexer, bon pour les résumés',
    expectedChunks: '~15-20 chunks',
    bestFor: ['Questions larges', 'Résumés de documents', 'Indexation rapide']
  }
};

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
  chunkProfile: ChunkProfile;
  chunkSize: number;
  chunkOverlap: number;
}

export interface NotificationState {
  message: string;
  type: 'success' | 'error' | 'info';
}

export type TabType = 'documents' | 'injection';