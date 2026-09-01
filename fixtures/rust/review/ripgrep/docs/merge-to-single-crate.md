# ripgrep 单包改造方案

本文档描述如何将 ripgrep 从多 crate 工作区合并为单一 crate，**专注于提供简洁的 Rust API**，同时可选保留 CLI 功能。

## 当前结构 vs 目标结构

### 当前（多 crate）
```
crates/
├── cli/           # CLI 工具函数（颜色、缓冲、解压缩等）
├── core/          # CLI 入口和参数处理（约 3000+ 行 CLI 逻辑）
├── globset/       # Glob 模式匹配
├── grep/          # facade（重新导出）
├── ignore/        # 文件遍历和 ignore 规则
├── matcher/       # Matcher trait 定义
├── pcre2/         # PCRE2 正则引擎实现
├── printer/       # 输出格式化（标准/JSON/Summary/超链接）
├── regex/         # Rust 正则引擎实现
└── searcher/      # 搜索执行引擎
```

### 目标（单 crate + 模块，专注于 API）
```
src/
├── lib.rs              # 库入口：重新导出核心 API
├── main.rs             # CLI 入口（可选，feature = "bin"）
├── matcher/            # 保留：Matcher trait 核心抽象
│   ├── mod.rs
│   └── interpolate.rs
├── globset/            # 保留：文件过滤必需
│   ├── mod.rs
│   └── glob.rs
├── ignore/             # 保留：文件遍历核心
│   ├── mod.rs
│   ├── walk.rs
│   ├── gitignore.rs
│   └── types.rs
└── search/             # 合并 searcher + regex + pcre2 + 精简 printer
    ├── mod.rs          # 导出所有搜索相关 API
    ├── engine.rs       # 原 searcher：Searcher, SearcherBuilder
    ├── sink.rs         # Sink trait 和 sinks 模块
    ├── regex.rs        # Rust 正则实现
    ├── pcre2.rs        # PCRE2 实现（条件编译，feature = "pcre2"）
    └── printer.rs      # 精简后的输出格式化（仅 Standard/JSON）
```

**注意**：`cli/` 和 `core/` 大部分代码是 CLI 专属，在 API -focused 改造中应移除或大幅精简。

---

## 功能精简策略：API vs CLI

### 1. 可完全移除的 CLI 专属功能

| 功能模块 | 当前位置 | 建议 | 原因 |
|---------|---------|------|------|
| 解压缩支持 | `cli/src/decompress.rs` | **移除** | 通过外部 crate 处理更灵活 |
| 颜色/终端控制 | `cli/src/wtr.rs`, `printer/src/color.rs` | **移除** | API 应返回原始数据，由调用方处理展示 |
| 超链接生成 | `printer/src/hyperlink/` | **移除** | 纯 CLI 终端功能 |
| 命令行参数解析 | `core/flags/` (约 2000 行) | **移除** | API 使用结构体配置而非 CLI 参数 |
| 人性化错误消息 | `core/messages.rs` | **移除** | API 返回标准 Error，消息由调用方处理 |
| 调试日志系统 | `core/logger.rs` | **移除** | 使用标准 `log` crate 或移除 |
| jemalloc 优化 | `core/main.rs` | **移除** | 仅 musl CLI 构建需要 |
| 参数模式处理 | `cli/src/pattern.rs` | **移除** | CLI 专属的文件读取模式 |
| 进程命令执行 | `cli/src/process.rs` | **移除** | 仅用于解压缩支持 |
| 主机名获取 | `cli/src/hostname.rs` | **移除** | 仅用于超链接功能 |
| Summary 统计 | `printer/src/summary.rs` | **移除** | 聚合统计更适合 CLI 层处理 |

### 2. 需要大幅精简的模块

#### `printer` 模块精简

```rust
// 当前 printer 提供：
pub use crate::{
    standard::{Standard, StandardBuilder, StandardSink},  // 保留（基础文本输出）
    json::{JSON, JSONBuilder, JSONSink},                  // 保留（可选 feature = "serde"）
    summary::{Summary, SummaryBuilder, SummaryKind},       // 移除（CLI 聚合统计）
    hyperlink::{HyperlinkConfig, ...},                    // 移除（终端超链接）
    stats::Stats,                                          // 移除（可由调用方统计）
    color::{ColorSpecs, ...},                             // 移除（API 不处理颜色）
};

// 精简后：
pub mod printer {
    pub struct Standard;      // 基础文本输出（无颜色）
    pub struct JSON;          // JSON Lines 输出（feature = "serde"）
    
    // 移除：Hyperlink、Summary、Stats、ColorSpecs
}
```

