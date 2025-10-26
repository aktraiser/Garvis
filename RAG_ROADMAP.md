# GRAVIS RAG - Feuille de Route Détaillée

## 🎯 Objectif
Intégrer un système RAG (Retrieval-Augmented Generation) robuste en Rust dans GRAVIS pour l'analyse et l'audit de code ET de documents utilisateur, utilisant candle + E5 embedder, hf-hub, et qdrant avec interface d'upload.

## 📋 Phases de Développement

### 🏗️ Phase 1: Infrastructure & Setup (Semaines 1-2)

#### Semaine 1: Configuration de Base
- **Jour 1-2: Setup Rust Dependencies**
  - [ ] Ajouter les crates RAG au Cargo.toml
  - [ ] Configuration candle-core avec support GPU/CPU
  - [ ] Setup hf-hub pour le téléchargement de modèles
  - [ ] Test de base avec un petit modèle d'embedding

- **Jour 3-4: Structure du Projet**
  - [ ] Créer l'architecture modulaire (/rag, /commands, /models)
  - [ ] Définir les structures de données (CodeDocument, CodeChunk, etc.)
  - [ ] Setup des tests unitaires de base
  - [ ] Configuration logging avec tracing

- **Jour 5-7: Qdrant Integration**
  - [ ] Installation et configuration Qdrant local
  - [ ] Client Rust pour Qdrant
  - [ ] Création des collections et schémas
  - [ ] Tests CRUD de base sur les vecteurs

#### Semaine 2: Core Components
- **Jour 8-10: Document Processor & OCR**
  - [ ] Setup Tesseract avec configuration optimale
  - [ ] Processor unifié pour tous types de documents
  - [ ] Extraction de texte natif des PDFs
  - [ ] Fallback OCR pour PDFs scannés et images
  - [ ] Preprocessing d'images pour améliorer l'OCR

- **Jour 11-14: Embedder Engine**
  - [ ] Chargement des modèles via hf-hub
  - [ ] Implémentation embeddings avec candle
  - [ ] Cache et optimisation des embeddings
  - [ ] Benchmark des différents modèles (CodeBERT, UniXcoder, StarEncoder)

### 🔧 Phase 2: Core RAG Engine (Semaines 3-4)

#### Semaine 3: Indexation Pipeline
- **Jour 15-17: Document Processing**
  - [ ] Pipeline d'ingestion des fichiers
  - [ ] Filtrage par extensions et gitignore
  - [ ] Processing asynchrone avec tokio
  - [ ] Gestion des erreurs et retry logic

- **Jour 18-21: Vector Storage**
  - [ ] Optimisation des insertions batch
  - [ ] Indexation incrémentale (detect changes)
  - [ ] Métadonnées enrichies (git info, dependencies)
  - [ ] Compression et optimisation mémoire

#### Semaine 4: Retrieval System
- **Jour 22-24: Search Engine**
  - [ ] Algorithmes de recherche vectorielle
  - [ ] Filtres avancés (langage, type, date)
  - [ ] Scoring et ranking des résultats
  - [ ] Recherche hybride (vectorielle + keyword)

- **Jour 25-28: Query Understanding**
  - [ ] Processing des requêtes naturelles
  - [ ] Expansion de requêtes
  - [ ] Context-aware search
  - [ ] Suggestions et auto-complétion

### 🎨 Phase 3: Interface & Integration (Semaines 5-6)

#### Semaine 5: Tauri Commands & Upload Interface
- **Jour 29-31: Backend Commands**
  - [ ] Commandes Tauri pour indexation projets
  - [ ] Commandes upload/indexation documents utilisateur
  - [ ] API de recherche asynchrone avec streaming
  - [ ] Gestion du statut d'indexation (progress events)

- **Jour 32-35: Frontend Integration - Modale RAG Avancée**
  - [ ] Bouton RAG (📄) à côté du bouton web avec indicateur
  - [ ] Modale plein écran pour gestion des groupes
  - [ ] Interface groupes : création, édition, suppression
  - [ ] Upload zone drag & drop avec sélection de groupe
  - [ ] Configuration chunking par upload (size, overlap, strategy)
  - [ ] Métadonnées enrichies (tags, priority, language)
  - [ ] Toggle activation par groupe + statut global

#### Semaine 6: UX & Polish
- **Jour 36-38: User Experience**
  - [ ] Progress indicators pour indexation
  - [ ] Gestion des erreurs utilisateur
  - [ ] Settings de configuration RAG
  - [ ] Help et documentation intégrée

