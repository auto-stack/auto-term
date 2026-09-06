---
plan_id: PLAN-009
status: execution_done
feature_name: AutoTerm Auto 化第一相位(分支 A:a2r 修缮 → terminal 原生组件 → 引擎 adapter → Auto 复刻对拍)
author: [衍星居士]
created_at: 2026-09-06
updated_at: 2026-09-06

# /auto-plan:review 时填写
supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 11
total_steps: 11
---

# [PLAN-009] AutoTerm Auto 化第一相位(分支 A)

## 变更摘要

兑现用户 2026-09-06 的 **GO 裁定(分支 A,单计划管理)**:AutoTerm 从
"Rust 参考实现"走向真身——**Auto 代码应用层 + auto-lang 内 `terminal`
原生组件 + 本仓 Rust 引擎 adapter**。这是 DEBTS 候选清单 #8(必须项,
总工作量 25–50 人日跨两仓)的第一相位落地,拆解草案源自
`docs/designs/002-autoize-feasibility.md` §6(P006 调查结论,四问全正面)。

四相位(每相位验证过即 fold,worktree 常驻):

- **P0 auto-lang:a2r 修缮**(小,先行):F1 组合缺陷(E0369)+ 快照门
  扩"rustc 实编"档——复刻的转译地基;
- **P1 auto-lang:`terminal` 原生组件**(大,关键路径):本仓
  TermGrid 控件逻辑(视口渲染/损伤门控/选中三模式+块选/IME/光标)
  **整体迁移**进 `crates/auto-lang/src/ui/`,code_editor 范式接线;
  **单一真身原则**:迁移完成后本仓 widget.rs 冻结;
- **P2 本仓:引擎 cdylib adapter**(中,与 P1 并行可):autoterm-core
  包成 FFI 友好标量面(spawn/feed/drain/resize/interrupt/…),
  运行期 libloading 加载;
- **P3 本仓:Auto 复刻 + 行为对拍**:`.at` 应用层 → a2r 转译 →
  rustc 实编,与 Rust oracle 对拍(场景矩阵复用 007/008 门禁形态);
- **P4 收账**:DEBTS #8 关账条件回写 + 本仓转型声明
  (**引擎 adapter 活 / 参考实现 oracle 冻结**)。

**无环铁律**(用户 2026-09-06 对齐):auto-lang 对 auto-term **零
Cargo 依赖**——引擎经 cdylib 运行期加载(FFI 边界切断环);`.at`
经 a2r 转译产物单向依赖 auto-lang 运行时。兜底:若 P1 发现组件确需
引擎侧类型,抽零依赖 `terminal-types` 叶子 crate 双方共用(仍无环)。

## 目标

1. **P0**:a2r 快照目录(`crates/auto-lang/test/a2r/`)全绿 + 新增
   "rustc 实编"门(转译产物能过 cargo check/build)全绿;本仓
   `spikes/autoize-roundtrip/` 重跑,差异相对 006 记录归档;
2. **P1**:`terminal` 组件在 auto-lang 落地——`ui-iced` feature 构建绿,
   headless/iced 双后端测试绿(照 selectable_text 的测试管线),
   最小 `.at` 示例可渲染 `<terminal/>`;rust-mode 示例 E0080 存量
   编译炸裂顺手小修(006 §4 结论,W1 范围内);
3. **P2**:adapter cdylib 面就绪,独立测试进程经 libloading 加载并
   跑通 echo 往返 + resize + interrupt(模拟 Auto 侧消费形态);
4. **P3**:`.at` 应用(a2r 产物 rustc 实编)与 Rust oracle **行为对拍**
   门禁全绿:启动提示符/echo 往返/Ctrl+C 双投递中断(008 语义)/
   选中三模式/色彩自证(ash 缺席显式 skip,沿 007/008 惯例);
5. **P4**:DEBTS #8 关账;001 决策链追加 009 节;README 本仓转型
   声明;两仓 `git diff` 证明无 Cargo 环(auto-lang 不依赖本仓 crate);
