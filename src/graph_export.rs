//! # GraphViz Export
//!
//! This module provides functionality to export the web crawler's graph
//! to GraphViz DOT format for visualization.

use std::fs::File;
use std::io::Write;
use tracing::{debug, info, instrument, trace, warn};
use url::Url;

use crate::crawler::WebTree;
use crate::error::{WebTreeError, Result};

impl WebTree {
    /// Export the graph to a DOT file for GraphViz visualization
    ///
    /// This function creates a DOT file representation of the crawled web structure
    /// that can be visualized using GraphViz tools like dot, neato, or online viewers.
    /// Each node is labeled with its domain name and path for easy identification.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path where the DOT file will be saved
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error that occurred during file creation
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use web_tree::crawler::WebTree;
    /// # let web_tree = WebTree::new("https://example.com".to_string(), vec![], 2);
    /// web_tree.export_dot("website_graph.dot").unwrap();
    /// ```
    #[instrument(name = "export_dot", skip(self), fields(nodes = self.graph.node_count(), edges = self.graph.edge_count()))]
    pub fn export_dot(&self, file_path: &str) -> Result<()> {
        info!(output_file = %file_path, "Exporting graph to DOT format");
        
        // Create a DOT file manually with proper labels
        let mut dot_contents = String::from("digraph {\n");
        debug!("Building DOT file contents");
        
        // Add nodes with proper labels
        for (url, idx) in &self.url_to_node {
            trace!(url = %url, node_id = idx.index(), "Adding node to DOT output");
            
            // Extract domain and path for readable labels
            let parsed_url = match Url::parse(url) {
                Ok(u) => u,
                Err(e) => {
                    warn!(error = %e, url = %url, "Failed to parse URL for label - using URL as is");
                    // Create a basic label for unparsable URLs
                    dot_contents.push_str(&format!("    {} [label=\"{}\"]\n", idx.index(), url));
                    continue;
                }
            };
            
            let domain = parsed_url.host_str().unwrap_or("unknown");
            let path = parsed_url.path();
            let path_str = if path.is_empty() || path == "/" { "" } else { path };
            
            // Create a concise but informative label
            let label = if path_str.is_empty() {
                domain.to_string()
            } else {
                format!("{}:{}", domain, path_str)
            };
            
            // Escape quotes in the label
            let escaped_label = label.replace("\"", "\\\"");
            
            // Add the node definition with label
            dot_contents.push_str(&format!("    {} [label=\"{}\"]\n", idx.index(), escaped_label));
        }
        
        // Add edges
        trace!("Adding edges to DOT output");
        for edge in self.graph.edge_indices() {
            let (source, target) = match self.graph.edge_endpoints(edge) {
                Some((s, t)) => (s, t),
                None => {
                    warn!(edge_id = ?edge, "Could not find endpoints for edge - skipping");
                    continue;
                }
            };
            
            dot_contents.push_str(&format!("    {} -> {}\n", source.index(), target.index()));
        }
        
        dot_contents.push_str("}\n");
        
        // Write the DOT file
        debug!(file_path = %file_path, size = dot_contents.len(), "Writing DOT file");
        let mut file = File::create(file_path).map_err(WebTreeError::IoError)?;
        file.write_all(dot_contents.as_bytes()).map_err(WebTreeError::IoError)?;
        
        info!(file_path = %file_path, "Graph exported successfully to DOT format");
        Ok(())
    }
}