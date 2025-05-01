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
//! - Comprehensive error handling with tracing

pub mod cli;
pub mod crawler;
pub mod graph_export;
pub mod error;

use std::error::Error;
use tracing_subscriber::{EnvFilter, filter::LevelFilter};

/// A type alias for errors that can be sent between threads
pub type BoxError = Box<dyn Error + Send + Sync>;

/// Initialize the application's error handling and tracing
///
/// This should be called near the beginning of the main function.
/// It sets up:
/// - color-eyre for pretty error reporting
/// - tracing subscribers for structured logging
///
/// # Arguments
///
/// * `log_level` - Optional log level string (trace, debug, info, warn, error)
pub fn init(log_level: Option<&str>) -> color_eyre::Result<()> {
    color_eyre::install()?;
    
    let filter = if let Some(level) = log_level {
        match level.to_lowercase().as_str() {
            "trace" => EnvFilter::default().add_directive(LevelFilter::TRACE.into()),
            "debug" => EnvFilter::default().add_directive(LevelFilter::DEBUG.into()),
            "info" => EnvFilter::default().add_directive(LevelFilter::INFO.into()),
            "warn" => EnvFilter::default().add_directive(LevelFilter::WARN.into()),
            "error" => EnvFilter::default().add_directive(LevelFilter::ERROR.into()),
            _ => {
                EnvFilter::from_default_env()
                    .add_directive(LevelFilter::INFO.into())
            }
        }
    } else {
        EnvFilter::from_default_env()
            .add_directive(LevelFilter::INFO.into())
    };
    
    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(filter)
        .init();
        
    tracing::debug!("Tracing initialized");
    
    Ok(())
}

pub use error::{WebTreeError, Result};