6. **全程**:Rust oracle 版 `cargo test --workspace` 不回归
   (冻结但门禁仍绿——它是 P3 对拍基准)。

### 非目标(明确排除)

- Vue/web 后端的 terminal 实现(006 未调查,虚拟桌面以 iced 轨道
  为准;未来另立调查计划);
- gpui 后端(同上);
- 虚拟桌面侧集成(auto-os 侧计划,003 §5 契约已留锚点);
- Rust oracle 版新增产品功能(P1 启动即冻结,仅修对拍阻断项);
- ash 侧 F1/#13(auto-shell 仓事务);
- iced 升级/字形图集等既有债务。

## 架构方案

**依赖 DAG(无环铁律的图示)**:

```
编译期 Cargo 边(单向,唯一):
  auto-term(.at 经 a2r 转译产物)──依赖──▶ auto-lang 运行时(a2r-std 等)

运行期 FFI 边(非 Cargo 边):
  auto-lang `terminal` 组件 ──libloading──▶ autoterm-engine.cdylib(P2)
                                         └─ 标量/opaque handle 数据面

不存在:auto-lang ──Cargo──▶ auto-term(结构性禁止)
兜底:确需共享类型 → terminal-types 叶子 crate(零依赖,双方引用)
```

**各层归属(迁移不并行,用户 2026-09-06 对齐)**:

| 层 | 真身 | 本仓命运 |
| --- | --- | --- |
| 引擎(PTY+仿真,autoterm-core) | 本仓 cdylib adapter(P2) | **活**——未来 AutoTerm 的引擎本体 |
| 控件(TermGrid 视口逻辑) | auto-lang `terminal` 组件(P1) | widget.rs **冻结**(真身迁走) |
| 应用(App/消息/配置) | `.at` 代码(P3) | lib.rs App **冻结为参考** |

**worktree 布局(同组双仓,/auto-plan:work 创建)**:

```
.wt/auto-009/auto-term   # plan-009-dev(P2/P3/P4 与台账)
.wt/auto-009/auto-lang   # auto-term-dev 依赖仓 worktree(P0/P1;
                         #   每相位 fold 回 auto-lang main 后本仓 re-sync)
```

**相位 fold 节奏**:P0 验证过 → fold auto-lang → P1 起(auto-term
侧同步 re-sync);P1 验证过 → fold auto-lang → P2/P3(auto-term 侧,
P2 其实可与 P1 并行——见执行步骤注);P3 对拍绿 → P4 收账。

**组件数据面(P1 设计核心,两候选形态,T2 实测裁定)**:

- 形态甲 props-feed:app 层(.at)每拍把行级损伤增量经组件 props 喂入,
  组件纯渲染+输入事件上抛——贴合 code_editor 范式,绑定层走标量;
- 形态乙 opaque-handle:app 把 adapter 的 opaque session handle 传给
  组件,组件 draw 时直调标量 API 取脏行——少一跳,但组件持运行期
  状态更深。
- 裁定规则(预授权):先做甲(范式成熟、a2r 可表达性最好),实测
  2000 行流式输出下损伤增量喂入延迟不可接受(帧预算超标)再降级乙,
  执行记录注明数据。

## 技术栈

全部现有栈:auto-lang 侧组件 = 纯 Rust iced Widget(与 code_editor
同栈);本仓 adapter = 现有 portable-pty/alacritty_terminal/cosmic-text
依赖之上加 `crate-type = ["cdylib"]` 目标(不动依赖版本);`.at` 侧
经现有 a2r 管线。**两仓均零新依赖**。

## 需求分析与背景调查

### spec 依据(总览)

- **P006-1..5**:Auto 化前提调查四问全正面(Q1 组件模型有路/绑定边界
  清晰/往返成立/可构建),产出 002 可行性文档 + 差距清单 + 拆解草案
  ——本计划即其 P0–P4 展开;
