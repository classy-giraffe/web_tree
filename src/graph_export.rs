//! # GraphViz Export
//!
//! This module provides functionality to export the web crawler's graph
//! to GraphViz DOT format for visualization.

use std::fs::File;
use std::io::Write;
use url::Url;

use crate::BoxError;
use crate::crawler::WebTree;

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
    pub fn export_dot(&self, file_path: &str) -> Result<(), BoxError> {
        // Create a DOT file manually with proper labels
        let mut dot_contents = String::from("digraph {\n");
        
        // Add nodes with proper labels
        for (url, idx) in &self.url_to_node {
            // Extract domain and path for readable labels
            let parsed_url = Url::parse(url).ok();
            let domain = parsed_url
                .as_ref()
                .and_then(|u| u.host_str())
                .unwrap_or("unknown");
                
            let path = parsed_url
                .as_ref()
                .map(|u| {
                    let path = u.path();
                    if path.is_empty() || path == "/" {
                        ""
                    } else {
                        path
                    }
                })
                .unwrap_or("");
                
            // Create a concise but informative label
            let label = if path.is_empty() {
                domain.to_string()
            } else {
                format!("{}:{}", domain, path)
            };
            
            // Escape quotes in the label
            let escaped_label = label.replace("\"", "\\\"");
            
            // Add the node definition with label
            dot_contents.push_str(&format!("    {} [label=\"{}\"]\n", idx.index(), escaped_label));
        }
        
        // Add edges
        for edge in self.graph.edge_indices() {
            let (source, target) = self.graph.edge_endpoints(edge).unwrap();
            dot_contents.push_str(&format!("    {} -> {}\n", source.index(), target.index()));
        }
        
        dot_contents.push_str("}\n");
        
        // Write the DOT file
        let mut file = File::create(file_path)?;
        file.write_all(dot_contents.as_bytes())?;
        println!("Graph exported to {} in DOT format with labeled nodes", file_path);
        Ok(())
    }
}