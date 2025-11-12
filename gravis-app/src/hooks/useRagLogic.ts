import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { InjectionMetadata, DocumentInfo, ChunkProfile } from '@/components/rag/types';
import { CHUNK_PROFILES } from '@/components/rag/types';

// Fonction pour calculer la similarité entre deux textes (Jaccard simple)
const calculateTextSimilarity = (text1: string, text2: string): number => {
  const words1 = new Set(text1.toLowerCase().split(/\s+/));
  const words2 = new Set(text2.toLowerCase().split(/\s+/));
  
  const intersection = new Set([...words1].filter(x => words2.has(x)));
  const union = new Set([...words1, ...words2]);
  
  return union.size === 0 ? 0 : intersection.size / union.size;
};

// Déduplication des résultats RAG par similarité de contenu
const deduplicateResults = (results: any[]): any[] => {
  const SIMILARITY_THRESHOLD = 0.7; // 70% de similarité = doublon
  const deduplicated: any[] = [];
  
  for (const result of results) {
    let isDuplicate = false;
    
    for (const existing of deduplicated) {
      const similarity = calculateTextSimilarity(result.content, existing.content);
      if (similarity >= SIMILARITY_THRESHOLD) {
        // Garder celui avec le meilleur score
        if (result.score > existing.score) {
          const index = deduplicated.indexOf(existing);
          deduplicated[index] = result;
        }
        isDuplicate = true;
        break;
      }
    }
    
    if (!isDuplicate) {
      deduplicated.push(result);
    }
  }
  
  return deduplicated.sort((a, b) => b.score - a.score); // Trier par score décroissant
};