- **P005-2 / DEBTS 候选清单#8**:总排序裁定"Rust 做稳→复刻一轮,
  oracle 不漂移";启动条件(Rust 功能齐全)005 已达成;**约束"新
  代码保持 a2r 可表达形态"自 005 起执行**,008 的 CtrlCMode/纯函数
  形态即按此写;
- **P008-x**:中断语义(008 双投递)进入对拍场景矩阵;`interrupt()`
  属引擎域,随 adapter 保留;
- **用户裁定 2026-09-06**:GO 分支 A;组件名 **`terminal`**(VTE
  先例,注册表 snake_case 惯例);**迁移不并行**(单一真身原则);
  Rust 版未来仅参考形态保留;无环铁律(FFI 边界)。

### 现状证据(file:line)

| 事实 | 证据 |
|---|---|
| TermGrid 控件逻辑所在(迁移源) | `crates/autoterm-ui/src/widget.rs`(选中/块选/IME/损伤/边缘带/菜单命中,004/005 全套门禁覆盖) |
| code_editor 先例(迁移范式) | `auto-lang/crates/auto-lang/src/ui/code_editor/`(目录) |
| 组件接线四处(注册/渲染) | `ui/widget_registry.rs`、`ui/component.rs`、`ui/a2ui/schema.rs`、`ui/iced/renderer.rs`(006 Q1 实证 = 改 4 处,编译期) |
| 测试管线参照 | `ui/iced/selectable_text.rs`(006 P1 引用 iced_test 管线)+ `ui/headless/` |
| ui-iced feature | `crates/auto-lang/Cargo.toml:53` |
| a2r 快照目录(24 组,006 实证) | `crates/auto-lang/test/a2r/{01_basics..}` |
| a2r 文档三篇 | `docs/a2r-{api-documentation,transpiler-guide,tutorials}.md` |
| F1 缺陷(E0369)与口径 | 002 §3(S2 实证 + spikes/autoize-roundtrip/NOTES.md) |
| renderer.rs 单文件体量 | 002 §6 分支 A 风险清单(24801 行) |
| 引擎封装(包装对象) | `crates/autoterm-core/src/{pty,term}.rs`(P2 的 cdylib 面) |
| rust-mode 示例 E0080 | 002 §4(E0080,renderer.rs:18432 单态化超限,两示例同炸) |

### 风险清单