#### `core` 模块精简

```rust
// 移除前：HiArgs（约 1500 行 CLI 参数解析结果）
// 移除后：纯数据结构 SearchConfig

pub struct SearchConfig {
    pub pattern: String,
    pub paths: Vec<PathBuf>,
    pub case_sensitive: bool,
    pub include_hidden: bool,
    pub max_depth: Option<usize>,
    pub follow_symlinks: bool,
    // ... 仅保留 API 相关配置
}

impl SearchConfig {
    pub fn new(pattern: impl Into<String>) -> Self { ... }
    pub fn paths(mut self, paths: impl IntoIterator<Item = impl AsRef<Path>>) -> Self { ... }
    pub fn search(self) -> Result<SearchResults> { ... }
}
```

---

## 合并步骤

### 1. 创建新的单 crate 结构

```bash
# 创建目录结构（不包含 cli/ 和 core/flags/）
mkdir -p src/{matcher,globset,ignore,search}
```

### 2. 迁移各 crate 代码

#### 2.1 matcher 模块（核心，完全保留）

**原路径**: `crates/matcher/src/lib.rs`
**新路径**: `src/matcher/mod.rs`

```rust
// src/matcher/mod.rs
pub use self::{
    interpolate::interpolate,
};

mod interpolate;

// 核心 trait：所有匹配器的基础
pub struct Match { ... }
pub trait Matcher { ... }
pub trait Captures { ... }
pub struct LineTerminator { ... }
pub struct ByteSet { ... }
```

#### 2.2 globset 模块（完全保留）

**修改内容**: 仅更新内部依赖路径

```rust
// 修改前
use globset::{Glob, GlobSet};

// 修改后（在 ignore 模块中）
use crate::globset::{Glob, GlobSet};
```

#### 2.3 ignore 模块（完全保留）

包含文件遍历、gitignore 处理、类型过滤等核心功能。

#### 2.4 search 模块（合并 + 精简）

整合 `searcher` + `regex` + `pcre2` + 精简版 `printer`：

```
src/search/
├── mod.rs              # 导出公共 API
├── engine.rs           # 原 searcher：Searcher, SearcherBuilder
├── sink.rs             # Sink trait 和 sinks 模块
├── regex.rs            # Rust 正则实现（Matcher）
├── pcre2.rs            # PCRE2 实现（#[cfg(feature = "pcre2")]）
└── printer.rs          # 精简后的输出格式化
```

**src/search/mod.rs**:
```rust
pub mod engine;
pub mod sink;

// 条件编译 PCRE2 支持
#[cfg(feature = "pcre2")]
pub mod pcre2;

// 正则实现
pub mod regex;

// 精简 printer
pub mod printer;

// 重新导出核心类型
pub use engine::{
    Searcher, 
    SearcherBuilder, 
    BinaryDetection, 
    Encoding, 
    MmapChoice
};
pub use sink::{Sink, SinkMatch, SinkContext, sinks};
pub use regex::RegexMatcher;
#[cfg(feature = "pcre2")]
pub use pcre2::RegexMatcher as Pcre2Matcher;
pub use printer::{Standard, JSON};

// 统一匹配器枚举（方便使用）
pub enum PatternMatcher {
    RustRegex(regex::RegexMatcher),
    #[cfg(feature = "pcre2")]
    PCRE2(pcre2::RegexMatcher),
}
```

#### 2.5 条件编译处理

```rust
// src/lib.rs
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod matcher;
pub mod globset;
pub mod ignore;
pub mod search;

// 可选：保留 CLI 支持（feature = "bin"）
#[cfg(feature = "bin")]
pub mod cli;

// 重新导出核心类型
pub use crate::{
    ignore::{Walk, WalkBuilder, Types},
    matcher::{Matcher, Match, Captures, LineTerminator},
    search::{
        engine::{Searcher, SearcherBuilder},
        sink::{Sink, sinks},
        PatternMatcher,
    },
};
```

---

## 3. 依赖精简

### Cargo.toml（API-focused 配置）

