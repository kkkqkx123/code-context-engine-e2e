# examples

展示性输出生成器，运行后可生成 `outputs/` 下的文件供人工审查。

## 原则

- 示例**不参与自动断言**，与 `tests/` 严格分离。
- 修改核心逻辑后，运行对应示例观察 `outputs/` 的输出变化。
- 示例不需要符合测试规范（不需要 `#[tokio::test]`、断言等）。

## 目录结构

按语言/场景划分子目录：

- `rust/` — Rust 语言相关的示例
- `java/` — Java 语言相关的示例
- `multi_language/` — 跨语言场景的示例

## 命名规范

- 文件名应反映 fixture 名称或场景用途（如 `basic.rs`、`export.rs`、`relation.rs`）。
- 示例名与文件名对应：`rust/basic.rs` 注册为 `rust_basic` 示例。
- 注册在 `Cargo.toml` 中通过 `[[example]]` 条目声明。

## 运行

```bash
# 列出所有可用示例
cargo run --example rust_basic
cargo run --example rust_export
cargo run --example rust_relation
cargo run --example java_spring_boot_export
cargo run --example multi_language_basic
```

## 添加新示例步骤

1. 在对应语言子目录下创建 `.rs` 文件
2. 在 `Cargo.toml` 中添加 `[[example]]` 条目
3. 文件顶部用 `//!` 注释说明输出内容
