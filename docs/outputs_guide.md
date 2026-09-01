# outputs目录输出文件分析

本文档分析 `outputs` 目录中各类输出文件的生成来源和用途。

## 目录结构概览

```
outputs/
├── README.md                                # 输出目录说明文档
├── scenarios/                               # 场景测试输出
│   ├── documents/                           # 结构化文档格式测试
│   │   ├── chunks/                          # 分块文本输出
│   │   │   ├── json/
│   │   │   ├── md/
│   │   │   ├── toml/
│   │   │   ├── xml/
│   │   │   └── yaml/
│   │   └── summary/                         # 文件级摘要输出
│   │       ├── json/
│   │       ├── toml/
│   │       ├── xml/
│   │       └── yaml/
│   ├── java/
│   │   └── summary/
│   │       └── springboot-minimal-demo/     # Java NL文档导出
│   ├── python/
│   │   ├── chunks/                          # Python分块验证输出
│   │   │   ├── basic/emb/
│   │   │   ├── flask/emb/
│   │   │   ├── flask/bm25/
│   │   │   └── index_sidecar/
│   │   │       ├── emb/
│   │   │       └── bm25/
│   │   └── summary/                         # Python NL文档导出
│   │       ├── basic/
│   │       ├── flask/
│   │       └── index_sidecar/
│   └── rust/
│       ├── alignment/                       # Hybrid对齐审查报告
│       │   ├── run_manifest.txt
│       │   ├── alignment_summary.md
│       │   └── per_query_alignment.md
│       ├── chunks/                          # Rust分块验证输出
│   │   │   ├── index_sidecar/
│   │   │   │   ├── emb/
│   │   │   │   └── bm25/
│   │   │   ├── once_cell/
│   │   │   │   ├── emb/
│   │   │   │   └── bm25/
│   │   │   └── ripgrep/
│   │   │       ├── emb/
│   │   │       └── bm25/
│   │   └── summary/                         # Rust NL文档导出
│   │       ├── index_sidecar/
│   │       ├── once_cell/
│   │       └── ripgrep/
│   └── test_reporter/                       # 测试报告器输出
├── relation/                                # 关系查询输出
│   └── rust/
│       └── presentation/
│           ├── relation_demo/               # relation_demo关系展示
│           └── relation_diamond/            # relation_diamond关系展示
├── index/                                   # 索引工作流输出
│   └── test_basic/
├── benchmark/                               # 基准测试输出
│   ├── once_cell/                           # once_cell基准评估
│   │   ├── aggregate_metrics_*.md
│   │   ├── aggregate_metrics_by_query_type_*.md
│   │   ├── per_query_results_*.md
│   │   ├── relevance_top5.md
│   │   ├── test_diagnostics.md
│   │   ├── run_manifest.txt
│   │   ├── evaluation_scope.md
│   │   ├── bm25_parameter_sweep/
│   │   └── retrieval_method/                # 召回方式基准测试（emb/bm25/hybrid 融合）
│   │       ├── run_manifest.txt
│   │       ├── alignment_coverage.md
│   │       ├── aggregate_top{k}.md
│   │       ├── aggregate_by_query_type_top{k}.md
│   │       ├── per_query_top{k}.md
│   │       ├── fusion_gain_by_query_type_top{k}.md
│   │       ├── weight_sensitivity.md
│   │       ├── rrf_k_sensitivity.md
│   │       └── relevance_top5.md
│   ├── ripgrep/                             # ripgrep基准评估
│   │   ├── (same structure as once_cell)
│   │   └── bm25_parameter_sweep/
│   └── flask/                               # Flask基准评估
│       ├── (same structure as once_cell)
│       └── bm25_parameter_sweep/
└── debug/                                   # 调试输出
    ├── direct_chunking/
    │   └── comparison.txt
    ├── full_pipeline/
    │   └── comparison.txt
    └── full_pipeline_raw_source/
        └── comparison.txt
```

