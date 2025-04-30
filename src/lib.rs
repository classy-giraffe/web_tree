//! # Web Tree
//! 
//! `web_tree` is a web crawler that builds and visualizes a website's link structure.
//! It crawls web pages starting from a specified URL, builds a graph of links,
//! and exports the structure to GraphViz DOT format for visualization.
//! 
//! ## Features
//! 
//! - Concurrent web crawling with configurable depth
//! - Domain filtering to limit crawl scope
//! - GraphViz DOT export for visualization
//! - Command-line interface with customizable options

pub mod cli;
pub mod crawler;
pub mod graph_export;

use std::error::Error;

/// A type alias for errors that can be sent between threads
pub type BoxError = Box<dyn Error + Send + Sync>;