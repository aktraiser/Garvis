#!/usr/bin/env python3
"""
GRAVIS RAG Benchmark Analysis Tool
Analyse les résultats CSV et génère un rapport Markdown
"""

import pandas as pd
import argparse
import glob
from pathlib import Path
import sys

def analyze_benchmarks(csv_pattern="benchmark_results/*.csv"):
    """Analyse les fichiers CSV de benchmark et génère un rapport"""
    
    # Trouver tous les fichiers CSV
    csv_files = glob.glob(csv_pattern)
    if not csv_files:
        print(f"❌ No CSV files found matching pattern: {csv_pattern}")
        return
    
    print(f"📊 Found {len(csv_files)} CSV files to analyze")
    
    # Charger et combiner tous les CSV
    dfs = []
    for csv_file in csv_files:
        try:
            df = pd.read_csv(csv_file)
            df['source_file'] = Path(csv_file).name
            dfs.append(df)
            print(f"✅ Loaded {csv_file}: {len(df)} rows")
        except Exception as e:
            print(f"❌ Error loading {csv_file}: {e}")
    
    if not dfs:
        print("❌ No valid CSV files loaded")
        return
    
    # Combiner tous les DataFrames
    combined_df = pd.concat(dfs, ignore_index=True)
    
    # Analyser les résultats
    generate_report(combined_df)

