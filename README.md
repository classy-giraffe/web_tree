# WebTree

A concurrent web crawler that visualizes website structures using GraphViz.

## Overview

WebTree is a command-line tool that crawls websites starting from a specified URL, builds a directed graph of the site structure, and exports the result to a GraphViz DOT file for visualization. It's built in Rust with a focus on concurrent performance and configurability.

## Features

- **Concurrent Crawling**: Uses asynchronous Rust with Tokio for efficient parallel web crawling
- **Configurable Depth**: Limit how deep the crawler traverses from the starting URL
- **Domain Filtering**: Restrict crawling to specific domains or subdomains
- **GraphViz Export**: Generate DOT files with properly labeled nodes for visualization
- **User-Friendly CLI**: Simple command-line interface with reasonable defaults
- **Robust Error Handling**: Comprehensive error handling with custom error types
- **Detailed Logging**: Structured logging with tracing for better diagnostics

## Installation

### Prerequisites

- Rust and Cargo (2021 edition or later)
- GraphViz (optional, for visualizing the output)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/classy-giraffe/web_tree.git
cd web_tree

# Build the project
cargo build --release

# The binary will be available in target/release/web_tree
```

## Usage

Basic usage with default settings (starting URL: https://google.com, depth: 15):

```bash
web_tree
```

Specify a custom URL to crawl:

```bash
web_tree --url https://example.com
```

Set the log level (trace, debug, info, warn, error):

```bash
web_tree debug --url https://example.com
```

Configure crawl depth and output file:

```bash
web_tree info --url https://example.com --depth 3 --output site-map.dot
```

Filter crawling to specific domains:

```bash
web_tree --url https://example.com --filters example.com,api.example.com
```

Combine all options:

```bash
web_tree debug --url https://example.com --depth 3 --output site-map.dot --filters example.com,api.example.com
```

### Command-line Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `LOG_LEVEL` | - | Optional log level (trace, debug, info, warn, error) | info |
| `--url` | `-u` | Starting URL to crawl | https://google.com/ |
| `--depth` | `-d` | Maximum crawl depth | 15 |
| `--output` | `-o` | Output file path for GraphViz DOT file | webtree.dot |
| `--filters` | `-f` | Comma-separated list of domain filters | (none) |

## Visualizing the Output

After running the crawler, you can visualize the generated DOT file using GraphViz:

```bash
# Generate a PNG image
dot -Tpng webtree.dot -o webtree.png

# Generate an SVG
dot -Tsvg webtree.dot -o webtree.svg

# For larger graphs, neato may produce better layouts
neato -Tpng webtree.dot -o webtree.png
```

## Project Structure

- `src/main.rs`: Entry point that handles command-line arguments
- `src/cli.rs`: Command-line interface using clap
- `src/crawler.rs`: Core web crawling logic
- `src/graph_export.rs`: GraphViz DOT export functionality
- `src/error.rs`: Custom error types and error handling
- `src/lib.rs`: Module exports, common definitions, and initialization

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request