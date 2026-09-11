# at-engine-face —— 引擎 12 符号面的 Auto 版替身 cdylib(PLAN-012)

并存形态:Rust 版 `autoterm_core.dll`(`crates/autoterm-core/src/ffi.rs`)
仍是 dist 生产引擎与对拍 oracle;本 crate 证明同一 C ABI 契约可由
Auto 源书写(004 §5⑤ 转正)。**替换生产物 = P-S 终局决策,留用户裁定。**

## 文件分工

| 文件 | 属性 | 说明 |
|---|---|---|
| `../at/engine_face.at` | **Auto 源(真身)** | 引擎面 12 符号 Auto 版(源流 auto-lang PLAN-610 capstone) |
| `src/lib.rs` | **转译产物(勿手改)** | a2r 产物入库;含 `auto_cabi_kit` 生成桥(unsafe 全居生成码) |

## 构建与验证

```bash
cargo build                                   # 产出 target/debug/autoterm_engine_face_auto.dll
# 符号对照(Rust 版 vs Auto 版,12 符号集恒等):
dumpbin //exports target\debug\autoterm_engine_face_auto.dll | findstr autoterm_engine
# 置换验证:parity 六场景跑在 Auto 面驱动下(见 PLAN-012 T3 规程)
```

## 转译再生成规程

```bash
# auto-lang 主检出(只读工具链):
auto trans -p <auto-term>/at/engine_face.at rust
# 产物 at/engine_face.a2r.rs → 复制为 at-engine-face/src/lib.rs
# (整文件自足,零手写补充;快照门禁在 auto-lang 27_c_abi/005 同源在案)
```
