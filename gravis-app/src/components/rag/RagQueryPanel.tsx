import React, { useState } from 'react';
import { Search, Loader2, FileText, CheckCircle2 } from 'lucide-react';
import { useRagQuery, type RagContextResponse } from '@/hooks/useRagQuery';

interface RagQueryPanelProps {
  onContextGenerated?: (context: RagContextResponse) => void;
  autoFocus?: boolean;
}

const RagQueryPanel: React.FC<RagQueryPanelProps> = ({ onContextGenerated, autoFocus = false }) => {
  const [query, setQuery] = useState('');
  const [selectedCollection, setSelectedCollection] = useState('default_group');
  const [resultLimit, setResultLimit] = useState(5);

  const { isQuerying, ragContext, error, queryRagWithContext, clearContext } = useRagQuery();

  const handleSearch = async () => {
    if (!query.trim()) {
      return;
    }

    const response = await queryRagWithContext({
      query: query.trim(),
      groupId: selectedCollection,
      limit: resultLimit
    });

    if (response && onContextGenerated) {
      onContextGenerated(response);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSearch();
    }
  };

  return (
    <div className="flex flex-col gap-4 p-4 bg-gray-50 rounded-lg border border-gray-200">
      {/* En-tête */}
      <div className="flex items-center gap-2">
        <FileText className="w-5 h-5 text-blue-600" />
        <h3 className="font-semibold text-gray-900">Recherche RAG Intelligente</h3>
      </div>

      {/* Configuration */}
      <div className="flex flex-col sm:flex-row gap-3">
        {/* Sélecteur de collection */}
        <div className="flex flex-col gap-1 flex-1">
          <label className="text-xs font-medium text-gray-600">Collection RAG</label>
          <select
            value={selectedCollection}
            onChange={(e) => setSelectedCollection(e.target.value)}
            className="px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            disabled={isQuerying}
          >
            <option value="default_group">📚 Groupe par défaut</option>
            {/* TODO: Charger dynamiquement les collections disponibles */}
          </select>
        </div>

        {/* Limite de résultats */}
        <div className="flex flex-col gap-1 w-24">
          <label className="text-xs font-medium text-gray-600">Résultats</label>
          <select
            value={resultLimit}
            onChange={(e) => setResultLimit(Number(e.target.value))}
            className="px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            disabled={isQuerying}
          >
            <option value={3}>3</option>
            <option value={5}>5</option>
            <option value={10}>10</option>
            <option value={15}>15</option>
          </select>
        </div>
      </div>

      {/* Barre de recherche */}
      <div className="flex gap-2">
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyPress={handleKeyPress}
          placeholder="Recherchez dans vos documents RAG..."
          className="flex-1 px-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          disabled={isQuerying}
          autoFocus={autoFocus}
        />
        <button
          onClick={handleSearch}
          disabled={isQuerying || !query.trim()}
          className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed flex items-center gap-2 transition-colors"
        >
          {isQuerying ? (
            <>
              <Loader2 className="w-4 h-4 animate-spin" />
              <span>Recherche...</span>
            </>
          ) : (
            <>
              <Search className="w-4 h-4" />
              <span>Rechercher</span>
            </>
          )}
        </button>
      </div>

      {/* Erreur */}
      {error && (
        <div className="p-3 bg-red-50 border border-red-200 rounded-md text-sm text-red-700">
          {error}
        </div>
      )}

      {/* Résultats */}
      {ragContext && (
        <div className="flex flex-col gap-3 p-4 bg-white border border-gray-200 rounded-md">
          {/* En-tête des résultats */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <CheckCircle2 className="w-5 h-5 text-green-600" />
              <span className="font-medium text-gray-900">
                {ragContext.total_chunks} chunk(s) trouvé(s)
              </span>
            </div>
            <span className="text-xs text-gray-500">
              {ragContext.search_time_ms}ms
            </span>
          </div>

          {/* Liste des sources */}
          <div className="flex flex-col gap-2">
            <h4 className="text-sm font-semibold text-gray-700">Sources détectées:</h4>
            <div className="grid grid-cols-1 gap-2">
              {ragContext.sources.map((source, idx) => (
                <div
                  key={source.chunk_id}
                  className="p-3 bg-gray-50 rounded border border-gray-200 hover:border-blue-300 transition-colors"
                >
                  <div className="flex items-start justify-between gap-2 mb-2">
                    <div className="flex items-center gap-2">
                      <span className="px-2 py-0.5 bg-blue-100 text-blue-700 text-xs font-medium rounded">
                        Source {idx + 1}
                      </span>
                      {source.source_file && (
                        <span className="text-xs text-gray-600 truncate max-w-[200px]" title={source.source_file}>
                          📄 {source.source_file}
                        </span>
                      )}
                    </div>
                    <span className="text-xs font-semibold text-green-600">
                      {(source.score * 100).toFixed(1)}%
                    </span>
                  </div>

                  {source.document_category && (
                    <div className="mb-2">
                      <span className="text-xs px-2 py-0.5 bg-purple-100 text-purple-700 rounded">
                        {source.document_category}
                      </span>
                    </div>
                  )}

                  <p className="text-xs text-gray-700 line-clamp-3">
                    {source.content_preview}...
                  </p>
                </div>
              ))}
            </div>
          </div>

          {/* Bouton pour effacer */}
          <button
            onClick={clearContext}
            className="text-sm text-gray-600 hover:text-gray-900 underline"
          >
            Effacer les résultats
          </button>
        </div>
      )}
    </div>
  );
};

export default RagQueryPanel;