## 输出文件生成来源分析

### 1. scenarios/ 目录

#### 1.1 Rust索引结果 (`scenarios/rust/summary/`)

**生成命令:**
```bash
cargo run --example basic -p cce-e2e-tests
```

**来源文件:** `examples/rust/basic.rs`

**生成逻辑:**
- 使用 `TestFixture::rust_basic()` 加载基本Rust项目fixture
- 通过 `IndexOrchestrator` 执行完整索引工作流
- 使用 `OutputReporter` 和 `OutputSerializer` 生成JSON格式的索引结果报告
- 输出包含：文件数、索引文件数、实体数、关系数、向量数、令牌数等统计信息

**输出内容:**
- `outputs/scenarios/rust/presentation/basic/result.json` - 索引结果统计（由 `OutputSerializer::json()` 序列化）

#### 1.2 Rust BM25索引结果 (`scenarios/rust/presentation/basic_bm25/`)

**生成命令:**
```bash
cargo run --example basic -p cce-e2e-tests
```

**来源文件:** `examples/rust/basic.rs` (第42-66行)

**生成逻辑:**
- 与basic相同，但配置为使用BM25索引
- 生成BM25特定的索引结果报告

**输出内容:**
- `outputs/scenarios/rust/presentation/basic_bm25/result.json` - BM25索引结果统计

#### 1.3 Rust NL文档导出 (`scenarios/rust/summary/{once_cell,ripgrep,index_sidecar}/`)

**生成命令:**
```bash
cargo run --example export_rs -p cce-e2e-tests
```

**来源文件:** `examples/rust/export_rs.rs` (第67-160行 once_cell, 第162-255行 ripgrep, 第257-349行 index_sidecar)

**生成逻辑:**
1. 使用 `TestFixture::rust_once_cell()` / `rust_ripgrep()` / `rust_index_sidecar()` 加载fixture
2. 使用 `FSScanner` 扫描项目目录
3. 使用 `FileProcessor::process_file_complete()` 处理每个文件
4. 使用 `DirectExporter` 导出NL文档到 `.cce/nl_docs/`
5. 将生成的 `.md` 文件从 `.cce/nl_docs/` 复制到outputs目录
6. 生成 `SUMMARY.md`

**输出内容:**
- `SUMMARY.md` - 导出摘要信息
- `src/*.md` - 各源文件的Markdown格式NL文档

#### 1.4 Rust分块验证 (`scenarios/rust/chunks/{once_cell,ripgrep,index_sidecar}/{emb,bm25}/`)

**生成命令:**
```bash
cargo run --example export_rs -p cce-e2e-tests
```

**来源文件:** `examples/rust/export_rs.rs` (第351-484行 once_cell, 第486-619行 ripgrep, 第621-723行 index_sidecar)

**生成逻辑:**
1. 使用 `TestFixture::rust_once_cell()` 加载 once_cell fixture
2. 使用 `FileProcessor::process_file_complete()` 分别处理 Embedding 和 BM25 路径
3. 按源文件路径对 chunk 分组并排序
4. 对每个源文件生成分段验证文档

**输出目录结构:**
```
once_cell/emb/
├── SUMMARY.txt
└── src/
    ├── lib.rs.txt
    ├── imp_cs.rs.txt
    └── ...
once_cell/bm25/
├── SUMMARY.txt
└── src/
    └── ...
```

**分段验证格式 (每个文件):**

```
{文件名}
total chunks: {chunk总数}
---

=== CHUNK 1/{总数} (lines {起始行}-{结束行}) ===
chunk_id: {id} | group: {group_id} | kind: {实体类型}
fragment: {n}/{总数} | split_reason: {split_reason}
title: {bm25_title} | tokens: {token_count}
keywords: [kw1, kw2, ...]
related: {group_id} ({relation_type}), ...
prev_overlap: {n} tokens from {chunk_id}
next_overlap: {n} tokens from {chunk_id}

{chunk 的 NL 描述文本}
```