```toml
[package]
name = "ripgrep"
version = "15.1.0"
edition = "2024"
rust-version = "1.85"

[lib]
name = "ripgrep"
path = "src/lib.rs"

# 可选 CLI 二进制（默认关闭）
[[bin]]
name = "rg"
path = "src/main.rs"
required-features = ["bin"]

[features]
default = ["pcre2", "serde"]

# PCRE2 正则引擎支持
pcre2 = ["dep:pcre2-sys"]

# JSON 输出支持
serde = ["dep:serde", "dep:serde_json"]

# CLI 支持（可选）
bin = [
    "dep:anyhow",
    "dep:lexopt", 
    "dep:termcolor",
]

[dependencies]
# === 核心必需 ===
bstr = "1"
encoding_rs = "0.8"
encoding_rs_io = "0.1"
log = "0.4"
regex = "1"
regex-automata = "0.4"
regex-syntax = "0.8"
walkdir = "2"

# === 可选：PCRE2 ===
pcre2-sys = { version = "0.2", optional = true }

# === 可选：JSON 输出 ===
serde = { version = "1", optional = true }
serde_json = { version = "1", optional = true }

# === 可选：CLI 二进制 ===
anyhow = { version = "1", optional = true }
lexopt = { version = "0.3", optional = true }
termcolor = { version = "1", optional = true }

# === 平台特定 ===
[target.'cfg(windows)'.dependencies]
winapi-util = "0.1"

[dev-dependencies]
# 测试依赖
```

### 移除的依赖

| 依赖 | 原因 |
|------|------|
| `anyhow` | API 使用标准错误类型 |
| `lexopt` | CLI 参数解析，API 不需要 |
| `termcolor` | 终端颜色，API 返回原始数据 |
| `textwrap` | 文本格式化，CLI 专属 |
| `tikv-jemallocator` | 仅 musl CLI 优化 |
| `grep-cli` | 解压缩/进程执行，API 不需要 |

---

## 4. 公开 API 设计（三层架构）

### 高层 API（一键搜索）

适合快速使用场景：

```rust
use ripgrep::{search_simple, SearchResult};

// 最简单的搜索
for result in search_simple("pattern", "./src")? {
    println!("{}:{}: {}", 
        result.path().display(),
        result.line_number(),
        result.line()
    );
}

// 带配置的搜索
for result in SearchConfig::new("pattern")
    .paths(["./src", "./tests"])
    .case_insensitive(true)
    .max_depth(3)
    .search()?
{
    // 处理结果
}
```

### 中层 API（协调搜索流程）

适合需要更多控制的场景：

```rust
use ripgrep::{
    ignore::WalkBuilder,
    search::{Searcher, RegexMatcher, StandardSink},
};

// 创建匹配器
let matcher = RegexMatcher::new(r"fn \w+")?;

// 配置搜索器
let searcher = Searcher::builder()
    .line_number(true)
    .encoding(Encoding::UTF8)
    .build();

// 遍历并搜索
for entry in WalkBuilder::new("./src").build() {
    let entry = entry?;
    let mut sink = Vec::new();
    searcher.search_path(&matcher, entry.path(), &mut sink)?;
    // 处理 sink 结果
}
```

### 底层 API（完全自定义）

适合扩展和自定义：

```rust
use ripgrep::matcher::{Matcher, Match, Captures};
use ripgrep::search::sink::{Sink, SinkMatch};

// 实现自定义 Matcher
struct MyMatcher { ... }

impl Matcher for MyMatcher {
    type Captures = NoCaptures;
    type Error = std::io::Error;
    
    fn find_at(&self, haystack: &[u8], at: usize) -> Result<Option<Match>, Self::Error> {
        // 自定义匹配逻辑
    }
    
    fn new_captures(&self) -> Result<Self::Captures, Self::Error> {
        Ok(NoCaptures::new())
    }
}

// 实现自定义 Sink
struct MySink { ... }

impl Sink for MySink {
    fn matched(&mut self, _searcher: &Searcher, mat: &SinkMatch<'_>) -> Result<bool, Self::Error> {
        // 自定义结果处理
        Ok(true)
    }
}
```

---

## 5. 功能保留矩阵