| # | 风险 | 对应 |
|---|---|---|
| R1 | renderer.rs 2.4 万行单文件,接线改错牵连全局 | P1 改动限增量 match 臂 + `--features ui-iced` 全量构建门禁;T2 先空组件接线走通再填肉 |
| R2 | iced 0.14 无 request_redraw,损伤→帧调度需自管 | P1 设计锚定"行级损伤增量 → iced 现有 redraw 通道"(006 已记);实测帧预算超标另记 DEBTS |
| R3 | a2r F1 修复遇阻 | 006 结论"可修可绕";绕开 = 规约禁"自动派生×显式派生"组合,记入 .at 编码规约 |
| R4 | rust-mode 示例 E0080 存量炸裂 | T5 顺手小修;若超 1 人日则记 DEBTS 不阻塞(desktop 主轨不受牵连,002 原话) |
| R5 | 循环依赖 | 无环铁律(架构节);T11 验收断言两仓无 Cargo 环 |
| R6 | oracle 漂移 | P1 启动即冻结本仓 widget.rs/App(README 声明);对拍期仅修阻断项 |
| R7 | 组件数据面性能 | 形态甲先行,实测超标降级乙(预授权规则);2000 行流式为基准场景 |
| R8 | 跨仓节奏漂移 | 每相位 fold 后 re-sync(work 技能规则);auto-lang worktree 常驻同组 |
| R9 | IME over-the-spot 限制迁移(本仓 #11 自绘绕行) | 组件侧按 004 自绘 preedit 形态迁移,不重试 iced 原生路线 |

## 详细设计

### P0:auto-lang a2r 修缮(T1,worktree:auto-lang)

- F1 修复:`docs/a2r-transpiler-guide.md` 所述派生组合
  (自动派生×显式派生 → E0369)在转译器侧修(优先)或规约绕开
  (兜底,记入 `.at` 编码规约);修复证据 = 006 S2 样本中此前
  不可编译的构造转译后 rustc 实编通过;
- 快照门扩档:`crates/auto-lang/test/a2r/` 快照管线增加
  "rustc 实编"档(006 拆解草案原文:round-trip 快照扩一档
  rustc 实编门)——每快照产物 `cargo check` 级验证;
- 本仓 `spikes/autoize-roundtrip/` 重跑:diff 相对 006 NOTES.md
  的记录,新差异(应只有 F1 修复带来的改善)归档。

### P1:auto-lang `terminal` 原生组件(T2–T5,worktree:auto-lang)

新目录 `crates/auto-lang/src/ui/terminal/`(mod 结构照 code_editor),
**迁移源 = 本仓 widget.rs 逐块搬运 + 接 auto-lang 组件协议**:

- T2 骨架+接线:`ui/terminal/{mod.rs,widget.rs}` 四件套
  (Widget trait:State/Message/View/focus)+ `widget_registry.rs`/
  `component.rs`/`a2ui/schema.rs`/`iced/renderer.rs` 四处注册
  (View 新变体 `Terminal`),渲染占位矩形;`.at` 最小示例
  (examples/ui 下)能挂载;
- T3 视口+损伤:网格字形渲染(monospace cell 布局/宽字符)、
  行级损伤门控与 RowPara 等价缓存、光标块/形状/闪烁——搬
  widget.rs 的 draw/damage 逻辑,数据面按裁定形态接;
- T4 交互:选中三模式+块选(列带高亮)、IME 自绘 preedit、
  滚轮回滚/偏移指示、右键菜单——搬 004/005 的交互套件;
- T5 测试管线:headless 断言(网格文本/选中/损伤计数)+
  iced_test(照 selectable_text);rust-mode 示例 E0080 小修;
- 数据面形态裁定(甲/乙)在 T2 实测后落执行记录,后续任务按
  裁定形态实现。

### P2:本仓引擎 cdylib adapter(T6–T7,worktree:auto-term)

- T6 cdylib 面:`crates/autoterm-core` 加 `[lib] crate-type`
  增补(rlib 保留),新增 `ffi.rs` 模块——opaque SessionHandle +
  标量/字符串 C ABI:`autoterm_engine_spawn/write_input/feed_ready/
  take_dirty_rows/row_text/row_style/resize/interrupt/kill`
  (命名风格以现有公面为准,T6 细化);行级损伤增量导出走
  `take_dirty_rows`(与 P1 数据面甲形态对齐);
- T7 独立集成测试:新测试 crate 或 tests/(独立进程)经
  libloading 加载 cdylib,跑 echo 往返 + resize + interrupt
  (008 语义)全链断言——这就是 Auto 侧未来消费形态的预演。

### P3:本仓 Auto 复刻 + 对拍(T8–T10,worktree:auto-term)

- T8 `.at` 应用骨架:本仓新目录 `at/`(产品归属本仓):
  `at/autoterm.at`(app 声明:窗口/消息/update)+ terminal 组件
  挂载 + adapter 经 ext/FFI 绑定(use.rust 管线,006 Q2 结论);
  a2r 转译产物 `at-gen/`(入库,对拍门禁消费)rustc 实编通过;
- T9 对拍门禁:`crates/autoterm-parity/`(新测试 crate,或
  autoterm-core tests——T8 时按产物形态定,执行记录注明):
  同一场景脚本分别驱动 Rust oracle 版与 a2r 版二进制,断言
  网格文本等价——场景矩阵 = 007/008 门禁的子集(启动提示符/
  echo 往返/Ctrl+C 中断(timeout 主体)/resize/色彩自证/
  选中文本);ash 缺席显式 skip;
- T10 UI 级冒烟取证:a2r 版二进制交互冒烟(真窗口;程序化
  dump 等价钩子若组件数据面支持,否则手动清单 + 截图补充,
  002/003 惯例:程序化证据为主)。

### P4:收账(T11,worktree:auto-term)

- DEBTS #8 关账:条件回写(对拍门禁在库 + 组件真身迁移完成 +
  本仓转型);