**验证要点:**
- `split_reason`: 检查分割原因是否合理 (member_boundary / hard_limit / sentence_boundary 等)
- `fragment`: 检查分片编号是否连续 (1/3, 2/3, 3/3)
- `original_entity_id`: 同一实体的分片应共享相同 ID
- `prev_overlap` / `next_overlap`: 检查相邻 chunk 的重叠区域是否一致
- `related_groups`: 检查前驱/后继/调用关系是否正确

#### 1.5 Java Spring Boot导出 (`scenarios/java/summary/springboot-minimal-demo/`)

**生成命令:**
```bash
cargo run --example java_spring_boot_export -p cce-e2e-tests
```

**来源文件:** `examples/java/spring_boot_export.rs`

**生成逻辑:**
- 使用 `TestFixture::java_spring_boot()` 加载fixture
- 使用 `NlDocumentExporter` 生成NL文档
- 将 `.md` 文件从 `.cce/nl_docs/` 复制到outputs目录

**输出内容:**
- `SUMMARY.md` - 导出摘要
- `*.md` - 各源文件的Markdown格式NL文档

#### 1.6 Python NL文档导出和分块验证 (`scenarios/python/summary/` 和 `scenarios/python/chunks/`)

**生成命令:**
```bash
cargo run --example export_py -p cce-e2e-tests
```

**来源文件:** `examples/python/export_py.rs`

**生成逻辑:**
- 依次处理 flask、basic、index_sidecar 三个fixture
- 每个fixture同时生成 summary (Markdown NL文档) 和 chunks (分块验证)
- chunks 按源文件路径分组并排序

**输出内容:**
- `summary/{fixture}/SUMMARY.md` - 导出摘要
- `summary/{fixture}/*.md` - Markdown格式NL文档
- `chunks/{fixture}/emb/*.txt` - Embedding路径分块
- `chunks/{fixture}/bm25/*.txt` - BM25路径分块

#### 1.7 多语言项目输出 (`scenarios/multi_language/`)

**生成命令:**
```bash
cargo run --example multi_language/basic -p cce-e2e-tests
```

**来源文件:** `examples/multi_language/basic.rs`

**生成逻辑:**
- 依次处理多语言fixture、Python基本fixture和Java基本fixture
- 使用 `OutputReporter` 生成JSON报告

**输出内容:**
- `multi_language_project.json` - 多语言项目索引结果
- `python_basic.json` - Python项目索引结果
- `java_basic.json` - Java项目索引结果

### 2. relation/ 目录

#### 2.1 relation_demo关系展示 (`relation/rust/presentation/relation_demo/`)

**生成命令:**
```bash
cargo run --example relation -p cce-e2e-tests
```

**来源文件:** `examples/rust/relation.rs` (第689-833行)

**生成逻辑:**
1. 使用 `TestFixture::rust_relation_demo()` 加载fixture
2. 使用 `BuildConfigParser` 扫描项目依赖
3. 配置 `IndexOptions` 启用关系构建 (`build_relations: true`)
4. 执行索引获取 `RelationIndex`
5. 使用 `CallChainQuery` 执行各种关系查询

**输出内容:**
- `SUMMARY.txt` - 关系展示摘要
- `config_scan.txt` - 构建配置扫描结果
- `scope_entities.txt` - 作用域实体和关系
- `relations.txt` - 直接关系列表
- `call_chains.txt` - 调用链信息
- `file_dependencies.txt` - 文件依赖关系

#### 2.2 relation_diamond关系展示 (`relation/rust/presentation/relation_diamond/`)

**生成逻辑:**
- 与relation_demo类似，但使用 `TestFixture::rust_relation_diamond()` fixture
- 该fixture展示菱形依赖模式

**输出内容:**
- `SUMMARY.txt` - 摘要
- `config_scan.txt` - 配置扫描
- `relations.txt` - 关系列表
- `call_chains.txt` - 调用链

