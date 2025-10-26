// RAG Interface TypeScript pour les commandes Tauri
import { invoke } from '@tauri-apps/api/core';

// === Types === 

export interface DocumentGroup {
  id: string;
  name: string;
  active: boolean;
  chunk_config: ChunkConfig;
  metadata_config: MetadataConfig;
  documents: GroupDocument[];
  qdrant_collection: string;
  created_at: string;
  updated_at: string;
}

export interface ChunkConfig {
  chunk_size: number;
  overlap: number;
  strategy: 'AstFirst' | 'Heuristic' | 'Hybrid';
}

export interface MetadataConfig {
  default_tags: string[];
  default_priority: 'Low' | 'Normal' | 'High';
  auto_language_detection: boolean;
}

export interface GroupDocument {
  id: string;
  file_path: string;
  language: string;
  content: string;
  chunks: EnrichedChunk[];
  metadata: EnrichedMetadata;
  last_modified: string;
  document_type: DocumentType;
  group_id: string;
}

export interface EnrichedChunk {
  id: string;
  content: string;
  start_line: number;
  end_line: number;
  chunk_type: 'Function' | 'Class' | 'Module' | 'TextBlock' | 'Comment';
  embedding?: number[];
  hash: string;
  metadata: ChunkMetadata;
  group_id: string;
}

export interface ChunkMetadata {
  tags: string[];
  priority: 'Low' | 'Normal' | 'High';
  language: string;
  symbol?: string;
  context?: string;
  confidence: number;
}

export interface EnrichedMetadata {
  tags: string[];
  priority: 'Low' | 'Normal' | 'High';
  description?: string;
  author?: string;
  project?: string;
  custom_fields: Record<string, string>;
}

export type DocumentType = 
  | { SourceCode: { language: string } }
  | { PDF: { has_text: boolean; pages: number } }
  | { Image: { ocr_confidence: number } }
  | 'Markdown'
  | 'PlainText';

// === RAG Client ===

export class RagClient {
  // === Gestion des Groupes ===
  
  static async createGroup(name: string, chunkConfig?: Partial<ChunkConfig>): Promise<DocumentGroup> {
    const config: ChunkConfig = {
      chunk_size: chunkConfig?.chunk_size || 512,
      overlap: chunkConfig?.overlap || 64,
      strategy: chunkConfig?.strategy || 'AstFirst'
    };
    
    return await invoke('rag_create_group', { 
      name,
      chunkConfig: config 
    });
  }
  
  static async listGroups(): Promise<DocumentGroup[]> {
    return await invoke('rag_list_groups');
  }
  
  static async updateGroup(groupId: string, updates: Partial<DocumentGroup>): Promise<DocumentGroup> {
    return await invoke('rag_update_group', { 
      groupId, 
      updates 
    });
  }
  
  static async deleteGroup(groupId: string): Promise<boolean> {
    return await invoke('rag_delete_group', { groupId });
  }
  
  static async toggleGroup(groupId: string, active: boolean): Promise<boolean> {
    return await invoke('rag_toggle_group', { groupId, active });
  }
  
  // === Upload et Indexation ===
  
  static async uploadToGroup(
    groupId: string, 
    files: File[], 
    metadata: Partial<EnrichedMetadata>
  ): Promise<any> {
    // Convert files to base64 for Tauri
    const fileUploads = await Promise.all(
      files.map(async (file) => ({
        path: file.name,
        content: Array.from(new Uint8Array(await file.arrayBuffer())),
        filename: file.name,
        mime_type: file.type
      }))
    );
    
    return await invoke('rag_upload_to_group', {
      groupId,
      files: fileUploads,
      metadata: {
        tags: metadata.tags || [],
        priority: metadata.priority || 'Normal',
        description: metadata.description,
        custom_fields: metadata.custom_fields || {}
      }
    });
  }
  
  static async indexGroupDocuments(groupId: string): Promise<any> {
    return await invoke('rag_index_group_documents', { 
      groupId,
      progressCallback: null 
    });
  }
  
