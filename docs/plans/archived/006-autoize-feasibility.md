---
plan_id: PLAN-006
status: archived
feature_name: AutoTerm Auto 化前提调查(AutoUI 组件模型表达力 + a2r 往返一致性 + Rust 绑定通路)
author: [zhaopuming]
created_at: 2026-09-05T21:20:00+08:00
updated_at: 2026-09-05T18:50:00+08:00

# Leave these EMPTY here — /auto-plan:review fills them:
supersedes_spec_components: []
new_spec_components:
  - "docs/designs/002-autoize-feasibility.md: 新增"
touched_goals:
  - "goal-1: Q1 裁定——.at 组件模型不承载自定义渲染(View 封闭 28 变体),TermGrid 唯一路径 = auto-lang 原生组件(code_editor 范式四件套接线)"
  - "goal-2: Q2 裁定——auto-bindgen 立项假设不成立(系 C 头 manifest 生成器);真通路 FFI 边界 = 固有 impl+标量/字符串+≤3 arity+无闭包/泛型/trait;引擎复用形态 = Rust adapter(portable-pty 部分自动,alacritty_terminal 手写)"
  - "goal-3: Q3 定性——往返约束口径修正为'语义等价 + rustc 实编 + 行为对拍'(S2 实证 F1-F6:F1 派生组合缺陷 E0369、int→i64、字段强制 pub;字节级往返不成立)"
  - "goal-4: Q4 基线——ui-iced lib 一键可构建(dev 1m38s,224 警告 0 错误);rust-mode 示例系统性 E0080 编译失败(renderer.rs:18432,已入差距清单 3.6)"
  - "goal-5: 台账——DEBTS #7 关账、#8 启动条件/往返口径更新;W1-W4 工作量分级(25-50 人日)与 P0-P4 拆解草案就绪(go/no-go 建议分支 A,终裁留用户)"

current_step: 8
total_steps: 8
---

# [PLAN-006] AutoTerm Auto 化前提调查(AutoUI 组件模型表达力 + a2r 往返一致性 + Rust 绑定通路)

## 变更摘要

DEBTS #8(Auto 化——必须项)的前置调查(DEBTS #7),为 #8 复刻
计划产出 go/no-go 裁定材料。**纯调查型计划**(同 PLAN-001 的 spike
性质):跨仓只读调查 `../auto-lang`(AutoUI 实现与 a2r 转译器的
真正所在地——**注意:../auto-ui 独立仓已废弃合并入 auto-lang,
DEBTS 旧引用一并修正**),产出可行性设计文档 + 差距清单 + 工作量
估计;唯一动代码的是两个 spike(auto-lang 只读构建 + 最小 .at
round-trip),不改任何仓的产品代码。与并行执行的 PLAN-005
(Rust 交互批,另一会话)零冲突——本计划不碰 auto-term 产品代码。

## 目标

1. **Q1 组件模型表达力**:AutoUI(`auto-lang/src/ui/`)能否表达
   "实现 iced `Widget` trait 的自定义 widget"(TermGrid 命门:
   选中/IME/损伤门控全在其 `update/draw`),还是只能组合既有
   组件?iced 后端(`ui/iced/`)的扩展点在哪;
2. **Q2 Rust 绑定通路**:Auto 调用 Rust crate(iced/
   alacritty_terminal/portable-pty)的机制(auto-bindgen?ext
   imports/use.web?custom-lib-paths?)与边界;
3. **Q3 a2r 往返一致性**:转译器产出子集(能否 emit trait impl/
   extern 绑定?idiomatic 程度)与 DEBTS #8 的"a2r 产出 ≈ 手写
   Rust"约束的差距;以 TermSession 薄封装为样本做最小 round-trip;
4. **Q4 iced 后端可构建性**:ui-iced feature 在本机可构建/可跑
   demo,记录构建面貌;