### 3. index/ 目录

#### 3.1 test_basic (`index/test_basic/`)

**生成来源:** 测试用例

**来源文件:** `tests/regression/e2e_outputs.rs`

**生成逻辑:**
- 由 `test_output_manager_basic` 测试生成
- 执行输出管理器基本功能测试

**输出内容:**
- `test.txt` - 测试结果输出

### 4. benchmark/ 目录

#### 4.1 once_cell 基准评估 (`benchmark/once_cell/`)

**生成命令:**
```bash
cargo run --example benchmark_oncecell -p cce-e2e-tests
```

**来源文件:** `examples/rust/benchmark_oncecell.rs`

**生成逻辑:**
1. 从 `data/benchmark/{baseline}/once_cell/bge-m3/bench_data.rkyv` 加载基准数据
2. 使用 `src/judgments/evaluate.rs` 中的评估函数
3. 计算BGE-M3余弦相似度分数和BM25分数
4. 使用 `src/judgments/report.rs` 中的报告函数生成报告
5. 输出Precision@K、Recall@K、MRR@K、F1@K等指标

**输出内容:**
- `aggregate_metrics_top{k}.md` - 聚合指标报告（top5/10/20/30/50）
- `aggregate_metrics_by_query_type_top{k}.md` - 按查询类型分组的聚合指标
- `per_query_results_top{k}.md` - 每条查询的详细结果
- `relevance_top5.md` - 相关性报告
- `test_diagnostics.md` - 测试诊断信息
- `run_manifest.txt` - 运行清单
- `evaluation_scope.md` - 评估范围说明
- `bm25_parameter_sweep/` - BM25参数扫描结果

#### 4.2 ripgrep 基准评估 (`benchmark/ripgrep/`)

**生成命令:**
```bash
cargo run --example benchmark_ripgrep -p cce-e2e-tests
```

**来源文件:** `examples/rust/benchmark_ripgrep.rs`

**生成逻辑:**
- 与 benchmark_oncecell 相同的评估流程
- 使用 ripgrep fixture 和 ripgrep 相关性判断
- 评估范围：CoreRetrieval

**输出内容:**
- (同 once_cell 结构)

#### 4.3 Flask 基准评估 (`benchmark/flask/`)

**生成命令:**
```bash
cargo run --example benchmark_flask -p cce-e2e-tests
```

**来源文件:** `examples/python/benchmark_flask.rs`

**生成逻辑:**
- 与 once_cell 相同的评估流程
- 使用 Python Flask fixture

**输出内容:**
- (同 once_cell 结构)

#### 4.4 BM25参数扫描 (`benchmark/{once_cell,ripgrep,flask}/bm25_parameter_sweep/`)

**生成命令:**
```bash
cargo run --example bm25_para_oncecell -p cce-e2e-tests
cargo run --example bm25_para_ripgrep -p cce-e2e-tests
cargo run --example bm25_para_flask -p cce-e2e-tests
```

**来源文件:**
- `examples/rust/bm25_para_oncecell.rs`
- `examples/rust/bm25_para_ripgrep.rs`
- `examples/python/bm25_para_flask.rs`

**生成逻辑:**
- 对BM25参数（k1, b）进行网格搜索
- 计算不同参数组合下的MRR@10、F1@10、Recall@20等指标

**输出内容:**
- `aggregate_metrics.md` - 聚合指标
- `aggregate_metrics_by_query_type.md` - 按查询类型分组
- `per_query_metrics.md` - 每条查询的指标
- `manifest.md` - 参数清单

#### 4.5 召回方式基准测试 (`benchmark/{once_cell,ripgrep,flask}/retrieval_method/`)

**生成命令:**
```bash
cargo run --example benchmark_retrieval_oncecell -p cce-e2e-tests
cargo run --example benchmark_retrieval_ripgrep -p cce-e2e-tests
cargo run --example benchmark_retrieval_flask -p cce-e2e-tests
```

