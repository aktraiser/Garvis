import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface SourceInfo {
  document_id: string;
  chunk_id: string;
  content_preview: string;
  score: number;
  source_file?: string;
  document_category?: string;
}

export interface RagContextResponse {
  formatted_context: string;
  sources: SourceInfo[];
  total_chunks: number;
  query: string;
  search_time_ms: number;
}

export interface RagQueryParams {
  query: string;
  groupId: string;
  limit?: number;
}

export const useRagQuery = () => {
  const [isQuerying, setIsQuerying] = useState(false);
  const [ragContext, setRagContext] = useState<RagContextResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  const queryRagWithContext = useCallback(async (params: RagQueryParams): Promise<RagContextResponse | null> => {
    setIsQuerying(true);
    setError(null);

    try {
      console.log('🔍 Querying RAG with context:', params);

      const response = await invoke<RagContextResponse>('query_rag_with_context', {
        query: params.query,
        groupId: params.groupId,
        limit: params.limit || 5
      });

      console.log('✅ RAG context response:', response);
      setRagContext(response);

      return response;

    } catch (err) {
      const errorMsg = `Erreur de requête RAG: ${err}`;
      console.error('❌ RAG query failed:', err);
      setError(errorMsg);
      return null;

    } finally {
      setIsQuerying(false);
    }
  }, []);

  const clearContext = useCallback(() => {
    setRagContext(null);
    setError(null);
  }, []);

  return {
    // State
    isQuerying,
    ragContext,
    error,

    // Actions
    queryRagWithContext,
    clearContext
  };
};