- **Jour 39-42: Performance UI**
  - [ ] Lazy loading des résultats
  - [ ] Virtualisation pour grandes listes
  - [ ] Caching côté frontend
  - [ ] Optimisation des re-renders

### 🚀 Phase 4: Features Avancées (Semaines 7-8)

#### Semaine 7: Intelligence Avancée
- **Jour 43-45: Code Understanding**
  - [ ] Analyse des dépendances
  - [ ] Détection de patterns et anti-patterns
  - [ ] Similarité sémantique entre fonctions
  - [ ] Suggestions de refactoring

- **Jour 46-49: Multi-Modal Search**
  - [ ] Recherche par similarité de code
  - [ ] Recherche par description fonctionnelle
  - [ ] Recherche par usage/exemples
  - [ ] Cross-language similarity

#### Semaine 8: Production Ready
- **Jour 50-52: Optimisations**
  - [ ] Profiling et optimization
  - [ ] Gestion mémoire avancée
  - [ ] Parallélisation des opérations
  - [ ] Benchmark et métriques

- **Jour 53-56: Déploiement**
  - [ ] Configuration production
  - [ ] Documentation utilisateur
  - [ ] Tests d'intégration complets
  - [ ] Packaging et distribution

## 🛠️ Outils & Technologies

### Stack Principal (Recommandations Experts Intégrées)
```toml
[dependencies]
# Core RAG - E5 Embedder robuste
candle-core = "0.3"
candle-nn = "0.3"
candle-transformers = "0.3"
hf-hub = "0.3"
qdrant-client = "1.7"
tokenizers = "0.13"

# Document Processing & Upload
tree-sitter = "0.20"
tree-sitter-rust = "0.20"
tree-sitter-typescript = "0.20"
tree-sitter-python = "0.20"
tree-sitter-javascript = "0.20"
pdf-extract = "0.6"
image = { version = "0.24", features = ["png", "jpeg", "tiff"] }

# Database & Storage
sqlx = { version = "0.7", features = ["sqlite"] }

# Async & Utils (experts)
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
rayon = "1.7"
dashmap = "5.5"
blake3 = "1.5"
notify = "6.0"
walkdir = "2.3"

# Optional GPU (Metal macOS / CUDA)
# candle-core = { version = "0.3", features = ["metal"] }
```

### Modèles d'Embedding (Stratégie Experts)
1. **E5-Small-v2** (`intfloat/e5-small-v2`) - 384d, robuste, tout-Rust ✅
2. **MiniLM-L6-v2** (`sentence-transformers/all-MiniLM-L6-v2`) - 384d, éprouvé ✅
3. **StarEncoder** (`bigcode/starencoder`) - 768d, code-centric (si candle support)
4. **UniXcoder** (`microsoft/unixcoder-base`) - 768d, multi-langage (ONNX fallback)

### Infrastructure
- **Qdrant** : Base vectorielle locale
- **SQLite** : Métadonnées et cache
- **Tree-sitter** : Parsing AST
- **Tokio** : Runtime async

## 📊 Métriques de Succès

### Performance
- **Indexation** : >1000 fichiers/min
- **Recherche** : <100ms latence
- **Mémoire** : <2GB pour 100k fichiers
- **Précision** : >85% relevance@10

### Fonctionnalités
- [ ] Support 5+ langages de programmation
- [ ] Recherche en langage naturel
- [ ] Indexation incrémentale temps réel
- [ ] Interface intuitive et rapide

## 🔧 Configuration Recommandée

### Développement
```bash
# Qdrant local
docker run -p 6333:6333 qdrant/qdrant

# Modèles (téléchargement automatique)
mkdir models/
# CodeBERT: ~500MB
# UniXcoder: ~500MB
```

### Production
- **RAM** : 8GB minimum, 16GB recommandé
- **Storage** : SSD pour performances vectorielles
- **CPU** : Support AVX2 pour optimisations SIMD
- **GPU** : Optionnel, améliore les embeddings

## 🎯 Livrables par Phase

### Phase 1
- [ ] Architecture Rust fonctionnelle
- [ ] Tests unitaires de base
- [ ] Documentation technique

### Phase 2
- [ ] Pipeline d'indexation complet
- [ ] Recherche vectorielle opérationnelle
- [ ] Benchmarks de performance

