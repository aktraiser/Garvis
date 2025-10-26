#!/bin/bash

# GRAVIS RAG Benchmark Suite - Production Validation
# Implémentation du runbook pour validation RAG complète

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
GRAVIS_DATA=${GRAVIS_DATA:-"/tmp/gravis_rag_data"}
RUST_SEED=${RUST_SEED:-42}
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_DIR="benchmark_results"

echo -e "${BLUE}🚀 GRAVIS RAG Benchmark Suite${NC}"
echo -e "${BLUE}Production Validation & Performance Analysis${NC}"
echo "=============================================="
echo -e "Data path: ${CYAN}$GRAVIS_DATA${NC}"
echo -e "Seed: ${CYAN}$RUST_SEED${NC}"
echo -e "Timestamp: ${CYAN}$TIMESTAMP${NC}"
echo ""

# 1. PRÉ-RUN - CONDITIONS PROPRES
echo -e "${YELLOW}🔧 Phase 1: Setup Clean Environment${NC}"

# Créer les répertoires
mkdir -p "$RESULTS_DIR"
mkdir -p "$GRAVIS_DATA/qdrant"

# Nettoyer Qdrant existant
echo "Stopping existing Qdrant containers..."
docker rm -f qdrant 2>/dev/null || true

# Démarrer Qdrant avec persistance
echo -e "${YELLOW}Starting Qdrant with persistent storage...${NC}"
docker run -d --name qdrant \
  -p 6333:6333 -p 6334:6334 \
  -v "$GRAVIS_DATA/qdrant:/qdrant/storage" \
  qdrant/qdrant:latest

# Attendre que Qdrant soit prêt
echo "Waiting for Qdrant to be ready..."
for i in {1..30}; do
    if curl -s http://localhost:6333/health > /dev/null; then
        echo -e "${GREEN}✅ Qdrant is ready!${NC}"
        break
    fi
    sleep 2
    echo -n "."
done

if [ $i -eq 30 ]; then
    echo -e "${RED}❌ Qdrant failed to start${NC}"
    exit 1
fi

# Build du benchmark
echo -e "${YELLOW}Building benchmark tool...${NC}"
cd src-tauri
cargo build --release --bin rag_benchmark

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Environment ready${NC}"
echo ""

# 2. WARM-UP - PETIT RUN POUR CACHES
echo -e "${YELLOW}🔥 Phase 2: Warm-up Run${NC}"
echo "Running small warm-up to populate caches..."

./target/release/rag_benchmark \
    --chunks 100 \
    --queries 10 \
    --ef-search "64" \
    --collection "warmup_test" \
    --seed $RUST_SEED \
    --output "../$RESULTS_DIR/warmup_$TIMESTAMP.json" \
    --qdrant-data "$GRAVIS_DATA/qdrant" || echo "Warm-up completed"

echo -e "${GREEN}✅ Warm-up complete${NC}"
echo ""

# 3. SUITE DE BENCHMARKS COMPLÈTE
echo -e "${YELLOW}📊 Phase 3: Full Benchmark Suite${NC}"

# Fonction pour nettoyer la collection entre les tests
cleanup_collection() {
    local collection=$1
    echo "Cleaning collection: $collection"
    curl -s -X POST "http://localhost:6333/collections/$collection/points/delete" \
         -H "Content-Type: application/json" \
         -d '{"filter":{}}' > /dev/null || true
}

# Tests progressifs avec métriques complètes
SIZES=("small" "medium" "large" "full")
CHUNKS_COUNTS=(1000 10000 100000 100000)
EF_VALUES="32,64,128"