5. **Q5 产出**:可行性设计文档(含差距清单 + 工作量分级 +
   go/no-go 建议 + #8 立项拆解草案);DEBTS #7 关账或转立项,
   #8 补齐启动条件;修正 DEBTS/001 的 auto-ui 旧仓引用。

非目标:不改 auto-term/auto-lang 产品代码;不做完整复刻(那是
#8);不做 gpui/Vue 后端调查(虚拟桌面 iced 轨道为准);不替用户
做 go/no-go 裁定(只交材料)。

## 架构方案

```
调查(只读)               产出(spikes/ + docs/designs/)
──────────────           ─────────────────────────────
auto-lang/src/ui/     ▶   002-autoize-feasibility.md
  ├ a2ui/schema.rs          ├ Q1-Q4 答案(每条带源码 file:line 证据)
  ├ ui/iced/(renderer…)     ├ 差距清单(按 #8 复刻面映射)
  ├ ui/ext_stubs.rs         ├ 工作量分级(核心封装/UI 层/引擎绑定)
  └ auto-bindgen/           ├ go/no-go 裁定材料(三分支)
                           └ #8 立项拆解草案
docs/a2r-*.md + 组件注册表   evidence/006-autoize/(spike 产物)
```

spike 两个:
- **S1 只读构建**:在 `../auto-lang` 主检出跑
  `cargo build -p auto-lang --features ui-iced`(产物仅 target/,
  零 tracked 改动);
- **S2 round-trip**:写最小 `.at` 样本(struct + enum message +
  方法,TermSession 子集形态)→ `auto.exe transpile rust` → 与
  手写 Rust 风格 diff → 差距记录。样本落本仓
  `spikes/autoize-roundtrip/`,不进 auto-lang。

## 技术栈

零新依赖;工具=读源码 + `cargo build`(auto-lang 只读)+ 
`auto.exe transpile rust`(auto-lang CLI)+ 本仓 004 定型的取证
惯例(证据目录 + file:line 引用)。

## 需求分析与背景调查

> 种子:DEBTS #7/#8(005 总排序裁定后的并行修正:调查只读、可
> 与 005 并行;#8 复刻仍留在闸后)+ spec ledger P004-3/4(选中
> 架构——TermGrid 是复刻面里最难的表达对象)。

1. **用户事实修正(2026-09-05)**:AutoUI 实现全部在
   `../auto-lang`(`../auto-ui` 独立仓已旧、已合并废弃)——本计划
   所有调查路径以 auto-lang 为准,DEBTS/001 的旧引用在 T8 修正;
2. **已核实的落点(drafting 期实地勘察)**:
   - `auto-lang/crates/auto-lang/src/ui/`——AutoUI 实现(app/
     component/event_router/dynamic/desktop_protocol…),iced
     后端在 `src/ui/iced/`(renderer.rs、selection.rs、
     selectable_text.rs、pointer_area.rs、virtual_window.rs、
     broker_surface.rs——**内置组件已含可选中文本**),由
     **可选 feature `ui-iced`** 挂载(Cargo.toml 注明"重后端,
     非 默认");
   - `src/a2ui/`——.at UI 格式 export/import/schema;
   - `docs/components/core.md`——组件注册表,逐后端状态(大量
     `iced: unknown`——本身就是差距线索);
   - `docs/a2r-transpiler-guide.md` 等 3 篇 + crates
     `a2r-std`/`a2r-actor-tests`;CLI:`auto.exe transpile rust
     input.at output.rs`;
   - `crates/auto-bindgen`——疑似 Rust 绑定生成器(Q2 主线索);
   - `src/ui/ext_stubs.rs`(Plan 442 A3)——ext imports 的 VM
     目标态(stub 语义、arity 精确约束)——绑定边界的重要细节;
3. **并行安全**:005(另一会话,worktree .wt/auto-005)改
   crates/;本计划只写 docs/+spikes/——文件级不相交;
4. **风险**:auto-lang 体量大,调查须钉着 TermGrid 的五个具体
   需求问(自定义 draw/update、每帧绘制权、剪贴板/IME shell、
   条件订阅、损伤门控数据流)——泛读必失焦。

## 详细设计

### T1 AutoUI 组件模型(只读)

读 `src/ui/component.rs`/`dynamic.rs`/`a2ui/schema.rs`/组件注册
表:组件定义形态、能否注册新组件、有无"原生/自定义 widget 挂载
点"。产出 Q1 前半(模型侧)+ file:line 证据。

### T2 iced 后端扩展点(只读)

读 `src/ui/iced/{renderer,mod,selectable_text,pointer_area,
virtual_window}.rs`:renderer 如何把组件树映射到 iced;内置组件
是"手写 Rust Widget 实现"还是"iced 既有组件包装";**从 .at 层
触达自定义 Widget 的路径存在与否**。产出 Q1 后半(后端侧)。

### T3 绑定通路(只读)

读 `crates/auto-bindgen`、`docs/custom-lib-paths-*.md`、
`src/ui/ext_stubs.rs`:Auto→Rust crate 的调用机制、类型封送、
限制(如 stub arity 约束)。产出 Q2。

### T4 a2r 能力面(只读)

读 a2r 三篇文档的 Implementation Status + `a2r-std` 公面:产出
子集清单(struct/enum/trait impl?/extern?),对照 DEBTS #8 的
往返约束。产出 Q3 前半。

### T5 S2 round-trip spike(动手,本仓内)

`spikes/autoize-roundtrip/sample.at`:TermSession 子集形态
(GridSize struct + Damage enum + begin/update/clear_selection
方法签名级)+ 运行 `auto.exe transpile rust`;人工 diff 产出 vs
`autoterm-core/src/term.rs` 风格,记录:哪些构造 1:1、哪些变形、
哪些不可表达。产出 Q3 后半(实证)。

### T6 S1 只读构建 spike(动手,auto-lang 不改)

`cargo build -p auto-lang --features ui-iced`(+ 若有 ui-iced
示例则跑一个);记录构建时长/警告面貌/示例可运行性。产出 Q4。

### T7 可行性文档汇总

`docs/designs/002-autoize-feasibility.md`:Q1-Q4 答案(带源码
证据)、**差距清单按 #8 复刻面映射**(core 封装/ui App/
TermGrid/引擎绑定四档)、工作量分级(人日量级)、go/no-go 三分
支材料(能表达→#8 直立;缺能力→auto-lang 补齐计划草案;短期
不可达→降级路线如宿主壳+Auto 面板)、#8 立项拆解草案。

### T8 台账回写

DEBTS:#7 关账(结论一句话+指向 002 文档);#8 启动条件按结论
更新;**修正 auto-ui 旧仓引用为 auto-lang 内 ui 模块**(含 001
附录相关表述);README 定位节同步。

## 测试设计

- 调查型计划:验证=每任务的产出物存在且含指定内容(grep 断言);
- S2 spike:output.rs 生成成功 + diff 记录入档;
- S1 spike:构建成功输出(或失败面貌也是结论,如实记录);
- 不做:本仓 cargo test 不动(零产品代码改动,
  `cargo test --workspace` 收尾跑一次确认无意外)。

## 验收标准

1. `docs/designs/002-autoize-feasibility.md` 存在,含 Q1-Q4 各
   一节(每节 ≥3 条 file:line 源码证据)+ 差距清单 + 工作量分级
   + go/no-go 材料 + #8 拆解草案;
2. `spikes/autoize-roundtrip/` 含 sample.at + transpile 产物 +
   diff 记录(或失败面貌记录);
3. ui-iced 构建结论在案(成功时长或失败原因);
4. DEBTS #7 关账、#8 条件更新、auto-ui 旧引用修正(grep
   "auto-ui 仓" 无残留误引);
