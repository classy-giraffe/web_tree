//! # Web Crawler
//!
//! This module implements the core web crawling functionality.
//! It provides a concurrent crawler that builds a directed graph
//! of web pages and their links.

use futures::StreamExt;
use futures::stream::FuturesUnordered;
use petgraph::graph::{DiGraph, NodeIndex};
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, error, info, instrument, span, trace, warn, Level};
use url::Url;

use crate::error::{WebTreeError, Result};

/// Represents a node in the web graph
///
/// Each node contains a URL and the depth at which it was discovered
/// during the crawl process.
#[derive(Debug, Clone)]
pub struct LinkNode {
    /// The full URL of the page
    pub url: String,
    /// The depth at which this URL was discovered (0 = starting URL)
    pub depth: usize,
}

/// Main web crawler that builds a graph of linked pages
///
/// WebTree crawls web pages starting from a specified URL and builds
/// a directed graph of the pages and their links. It supports concurrent
/// crawling with configurable depth and domain filtering.
pub struct WebTree {
    /// The starting URL for the crawl
    pub start_url: String,
    /// List of domain filters to limit crawl scope (empty = no filter)
    pub filter_list: Vec<String>,
    /// Maximum depth to crawl from the starting URL
    pub max_depth: usize,
    /// HTTP client for making requests
    client: Client,
    /// Set of normalized URLs that have already been visited
    pub visited: HashSet<String>,
    /// Directed graph representing the web page structure
    pub graph: DiGraph<LinkNode, ()>,
    /// Mapping from URL to its node index in the graph
    pub url_to_node: HashMap<String, NodeIndex>,
}

impl WebTree {
    /// Create a new WebTree crawler instance
    ///
    /// # Arguments
    ///
    /// * `start_url` - The URL to start crawling from
    /// * `filter_list` - List of domain strings to limit crawling to (empty = no filter)
    /// * `max_depth` - Maximum depth to crawl from the starting URL
    ///
    /// # Returns
    ///
    /// A new WebTree instance ready to start crawling
    pub fn new(start_url: String, filter_list: Vec<String>, max_depth: usize) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        debug!("Creating new WebTree crawler");
        trace!(start_url = %start_url, max_depth = %max_depth, filter_count = %filter_list.len(), 
            "WebTree initialization parameters");

