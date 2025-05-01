use thiserror::Error;
use url::ParseError;
use std::io;

/// WebTree custom error types
#[derive(Error, Debug)]
pub enum WebTreeError {
    /// Error when HTTP request fails
    #[error("HTTP request error: {0}")]
    RequestError(#[from] reqwest::Error),
    
    /// Error when URL parsing fails
    #[error("URL parsing error: {0}")]
    UrlError(#[from] ParseError),
    
    /// Error when file operations fail
    #[error("File I/O error: {0}")]
    IoError(#[from] io::Error),
    
    /// Crawl errors with context
    #[error("Crawl error for {url}: {message}")]
    CrawlError {
        url: String,
        message: String,
    },
    
    /// General error with context
    #[error("{0}")]
    General(String),
    
    /// Error from color-eyre
    #[error("Internal error: {0}")]
    EyreError(#[from] color_eyre::Report),
}

/// Result type alias with WebTreeError as error type
pub type Result<T> = std::result::Result<T, WebTreeError>;