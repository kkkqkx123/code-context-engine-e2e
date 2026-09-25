# File Fold Tool Fixture

This fixture group demonstrates the stateless file fold export.

## Language samples

- Rust skeleton extraction from `src/main.rs`
- Rust section dropping from `src/large.rs`
- Python class and function skeleton from `src/service.py`
- TypeScript interface and class skeleton from `src/handler.ts`
- Go struct and method skeleton from `src/server.go`
- Java class and interface skeleton from `src/Order.java`

## Degradation samples

- Markdown files do not have a tree-sitter AST language variant, so fold output
  keeps the source text and marks `structure_known=false`.
- Unknown suffixes are not parsed and also degrade to truncated text.

## Review goal

The generated files under `outputs/tools/file-fold` should make three effects
visible:

1. Structured code files produce compact symbol skeletons.
2. Minimal mode is useful for structured files; it is skipped for degraded
   files because mode does not change fallback text.
3. A tight token budget drops lower-priority sections, truncates degraded text,
   or reports `structure_known=false` when the skeleton cannot fit.