| 功能 | 状态 | 优先级 | 说明 |
|------|------|--------|------|
| Matcher trait | 保留 | P0 | 核心抽象 |
| Rust 正则引擎 | 保留 | P0 | 默认正则支持 |
| 文件遍历 + ignore | 保留 | P0 | 核心功能 |
| WalkBuilder | 保留 | P0 | 文件遍历配置 |
| Searcher | 保留 | P0 | 搜索执行引擎 |
| Sink trait | 保留 | P0 | 结果处理 |
| PCRE2 引擎 | 保留 | P1 | `feature = "pcre2"` |
| JSON 输出 | 保留 | P1 | `feature = "serde"` |
| 编码检测 | 保留 | P1 | UTF-8/编码自动检测 |
| 多行搜索 | 保留 | P1 | Searcher 支持 |
| 二进制检测 | 保留 | P2 | BinaryDetection |
| 内存映射 | 保留 | P2 | MmapChoice |
| 超链接 | **移除** | - | CLI 专属 |
| 颜色输出 | **移除** | - | 调用方处理 |
| Summary 统计 | **移除** | - | 调用方统计 |
| 解压缩 | **移除** | - | 外部处理 |
| 参数解析 | **移除** | - | 调用方处理 |

---

## 6. 错误处理策略

统一错误类型设计：

```rust
// src/error.rs
#[derive(Debug)]
pub enum Error {
    Matcher(crate::matcher::MatcherError),
    Ignore(crate::ignore::IgnoreError),
    Regex(regex::Error),
    #[cfg(feature = "pcre2")]
    Pcre2(crate::search::pcre2::Error),
    Io(std::io::Error),
    Encoding(EncodingError),
    Pattern(String),
}

impl std::fmt::Display for Error { ... }
impl std::error::Error for Error { ... }

// 方便的类型别名
pub type Result<T> = std::result::Result<T, Error>;
```

---

## 7. 文件迁移映射表

| 原路径 | 新路径 | 说明 |
|--------|--------|------|
| `crates/matcher/src/*.rs` | `src/matcher/` | 直接迁移 |
| `crates/globset/src/*.rs` | `src/globset/` | 直接迁移 |
| `crates/ignore/src/*.rs` | `src/ignore/` | 修改 `use globset` → `use crate::globset` |
| `crates/searcher/src/lib.rs` | `src/search/engine.rs` | 重命名为 engine |
| `crates/searcher/src/searcher/*.rs` | `src/search/engine/` | 整体迁移 |
| `crates/searcher/src/sink.rs` | `src/search/sink.rs` | 直接迁移 |
| `crates/regex/src/*.rs` | `src/search/regex/` | 移入 search 模块 |
| `crates/pcre2/src/*.rs` | `src/search/pcre2/` | 条件编译 |
| `crates/printer/src/standard.rs` | `src/search/printer.rs` | 精简后合并为单文件 |
| `crates/printer/src/json.rs` | `src/search/printer.rs` | 合并到 printer（feature = "serde"） |
| `crates/cli/src/*.rs` | **移除** | CLI 专属功能 |
| `crates/core/flags/` | **移除** | CLI 参数解析 |
| `crates/core/main.rs` | `src/main.rs` | CLI 入口（feature = "bin"） |
| `crates/core/search.rs` | **精简后移除** | 逻辑合并到 search 模块 |

---

## 8. 合并后的优势

1. **更简洁的 API**：移除了 CLI 特有的复杂性，API 更专注
2. **减少依赖**：移除 CLI 相关依赖，核心更轻量
3. **更快的编译**：单 crate 减少链接开销
4. **更容易集成**：单一包导入，无 workspace 协调问题
5. **清晰的层次**：三层 API 设计满足不同使用场景

## 9. 注意事项

1. **CLI 功能分离**：如果需要 CLI，建议作为独立 crate 或 `feature = "bin"`
2. **向后兼容**：现有用户需要迁移到新 API（提供迁移指南）
3. **测试组织**：重新组织测试，确保各模块独立测试
4. **文档更新**：需要更新 API 文档和示例

---

## 快速迁移检查清单

- [ ] 创建新的 `src/` 目录结构
- [ ] 迁移 `matcher/` 模块（完全保留）
- [ ] 迁移 `globset/` 模块（完全保留）
- [ ] 迁移 `ignore/` 模块（更新内部依赖路径）
- [ ] 创建 `search/` 模块（合并 searcher + regex + pcre2）
- [ ] 精简 `printer/` 并移入 `search/`
- [ ] 移除 `cli/` 模块
- [ ] 精简 `core/` 模块（仅保留高层 API 配置）
- [ ] 更新 `Cargo.toml`（精简依赖，添加 features）
- [ ] 创建统一错误类型
- [ ] 编写新的三层 API 接口
- [ ] 更新测试和文档