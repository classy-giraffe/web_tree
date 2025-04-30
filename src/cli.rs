//! # Command Line Interface
//!
//! This module provides the command-line interface for the web crawler
//! using the `clap` crate for argument parsing.

use clap::Parser;

/// WebTree - A web crawler that visualizes website structure
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Starting URL to crawl
    #[arg(short, long, default_value = "https://google.com/")]
    pub url: String,

    /// Maximum crawl depth
    #[arg(short, long, default_value_t = 15)]
    pub depth: usize,

    /// Output file path for GraphViz DOT file
    #[arg(short, long, default_value = "webtree.dot")]
    pub output: String,

    /// Filters to limit crawling to specific domains (comma separated)
    #[arg(short, long)]
    pub filters: Option<String>,
}