export const useRagLogic = () => {
  const [isProcessing, setIsProcessing] = useState<Record<string, boolean>>({});
  const [processResults, setProcessResults] = useState<Record<string, any>>({});
  const [ragQuery, setRagQuery] = useState<string>('');
  const [ragResults, setRagResults] = useState<any[]>([]);
  const [isSearching, setIsSearching] = useState<boolean>(false);
  const [ragDocuments, setRagDocuments] = useState<any[]>([]);
  const [isLoadingRagDocs, setIsLoadingRagDocs] = useState<boolean>(false);
  const [showInjectionModal, setShowInjectionModal] = useState<string | null>(null);
  const [injectionMetadata, setInjectionMetadata] = useState<InjectionMetadata>({
    title: '',
    description: '',
    author: '',
    category: 'document',
    groupId: 'default_group',
    tags: '',
    priority: 'normal',
    language: 'auto',
    forceOcr: false,
    chunkProfile: 'balanced',  // Profil par défaut
    chunkSize: 384,  // Optimisé pour E5-small-v2 (256-512 tokens idéal)
    chunkOverlap: 48  // 12.5% d'overlap pour continuité sans redondance
  });



  // Fonction pour changer le profil de chunking
  const setChunkProfile = useCallback((profile: ChunkProfile) => {
    const config = CHUNK_PROFILES[profile];
    setInjectionMetadata(prev => ({
      ...prev,
      chunkProfile: profile,
      chunkSize: config.chunkSize,
      chunkOverlap: config.chunkOverlap
    }));
  }, []);

  const prepareInjectionMetadata = useCallback((docName: string, documents: DocumentInfo[], extractedContent: Record<string, any>) => {
    const doc = documents.find(d => d.name === docName);
    const extractedDoc = extractedContent[docName];
    
    const title = docName.replace(/\.[^/.]+$/, "").replace(/[_-]/g, " ");
    
    let category = 'document';
    if (docName.toLowerCase().includes('facture')) category = 'facture';
    else if (docName.toLowerCase().includes('contrat')) category = 'contrat';
    else if (docName.toLowerCase().includes('rapport')) category = 'rapport';
    else if (docName.toLowerCase().includes('manuel')) category = 'manuel';
    
    let author = '';
    if (extractedDoc?.business_metadata?.author) {
      author = extractedDoc.business_metadata.author;
    }
    
    let description = `Document: ${title}`;
    if (doc) {
      description += ` (${doc.pages} pages, ${doc.size})`;
    }
    if (extractedDoc?.business_metadata?.title) {
      description = extractedDoc.business_metadata.title;
    }
    
    const suggestedTags = [];
    if (category !== 'document') suggestedTags.push(category);
    if (doc?.type) suggestedTags.push(doc.type);
    if (extractedDoc?.language) suggestedTags.push(extractedDoc.language);
    
    setInjectionMetadata({
      title,
      description,
      author,
      category,
      groupId: 'default_group',
      tags: suggestedTags.join(', '),
      priority: 'normal',
      language: extractedDoc?.language || 'auto',
      forceOcr: false,
      chunkProfile: 'balanced',
      chunkSize: 384,
      chunkOverlap: 48
    });
  }, []);

  const handleInjectDocumentWithMetadata = useCallback(async (docName: string, extractedContent: Record<string, any>, notificationCallback: (msg: string, type: 'success' | 'error' | 'info') => void) => {
    setIsProcessing(prev => ({ ...prev, [docName]: true }));
    
    try {
      const filePath = `exemple/${docName}`;
      console.log(`🚀 Injecting document: ${filePath} into group: ${injectionMetadata.groupId}`);

      const preExtracted = extractedContent[docName];
      const extractedText = preExtracted?.content || null;

      if (extractedText) {
        console.log(`📄 Using pre-extracted text (${extractedText.length} chars) from previous extraction`);
      } else {
        console.log('🔍 No pre-extracted text, will extract during injection');
      }


      const result = await invoke<any>('add_document_intelligent', {
        filePath: filePath,
        groupId: injectionMetadata.groupId,
        extractedText: extractedText
      });
      
      console.log('✅ Document injection result:', result);
      
      setProcessResults(prev => ({
        ...prev,
        [docName]: result
      }));
      
      notificationCallback(
        `Document "${docName}" injecté avec succès! (${result.chunks_created} chunks créés)`,
        'success'
      );
      
      setShowInjectionModal(null);
      
    } catch (error) {
      console.error('❌ Injection failed:', error);
      notificationCallback(`Erreur d'injection: ${error}`, 'error');
    } finally {
      setIsProcessing(prev => ({ ...prev, [docName]: false }));
    }
  }, [injectionMetadata]);

  const handleRagSearch = useCallback(async (notificationCallback: (msg: string, type: 'success' | 'error' | 'info') => void) => {
    if (!ragQuery.trim()) {
      notificationCallback('Veuillez saisir une requête', 'error');
      return;
    }

    setIsSearching(true);
    try {
      console.log(`🔍 Searching RAG for: "${ragQuery}"`);
      
      const searchResults = await invoke<any>('search_with_metadata', {
        params: {
          query: ragQuery,
          groupId: 'default_group',
          limit: 10,
          includeContent: true,
          includeBusinessMetadata: true
        }
      });
      
      console.log('🔍 RAG search results:', searchResults);
      
      // Déduplication par similarité de contenu
      const deduplicatedResults = deduplicateResults(searchResults.results || []);
      console.log(`📊 Déduplication: ${searchResults.results?.length || 0} → ${deduplicatedResults.length} résultats`);
      
      setRagResults(deduplicatedResults);
      
      if (deduplicatedResults.length === 0) {
        notificationCallback('Aucun résultat trouvé', 'info');
      } else {
        const originalCount = searchResults.results?.length || 0;
        const dedupCount = deduplicatedResults.length;
        
        if (originalCount === dedupCount) {
          notificationCallback(`${dedupCount} résultat(s) trouvé(s)`, 'success');
        } else {
          notificationCallback(`${dedupCount} résultat(s) uniques trouvé(s) (${originalCount - dedupCount} doublons filtrés)`, 'success');
        }
      }
      
    } catch (error) {
      console.error('❌ RAG search failed:', error);
      notificationCallback(`Erreur de recherche: ${error}`, 'error');
    } finally {
      setIsSearching(false);
    }
  }, [ragQuery]);

  const loadRagDocuments = useCallback(async (notificationCallback: (msg: string, type: 'success' | 'error' | 'info') => void) => {
    setIsLoadingRagDocs(true);
    try {
      console.log('📚 Loading RAG documents...');
      
      const documents = await invoke<any[]>('list_rag_documents', {
        groupId: 'default_group'
      });
      
      console.log('📚 RAG documents loaded:', documents);
      setRagDocuments(documents);
      
      notificationCallback(`${documents.length} document(s) RAG chargé(s)`, 'success');
      
    } catch (error) {
      console.error('❌ Failed to load RAG documents:', error);
      notificationCallback(`Erreur de chargement RAG: ${error}`, 'error');
    } finally {
      setIsLoadingRagDocs(false);
    }
  }, []);

  const handleDeleteRagDocument = useCallback(async (documentId: string, notificationCallback: (msg: string, type: 'success' | 'error' | 'info') => void) => {
    try {
      console.log(`🗑️ Deleting RAG document: ${documentId}`);
      
      const result = await invoke<any>('delete_rag_document', {
        documentId: documentId,
        groupId: 'default_group'
      });
      
      console.log('🗑️ RAG document deletion result:', result);
      
      setRagDocuments(prev => prev.filter(doc => doc.document_id !== documentId));
      
      notificationCallback(`Document RAG supprimé (${result.chunks_deleted} chunks)`, 'success');
      
    } catch (error) {
      console.error('❌ RAG document deletion failed:', error);
      notificationCallback(`Erreur de suppression RAG: ${error}`, 'error');
    }
  }, []);

  return {
    // State
    isProcessing,
    processResults,
    ragQuery,
    ragResults,
    isSearching,
    ragDocuments,
    isLoadingRagDocs,
    showInjectionModal,
    injectionMetadata,

    // Actions
    setRagQuery,
    setShowInjectionModal,
    setInjectionMetadata,
    setChunkProfile,
    prepareInjectionMetadata,
    handleInjectDocumentWithMetadata,
    handleRagSearch,
    loadRagDocuments,
    handleDeleteRagDocument
  };
};