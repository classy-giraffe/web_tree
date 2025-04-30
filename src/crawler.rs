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
use url::Url;

use crate::BoxError;

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
            return true;
        }

        self.filter_list.iter().any(|filter| url.contains(filter))
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
    pub fn normalize_url(&self, url: &str) -> Option<String> {
        let mut u = Url::parse(url).ok()?;
        u.set_fragment(None);
        if (u.scheme() == "http" && u.port_or_known_default() == Some(80))
            || (u.scheme() == "https" && u.port_or_known_default() == Some(443))
        {
            let _ = u.set_port(None);
        }
        Some(u.as_str().trim_end_matches('/').to_string())
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
        let document = Html::parse_document(html);
        let selector = Selector::parse("a[href]").unwrap();

        document
            .select(&selector)
            .filter_map(|element| {
                let href = element.value().attr("href")?;
                let full = base_url.join(href).ok()?.to_string();
                self.normalize_url(&full)
            })
            .filter(|url| self.should_follow(url))
            .collect()
    }

    /// Print a text representation of the crawled tree to the console
    pub fn print_tree(&self) {
        println!("\n--- WebTree Results ---");
        println!("Total nodes: {}", self.graph.node_count());
        println!("Total edges: {}", self.graph.edge_count());

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
    /// * `crawler` - An `Arc<Mutex<WebTree>>` for shared access across tasks
    ///
    /// # Returns
    ///
    /// Result indicating success or an error that occurred during crawling
    pub async fn crawl_concurrent(crawler: Arc<Mutex<Self>>) -> Result<(), BoxError> {
        let start_url = { crawler.lock().await.start_url.clone() };
        // Kick off recursive concurrent crawl
        WebTree::crawl_url_concurrent(crawler.clone(), start_url, 0, None).await
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
    async fn crawl_url_concurrent(
        crawler: Arc<Mutex<WebTree>>,
        url: String,
        depth: usize,
        parent_idx: Option<NodeIndex>,
    ) -> Result<(), BoxError> {
        let (normalized, current_idx) = {
            let mut guard = crawler.lock().await;
            let norm = match guard.normalize_url(&url) {
                Some(u) => u,
                None => return Ok(()),
            };
            if depth > guard.max_depth || guard.visited.contains(&norm) {
                return Ok(());
            }
            guard.visited.insert(norm.clone());
            println!("Crawling: {} (depth: {})", norm, depth);

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
            }
            (norm, idx)
        };

        let client =  crawler.lock().await.client.clone();

        let resp = match client.get(&normalized).send().await {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => {
                eprintln!("Non-200 from {}: {}", normalized, r.status());
                return Ok(());
            }
            Err(e) => {
                eprintln!("Error fetching {}: {}", normalized, e);
                return Ok(());
            }
        };

        let html = match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Error reading body {}: {}", normalized, e);
                return Ok(());
            }
        };

        let base = Url::parse(&normalized)?;

        let links = crawler.lock().await.extract_links(&html, &base);
        
        let mut tasks = FuturesUnordered::new();
        for link in links {
            let crawler2 = crawler.clone();
            let idx = current_idx;
            tasks.push(async move {
                WebTree::crawl_url_concurrent(crawler2, link, depth + 1, Some(idx)).await
            });
        }

        while let Some(child_result) = tasks.next().await {
            child_result?;
        }

        Ok(())
    }
}