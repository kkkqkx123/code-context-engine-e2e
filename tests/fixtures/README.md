# tests/fixtures

这里存放 integration tests 专用的输入数据。

## 约定

- 只放测试输入，不放输出快照。
- 这里的数据只供 `tests/` 下的集成测试使用。
- 库 crate 根目录的 `fixtures/` 仅保留给 `src/` 里的输出检查和库级测试。

## 目录说明

- `rust/basic`：基础 Rust 项目样例。
- `rust/edge_cases`：Rust 语法边界样例。
- `rust/review/once_cell`：用于导出与展示验证的 review 样例。
- `rust/review/index_sidecar`：用于索引侧边数据验证的 review 样例。
- `rust/review/relation_demo`：用于关系输出展示的 review 样例。
- `python/basic`：Python 基础样例。
- `python/review/index_sidecar`：Python 侧边数据验证的 review 样例。
- `java/basic`：Java 基础样例。
- `java/review/index_sidecar`：Java 侧边数据验证的 review 样例。
- `typescript/basic`：TypeScript 基础样例。
- `typescript/review/index_sidecar`：TypeScript 侧边数据验证的 review 样例。
- `multi_language`：多语言样例。
