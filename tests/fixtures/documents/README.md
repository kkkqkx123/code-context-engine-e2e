# Code Context Engine

A codebase indexing engine that converts source code into natural language descriptions and enables semantic search.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
cce-core = "0.1"
```

## Quick Start

```rust
use cce_core::indexer::Indexer;

let indexer = Indexer::new("path/to/project");
indexer.build().await?;
```

## Features

- **AST-based parsing**: Uses tree-sitter for accurate code understanding
- **Natural Language Conversion**: Transforms code entities into readable descriptions
- **Multiple Retrieval Methods**: Supports both embedding-based and BM25 retrieval
- **Cross-language Support**: Works with Rust, Python, Java, TypeScript, and more

## API Reference

### Indexer

The main entry point for indexing operations.

### Retriever

Handles query execution and result ranking.

## License

MIT License