5. 本仓 `cargo test --workspace` 仍绿(零改动自证);
6. 与 005 无文件交集(改动仅 docs/ + spikes/ + DEBTS/README)。

## 执行步骤

- [x] **T1** 组件模型调查:读
      `../auto-lang/crates/auto-lang/src/ui/{component,dynamic}.rs`
      + `a2ui/schema.rs` + `docs/components/core.md`,答案与
      file:line 证据写入 002 文档 Q1 节(草)。
      验证:`grep -c "component.rs" docs/designs/002-autoize-feasibility.md` ≥1
      [✅ 已完成] View 封闭 28 变体/component_registry 三源注册/注册表 iced 状态分布(unknown 37·none 32)入 Q1 §1.1;grep=3 通过
- [x] **T2** iced 后端调查:读
      `src/ui/iced/{renderer,mod,selectable_text,pointer_area,
      virtual_window}.rs`,自定义 Widget 路径结论入 Q1 节。
      验证:`grep -c "ui/iced/renderer" 002 文档` ≥1
      [✅ 已完成] 5 处手写 Widget 先例 + code_editor 四件套接线点(view 变体/renderer 臂 2823-4600/aura_view_builder 臂/mod.rs)入 Q1 §1.2;grep=1 通过
- [x] **T3** 绑定通路:读 `crates/auto-bindgen` +
      `docs/custom-lib-paths-{api,implementation}.md` +
      `ui/ext_stubs.rs`,Q2 节成文。
      验证:`grep -c "auto-bindgen" 002 文档` ≥1
      [✅ 已完成] 关键修正:auto-bindgen 是 C 头 manifest 生成器与 Rust crate 无关;真通路=dep/use.rust/方法包 FFI(固有 impl+标量+≤3 arity 边界);三 crate 判定表入 Q2;grep=3 通过
