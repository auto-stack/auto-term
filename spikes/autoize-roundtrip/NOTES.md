# S2 spike:a2r round-trip 实证记录(PLAN-006 / DEBTS #7)

日期:2026-09-05 · 样本:TermSession 子集(约 60 行 .at)
工具:`../auto-lang` 主检出 `target/debug/auto.exe`(与 S1 同日构建,commit f3185a7 时代)
方法:写 `.at` → 转译 → `rustc --edition 2024` 独立实编 → 运行 → 与
`crates/autoterm-core/src/term.rs` 手写形状逐构造 diff。

## 做了什么

1. `sample.at`:按手写形状写 Auto 子集——
   `enum Damage { Full, Lines(List<int>) }`、
   `#[derive(Clone, Copy, Debug)] type GridSize`、`type TermSession` +
   `ext` 方法块(`new` / `update` 含 `is` 模式匹配 / `clear_selection`)+
   `fn main` 冒烟;
2. 转译(实际可用命令,与文档写法不符,见 F6):
   `auto.exe trans -p sample.at rust -o output.rs`;
3. 产物 `sample.a2r.rs` 用 rustc 实编(转译器自带快照测试只比文本,不编译);
4. 变体 B `sample2.at`:仅给 TermSession 加显式 `#[derive(Clone, Debug)]`,
   再转译、实编、运行,隔离 F1 缺陷的归因。

## 结论速览

- 变体 B **编译通过并正确运行**:`area = 1920` / `damage: Full`
  (仅 1 条 unreachable 警告,源自 F4);
- 原样本(GridSize 显式派生、TermSession 走自动派生)**编译失败 E0369**(F1);
- 构造面 1:1:struct / enum(含数据变体)/ `ext`→impl 方法块 / `&self`
  隐式接收者 / `is`→`match` / 显式 `#[derive]` 透传 / 命名字段构造;
- 系统性变形集中在:int→i64、字段强制 pub、自动派生启发式、杂点噪声。

## 逐构造 diff(vs `crates/autoterm-core/src/term.rs` 手写)

| 手写形状 | a2r 产出(sample.a2r.rs) | 判定 |
|---|---|---|
| `#[derive(Clone, Debug, PartialEq, Eq)] enum Damage` | 自动升级为 `+ PartialOrd, Ord` | 变形(多 Ord) |
| `Lines(Vec<usize>)` | `Lines(Vec<i64>)` | 变形(int 固定 i64,usize 不可选) |
| `#[derive(Clone, Copy, Debug)] struct GridSize` | 原样透传(第 10 行) | **1:1** |
| `impl GridSize { fn area(&self) -> usize }` | `impl GridSize { fn area(&self) -> i64 }` | 1:1(除整型宽度) |
| `pub struct TermSession`(字段私有) | `struct TermSession`(类型无 pub,字段强制 `pub`) | 变形(封装丢失) |
| `match damage { … }`(穷尽即止) | `match dmg { … }` + 尾部冗余 `return`(rustc unreachable 警告) | 变形 |
| `damage: Damage::Full` | `damage: Damage::Full.clone()`(第 34 行) | 变形(多余 clone) |
| `GridSize { cols, rows }` 构造 | `GridSize { cols: cols, rows: rows }` | 1:1 |
| `println!` 输出 | `println!("area = {}", …)` | 1:1 |

## 发现清单(F1–F6)

- **F1(缺陷,组合级)自动派生 × 显式派生组合炸裂**:GridSize 显式
  `#[derive(Clone, Copy, Debug)]`(无 PartialEq),TermSession 未写派生 →
  a2r 自动给后者加 `PartialEq, Eq, PartialOrd, Ord` → 字段 GridSize 不满足
  约束 → **E0369 编译失败**。"a2r 产出 ≈ 手写"在"显式派生类型被容器引用"
  时不成立;变体 B 证明绕法可行(所有容器类型也显式写派生)。修法方向:
  自动派生前检查字段类型的显式派生面(a2r 侧小改,见 002 文档差距清单)。
- **F2(变形)int 固定映射 i64**:手写 cols/rows/行号惯用 usize;a2r 的
  `int→i64` 不可选(T4 调查:rust.rs:1531-1537 注释 "Now unified at the
  source",连 docs 里的 int→i32 表都过时了)。索引场景会在产物里积累
  `as usize` 转换噪声。
- **F3(变形)可见性**:字段一律强制 `pub`(rust.rs 侧既定行为),类型级
  `pub` 需 `#[pub]` 显式声明——本样本未写故缺失。手写的封装边界
  (TermSession 字段私有)不可 1:1 往返。
- **F4(变形)is 语句冗余出口**:穷尽 match 后仍生成尾部 `return`
  (rustc unreachable 警告佐证);手写不会保留这种死代码。
- **F5(变形)杂点噪声**:`Damage::Full.clone()`、连续双空行、
  `// Auto-generated` 头注入。逐条皆小,但字节级 diff 永远不干净。
- **F6(工具面貌)CLI 文档与实现不符 + 未知类型静默透传**:
  `docs/a2r-transpiler-guide.md:9-11` 写 `auto.exe transpile rust input.at
  output.rs`;实际 clap 定义为 `auto trans -p <path> rust -o <out>`
  (crates/auto/src/main.rs:531-539),且实测 **`-o` 未生效**——产物固定落在
  输入旁 `<stem>.a2r.rs`。更值得警惕:未知类型名**静默透传**——初版样本误写
  `list<int>`(正确拼写 `List<int>`),转译器无告警输出非法 Rust
  `list<i64>`,直到 rustc 实编才暴露。拼写错误只能靠编译期兜底。

## 对 Q3(DEBTS #8 往返约束)的含义

- **构造层面基本成立**:核心构造全部到位且产物可独立编译运行,与手写
  Rust 的差距是"风格变形"而非"结构失真";
- **字节级往返不成立**:i64 / pub / clone / 冗余 return 四类系统性变形,
  要么 a2r 增强(int 宽度可选、字段可见性、clone 消除),要么 #8 的验收
  口径定为"语义等价 + 编译通过"而非文本等价;
- **F1 必须在 #8 立项前处置**:修 a2r,或编码规约强制"所有跨引用类型
  显式写派生";
- **实证口径建议**:凡 a2r 产物的验收,以 rustc 实编 + 运行对拍为准,
  不信快照文本(本 spike 的 E0369 正是快照测试抓不到的类别)。
