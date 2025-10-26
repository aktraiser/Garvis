// GRAVIS RAG Benchmark CLI Tool
// Outil de test de performance isolé

use anyhow::Result;
use clap::{Arg, Command};
use gravis_app_lib::rag::{BenchmarkConfig, run_benchmark_cli};
use std::env;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup du logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    // CLI avec clap
    let matches = Command::new("GRAVIS RAG Benchmark")
        .version("1.0.0")
        .about("Performance testing tool for GRAVIS RAG system")
        .arg(
            Arg::new("chunks")
                .long("chunks")
                .short('c')
                .value_name("COUNT")
                .help("Number of chunks to test (default: 1000)")
                .default_value("1000")
        )
        .arg(
            Arg::new("queries")
                .long("queries")
                .short('q')
                .value_name("COUNT")
                .help("Number of search queries to test (default: 100)")
                .default_value("100")
        )
        .arg(
            Arg::new("ef-search")
                .long("ef-search")
                .value_name("VALUES")
                .help("Comma-separated ef_search values to test (default: 32,64,128)")
                .default_value("32,64,128")
        )
        .arg(
            Arg::new("output")
                .long("output")
                .short('o')
                .value_name("FILE")
                .help("Output file for results (default: benchmark_results.json)")
                .default_value("benchmark_results.json")
        )
        .arg(
            Arg::new("collection")
                .long("collection")
                .value_name("NAME")
                .help("Qdrant collection name (default: benchmark_test)")
                .default_value("benchmark_test")
        )
        .arg(
            Arg::new("full")
                .long("full")
                .help("Run full 100k chunks benchmark (production test)")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("csv")
                .long("csv")
                .value_name("FILE")
                .help("Export results to CSV file for analysis")
        )
        .arg(
            Arg::new("seed")
                .long("seed")
                .value_name("NUMBER")
                .help("Random seed for reproducibility (default: 42)")
                .default_value("42")
        )
        .arg(
            Arg::new("qdrant-data")
                .long("qdrant-data")
                .value_name("PATH")
                .help("Qdrant data directory path for disk usage measurement")
        )
        .get_matches();

    // Parse des arguments
    let chunks_count = if matches.get_flag("full") {
        100_000
    } else {
        matches.get_one::<String>("chunks").unwrap().parse::<usize>()?
    };

    let queries_count = matches.get_one::<String>("queries").unwrap().parse::<usize>()?;
    let output_path = matches.get_one::<String>("output").unwrap().clone();
    let collection = matches.get_one::<String>("collection").unwrap().clone();
    
    // Nouveaux paramètres
    let csv_output = matches.get_one::<String>("csv").cloned();
    let random_seed = Some(matches.get_one::<String>("seed").unwrap().parse::<u64>()?);
    let qdrant_data_path = matches.get_one::<String>("qdrant-data").cloned();

    let ef_search_values: Vec<u64> = matches
        .get_one::<String>("ef-search").unwrap()
        .split(',')
        .map(|s| s.trim().parse::<u64>())
        .collect::<Result<Vec<_>, _>>()?;

    // Configuration du benchmark
    let config = BenchmarkConfig {
        chunks_count,
        search_queries: queries_count,
        ef_search_values,
        collections: vec![collection],
        output_path,
        csv_output,
        random_seed,
        qdrant_data_path,
        ..Default::default()
    };

    info!("🚀 Starting GRAVIS RAG Benchmark");
    info!("Configuration:");
    info!("  • Chunks: {}", config.chunks_count);
    info!("  • Queries: {}", config.search_queries);
    info!("  • ef_search values: {:?}", config.ef_search_values);
    info!("  • Output: {}", config.output_path);

    if config.chunks_count >= 10_000 {
        info!("⚠️  Large benchmark detected. This may take 15-30 minutes.");
        info!("   Make sure Qdrant is running: docker-compose up -d");
    }

    // Lancement du benchmark
    run_benchmark_cli(vec!["rag_benchmark".to_string()]).await?;

    Ok(())
}