**来源文件:**
- `examples/rust/benchmark_retrieval_oncecell.rs`
- `examples/rust/benchmark_retrieval_ripgrep.rs`
- `examples/python/benchmark_retrieval_flask.rs`
- 核心逻辑：`src/retrieval_method/{fusion,benchmark,report}.rs`

**生成逻辑:**
- 直接复用 `data/benchmark/{baseline}/{fixture}/bge-m3/bench_data.rkyv`（emb 向量 + bm25 文本 + 查询），无需重新生成数据、无需 LLM API
- 评测方法矩阵：`emb`、`bm25`、`minmax-*`（7 组 vector/bm25 权重）、`rrf-*`（k=30/60/100）
- 跨路径对齐键为 `(file_path, entity_name)`（当前 ChunkData 无 `content_entity_ids`）；所有方法统一按对齐键去重后评测，保证 hybrid 与单路口径一致
- 复用 `evaluate_ranked_chunks_range_based` 计算 P/R/F1，另加首次命中排位、同 range 重复占位、fusion_gain、权重敏感性指标

**输出内容:**
- `run_manifest.txt` - 运行清单（数据源、方法矩阵、对齐键、各 baseline chunk/键统计）
- `alignment_coverage.md` - 跨路径对齐键覆盖统计（失真可见性）
- `aggregate_top{k}.md` - 按 (baseline, method) 聚合的 P/R/F1/1st_hit/redund
- `aggregate_by_query_type_top{k}.md` - 按查询类型聚合
- `per_query_top{k}.md` - 逐 query × 逐 method 明细
- `fusion_gain_by_query_type_top{k}.md` - 融合相对最优单路的提升
- `weight_sensitivity.md` - minmax 7 权重敏感性（含 std）
- `rrf_k_sensitivity.md` - rrf 3 个 k 值对比（含 std）
- `relevance_top5.md` - top-5 strong/related 命中明细

#### 4.6 Hybrid对齐审查报告 (`scenarios/rust/alignment/`)

**生成命令:**
```bash
cargo run --example alignment_report -p cce-e2e-tests
```

**来源文件:** `examples/rust/alignment_report.rs`

**生成逻辑:**
- 加载 `tests/fixtures/rust/basic` fixture，使用确定性 mock 嵌入服务（`src/mock_embedding_server.rs`），无需 LLM API
- 探测 Qdrant 可用性：可用时执行 vector/BM25/hybrid 三路查询，不可用时降级为 BM25-only 并在 manifest 中记录
- 使用 `OutputManager`（`Scenarios` 类别 + `rust` 语言 + `alignment` 场景）写入输出

**输出内容:**
- `run_manifest.txt` - 运行清单（fixture、Qdrant 可用性、mock 嵌入、对齐键口径、查询列表）
- `alignment_summary.md` - 逐 query 的 vector/bm25 键数、跨路径匹配键数与对齐率、hybrid 键包含关系
- `per_query_alignment.md` - 逐 query 的 vector/bm25/hybrid 对齐键明细

### 5. debug/ 目录

#### 5.1 检索器对比 (`debug/{direct_chunking,full_pipeline,full_pipeline_raw_source}/`)

**生成命令:**
```bash
cargo run --example debug_matrix -p cce-e2e-tests
```

**来源文件:** `examples/rust/debug_matrix.rs`

**生成逻辑:**
- 加载基准数据
- 对比BGE-M3和BM25两种检索器的性能
- 生成对比报告

**输出内容:**
- `comparison.txt` - 检索器对比报告

### 6. 其他输出

#### 6.1 文档分块演示 (`outputs/demo/documents/`)

**生成命令:**
```bash
cargo run --example dump_document_chunks -p cce-e2e-tests
```

**来源文件:** `examples/rust/dump_document_chunks.rs`

