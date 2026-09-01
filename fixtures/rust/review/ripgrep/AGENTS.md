# ripgrep Project Context

## Project Overview

**ripgrep** (`rg`) is a high-performance line-oriented search tool that recursively searches directories for regex patterns.

**Key Features:**

- Recursive directory search
- automatic gitignore filtering
- Full Unicode support with consistent performance
- Optional PCRE2 support for lookaround and backreferences
- Multiple output formats (standard, JSON, summary)
- Configuration file support
- Compressed file search (gzip, bzip2, lzma, etc.)
- Multi-line search capability

## Project Structure

### Current Architecture (Multi-Crate Workspace)

```
ripgrep/
├── Cargo.toml              # Workspace root (Rust 1.85+, Edition 2024)
├── build.rs                # Build script (git hash embedding, Windows manifest)
├── crates/
│   ├── core/               # Main entry point and CLI coordination
│   ├── grep/               # Facade crate aggregating other crates
│   ├── matcher/            # Matcher trait definitions
│   ├── regex/              # Regex matcher implementation (rust regex)
│   ├── pcre2/              # PCRE2 matcher implementation (optional)
│   ├── searcher/           # Search execution engine
│   ├── printer/            # Output formatting (standard, JSON, summary)
│   ├── ignore/             # Directory traversal and ignore rules
│   ├── globset/            # Glob pattern matching
│   └── cli/                # CLI utility functions
├── docs/
│   ├── core-code-analysis.md     # Core code architecture analysis
│   └── merge-to-single-crate.md  # Single-crate refactoring plan
├── scripts/                # Build and release scripts
└── tests/                  # Integration tests
```

### Target Architecture (Single Crate - Planned)

Per `docs/merge-to-single-crate.md`, the project is planned to be refactored into a single crate with modular structure:

```
src/
├── lib.rs                  # Library entry point
├── main.rs                 # CLI binary entry
├── cli/                    # CLI utilities
├── core/                   # High-level coordination
├── globset/                # Glob patterns
├── ignore/                 # File traversal
├── matcher/                # Matcher traits
├── search/                 # Unified search module
│   ├── engine.rs           # Search execution
│   ├── matcher/            # Matcher implementations
│   ├── regex.rs            # Rust regex
│   ├── pcre2.rs            # PCRE2 (feature-gated)
│   └── printer.rs          # Output formatting
└── util/                   # Utilities
```

## Building and Running

### Prerequisites

- **Rust**: 1.85.0 or newer (stable)
- **Optional**: PCRE2 library (for `--pcre2` feature)

### Running

```bash
# Run from source
cargo run -- <pattern> [path]

# Common usage examples
rg "pattern"                    # Search current directory
rg -w "pattern"                 # Word-boundary match
rg -t py "pattern"              # Search only Python files
rg -T js "pattern"              # Exclude JavaScript files
rg -n "pattern"                 # Show line numbers
rg -C 3 "pattern"               # Show 3 context lines
rg -P "pattern"                 # Use PCRE2 engine
rg --json "pattern"             # JSON output
```

### Testing

```bash
# Run all tests
cargo test --all

# Run specific test suite
cargo test --test integration

# Run with output
cargo test --all -- --nocapture
```

## Development Conventions

### Code Style

- **Edition**: Rust 2024
- **MSRV**: 1.85.0 (tracks latest stable Rust)
- **License**: Dual-licensed under MIT or UNLICENSE
- **Formatting**: Standard `rustfmt` conventions

### Module Organization

Each crate/module follows a consistent pattern:

- `lib.rs` - Module exports and documentation
- `mod.rs` - Submodule organization
- Clear separation between traits (interfaces) and implementations

### Error Handling

- Uses `anyhow` for application-level error handling
- Custom error types for library crates
- Errors should be descriptive and actionable

### Testing Practices

- Unit tests alongside source code (`#[cfg(test)]`)
- Integration tests in `tests/` directory
- Tests should be fast and deterministic
- Mock file systems for edge cases

### Performance Considerations

- SIMD optimizations via `regex-automata`
- Memory-mapped file I/O for large files
- Parallel directory traversal via `crossbeam`
- Lazy evaluation where possible

### Key Dependencies

| Dependency       | Purpose                   |
| ---------------- | ------------------------- |
| `regex-automata` | Core regex engine         |
| `aho-corasick`   | Literal optimizations     |
| `bstr`           | Byte string handling      |
| `crossbeam`      | Parallel iteration        |
| `encoding_rs`    | Text encoding support     |
| `termcolor`      | Colored output            |
| `lexopt`         | Argument parsing          |
| `pcre2-sys`      | PCRE2 bindings (optional) |

### Feature Flags

- `pcre2` - Enable PCRE2 regex engine support
- `bin` - Build CLI binary (default)
- `serde` - Enable serde serialization

## Architecture Notes

### Core Data Flow

```
CLI Args → flags/ → HiArgs
                    ↓
WalkBuilder → ignore/ → File Iteration
                    ↓
Searcher + Matcher → searcher/ → Search Execution
                    ↓
Printer (Sink) → printer/ → Formatted Output
```

### Key Traits

```rust
// matcher/
pub trait Matcher {
    fn find_at(&self, haystack: &[u8], at: usize) -> Result<Option<Match>, Error>;
    fn find_iter<'a, 'b>(&'a self, haystack: &'b [u8]) -> MatchesIterator<'a, 'b>;
}

// searcher/
pub trait Sink {
    fn matched(&mut self, searcher: &Searcher, mat: &Match) -> Result<bool, Error>;
}
```

### Trait Implementations

```
Matcher
├── RegexMatcher (regex/)      # Standard Rust regex
└── RegexMatcher (pcre2/)      # PCRE2 engine (optional)

Sink
├── StandardSink (printer/)    # grep-like output
├── JSONSink (printer/)        # JSON Lines format
└── SummarySink (printer/)     # Aggregate statistics
```