  static async reindexDocument(documentId: string, newConfig?: ChunkConfig): Promise<any> {
    return await invoke('rag_reindex_document', { 
      documentId, 
      newConfig 
    });
  }
  
  // === Recherche ===
  
  static async searchInGroups(
    query: string, 
    activeGroups: string[],
    filters?: any,
    limit: number = 10
  ): Promise<any[]> {
    return await invoke('rag_search_in_groups', {
      query,
      activeGroups,
      filters: filters || {},
      limit
    });
  }
  
  static async getContextForQuery(query: string, maxChunks: number = 5): Promise<any> {
    return await invoke('rag_get_context_for_query', {
      query,
      maxChunks
    });
  }
  
  // === Gestion Documents ===
  
  static async listGroupDocuments(groupId: string): Promise<GroupDocument[]> {
    return await invoke('rag_list_group_documents', { groupId });
  }
  
  static async removeDocument(documentId: string): Promise<boolean> {
    return await invoke('rag_remove_document', { documentId });
  }
  
  static async getDocumentChunks(documentId: string): Promise<EnrichedChunk[]> {
    return await invoke('rag_get_document_chunks', { documentId });
  }
  
  // === Status ===
  
  static async getStatus(): Promise<string> {
    return await invoke('rag_get_status');
  }
}

// === Store local pour la gestion des groupes ===

export class RagStore {
  private static groups: DocumentGroup[] = [];
  private static listeners: ((groups: DocumentGroup[]) => void)[] = [];
  
  static subscribe(listener: (groups: DocumentGroup[]) => void) {
    this.listeners.push(listener);
    listener(this.groups);
    
    // Return unsubscribe function
    return () => {
      const index = this.listeners.indexOf(listener);
      if (index > -1) {
        this.listeners.splice(index, 1);
      }
    };
  }
  
  private static notify() {
    this.listeners.forEach(listener => listener(this.groups));
  }
  
  static async loadGroups() {
    try {
      this.groups = await RagClient.listGroups();
      this.notify();
    } catch (error) {
      console.error('Error loading RAG groups:', error);
    }
  }
  
  static async createGroup(name: string, chunkConfig?: Partial<ChunkConfig>) {
    try {
      const group = await RagClient.createGroup(name, chunkConfig);
      this.groups.push(group);
      this.notify();
      return group;
    } catch (error) {
      console.error('Error creating RAG group:', error);
      throw error;
    }
  }
  
  static async toggleGroup(groupId: string) {
    try {
      const group = this.groups.find(g => g.id === groupId);
      if (!group) return;
      
      const newActive = !group.active;
      await RagClient.toggleGroup(groupId, newActive);
      
      group.active = newActive;
      this.notify();
    } catch (error) {
      console.error('Error toggling RAG group:', error);
      throw error;
    }
  }
  
  static async deleteGroup(groupId: string) {
    try {
      await RagClient.deleteGroup(groupId);
      this.groups = this.groups.filter(g => g.id !== groupId);
      this.notify();
    } catch (error) {
      console.error('Error deleting RAG group:', error);
      throw error;
    }
  }
  
  static getGroups() {
    return this.groups;
  }
  
  static getActiveGroups() {
    return this.groups.filter(g => g.active);
  }
}

// === Test des commandes RAG ===

export async function testRagCommands() {
  try {
    console.log('🧪 Testing RAG Commands Phase 2...');
    
    // Test statut RAG
    const status = await RagClient.getStatus();
    console.log('✅ RAG Status:', status);
    
    // Test création groupe
    const group = await RagClient.createGroup('Test Group Frontend', {
      chunk_size: 512,
      overlap: 64,
      strategy: 'AstFirst'
    });
    console.log('✅ Created Group:', group);
    
    // Test liste groupes
    const groups = await RagClient.listGroups();
    console.log('✅ Listed Groups:', groups);
    
    console.log('🎉 All RAG commands working!');
    return { success: true, status, group, groups };
    
  } catch (error) {
    console.error('❌ RAG Commands Error:', error);
    return { success: false, error };
  }
}