### Phase 3
- [ ] Interface utilisateur intégrée
- [ ] Commandes Tauri exposées
- [ ] Documentation utilisateur

### Phase 4
- [ ] Features avancées fonctionnelles
- [ ] Optimisations de production
- [ ] Système prêt pour déploiement

## 🚨 Risques & Mitigation

### Risques Techniques
1. **Performance des embeddings** → Benchmark multiple modèles early
2. **Scalabilité Qdrant** → Tests avec gros datasets
3. **Complexité tree-sitter** → Start avec langages simples

### Risques Projet
1. **Scope creep** → Phases strictes et bien définies
2. **Intégration Tauri** → POC rapide en Phase 1
3. **UX complexe** → Prototypage utilisateur en Phase 3

## 📚 Resources & Learning

### Documentation Essentielle
- [Candle Book](https://huggingface.co/docs/candle/)
- [Qdrant Docs](https://qdrant.tech/documentation/)
- [Tree-sitter Guide](https://tree-sitter.github.io/tree-sitter/)

### Références Code
- [Candle Examples](https://github.com/huggingface/candle/tree/main/candle-examples)
- [Code Search Benchmarks](https://github.com/github/CodeSearchNet)

## 🎯 Prochaines étapes prêtes à exécuter (Recommandations Experts)

### Phase 1: Squelette RAG Core
1. **Setup dependencies** : candle-nn, anyhow, thiserror, tracing, rayon, dashmap, blake3
2. **Device detection** : CUDA/Metal/CPU avec logging
3. **E5 Embedder** : hf-hub + candle + tokenizers (384d robuste)
4. **Qdrant local** : docker + 2 collections (code_chunks + documents)

### Phase 2: Document Pipeline + Interface Modale
1. **Chunking configuré** : AST/heuristique avec paramètres par upload
2. **Interface modale RAG** : Gestion groupes + configuration avancée
3. **Upload avec métadonnées** : Tags, priority, language, chunking params
4. **Commands Tauri** : create_group, upload_to_group, toggle_group, search_groups

### Phase 3: Optimisations Experts
1. **Batch processing** : 512-2048 points/batch
2. **Cache embeddings** : blake3 hash (contenu + model_id)
3. **HNSW config** : m=16, ef_construct=128, quantization si >1M points
4. **Streaming UI** : pagination + virtualisation résultats

### 🧭 Mini-POC E5 Embedder (Copier-Coller)
```rust
use anyhow::Result;
use hf_hub::{api::sync::Api, Repo, RepoType};
use candle_core::Device;
use qdrant_client::prelude::*;
use serde_json::json;

// Device detection (recommandation experts)
pub fn pick_device() -> Result<Device> {
    #[cfg(all(target_os="macos", feature="metal"))] { 
        return Ok(Device::new_metal(0)?); 
    }
    #[cfg(feature="cuda")] { 
        return Ok(Device::new_cuda(0)?); 
    }
    Ok(Device::Cpu)
}

// Schema Qdrant optimisé
pub async fn ensure_collections(client: &QdrantClient) -> Result<()> {
    // Collection code_chunks (granularité fine)
    ensure_collection(client, "code_chunks", 384).await?;
    // Collection documents (documents utilisateur)  
    ensure_collection(client, "user_documents", 384).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1) Device detection
    let device = pick_device()?;
    tracing::info!("Using device: {:?}", device);

    // 2) E5 Model (intfloat/e5-small-v2 - 384d robuste)
    let api = Api::new()?;
    let repo = api.repo(Repo::new("intfloat/e5-small-v2", RepoType::Model));
    let _model_path = repo.get("model.safetensors")?;
    // TODO: charger E5 embedder + tokenizer

    // 3) Qdrant setup
    let client = QdrantClient::from_url("http://localhost:6333").build()?;
    ensure_collections(&client).await?;

    // 4) Test embedding + upsert
    let embedding = vec![0.1_f32; 384]; // Remplacer par E5 output
    let payload = json!({
        "type": "code",
        "path": "src/main.rs",
        "language": "rust", 
        "hash": "blake3_hash_here",
        "ts": 1698765432
    });
    
    upsert_embeddings(&client, "code_chunks", vec![1], vec![embedding.clone()], vec![payload]).await?;
    
    let results = search(&client, "code_chunks", embedding, 5, None).await?;
    tracing::info!("Search results: {}", results.len());
    
    Ok(())
}
```

---

**Prochaine étape** : Commencer Phase 1 avec setup E5 embedder + Qdrant collections