**输出内容:**
- `{file_name}.emb/` - Embedding路径分块
- `{file_name}.bm25/` - BM25路径分块
- `SUMMARY.md` - 概览

#### 6.2 检索结果转储 (`outputs/benchmark/results/{baseline}/{retriever}/ranked/`)

**生成命令:**
```bash
cargo run --example dump_retrieval -p cce-e2e-tests
```

**来源文件:** `examples/rust/dump_retrieval.rs`

**输出内容:**
- `{query_id}.txt` - 每条查询的检索 ranked 结果

### 7. 基准数据生成 (`data/benchmark/`)

**生成命令:**
```bash
cargo run --example gen_bench_oncecell -p cce-e2e-tests
cargo run --example gen_bench_ripgrep -p cce-e2e-tests
cargo run --example gen_bench_flask -p cce-e2e-tests
```

**来源文件:**
- `examples/rust/gen_bench_oncecell.rs`
- `examples/rust/gen_bench_ripgrep.rs`
- `examples/python/gen_bench_flask.rs`

**生成逻辑:**
1. 加载目标代码库和 distractor 代码库的源文件
2. 执行三种分块基线：full_pipeline、direct_chunking、text_cleaner
3. 对所有分块文本和查询文本进行批量嵌入（如配置了API key）
4. 将结果序列化为 `rkyv` 格式

**输出内容:**
- `data/benchmark/{baseline}/{fixture}/bge-m3/bench_data.rkyv`

## 核心生成模块说明

### OutputManager (`src/output_manager.rs`)

输出管理器，负责管理输出目录结构和文件创建。

**关键功能:**
- `OutputCategory` 枚举定义输出类别：Index、Query、Relation、Scenarios、HotUpdate
- `OutputBuilder` 用于构建输出路径（支持语言、场景、时间戳等维度）
- `write()` 方法写入文件内容

### OutputReporter (`src/output_reporter.rs`)

高级报告生成器。

**关键功能:**
- `report_index_result()` - 生成索引结果报告
- `write_custom()` - 写入自定义内容
- 支持元数据（描述、类别、标签）

### OutputSerializer (`src/output_serializer.rs`)

序列化器，支持JSON和Markdown格式。

**关键功能:**
- `serialize_index_result()` - 序列化索引结果
- `SerializableIndexResult` - 可序列化的索引结果结构
- `json()` / `markdown()` - 快速构造序列化器

### PipelineDebugExporter (`src/pipeline_debug.rs`)

流水线调试导出器。

**关键功能:**
- `render_entity_tree()` - 渲染实体树
- `render_groups()` - 渲染分组信息
- `render_pipeline_summary()` - 渲染流水线摘要
- `render_chunk_alignment()` - 渲染分块对齐

### DirectExporter (`cce_orchestrator::export`)

NL文档导出器，用于导出Markdown格式的自然语言文档。

**关键功能:**
- `export_groups()` - 导出实体组为NL文档

### NlDocumentExporter (`cce_orchestrator::export`)

文件级NL文档导出器。

**关键功能:**
- 导出文件级的Markdown摘要文档

### FileProcessor (`cce_orchestrator::index`)

文件处理器，负责完整流水线处理。

**关键功能:**
- `process_file_complete()` - 处理单个文件的完整流水线
- 支持 Embedding 和 BM25 两种路径

## 生成命令汇总