        WebTree {
            start_url,
            filter_list,
            max_depth,
            client,
            visited: HashSet::new(),
            graph: DiGraph::new(),
            url_to_node: HashMap::new(),
        }
    }

    /// Determines if a URL should be followed based on filter settings
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to check
    ///
    /// # Returns
    ///
    /// `true` if the URL should be followed, `false` otherwise
    pub fn should_follow(&self, url: &str) -> bool {
        if self.filter_list.is_empty() {
            trace!(url = %url, "URL allowed (no filters active)");
            return true;
        }

        let should_follow = self.filter_list.iter().any(|filter| url.contains(filter));
        if should_follow {
            trace!(url = %url, "URL allowed by filter");
        } else {
            trace!(url = %url, "URL filtered out");
        }
        should_follow
    }

    /// Normalize a URL by removing fragments, default ports, and trailing slashes
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to normalize
    ///
    /// # Returns
    ///
    /// An Option containing the normalized URL string, or None if parsing failed
    pub fn normalize_url(&self, url: &str) -> Result<String> {
        trace!(url = %url, "Normalizing URL");
        
        let mut u = Url::parse(url).map_err(WebTreeError::UrlError)?;
        u.set_fragment(None);
        
        if (u.scheme() == "http" && u.port_or_known_default() == Some(80))
            || (u.scheme() == "https" && u.port_or_known_default() == Some(443))
        {
            let _ = u.set_port(None);
        }
        
        let normalized = u.as_str().trim_end_matches('/').to_string();
        trace!(original = %url, normalized = %normalized, "URL normalized");
        
        Ok(normalized)
    }

    /// Extract and normalize links from HTML content
    ///
    /// # Arguments
    ///
    /// * `html` - The HTML content to parse
    /// * `base_url` - The base URL for resolving relative links
    ///
    /// # Returns
    ///
    /// A vector of normalized URLs found in the HTML
    pub fn extract_links(&self, html: &str, base_url: &Url) -> Vec<String> {
        let span = span!(Level::DEBUG, "extract_links", base_url = %base_url);
        let _enter = span.enter();
        
        let document = Html::parse_document(html);
        let selector = match Selector::parse("a[href]") {
            Ok(s) => s,
            Err(e) => {
                error!(error = %e, "Failed to parse selector");
                return Vec::new();
            }
        };

        let links: Vec<String> = document
            .select(&selector)
            .filter_map(|element| {
                let href = element.value().attr("href")?;
                
                // Resolve the URL relative to the base URL
                let full_url = match base_url.join(href) {
                    Ok(url) => url.to_string(),
                    Err(e) => {
                        debug!(href = %href, error = %e, "Failed to join URL");
                        return None;
                    }
                };
                
                // Normalize the URL
                let normalized = match self.normalize_url(&full_url) {
                    Ok(url) => url,
                    Err(e) => {
                        debug!(url = %full_url, error = %e, "Failed to normalize URL");
                        return None;
                    }
                };
                
                // Check if we should follow this link
                if self.should_follow(&normalized) {
                    Some(normalized)
                } else {
                    None
                }
            })
            .collect();
            
        debug!(link_count = links.len(), "Extracted links from page");
        links
    }

    /// Print a text representation of the crawled tree to the console
    pub fn print_tree(&self) {
        info!("\n--- WebTree Results ---");
        info!(nodes = self.graph.node_count(), edges = self.graph.edge_count(), "Crawl statistics");

        // Find the starting node
        if let Some(&start_idx) = self.url_to_node.get(&self.start_url) {
            self.print_node(start_idx, 0);
        }
    }

    /// Helper method to recursively print a node and its children
    ///
    /// # Arguments
    ///
    /// * `idx` - The node index to print
    /// * `indent` - The indentation level for formatting
    fn print_node(&self, idx: NodeIndex, indent: usize) {
        let node = &self.graph[idx];
        println!(
            "{}{} (depth: {})",
            "  ".repeat(indent),
            node.url,
            node.depth
        );

        for neighbor in self.graph.neighbors(idx) {
            self.print_node(neighbor, indent + 1);
        }
    }

    /// Start a concurrent crawl process
    ///
    /// # Arguments
    ///
    /// * `crawler` - An Arc<Mutex<WebTree>> for shared access across tasks
    ///
    /// # Returns
    ///
    /// Result indicating success or an error that occurred during crawling
    pub async fn crawl_concurrent(crawler: Arc<Mutex<Self>>) -> Result<()> {
        let start_url = {
            let web_tree = crawler.lock().await;
            info!(url = %web_tree.start_url, "Starting concurrent crawl");
            web_tree.start_url.clone()
        };
        
        WebTree::crawl_url_concurrent(crawler.clone(), start_url, 0, None).await?;
        
        let stats = {
            let web_tree = crawler.lock().await;
            (web_tree.graph.node_count(), web_tree.graph.edge_count(), web_tree.visited.len())
        };
        
        info!(nodes = stats.0, edges = stats.1, visited = stats.2, "Crawl completed");
        
        Ok(())
    }

    /// Recursively crawl a URL and its links concurrently
    ///
    /// # Arguments
    ///
    /// * `crawler` - An Arc<Mutex<WebTree>> for shared access across tasks
    /// * `url` - The URL to crawl
    /// * `depth` - The current depth level
    /// * `parent_idx` - Optional parent node index for building the graph
    ///
    /// # Returns
    ///
    /// Result indicating success or an error that occurred during crawling
    #[instrument(skip(crawler, parent_idx), fields(url = %url, depth = depth))]
    async fn crawl_url_concurrent(
        crawler: Arc<Mutex<WebTree>>,
        url: String,
        depth: usize,
        parent_idx: Option<NodeIndex>,
    ) -> Result<()> {
        // 1) URL normalization and graph node creation
        let (normalized, current_idx) = {
            let mut guard = crawler.lock().await;
            
            let norm = match guard.normalize_url(&url) {
                Ok(u) => u,
                Err(e) => {
                    warn!(error = %e, "Skipping invalid URL");
                    return Ok(());
                }
            };
            
            if depth > guard.max_depth {
                debug!("Reached maximum depth, not crawling further");
                return Ok(());
            }
            
            if guard.visited.contains(&norm) {
                trace!("Already visited, skipping");
                return Ok(());
            }
            
            guard.visited.insert(norm.clone());
            debug!(url = %norm, depth = depth, "Crawling page");

            let idx = match guard.url_to_node.get(&norm) {
                Some(&i) => i,
                None => {
                    let i = guard.graph.add_node(LinkNode {
                        url: norm.clone(),
                        depth,
                    });
                    guard.url_to_node.insert(norm.clone(), i);
                    i
                }
            };
            
            if let Some(parent) = parent_idx {
                guard.graph.add_edge(parent, idx, ());
                trace!(from = %guard.graph[parent].url, to = %norm, "Adding edge");
            }
            
            (norm, idx)
        };

        // 2) Fetch & parse outside lock
        let client = { crawler.lock().await.client.clone() };
        
        debug!(url = %normalized, "Sending HTTP request");
        let resp = match client.get(&normalized).send().await {
            Ok(r) if r.status().is_success() => {
                debug!(status = %r.status(), "Received successful response");
                r
            },
            Ok(r) => {
                warn!(status = %r.status(), "Received non-200 response");
                return Err(WebTreeError::CrawlError { 
                    url: normalized, 
                    message: format!("Non-200 status: {}", r.status()) 
                });
            }
            Err(e) => {
                warn!(error = %e, "Request failed");
                return Err(WebTreeError::CrawlError { 
                    url: normalized.clone(), 
                    message: format!("Request error: {}", e) 
                });
            }
        };
        
        let html = match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                warn!(error = %e, "Failed to read response body");
                return Err(WebTreeError::CrawlError { 
                    url: normalized.clone(), 
                    message: format!("Failed to read response body: {}", e) 
                });
            }
        };
        
        let base = match Url::parse(&normalized) {
            Ok(url) => url,
            Err(e) => return Err(WebTreeError::UrlError(e)),
        };

        // 3) Extract links 
        let links = { crawler.lock().await.extract_links(&html, &base) };
        debug!(link_count = links.len(), "Found links on page");

        // 4) Fan-out all child crawls in parallel
        let mut tasks = FuturesUnordered::new();
        for link in links {
            let crawler2 = crawler.clone();
            let idx = current_idx;
            tasks.push(async move {
                WebTree::crawl_url_concurrent(crawler2, link, depth + 1, Some(idx)).await
            });
        }

        // 5) Drive them to completion
        while let Some(child_result) = tasks.next().await {
            if let Err(e) = child_result {
                // We just log the error, as we don't want to stop the entire crawl
                warn!(error = %e, "Error in child crawl");
            }
        }

        Ok(())
    }
}