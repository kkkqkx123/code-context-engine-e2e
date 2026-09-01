# fixtures

供 `src/`（lib 级测试、`examples/`）使用的 fixture 目录。

## 与 `tests/fixtures/` 的并行关系

- `fixtures/` — 由 `src/fixture.rs` 加载，供 lib 测试和 examples 使用。
- `tests/fixtures/` — 由 `tests/e2e/helper/fixture.rs` 加载，供 integration tests 使用。

两个目录**保持同一份内容**。添加 fixture 时须同时创建或同步到两个位置。

## 目录结构

按语言组织：

- `rust/` — Rust fixture
- `java/` — Java fixture
- `python/` — Python fixture
- `typescript/` — TypeScript fixture
- `multi_language/` — 多语言混合 project fixture

## 添加新 fixture

1. 在 `fixtures/<lang>/` 下创建 fixture 目录
2. 在 `src/fixture.rs` 中注册 `TestFixture::<name>()` 方法
3. 同步到 `tests/fixtures/<lang>/`
4. 如果需要对应示例，在 `examples/<lang>/` 下创建示例文件