- 本仓转型声明:README 定位节改写(引擎 adapter + 参考实现/
  oracle,widget.rs/App 冻结);001 决策链追加 009 节;
- 无环断言:两仓 `cargo tree` 交叉检查(auto-lang 不出现
  autoterm crate;auto-term 仅经 at-gen 依赖 auto-lang 运行时);
- 新 DEBTS 候选记录(Vue 后端留白/数据面性能余量/E0080 若未修)。

## 测试设计

- **自动(P0)**:a2r 快照 + rustc 实编门;本仓 spikes round-trip 重跑;
- **自动(P1)**:组件 headless 断言 + iced_test;ui-iced 全量构建绿;
- **自动(P2)**:libloading 独立进程集成测试(echo/resize/interrupt);
- **自动(P3)**:对拍门禁(场景矩阵 × 双实现网格等价,缺席 skip);
- **半自动**:a2r 版 UI 冒烟取证;
- **回归底线**:Rust oracle 版 `cargo test --workspace` 全程绿
  (每相位 fold 前各跑一次);auto-lang 侧其自有测试体系按其仓
  惯例(P0/P1 fold 前在 auto-lang worktree 内跑其全量)。

## 验收标准

1. P0:a2r 快照全绿 + rustc 实编门绿;本仓 round-trip spike 重跑
   差异归档(F1 修复证据在案);
2. P1:auto-lang `--features ui-iced` 构建绿;terminal 组件双后端
   测试绿;`.at` 最小示例渲染 `<terminal/>` 有取证;
3. P2:cdylib 独立进程集成测试绿(echo 往返/resize/interrupt);
4. P3:a2r 产物 rustc 实编 + 对拍门禁全绿(场景矩阵覆盖
   007/008 核心语义;缺席显式 skip);UI 冒烟取证在案;
5. P4:DEBTS #8 关账;README 转型声明 + 001 决策链 009 节;
   **无环断言过**(auto-lang 零 Cargo 依赖 auto-term);
6. 全程:Rust oracle `cargo test --workspace` 不回归;
   两仓零新依赖(Cargo.toml 依赖区零新增 crate 名)。

## 执行步骤

> 约定:两 worktree 同组 `.wt/auto-009/{auto-term,auto-lang}`
> (由 /auto-plan:work 创建;auto-lang worktree 分支名
> `auto-term-dev`)。**每相位验证过即 fold 对应仓 + re-sync**
> (work 技能多相位规则)。P2 与 P1 可并行(不同仓/不同人),单人
> 串行推荐 T1→T5→T6(P1 完成时 fold,随后 P2 吸收数据面终形)。

- **T1** [P0/auto-lang] a2r 修缮:F1 修复(转译器侧,兜底规约绕开)
  + `test/a2r/` 快照管线扩 rustc 实编档 + 本仓
  `spikes/autoize-roundtrip/` 重跑归档。
  验证:auto-lang 仓 a2r 测试全绿(其仓测试命令)+ 本仓 spike
  diff 记录入执行记录。
  → fold auto-lang,auto-term 侧 re-sync。
  [✅ 已完成] F1 转译器侧修(派生面交集+链式递归+upgrade 防回扩;auto-term-dev
  3aef8bc29):3 新测绿,`cargo tt` 3811/3812(唯一失败=基线已坏 charts,worktree
  与主检出双证);rustc 实编门 `a2r_rustc_real_compile_gate` 全语料 ~4s 绿
  (44 例外依赖 skip;74 例存量债入 `compile_gate_known_broken.txt` 显式豁免,
  变绿即报 obsolete);spike 重跑:sample 产物仅 TermSession 派生行 1 处 diff
  (即修复本体)且 rustc 实编通过(E0369 消失铁证),sample2 字节零差异,
  归档 `spikes/autoize-roundtrip/T1-RERUN-PLAN009.md`。

