# 🎯 GRAVIS RAG Benchmark Suite

Outil de validation production pour l'architecture RAG GRAVIS, implémentant le runbook complet avec métriques et analyse automatisée.

## 🚀 Quick Start

```bash
# Lancement rapide de la suite complète
./run_benchmark_suite.sh

# Test manuel avec options avancées
cd src-tauri
./target/release/rag_benchmark \
    --chunks 10000 \
    --queries 100 \
    --ef-search "32,64,128" \
    --csv "results.csv" \
    --seed 42 \
    --qdrant-data "/path/to/qdrant"
```

## 📊 Fonctionnalités

### ✅ Métriques Production

- **Indexation**: Débit (chunks/min), temps total, RAM max
- **Recherche**: p95 latence @ top-10, recall@10 avec qrels
- **Stockage**: Taille disque Qdrant, compression
- **Tuning**: ef_search ∈ {32, 64, 128} avec trade-offs

### ✅ QRels & Recall@10

- Génération automatique de qrels (même document = relevant)
- Calcul précis du recall@10 pour validation qualité
- 10 chunks par document synthétique pour tests réalistes

### ✅ Export & Analyse

- **CSV automatique** : Compatible Excel/Google Sheets
- **Rapport Markdown** : Analyse Go/No-Go avec seuils production
- **Recommandations** : Configurations optimales par objectif

## 🛠️ Configuration

### Seuils Production (100k chunks)

| Métrique | Seuil | Description |
|----------|-------|-------------|
| Throughput | ≥1000 chunks/min | Indexation acceptable |
| P95 Latency | ≤120ms | Recherche interactive |
| Recall@10 | ≥0.85 | Qualité minimale |
| Disk Usage | ≤0.6GB | Stockage raisonnable |

### Tailles de Test

- **small** (1k chunks): Développement rapide
- **medium** (10k chunks): Validation intermédiaire  
- **large** (100k chunks): Test de charge
- **full** (100k chunks): Validation production complète

## 📋 Structure des Fichiers

```
benchmark_results/
├── benchmark_small_TIMESTAMP.json     # Résultats détaillés
├── benchmark_small_TIMESTAMP.csv      # Export pour analyse
├── benchmark_small_TIMESTAMP_memory.log # Métriques système
└── benchmark_analysis_report.md       # Rapport final
```

## 🔧 Options CLI

```bash
Usage: rag_benchmark [OPTIONS]

Options:
  -c, --chunks <COUNT>      Nombre de chunks (défaut: 1000)
  -q, --queries <COUNT>     Nombre de requêtes (défaut: 100)
      --ef-search <VALUES>  Valeurs ef_search séparées par virgules
  -o, --output <FILE>       Fichier JSON de sortie
      --csv <FILE>          Export CSV pour analyse
      --seed <NUMBER>       Seed aléatoire (défaut: 42)
      --qdrant-data <PATH>  Chemin des données Qdrant
      --collection <NAME>   Nom de la collection Qdrant
      --full                Benchmark complet 100k chunks
```

## 📈 Analyse des Résultats

### 1. Rapport Automatique

```bash
python3 scripts/benchmark_analysis.py --pattern "benchmark_results/*.csv"
```

Génère `benchmark_analysis_report.md` avec :
- Analyse Go/No-Go par seuils production
- Configurations recommandées par objectif
- Trade-offs latence/qualité/throughput

### 2. Métriques Clés

**Indexation**
- `throughput_chunks_per_min`: Performance d'ingestion
- `indexing_time_min`: Temps total d'indexation
- `ram_max_gb`: Pic mémoire observé

**Recherche**  
- `p95_latency_ms`: Latence 95e percentile
- `recall_at_10`: Qualité des résultats
- `ef_search`: Paramètre de tuning testé

**Stockage**
- `qdrant_disk_gb`: Taille sur disque
- `chunks_count`: Nombre de chunks indexés

## 🎯 Interprétation

### Go/No-Go Production

✅ **GO** si tous les seuils sont respectés :
- Throughput ≥ 1000 chunks/min  
- P95 latency ≤ 120ms
- Recall@10 ≥ 0.85
- Disk usage ≤ 0.6GB (100k chunks)

### Configurations Recommandées

- **Latence optimale**: `ef_search=32` (-2-5% recall)
- **Équilibre**: `ef_search=64` (baseline production)
- **Qualité maximale**: `ef_search=128` (+20-40% latence)

## 🔍 Dépannage

### Erreurs Communes

1. **Qdrant non démarré**
   ```bash
   docker run -d --name qdrant -p 6333:6333 qdrant/qdrant:latest
   ```

2. **Mémoire insuffisante** (benchmark full)
   - RAM recommandée : 8GB+ 
   - Réduire `--chunks` si nécessaire

3. **Timeout de compilation**
   ```bash
   cargo build --release --bin rag_benchmark
   ```

### Logs Détaillés

```bash
RUST_LOG=info ./target/release/rag_benchmark [options]
```

## 🏗️ Architecture

Le benchmark teste l'architecture RAG complète :
- **E5-Small-v2** (384D, tout-Rust)
- **Qdrant** (vector DB optimisé)
- **Candle** (ML inference)
- **DevicePool** (gestion mémoire)
- **EmbeddingBatcher** (traitement par lots)

---

*Pour plus de détails, voir `GRAVIS_RAG_DOCUMENTATION.md`*