def generate_report(df):
    """Génère un rapport Markdown avec les analyses"""
    
    report = []
    report.append("# 🎯 GRAVIS RAG Benchmark Analysis Report")
    report.append("")
    report.append("## 📊 Executive Summary")
    report.append("")
    
    # Résumé global
    total_tests = len(df)
    unique_sizes = df['size'].unique()
    unique_ef_values = df['ef_search'].unique()
    
    report.append(f"- **Total tests run**: {total_tests}")
    report.append(f"- **Benchmark sizes**: {', '.join(unique_sizes)}")
    report.append(f"- **ef_search values tested**: {', '.join(map(str, unique_ef_values))}")
    report.append("")
    
    # Go/No-Go Analysis
    report.append("## 🚦 Go/No-Go Analysis")
    report.append("")
    
    # Seuils recommandés
    thresholds = {
        'throughput_min': 1000,  # chunks/min
        'p95_latency_max': 120,  # ms
        'recall_min': 0.85,      # recall@10
        'disk_max': 0.6          # GB pour 100k chunks
    }
    
    # Analyse pour chaque taille
    for size in unique_sizes:
        size_df = df[df['size'] == size]
        report.append(f"### {size.upper()} Benchmark")
        report.append("")
        
        if size == 'full':
            # Critères production pour 100k chunks
            best_row = size_df.loc[size_df['recall_at_10'].idxmax()]
            
            throughput_ok = best_row['throughput_chunks_per_min'] >= thresholds['throughput_min']
            latency_ok = best_row['p95_latency_ms'] <= thresholds['p95_latency_max']
            recall_ok = best_row['recall_at_10'] >= thresholds['recall_min']
            disk_ok = best_row['qdrant_disk_gb'] <= thresholds['disk_max']
            
            status = "✅ GO" if all([throughput_ok, latency_ok, recall_ok, disk_ok]) else "❌ NO-GO"
            
            report.append(f"**Status**: {status}")
            report.append("")
            report.append("| Metric | Value | Threshold | Status |")
            report.append("|--------|-------|-----------|--------|")
            report.append(f"| Throughput | {best_row['throughput_chunks_per_min']:.0f} chunks/min | ≥{thresholds['throughput_min']} | {'✅' if throughput_ok else '❌'} |")
            report.append(f"| P95 Latency | {best_row['p95_latency_ms']:.1f}ms | ≤{thresholds['p95_latency_max']}ms | {'✅' if latency_ok else '❌'} |")
            report.append(f"| Recall@10 | {best_row['recall_at_10']:.3f} | ≥{thresholds['recall_min']} | {'✅' if recall_ok else '❌'} |")
            report.append(f"| Disk Usage | {best_row['qdrant_disk_gb']:.2f}GB | ≤{thresholds['disk_max']}GB | {'✅' if disk_ok else '❌'} |")
            report.append("")
        
        # Meilleure configuration par objectif
        report.append("#### Recommended Configurations")
        report.append("")
        
        best_latency = size_df.loc[size_df['p95_latency_ms'].idxmin()]
        best_recall = size_df.loc[size_df['recall_at_10'].idxmax()]
        best_throughput = size_df.loc[size_df['throughput_chunks_per_min'].idxmax()]
        
        report.append("| Objective | ef_search | P95 Latency | Recall@10 | Throughput |")
        report.append("|-----------|-----------|-------------|-----------|------------|")
        report.append(f"| **Lowest Latency** | {best_latency['ef_search']} | {best_latency['p95_latency_ms']:.1f}ms | {best_latency['recall_at_10']:.3f} | {best_latency['throughput_chunks_per_min']:.0f} |")
        report.append(f"| **Best Recall** | {best_recall['ef_search']} | {best_recall['p95_latency_ms']:.1f}ms | {best_recall['recall_at_10']:.3f} | {best_recall['throughput_chunks_per_min']:.0f} |")
        report.append(f"| **Best Throughput** | {best_throughput['ef_search']} | {best_throughput['p95_latency_ms']:.1f}ms | {best_throughput['recall_at_10']:.3f} | {best_throughput['throughput_chunks_per_min']:.0f} |")
        report.append("")
    
    # Détails complets
    report.append("## 📋 Complete Results")
    report.append("")
    report.append("| Size | Chunks | ef_search | Indexing (min) | Throughput | RAM (GB) | Disk (GB) | P95 Latency (ms) | Recall@10 |")
    report.append("|------|--------|-----------|----------------|------------|----------|-----------|------------------|-----------|")
    
    for _, row in df.iterrows():
        report.append(f"| {row['size']} | {row['chunks_count']:,} | {row['ef_search']} | {row['indexing_time_min']:.1f} | {row['throughput_chunks_per_min']:.0f} | {row['ram_max_gb']:.1f} | {row['qdrant_disk_gb']:.2f} | {row['p95_latency_ms']:.1f} | {row['recall_at_10']:.3f} |")
    
    report.append("")
    report.append("## 🔧 Tuning Recommendations")
    report.append("")
    
    # Recommandations basées sur les résultats
    if 'full' in unique_sizes:
        full_df = df[df['size'] == 'full']
        
        # Analyse du trade-off latence/qualité
        ef_32 = full_df[full_df['ef_search'] == 32].iloc[0] if not full_df[full_df['ef_search'] == 32].empty else None
        ef_128 = full_df[full_df['ef_search'] == 128].iloc[0] if not full_df[full_df['ef_search'] == 128].empty else None
        
        if ef_32 is not None and ef_128 is not None:
            latency_improvement = ((ef_128['p95_latency_ms'] - ef_32['p95_latency_ms']) / ef_32['p95_latency_ms']) * 100
            recall_improvement = ((ef_128['recall_at_10'] - ef_32['recall_at_10']) / ef_32['recall_at_10']) * 100
            
            report.append(f"- **ef_search=32 vs 128**: {latency_improvement:+.1f}% latency, {recall_improvement:+.1f}% recall")
    
    report.append("- **For production**: Use ef_search=64 as baseline, tune based on requirements")
    report.append("- **Low latency**: ef_search=32 (trade-off: -2-5% recall)")
    report.append("- **High quality**: ef_search=128 (trade-off: +20-40% latency)")
    report.append("")
    report.append("---")
    report.append(f"*Report generated by GRAVIS RAG Benchmark Analysis Tool*")
    
    # Sauvegarder le rapport
    report_content = "\n".join(report)
    report_path = "benchmark_analysis_report.md"
    
    with open(report_path, 'w') as f:
        f.write(report_content)
    
    print(f"📄 Report generated: {report_path}")
    
    # Afficher le résumé
    print("\n" + "="*50)
    print("📊 BENCHMARK ANALYSIS SUMMARY")
    print("="*50)
    for size in unique_sizes:
        size_df = df[df['size'] == size]
        best = size_df.loc[size_df['recall_at_10'].idxmax()]
        print(f"{size.upper():>6}: ef={best['ef_search']:>3}, latency={best['p95_latency_ms']:>5.1f}ms, recall={best['recall_at_10']:.3f}")

def main():
    parser = argparse.ArgumentParser(description='Analyze GRAVIS RAG benchmark results')
    parser.add_argument('--pattern', '-p', default='benchmark_results/*.csv',
                       help='Glob pattern for CSV files (default: benchmark_results/*.csv)')
    
    args = parser.parse_args()
    
    try:
        analyze_benchmarks(args.pattern)
    except KeyboardInterrupt:
        print("\n⚠️  Analysis interrupted")
        sys.exit(1)
    except Exception as e:
        print(f"❌ Error during analysis: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()