| 输出目录 | 生成命令 | 来源文件 | 是否依赖向量化 |
|---------|---------|---------|---------------|
| scenarios/rust/summary/once_cell/ | `cargo run --example export_rs` | examples/rust/export_rs.rs | 否（直接调用 `FileProcessor`） |
| scenarios/rust/summary/ripgrep/ | `cargo run --example export_rs` | examples/rust/export_rs.rs | 否（直接调用 `FileProcessor`） |
| scenarios/rust/summary/index_sidecar/ | `cargo run --example export_rs` | examples/rust/export_rs.rs | 否（直接调用 `FileProcessor`） |
| scenarios/rust/chunks/once_cell/{emb,bm25}/ | `cargo run --example export_rs` | examples/rust/export_rs.rs | 否（直接调用 `FileProcessor`） |
| scenarios/rust/chunks/ripgrep/{emb,bm25}/ | `cargo run --example export_rs` | examples/rust/export_rs.rs | 否（直接调用 `FileProcessor`） |
| scenarios/rust/chunks/index_sidecar/{emb,bm25}/ | `cargo run --example export_rs` | examples/rust/export_rs.rs | 否（直接调用 `FileProcessor`） |
| scenarios/java/summary/springboot-minimal-demo/ | `cargo run --example java_spring_boot_export` | examples/java/spring_boot_export.rs | 否 |
| scenarios/python/summary/{basic,flask,index_sidecar}/ | `cargo run --example export_py` | examples/python/export_py.rs | 否 |
| scenarios/python/chunks/{basic,flask,index_sidecar}/{emb,bm25}/ | `cargo run --example export_py` | examples/python/export_py.rs | 否 |
| scenarios/documents/chunks/{json,md,toml,xml,yaml}/ | `cargo run --example export_doc` | examples/rust/export_doc.rs | 否 |
| scenarios/documents/summary/{json,toml,xml,yaml}/ | `cargo run --example summary_doc` | examples/rust/summary_doc.rs | 否 |
| relation/rust/presentation/relation_demo/ | `cargo run --example relation` | examples/rust/relation.rs | 否（关系查询） |
| relation/rust/presentation/relation_diamond/ | `cargo run --example relation` | examples/rust/relation.rs | 否（关系查询） |
| benchmark/once_cell/ | `cargo run --example benchmark_oncecell` | examples/rust/benchmark_oncecell.rs | 是（使用预计算向量） |
| benchmark/ripgrep/ | `cargo run --example benchmark_ripgrep` | examples/rust/benchmark_ripgrep.rs | 是（使用预计算向量） |
| benchmark/flask/ | `cargo run --example benchmark_flask` | examples/python/benchmark_flask.rs | 是（使用预计算向量） |
| benchmark/{once_cell,ripgrep,flask}/bm25_parameter_sweep/ | `cargo run --example bm25_para_{oncecell,ripgrep,flask}` | examples/rust|python/bm25_para_*.rs | 否 |
| benchmark/{once_cell,ripgrep,flask}/retrieval_method/ | `cargo run --example benchmark_retrieval_{oncecell,ripgrep,flask}` | examples/rust|python/benchmark_retrieval_*.rs | 否（使用预计算向量与文本） |
| scenarios/rust/alignment/ | `cargo run --example alignment_report` | examples/rust/alignment_report.rs | 否（确定性 mock 嵌入；hybrid 路需 Qdrant） |
| debug/ | `cargo run --example debug_matrix` | examples/rust/debug_matrix.rs | 是（使用预计算向量） |
| data/benchmark/ (once_cell) | `cargo run --example gen_bench_oncecell` | examples/rust/gen_bench_oncecell.rs | **可选**（无 API key 时仅生成 chunk） |
| data/benchmark/ (ripgrep) | `cargo run --example gen_bench_ripgrep` | examples/rust/gen_bench_ripgrep.rs | **可选**（无 API key 时仅生成 chunk） |
| data/benchmark/ (flask) | `cargo run --example gen_bench_flask` | examples/python/gen_bench_flask.rs | **可选**（无 API key 时仅生成 chunk） |

## 注意事项

1. **不提交到版本控制**: outputs目录已被 `.gitignore` 忽略
2. **人工审查用途**: 这些输出仅用于人工审查，不参与自动断言
3. **重新生成**: 修改核心逻辑后，应重新运行对应示例生成输出文件
4. **对比审查**: 使用 `git diff` 观察输出变化是否符合预期