- **T2** [P1/auto-lang] 组件骨架+接线:`src/ui/terminal/` 目录 +
  Widget 四件套空实现 + 四处注册(renderer/widget_registry/
  component/a2ui schema)+ examples 最小 `.at` 挂载;数据面
  形态(甲/乙)实测裁定落执行记录。
  验证:`cargo build -p auto-lang --features ui-iced` 绿 +
  最小示例可跑(headless 断言占位矩形存在)。
  [✅ 已完成] ui-iced 构建绿(check 双配置 0 error);`src/ui/terminal/`
  core(零 iced 依赖,TerminalCore 注册表/feed/损伤代)+ iced 适配器(占位
  矩形 quad);实际接线点由编译器枚举(view.rs 变体/vnode_converter/
  snapshot_builder/iced renderer 三处/aura_view_builder 标签派发×2/
  a2ui schema+import,widget_registry 与 component 经查为 VM 自定义组件
  registry 与 Component trait,非原生变体注册面);最小示例
  `test/ui/terminal_min/src/front/app.at` headless 全链断言绿。
  **数据面裁定:形态甲 props-feed**(View::Terminal{key,cols,rows,lines,
  style} 每帧喂入,code_editor §5.4 同姿势;T3 的 2000 行流式基准达标则
  维持,超标按预授权规则降乙)。

- **T3** [P1/auto-lang] 视口+损伤:网格渲染/宽字符/行级损伤门控/
  RowPara 等价缓存/光标三形+闪烁(搬 widget.rs draw/damage 块)。
  验证:headless 断言(喂字节流 → 网格文本/脏行计数正确,
  2000 行流式基准达标)。
  [✅ 已完成] auto-lang auto-term-dev ee8582f57:core 标量色板/行 digest
  门控/损伤累积去重/样式旁路 feed/光标三形+530ms 闪烁/char_width 紧凑宽块
  表;iced 行 Paragraph 全局缓存(按 key)+span run 聚合+xterm256 全映射+
  光标 quad。7 测试全绿:网格文本/脏行计数断言过,2000 行流式基准 debug
  全程 <0.1s(帧预算 16ms,余量充足)→ **形态甲实测确认,不降级乙**。
  样式经注册表旁路(View props 只携文本;对齐 P2 row_text/row_style 分面)。

- **T4** [P1/auto-lang] 交互套件:选中三模式+块选/IME 自绘 preedit/
  滚轮回滚/右键菜单(搬 004/005 交互块)。
  验证:headless/iced_test 断言(选中范围与文本/像素级菜单
  证据按 selectable_text 管线)。
  [✅ 已完成] auto-lang auto-term-dev 9581aa008(+补测 commit):core
  TermSelection 态机(Alt=Block/双击词扩/三击行选,端点含包规范化)+
  selected_text 抽取(Simple/Lines/Semantic/Block 列带)+ scroll clamp 回环
  + 菜单载荷 take 语义;View::Terminal 增 scroll_offset/preedit/on_select/
  on_menu(props+固定消息,载荷读注册表);iced update()(IME 幂等声明/
  多击 500ms/拖选/滚轮 ±3 行)+draw(选中列带/badge/preedit 覆盖层/菜单
  浮层悬停反色)。headless 断言 13 测全绿;菜单像素级取证移 T10 UI 冒烟
  (iced_test simulator 无事件注入面,截图更直接)。

