# at-gen —— Auto 复刻应用产物 crate(PLAN-009 T8)

Auto 侧应用源 + a2r 转译产物 + 手写壳的共生目录。

## 文件分工

| 文件 | 属性 | 说明 |
|---|---|---|
| `../at/autoterm.at` | **Auto 源(真身)** | app 状态机 + 引擎驱动逻辑(纯 a2r 可表达构造) |
| `src/app_logic.rs` | **转译产物(勿手改)** | `at/autoterm.at` 经 a2r 转译入库;对拍门禁消费 |
| `src/engine.rs` | 手写胶水 | libloading 包装 `autoterm_core.dll`(P2 FFI 面安全化) |
| `src/shell.rs` | 手写窗口壳 | auto-lang ui `Component`,挂 `View::Terminal`(P1 真身组件),每拍调 `.tick()` |

## 构建与运行

```bash
# 0) 引擎 cdylib(在 auto-term workspace 根)
cargo build -p autoterm-core          # 产出 target/debug/autoterm_core.dll

# 1) 产物 crate(at-gen 自己的 target)
cargo build                            # bin = autoterm-at(GUI,ui-iced 默认开)

# 2) headless 对拍驱动(T9 门禁消费)
./target/debug/autoterm-at scenario echo   # stdout 按 ROW <n> <text> 打网格
```

## 转译再生成规程

CLI `auto trans` 暂挂(PLAN-009 待澄清9),用 in-process 同源 API:

```rust
// 放入 auto-lang 仓 crates/auto-lang/src/tests/a2r_tests.rs 跑一次:
let src = read_to_string("…/auto-term/at/autoterm.at").unwrap();
let mut r = transpile_rust("autoterm", &src).unwrap();
std::fs::write("…/auto-term/at-gen/src/app_logic.rs", r.done().unwrap()).unwrap();
```

## 已知转译变形(006 S2 F 清单在案 + T8 新增)

- `int` → `i64`、字段强制 `pub`、杂点 `.clone()` 噪声(006 F2/F3/F5);
- **新增(T8 记债)**:`List.new()` 被错误路径限定为 `<最后 use.rs 符号>::Vec::new()`
  ——绕开:空列表用 `[]` 字面量(语料 19_ownership/003 先例);
- `use.rs` 导入清单会丢失「声明后未再显式调用」的符号——绕开:不保留
  只导入不调用的面。

## 无环铁律(本 crate 是唯一 Cargo 边)

`at-gen → auto-lang 运行时` 是 auto-term 侧唯一的 auto-lang Cargo 依赖;
引擎经 DLL 运行期加载,auto-lang 对 auto-term 零依赖。