- [x] **T4** a2r 能力面:读 a2r 三篇 + a2r-std 公面,Q3 前半
      (产出子集清单)成文。
      验证:`grep -c "Implementation Status\|a2r-std" 002 文档` ≥1
      [✅ 已完成] 构造面清单(trait 定义+impl/泛型/match/闭包/actor 全支持)+ CLI 文档不符发现入 Q3 §3.1;grep=3 通过
- [x] **T5** S2 round-trip:写
      `spikes/autoize-roundtrip/sample.at`(TermSession 子集),
      `auto.exe transpile rust` 生成 + diff 记录入
      `spikes/autoize-roundtrip/NOTES.md`。
      验证:`spikes/autoize-roundtrip/` 含 .at + .rs + NOTES.md
      [✅ 已完成] sample.at + sample.a2r.rs + 变体 B + NOTES.md(F1-F6 六发现:E0369 派生组合缺陷/rustc 实编+运行通过/CLI 实为 `auto trans -p … rust`);实际命令 `auto.exe trans -p sample.at rust -o output.rs`(`-o` 未生效,产物落 `<stem>.a2r.rs`)
- [x] **T6** S1 只读构建:在 `../auto-lang` 跑
      `cargo build -p auto-lang --features ui-iced`,结论(时长/
      警告/失败面貌)入 002 文档 Q4 节。
      验证:002 文档含构建结论(grep "ui-iced 构建" ≥1)
      [✅ 已完成] dev 1m38s 成功/224 警告 0 错误/零 tracked 改动;示例补测:rust-mode 示例系统性 E0080 编译失败(renderer.rs:18432 单态化超限,ui_hello_loader+ui_counter 同炸)——如实入档为存量问题;grep=1 通过
