#!/bin/bash

# GRAVIS RAG Benchmark Script
# Usage: ./run_benchmark.sh [small|medium|large|full]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 GRAVIS RAG Benchmark Tool${NC}"
echo "=============================="

# Check if Qdrant is running
echo -e "${YELLOW}Checking Qdrant status...${NC}"
if ! curl -s http://localhost:6333/health > /dev/null; then
    echo -e "${RED}❌ Qdrant is not running!${NC}"
    echo -e "${YELLOW}Starting Qdrant with Docker...${NC}"
    docker-compose up -d qdrant
    
    # Wait for Qdrant to be ready
    echo "Waiting for Qdrant to start..."
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
else
    echo -e "${GREEN}✅ Qdrant is running${NC}"
fi

# Build the benchmark binary
echo -e "${YELLOW}Building benchmark tool...${NC}"
cd src-tauri
cargo build --release --bin rag_benchmark

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Build successful${NC}"

# Determine benchmark size
SIZE=${1:-small}
case $SIZE in
    "small")
        CHUNKS=100
        QUERIES=20
        echo -e "${BLUE}📊 Running SMALL benchmark (100 chunks, 20 queries)${NC}"
        ;;
    "medium")
        CHUNKS=1000
        QUERIES=100
        echo -e "${BLUE}📊 Running MEDIUM benchmark (1k chunks, 100 queries)${NC}"
        ;;
    "large")
        CHUNKS=10000
        QUERIES=500
        echo -e "${BLUE}📊 Running LARGE benchmark (10k chunks, 500 queries)${NC}"
        echo -e "${YELLOW}⚠️  This will take 5-10 minutes${NC}"
        ;;
    "full")
        CHUNKS=100000
        QUERIES=1000
        echo -e "${BLUE}📊 Running FULL benchmark (100k chunks, 1k queries)${NC}"
        echo -e "${RED}⚠️  This will take 15-30 minutes and requires significant RAM${NC}"
        read -p "Continue? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "Benchmark cancelled"
            exit 0
        fi
        ;;
    *)
        echo -e "${RED}❌ Invalid size: $SIZE${NC}"
        echo "Usage: $0 [small|medium|large|full]"
        exit 1
        ;;
esac

# Create output directory
mkdir -p ../benchmark_results
OUTPUT_FILE="../benchmark_results/benchmark_${SIZE}_$(date +%Y%m%d_%H%M%S).json"

echo -e "${YELLOW}📝 Results will be saved to: $OUTPUT_FILE${NC}"

# Run the benchmark
echo -e "${GREEN}🚀 Starting benchmark...${NC}"
./target/release/rag_benchmark \
    --chunks $CHUNKS \
    --queries $QUERIES \
    --ef-search "32,64,128" \
    --output "$OUTPUT_FILE" \
    --collection "benchmark_test_$SIZE"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Benchmark completed successfully!${NC}"
    echo -e "${BLUE}📊 Results saved to: $OUTPUT_FILE${NC}"
    
    # Show quick summary if jq is available
    if command -v jq &> /dev/null; then
        echo -e "${YELLOW}📋 Quick Summary:${NC}"
        echo "Indexing throughput: $(jq -r '.indexing_results.chunks_per_minute' "$OUTPUT_FILE") chunks/min"
        echo "Search p95 latency: $(jq -r '.search_results.latency_p95_ms' "$OUTPUT_FILE")ms"
        echo "Failed chunks: $(jq -r '.indexing_results.failed_chunks' "$OUTPUT_FILE")"
    fi
else
    echo -e "${RED}❌ Benchmark failed${NC}"
    exit 1
fi

echo -e "${GREEN}🎉 All done!${NC}"