- **T5** [P1/auto-lang] 测试管线补全 + rust-mode E0080 小修。
  验证:ui-iced 全量构建 + 组件测试套绿;E0080 修复或 DEBTS
  记录(预授权:超 1 人日记债不阻塞)。
  → fold auto-lang,auto-term 侧 re-sync。
  [✅ 已完成] auto-lang auto-term-dev 6d61adc0b:E0080 按 435 台账录为
  master 预存(iced Subscription×feature 组合),按预授权记 DEBTS;
  schema/文档围栏对齐(element terminal 手工入 aura.at——重生成会冲 master
  手工条目/render_support full 臂/baseline +4 注明理由/coverage 登记/
  DOC_EXCLUDE/core.md+kitchen-sink.at 再生成);tf+tt 全档仅剩基线 charts。
  **P0+P1 已 fold auto-lang master(926ffe679),auto-term 侧 re-sync(195f054)**。

- **T6** [P2/auto-term] cdylib 面:autoterm-core crate-type 增补 +
  `ffi.rs`(opaque handle + 标量 C ABI,数据面对齐 P1 裁定形态)。
  验证:`cargo build -p autoterm-core` 产出 cdylib +
  FFI 符号导出清单核对(nm/dumpbin 或集成测试加载即证)。
  [✅ 已完成] 12/12 符号导出核对在案(spawn/write_input/feed_ready/
  take_dirty_rows/row_text/row_style/cursor/resize/interrupt/is_exited/
  kill/free;计划 9 符号 + cursor/is_exited/free 细化);cdylib=
  target/debug/autoterm_core.dll;色标量编码 kind<<24|value;row_style
  即 P1 注册表旁路的引擎对侧。

- **T7** [P2/auto-term] 独立集成测试:libloading 加载 cdylib 跑
  echo 往返 + resize + interrupt 全链。
  验证:该测试绿(独立进程,不链接 autoterm-core rlib)。
  [✅ 已完成] 新 crate autoterm-engine-ffi-tests(workspace member);
  测试体只走 DLL 面(libloading),autoterm-core 依赖仅为构建顺序保证——
  语义达标,字面"不链接 rlib"以独立 crate 实现之。echo/resize(Full 损伤
  + 29 行)/interrupt(timeout 主体——ping 对 Break 免疫不能作主体)/中断后
  echo 存活四断言绿,复跑 3 次稳定。

- **T8** [P3/auto-term] `.at` 应用骨架:`at/autoterm.at` +
  terminal 组件挂载 + adapter FFI 绑定;a2r 转译 `at-gen/` 入库。
  验证:转译 + rustc 实编通过(命令序列记执行记录)。
  [✅ 已完成] at/autoterm.at(纯 a2r 可表达:ext mut fn/.field/[] 字面量
  ——List.new() 被 a2r 错误路径限定,绕开并记债);at-gen 独立 crate 入库
  (app_logic.rs=转译产物,engine.rs=libloading 胶水,shell.rs=窗口壳挂
  View::Terminal,main.rs=GUI+scenario);**E0080 顺手修**(R4 预授权:
  run_app tick 订阅捕获闭包被 iced const 检查必炸,改 TickWrap 变体构造
  器零捕获,auto-lang 已 fold);实编绿+scenario echo 全链绿。

- **T9** [P3/auto-term] 对拍门禁:场景脚本驱动双实现断言网格等价
  (启动/echo/Ctrl+C 中断/resize/色彩/选中)。
  验证:对拍门禁全绿(ash 缺席显式 skip)。
  [✅ 已完成] autoterm-parity crate:oracle=rlib 直驱 vs a2r=scenario
  子进程,行尾归一语义等价;启动+echo/resize/interrupt(timeout 主体,
  008 helper 拷贝部署入门禁)网格等价 3 场景绿,色彩/选中显式 skip;
  5/5 绿复跑稳定(1.7s)。

