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
- `javascript/` — JavaScript fixture
- `csharp/` — C# fixture
- `cpp/` — C++ fixture
- `go/` — Go fixture
- `kotlin/` — Kotlin fixture
- `scala/` — Scala fixture
- `ruby/` — Ruby fixture
- `php/` — PHP fixture
- `dart/` — Dart fixture
- `multi_language/` — 多语言混合 project fixture
- `documents/` — 文档类 fixture

各语言下可用 `type_inference/<case>/` 存放类型推断专项用例
（每个用例聚焦一种推断模式：泛型、控制流收窄、构造调用、注解等），
用例规模保持小文件单意图，便于 `TYPE_INFERENCE.md` 肉眼核查。

## 类型推断可视化与断言

- 可视化：运行 `cargo run -p cce-e2e-tests --example export_type_inference`，
  为全部 `type_inference` 用例生成 `outputs/scenarios/<lang>/structured/<case>/`
 （含 `SUMMARY.md`、按文件组织的 `<path>.txt` 与独立 `TYPE_INFERENCE.md`，
  表格列为变量/返回/收窄/类型形状），输出目录被 `.gitignore` 忽略。
- 机器断言：`src/type_inference_assert.rs` 提供
  `collect_type_bindings`（与可视化同引擎、同合并策略）与
  `assert_variable_has_type` / `assert_narrowed_has_type` /
  `assert_return_has_type` / `assert_type_snapshot_eq!`，
  在 `tests/regression/type_inference_integration.rs` 中使用。

## 添加新 fixture

1. 在 `fixtures/<lang>/` 下创建 fixture 目录
2. 在 `src/fixture.rs` 中注册 `TestFixture::<name>()` 方法
3. 同步到 `tests/fixtures/<lang>/`
4. 如果需要对应示例，在 `examples/<lang>/` 下创建示例文件