- [x] **T7** 汇总 002 文档:差距清单(按 #8 复刻面四档映射)+
      工作量分级 + go/no-go 材料 + #8 拆解草案。
      验证:`grep -c "go/no-go" 002 文档` ≥1 且四档各有小节
      [✅ 已完成] §5 差距清单四档(core/ui App/TermGrid/引擎绑定,🔴3🟡8🟢6)+ W1-W4 工作量(25–50 人日)+ §6 三分支 go/no-go(建议 A)+ P0-P4 拆解草案;grep=3 通过
- [x] **T8** 台账回写:DEBTS #7 关账/#8 更新/auto-ui 旧引用
      修正;README 定位同步;收尾 `cargo test --workspace`。
      验证:`grep -A2 "Phase 4 候选清单" DEBTS.md` 含 006 结论;
      `cargo test --workspace` 绿
      [✅ 已完成] DEBTS:#7 关账(结论+指向 002)、#8 启动条件按 006 更新(语义等价口径/Rust adapter 形态/W1-W2 一相位/25-50 人日);auto-ui 旧引用修正(DEBTS/001/README,活文档零误引;归档计划为历史记录不动);README 定位/布局/下一步同步;`cargo test --workspace` 绿(24 passed,CARGO_EXIT=0);worktree 提交 babe2dc + bf6c983

## 复审记录

- **复审人**:auto-plan:review(ZCode 会话,2026-09-05 18:40 +08:00)
- **复审对象**:worktree `.wt/auto-term-006/auto-term` @ `plan-006-dev`
  (babe2dc + bf6c983,base f3185a7;diff = 9 文件,+752/-15,
  全部在 docs/ + spikes/ + DEBTS/README,worktree 干净)

**逐条验收复验(worktree 内重跑,不信任执行期勾选)**:

1. **PASS** — `docs/designs/002-autoize-feasibility.md` 存在(408 行);
   Q1-Q4 每节 file:line 源码证据计数 = 28/12/12/5(≥3);差距清单四档
   小节齐(`grep -c "^### 档"` = 4)+ 工作量分级(W1-W4)+ go/no-go
   三分支(grep=3)+ P0-P4 拆解草案均在;
2. **PASS** — `spikes/autoize-roundtrip/` 含 sample.at + sample.a2r.rs +
   变体 B(sample2.at/.a2r.rs)+ NOTES.md(F1-F6 六发现含 diff 表);
3. **PASS** — ui-iced 构建结论在案(grep "ui-iced 构建"=1):lib 成功
   (dev 1m38s,224 警告 0 错误)+ rust-mode 示例系统性 E0080 失败
   (ui_hello_loader/ui_counter 同炸 renderer.rs:18432)——失败面貌
   如实记录,符合"调查型计划"的验收口径;
4. **PASS** — DEBTS #7 关账(候选清单条目改写+结论+指向 002)、#8
   启动条件按 006 结论重写(往返口径/adapter 形态/W1-W2 一相位/
   25-50 人日);grep "auto-ui 仓" 活文档零误引——残留仅:归档计划
   001/002(冻结的历史记录,有意不动)、计划文件自身(引用验证命令)、
   DEBTS 修正性表述("原独立 auto-ui 仓已废弃");
5. **PASS** — 复审门禁重跑 `cargo test --workspace`:24 passed /
   0 failed,CARGO_EXIT=0(热缓存二次运行,与执行期收尾一致);
6. **PASS** — 与 005 零文件交集:006 分支改动 = docs/designs/002+001、
   spikes/、DEBTS.md、README.md;005 分支(plan-005-dev,已推进至其 T3)
   改动 = crates/* + docs/designs/evidence/005-interaction/*;无重合文件。

**遗漏/延后/workaround 排查**:

- 遗漏:8 个任务均有对应 diff;目标 5(DEBTS #7/#8/旧引用/README)全兑现;
  非目标(产品代码/gpui/Vue 调查/替用户裁定)全部守住;
- 延后:无未经批准的延后——W1/W2/P0-P4 等"后续"即本计划 Q5 的交付物
  (拆解草案),非偷工;
- Workaround:CLI `-o` 失效→接受默认产物位置(发现 F6,上游问题已档);
  F1 E0369 绕法(显式派生)已实证并记录;无隐藏式糊弄;
- 计划图示与步骤的小分歧:架构图把 spike 产物画在 `evidence/006-autoize/`,
  执行步骤 T5 明确写 `spikes/autoize-roundtrip/`——以步骤为准执行,
  记录备查;
- 新增债务候选(auto-lang 侧,均已入 002 §5 差距清单,无本仓动作):
  3.6 rust-mode E0080、F1 a2r 派生组合、F6 CLI 文档漂移+未知类型静默透传。

**裁定:PASS → status: reviewed**。可执行 `/auto-plan:merge`。

## 待澄清事项

1. **S1 构建位置**:建议在 `../auto-lang` **主检出只读构建**
   (产物仅 target/,零 tracked 改动)。备选:不为 auto-lang 开
   worktree(本计划不改它,无必要)。默认按建议;
2. **S2 样本深度**:建议**TermSession 子集**(struct+enum+方法
   签名级,~40 行 .at)而非整个 core——调查要的是构造覆盖面,
   不是移植。默认按建议;
3. **go/no-go 呈现**:建议**只交裁定材料**(三分支+建议),最终
   裁定留给用户——#8 立项保持用户闸门。默认按建议。
