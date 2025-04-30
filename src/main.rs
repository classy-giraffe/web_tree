//! # Web Tree - Main Entry Point
//!
//! This is the main entry point for the web crawler application.
//! It parses command-line arguments and starts the crawling process.

use std::sync::Arc;
use tokio::sync::Mutex;
use clap::Parser;

use web_tree::cli::Cli;
use web_tree::crawler::WebTree;
use web_tree::BoxError;

/// Main entry point for the web crawler application
///
/// Parses command-line arguments, initializes the crawler,
/// performs the crawl, and exports the result to a GraphViz DOT file.
#[tokio::main]
async fn main() -> Result<(), BoxError> {
    // Parse command line arguments using clap
    let cli = Cli::parse();

    let start_url = cli.url;
    let max_depth = cli.depth;
    let output_file = cli.output;
    let filter_list = cli.filters
        .map(|f| f.split(',').map(|s| s.to_string()).collect())
        .unwrap_or_else(Vec::new);

    println!("Starting WebTree crawler");
    println!("Root URL: {}", start_url);
    println!("Output file: {}", output_file);

    let crawler = Arc::new(Mutex::new(WebTree::new(
        start_url.clone(),
        filter_list,
        max_depth,
    )));
    WebTree::crawl_concurrent(crawler.clone()).await?;
    crawler.lock().await.print_tree();

    // Export the graph to a DOT file
    crawler.lock().await.export_dot(&output_file)?;

    Ok(())
}