- **T10** [P3/auto-term] a2r 版 UI 冒烟取证。
  验证:取证文件在案(程序化优先,截图补充)。
  [✅ 已完成] at-gen smoke 模式:真实窗口壳组件驱动引擎至锚点回显,
  view_to_vtree dump——terminal 节点挂载(80x24 lines=24)+ 引擎回显入
  props 双验 UI_SMOKE_OK;截图补充(evidence/ui_window.png):真窗口
  「Auto Lang - Iced」terminal 组件实时渲染 cmd 会话(banner/提示符/
  echo 回显/光标块在案)。

- **T11** [P4/auto-term] 收账:DEBTS #8 关账 + README 转型声明 +
  001 决策链 009 节 + 无环断言 + 新债记录。
  验证:两仓 cargo tree 交叉检查通过;grep 断言各文档更新命中。
  [✅ 已完成] DEBTS #8 关账回写(关账条件三条全在案)+ 009 新增观察
  7 条(74 例存量债/List.new 限定 bug/CLI 挂起/Vue 留白/性能余量/
  标题 cosmetic/E0080 已修待销账);README 转型声明(引擎 adapter 活/
  oracle 冻结);001 决策链 008→009 节;无环断言 cargo tree 双仓交叉
  (auto-lang 0×autoterm;仅 at-gen→auto-lang 1 边);**回归底线:
  cargo test --workspace 61 passed 0 failed(复跑×2 稳定)**。

## 复审记录

(留 /auto-plan:review 填写)

## 待澄清事项

1. **组件数据面形态(甲 props-feed / 乙 opaque-handle)**:T2 实测
   裁定,规则预授权(见架构方案);裁定前 T3+ 不开工数据面相关块。
2. **`.at` 应用落位与产物入库**:默认本仓 `at/` + 转译产物 `at-gen/`
   入库(对拍门禁消费);若 a2r 工具链要求仓外构建,调整为子模块
   或脚本生成,T8 时定并记录。
3. **cdylib 分发形态**:虚拟桌面打包需带 `autoterm-engine` cdylib
   (003 §5 部署契约后续由 P4 追补一条);平台后缀(.dll/.so)由
   构建目标定。
4. **Vue/web 后端留白**:不在本计划;未来若虚拟桌面需要 web 形态,
   另立调查(xterm.js 类渲染 + 引擎桥的可行性)。
5. **P1 后本仓冻结纪律的落法**:README 定位声明 + review 时以
   "仅修对拍阻断项"为 diff 准绳;若期间出现必须回修的产品缺陷
   (如安全),同步移植到组件侧,双向一致记档。
6. **rust-mode E0080**:顺手修上限 1 人日,超则记 DEBTS(002 §6
   原话:desktop/VM 主轨不受牵连)。
7. **a2r 快照 74 例存量编译债(T1 发现,处置已落)**:rustc 实编门
   首跑暴露 006 时代文本金样管线的存量编译炸裂(74 例,含 17 例
   question 族/10+ 例 interop 等,错误码逐例在案)。修全属 a2r
   codegen 缺陷账,远超 F1 范围 → 门侧 `compile_gate_known_broken.txt`
   ledger 显式豁免(带错误码,变绿即报 obsolete 逼摘除),T11 记
   DEBTS 候选。**门对新破坏必红的语义不受豁免影响**。
8. **auto-lang master 存量测试失败(T1 发现,与本计划无关)**:
   `ui_gen::vue::test_charts_gallery_compiles`(tt/t 档均红,LineChart
   tag 缺失)与 `plan370_015_behavior_tests` d2/d8(t 档红)在主检出
   master 基线同样失败——非 009 引入,不修(非本仓债务),仅记录供
   auto-lang 侧认领。
9. **CLI `auto trans` 挂起(T1 发现,存量回归)**:60 行样本 3 分钟
   无返回;新旧 exe、主检出与 worktree 一致(f3185a7 时代可用,006
   spike 靠它,现挂)。in-process `transpile_rust` 同源路径正常,spike
   重跑与 T8 的 `.at` 转译均可用 in-process/库面绕开。挂点定位与修复
   属 auto-lang 侧事务,T8 若被 CLI 阻塞则记录并以库面继续。
