# fixtures

供 `src/`（lib 级测试、`examples/`）与 `tests/`（集成测试）共用的唯一夹具目录。

## 单树约定

- 只有 crate 根目录 `fixtures/` 这一套夹具树，`tests/` 下不再保留单独副本。
- `src/fixture.rs` 提供唯一的 `FixtureCategory`、`FixtureSpec`、`TestFixture` 加载器；`tests/e2e/helper/fixture.rs` 仅做重导出，不重复实现。
- 添加 fixture 时只需在 `fixtures/<lang>/` 下创建目录，并在 `src/fixture.rs` 中注册对应的 `FixtureSpec`/`TestFixture` 方法。

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
2. 在 `src/fixture.rs` 中注册 `FixtureSpec::<name>()` / `TestFixture::<name>()` 方法
3. 断言性测试控制范围：只对已在测试中使用的夹具补充断言，不要求覆盖全部夹具
4. 如果需要对应示例，在 `examples/<lang>/` 下创建示例文件