for i in "${!SIZES[@]}"; do
    SIZE=${SIZES[$i]}
    CHUNKS=${CHUNKS_COUNTS[$i]}
    
    echo -e "${CYAN}📈 Running $SIZE benchmark ($CHUNKS chunks)${NC}"
    
    # Nettoyer avant le test
    cleanup_collection "benchmark_test_$SIZE"
    
    # Configuration spécifique par taille
    QUERIES=100
    if [ "$SIZE" = "full" ]; then
        QUERIES=1000
        echo -e "${RED}⚠️  This will take 15-30 minutes and requires significant RAM${NC}"
        read -p "Continue with full benchmark? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "Skipping full benchmark"
            continue
        fi
    fi
    
    # Lancer le benchmark avec toutes les métriques
    echo "Starting $SIZE benchmark..."
    /usr/bin/time -l ./target/release/rag_benchmark \
        --chunks $CHUNKS \
        --queries $QUERIES \
        --ef-search "$EF_VALUES" \
        --collection "benchmark_test_$SIZE" \
        --seed $RUST_SEED \
        --output "../$RESULTS_DIR/benchmark_${SIZE}_$TIMESTAMP.json" \
        --csv "../$RESULTS_DIR/benchmark_${SIZE}_$TIMESTAMP.csv" \
        --qdrant-data "$GRAVIS_DATA/qdrant" \
        2> "../$RESULTS_DIR/benchmark_${SIZE}_${TIMESTAMP}_memory.log"
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ $SIZE benchmark completed${NC}"
        
        # Mesurer l'utilisation disque Qdrant
        DISK_USAGE=$(du -sh "$GRAVIS_DATA/qdrant" | cut -f1)
        echo "Qdrant disk usage: $DISK_USAGE"
        
        # Sauvegarder les métriques système
        echo "disk_usage_qdrant: $DISK_USAGE" >> "../$RESULTS_DIR/benchmark_${SIZE}_${TIMESTAMP}_system.log"
        
    else
        echo -e "${RED}❌ $SIZE benchmark failed${NC}"
    fi
    
    echo ""
done

cd ..

# 4. ANALYSE ET RAPPORT
echo -e "${YELLOW}📋 Phase 4: Analysis & Report Generation${NC}"

# Vérifier si Python est disponible pour l'analyse
if command -v python3 &> /dev/null; then
    echo "Generating analysis report..."
    
    # Installer pandas si nécessaire (optionnel)
    pip3 install pandas > /dev/null 2>&1 || echo "Note: pandas not available, install with: pip3 install pandas"
    
    # Lancer l'analyse
    python3 scripts/benchmark_analysis.py --pattern "$RESULTS_DIR/benchmark_*_$TIMESTAMP.csv" || echo "Analysis script completed"
else
    echo "Python3 not found, skipping automated analysis"
fi

# 5. RÉSUMÉ FINAL
echo -e "${GREEN}🎉 Benchmark Suite Complete!${NC}"
echo "============================================="
echo "Results directory: $RESULTS_DIR"
echo "Files generated:"
ls -la "$RESULTS_DIR/"*"$TIMESTAMP"* 2>/dev/null || echo "No files found"

# Quick summary avec jq si disponible
if command -v jq &> /dev/null; then
    echo ""
    echo -e "${BLUE}📊 Quick Summary:${NC}"
    for result_file in "$RESULTS_DIR"/benchmark_*_"$TIMESTAMP".json; do
        if [ -f "$result_file" ]; then
            SIZE=$(basename "$result_file" | cut -d'_' -f2)
            echo "[$SIZE] Throughput: $(jq -r '.indexing_results.chunks_per_minute // "N/A"' "$result_file") chunks/min, P95: $(jq -r '.search_results.latency_p95_ms // "N/A"' "$result_file")ms"
        fi
    done
fi

echo ""
echo -e "${CYAN}📄 Next steps:${NC}"
echo "1. Review benchmark_analysis_report.md for detailed analysis"
echo "2. Check CSV files for data analysis in your preferred tool"
echo "3. Use results to configure production RAG parameters"

# Nettoyer (optionnel)
read -p "Stop Qdrant container? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    docker stop qdrant
    echo "Qdrant stopped (data preserved in $GRAVIS_DATA)"
fi

echo -e "${GREEN}✨ All done!${NC}"