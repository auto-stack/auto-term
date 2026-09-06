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

CLI 已恢复(PLAN-010 T3:兄弟扫描有界化;亦可用 in-process 同源 API):

```bash
auto trans -p at/autoterm.at rust
# 产物 at/autoterm.a2r.rs → 复制为 at-gen/src/app_logic.rs
# (in-process 等价入口:auto-lang a2r_tests::temp_plan009_t8_transpile_at_app)
```

## 已知转译变形(006 S2 F 清单在案 + T8 新增;PLAN-010 T1/T2 后复核)

- `int` → `i64`、字段强制 `pub`、杂点 `.clone()` 噪声(006 F2/F3/F5);
- ~~`List.new()` 被错误路径限定~~ **已修(PLAN-010 T1)**:内置集合映射
  (List/Map/Set→Vec/HashMap/HashSet)豁免 use.rs 前缀;`[]` 绕开不再必要;
- `use.rs` 导入清单会丢失「声明后未再显式调用」的符号——绕开:不保留
  只导入不调用的面。

## 无环铁律(本 crate 是唯一 Cargo 边)

`at-gen → auto-lang 运行时` 是 auto-term 侧唯一的 auto-lang Cargo 依赖;
引擎经 DLL 运行期加载,auto-lang 对 auto-term 零依赖。
