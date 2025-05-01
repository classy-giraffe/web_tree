//! # Web Tree - Main Entry Point
//!
//! This is the main entry point for the web crawler application.
//! It parses command-line arguments and starts the crawling process.

use std::sync::Arc;
use tokio::sync::Mutex;
use clap::Parser;
use tracing::{info, debug};

use web_tree::cli::Cli;
use web_tree::crawler::WebTree;
use web_tree::Result;

/// Main entry point for the web crawler application
///
/// Parses command-line arguments, initializes the crawler,
/// performs the crawl, and exports the result to a GraphViz DOT file.
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    web_tree::init(cli.log_level.as_deref())?;
    
    let start_url = cli.url;
    let max_depth = cli.depth;
    let output_file = cli.output;
    let filter_list = cli.filters
        .map(|f| f.split(',').map(|s| s.to_string()).collect())
        .unwrap_or_else(Vec::new);

    info!("Starting WebTree crawler");
    info!(url = %start_url, depth = max_depth, "Crawler configuration");
    debug!(filters = ?filter_list, output = %output_file, "Additional settings");

    let crawler = Arc::new(Mutex::new(WebTree::new(
        start_url.clone(),
        filter_list,
        max_depth,
    )));
    
    WebTree::crawl_concurrent(crawler.clone()).await?;
    
    crawler.lock().await.print_tree();
    crawler.lock().await.export_dot(&output_file)?;

    info!("Crawl completed successfully");
    Ok(())
}
