---
plan_id: PLAN-023
status: archived
feature_name: 交互投递死亡根因调查与根修——分屏双浮层首轮交互后全灭(iced Stack/opaque/capture 链)
author: [zhaopuming/zcode-session]
created_at: 2026-09-19T00:00:00Z
updated_at: 2026-09-19T00:00:00Z
plan_revision: 1
current_step: 6
total_steps: 6
supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-lang/ui/design/overlay-interaction.md(add,SD-01 投递语义五规则+测试断言纪律)
  - docs/specs/auto-lang/ui/architecture.md(modify,ADR-22 索引)
  - docs/specs/terminal-widget-chrome.md(互引备案:022 SD-01 命中区几何为本篇下层消费方,无文本改动)
touched_goals: []
---

# PLAN-023 · 交互投递死亡根因调查与根修

## 0. 变更摘要

来源 = PLAN-022 T-05 用户实点反馈 + headless 确定性复现(auto-lang
`plan-022-dev` e5ddf41ba `p022_stack_click_tests.rs`);用户裁定
2026-09-19 独立立项("复现测试已就位,比之前盲修快得多")。

症状:分屏双浮层(absolute 叠层 → iced Stack)场景下,鼠标按压被
系统性吞掉——模拟器内交替点击两浮层,第一轮后计数冻结(实测轨迹
A=1/B=3 型:每按钮最多收到首击);实车对照 = rust 载具分屏后右面板
聚焦输入、滚动条拖拽、滚轮滚动全灭,连终端组件都收不到点击事件,
聚焦永远切不过去。

已排除(022 T-05 实验固化,勿重复试错):
1. Opaque 满幅捕获——移除 opaque 包装轨迹不变(e5ddf41ba 提交语);
2. 终端 mouse_interaction 光标抬升级联——修后无效(8d56cf936 埋点);
3. Init 双跑 / 载具路径重置——022 已定界为独立陷阱(f450d95、§10.6),
   与交互投递无关。

本计划 = ①T-01 有界调查(判决工件);②按判决根修;③复现测试转正;
④实车交互恢复(022 T-06 解锁);⑤浮层投递语义 spec 沉淀。

## 1. 目标

- G1 根因判决工件:交互投递死亡机理可复述(一次按压从注入到
  widget.update 的每一跳:事件 → 分发者 → 捕获态变化 → 终点),
  符号级证据,归属三选一或多因组合——**iced 上游 bug / auto-lang
  into_iced 映射误用 / iced_test 模拟器伪影**,并回答"模拟器现象
  与实车症状是否同根"。
- G2 复现测试转正:`p022_repeat_clicks_on_both_layers` 去
  `#[ignore]`,改为判定式断言(三轮交替点击 A=3/B=3),进常规
  CI 面。
- G3 实车交互恢复:rust 载具右面板聚焦输入、滚轮滚动、滚动条拖拽
  三动作恢复(即 022 T-06 门)。
- G4 修复制度化:若判上游 → fork/[patch.crates-io] 或升级的决策
  记录;若判本仓 → into_iced 根改 + 回归用例覆盖。
- G5 规范沉淀:浮层(absolute/Overlay → Stack+opaque)交互投递
  语义入册,终结该层"凭直觉修"的历史。

### 非目标

- PLAN-653 的 review/merge(独立轨道,另行推进);
- 022 其余任务(T-06/T-07 等 AC-03 达成后由 022 恢复);
- vue 轨交互面(653 已覆盖其空白根因,交互缺陷另有归属);
- 非 Stack/浮层路径的一般交互问题。

## 2. 架构方案

调查先行(T-01 有界调查,判决工件),三轨取证:

1. **模拟器轨**:iced_test-0.14.0 事件注入生命周期
   (simulator.rs `simulate`/`click`/`point_at`,emulator.rs,ice.rs
   Shell 实现)——重点:`Shell::is_event_captured` 标志跨事件是否
   复位、由谁复位。grep 证据(2026-09-19):capture_event/
   is_event_captured 在 iced_core/runtime/program/test 四包内**仅
   定义无调用点**(iced_core shell.rs:45-61;唯一读者为
   iced_widget stack.rs:262)——真实复位路径未定位,头号疑点。
2. **分发原语轨**:iced_widget-0.14.2 stack.rs:231-262 `Stack::update`
   子节点逆序遍历 + `is_event_captured()` 短路;helpers.rs
   `pub fn opaque` 捕获语义(无条件捕获 vs bounds 判定——决定
   顶层浮层是否吞掉落点在下层的事件)。
3. **实车对照轨**:auto-term 生成 main.rs 同构最小面——实车症状与
   模拟器现象是否同一机理。载具复建按 022 §10.6 配方(注意
   Cargo.toml 重指陷阱),AUTO_MA_DBG 埋点面已有(8d56cf936)。

判决矩阵 → 修复分叉(T-02,详设计在判决工件后细化,计划内演进):

- **上游 bug** → 最小 vendored patch 走 `[patch.crates-io]`,或勘验
  0.14.x patch/0.15 是否已修走升级;
- **本仓映射误用** → renderer.rs 根改,四处消费点一次收口:
  build_floating_layer:2614-2629、Stack 组装 4026-4038/4102-4113、
  Overlay 臂 4544-4549;
- **模拟器伪影**(实车不同根)→ 补齐实车复现手段,修实车链路,
  测试标注修偏并另立实车回归。

## 3. 技术栈

- auto-lang(主):ui/iced renderer.rs(into_iced Stack/opaque 组装)、
  p022_stack_click_tests.rs 复现与回归面 —— worktree 流程;
- iced 0.14(crates.io,只读调查对象):若需补丁则 vendored fork +
  `[patch.crates-io]`,不直接改 registry 源;
- auto-term(辅):实车对照(生成 main.rs + AUTO_MA_DBG 埋点)、
  022 T-06 验收面 —— 主检出直落。

## 4. 需求分析与背景调查

### 4.1 授权记录

用户 2026-09-19:"先继续检查,把'交互投递死亡'作为独立调查计划
立项(复现测试已就位,比之前盲修快得多)"。授权范围 = 调查 + 根修
+ 测试转正 + 实车验证;仓 = auto-lang(worktree)+ auto-term
(主检出)+ 必要时 iced vendored fork。无用户明示预算上限;调查任务
按有界(bounded)执行,T-02 修复形态视判决工件,若涉 fork 维护/
升级跨度等成本决策,在工件中列显式决策项归用户(见 §10.1)。

### 4.2 复现测试在册

- 位置:`D:/autostack/.wt/lang-022/auto-lang/crates/auto-lang/src/
  ui/iced/p022_stack_click_tests.rs`(branch plan-022-dev,提交
  e5ddf41ba;worktree/分支生命周期现由 022 管理,归属移交见 §10.3);
- 运行:`cargo test -p auto-lang --features ui-iced,iced-layout-tests
  --lib p022_stack`;
- 两用例:`p022_both_absolute_float_layers_clickable`(单轮双击,
  已通过——单轮投递无恙,死亡在第二轮起)与
  `p022_repeat_clicks_on_both_layers`(三轮交替,#[ignore],panic
  打印轨迹,实测 A=1/B=3 冻结)。

### 4.3 已勘符号锚点(2026-09-19 背景调查实录)

- `iced_core-0.14.0/src/shell.rs:45-61`:`Shell::capture_event()` /
  `is_event_captured()`——capture 概念唯一定义处,注释明言
  "Prevents event bubbling";
- `iced_widget-0.14.2/src/stack.rs:231-262`:`Stack::update` 逆序
  (顶层先)更新,`is_event_captured()` 即短路——capture 的唯一
  消费者;
- `iced_widget-0.14.2/src/helpers.rs`:`pub fn opaque` 定义(捕获
  边界语义待 T-01 勘定);
- `iced_test-0.14.0/src/{simulator,emulator,ice,instruction}.rs`:
  模拟器四件套;四包 grep 无 capture 复位调用点(见 §2.1);
- auto-lang renderer.rs 消费点(前述 §2 判决矩阵);
- 实车:auto-term `app/` 生成 main.rs(untracked,重生成即探针
  消失)+ AUTO_MA_DBG 埋点(8d56cf936 FOCUS/KEY/WHEEL/TERM_PRESS)。

### 4.4 关联计划

- PLAN-022(executing 5/8):T-06 依赖本计划 AC-03;其 SD-01
  命中区归属规则与本案同层,合流点见 §10.2;复现测试与 e5ddf41ba
  现居 022 分支,归属移交见 §10.3。
- PLAN-653(execution_done,`plan-653-dev` 未合并):与本计划
  rust 轨修复无冲突面(022 复核记录 2026-09-19 在案);复现测试
  ignore 标记中的"归属 653 T-00"表述由本计划承接取代(022 记录
  保留为历史,不改写)。

### 4.5 约束

- iced 来自 crates.io,不可直接改 registry 源;补丁必须走 vendored
  fork + `[patch.crates-io]`;
- auto-term 主检出 tracked 零 WIP 惯例(020/021/022 连续在案);
- 载具重生成会冲 `app/rust-workspace/Cargo.toml` 路径覆盖(022
  §10.6 陷阱,重生成后必须重指);
- 与并行会话(lang-026/lang-642/lang-647/lang-656 在途)的 auto-lang
  冲突面:T-01 开工前按 022 惯例重新勘定。

## 5. 详细设计

### 5.1 T-01 判决工件结构

产物 `docs/plans/evidence/023/t01-verdict.md`(auto-term 侧)+ 复跑
指引(测试命令 + 期望轨迹):

1. 三轨证据表:每轨(file:symbol)× 观测 × 推断;
2. 单次按压全跳事件流:注入(point_at/click)→ 模拟器分发 →
   Shell capture 态迁移 → Stack 逆序遍历 → 终点 widget(含
   "谁吃了事件、为什么");
3. 判决:上游 / 本仓 / 伪影(或多因),附"模拟器与实车同根性"
   结论;
4. 修复路径建议 + 显式决策项(若涉成本权衡,归用户);
5. 复跑指引:第三方按指引能在本机复现冻结轨迹与根因断点。

### 5.2 T-02 根修(依赖 T-01,形态待判决)

三岔预案(骨架,不预填实现细节):

- 上游:vendor iced 相关 crate(s) → 最小补丁 → `[patch.crates-io]`
  接线(auto-lang 或 rust-workspace workspace 级)→ 复现测试行为
  翻转;或升级勘验通过则直接 bump;
- 本仓:renderer.rs 四消费点根改(投递语义对齐 CSS absolute 命中:
  落点在谁的事件归谁,捕获不跨轮泄漏);
- 伪影:修实车链路 + 测试标注修偏,实车回归另立。

### 5.3 T-03 测试转正与回归面(依赖 T-02)

- `p022_repeat_clicks_on_both_layers` 去 ignore,断言改判定式:
  三轮交替 A=3/B=3(轨迹无冻结);
- 追加回归(用例面视 T-01 判决裁剪,防过度设计):①单浮层连击
  5 次=5;②先右后左顺序反转;③浮层 + 终端 widget 组合(终端收
  得到点击);④滚轮事件穿透 scrollable 层。

### 5.4 T-04 实车验证(依赖 T-02)

载具复建按 022 §10.6 配方(worktree codegen → 重生成 main.rs →
Cargo.toml 重指 → AUTOTERM_ENGINE_DLL 挂载),AUTO_MA_DBG 埋点验证
FOCUS/KEY/WHEEL 到达终端组件;用户实点门(022 T-06 三动作)。

### 5.5 T-05 spec 沉淀(依赖 T-01/T-02)

见规范增量表;与 022 SD-01(命中区归属)的合流/互引关系在此定。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/ui/architecture.md(auto-lang 仓;若勘验后判定独立成篇,改 design/overlay-interaction.md 新篇,架构文留索引) | before:浮层(absolute/Overlay→Stack+opaque)投递语义无成文规则,历轮凭直觉修(Opaque 满幅/光标抬升假设皆错);after:判决固化的投递语义——落点归属、capture 生命周期(谁置位/谁复位/跨轮是否泄漏)、模拟器与实车一致性保证 | 四层嵌套交互缺陷的最后一层,沉淀后终结盲修 | AC-01/AC-05 |
| SD-02 | add(视 T-01 判决增设) | 同上或 auto-term 侧 specs | before:无 iced 上游补丁/升级策略;after:fork 维护面、patch 接线点、升级勘验记录的成文策略 | 上游归属时防 fork 漂移 | AC-04 |

`new_spec_components` frontmatter 为暂填,review 定稿;SD-02 视判决
增设,不预设。

## 6. 测试设计

- 转正面:`p022_repeat_clicks_on_both_layers` 去 ignore + 判定式
  断言(§5.3);
- 回归面:新增用例四类(§5.3),数量视判决裁剪;
- CI 面:`cargo test -p auto-lang --features ui-iced,iced-layout-tests
  --lib p022`(转正后常规跑,auto-lang worktree 流程);
- 实车面:022 T-06 门(用户实点三动作:右面板聚焦输入/滚轮滚动/
  滚动条拖拽)+ AUTO_MA_DBG 探针面(FOCUS/KEY/WHEEL 到达日志);
- 防回退:判决工件复跑指引入 evidence/023,根因断点可复验。

## 7. 验收标准

- **AC-01** 根因判决工件存在且可复述:`docs/plans/evidence/023/
  t01-verdict.md` 含三轨证据表 + 单次按压全跳事件流 + 归属判定
  (上游/本仓/伪影)+ 同根性结论 + 复跑指引;验证 = 按复跑指引
  可在本机复现冻结轨迹与根因断点。
- **AC-02** 复现测试转正:`p022_repeat_clicks_on_both_layers` 无
  `#[ignore]`,断言三轮交替 A=3/B=3;`cargo test -p auto-lang
  --features ui-iced,iced-layout-tests --lib p022_stack` 全绿;
  验证 = 测试输出。
- **AC-03** 实车交互恢复:rust 载具(022 §10.6 配方重构建)右面板
  聚焦输入、滚轮滚动、滚动条拖拽三动作可用;验证 = 用户实点确认
  (022 T-06 门,用户为验收人)。
- **AC-04** 修复制度化:上游归属 → fork/[patch] 或升级决策记录
  在案(SD-02)且带补丁构建可复现;本仓归属 → 根改提交 + 回归
  用例覆盖;验证 = 构建命令 + 提交 SHA。
- **AC-05** spec 沉淀:SD-01 落册(auto-lang 仓 specs);验证 =
  spec 文件 + 账本刷新(auto-plan:merge 面做)。

## 8. 执行步骤

- **T-01** [x] 有界调查·判决工件(AC-01):三轨取证(§2),产物
  evidence/023/t01-verdict.md + 复跑指引;验证 = 指引可执行、判决
  有符号级证据。依赖:无。归属:auto-lang/auto-term 双侧只读取证
  (+iced registry 只读),不产代码改动。
  [✅ 已完成 2026-09-19] 判决 = **本仓映射误用(opaque 捕获边界×
  spacer 几何错位)+ 测试伪影(on_click 构建期调闭包)双案,iced
  上游无 bug**;同根性 = 同根(实车按压死与模拟器冻结同收敛于
  opaque 边界≠内容矩形)。证据:t01-verdict.md(13 项符号级证据表
  + 全跳事件流)+ t01-probe-output.txt(探针矩阵)+ 023 worktree
  examples/p023_probe.rs(复跑指引第 2 步)。勘误:§0"已排除 1"
  反转、"A=1/B=3"实为 A=1/B=1 双冻结(计数恒 1 伪影)。
- **T-02** [x] 根修(AC-04):依赖 T-01;按判决三岔(§5.2);若涉成本
  决策,先出决策项归用户;验证 = 复现测试行为翻转(冻结消失)。
  [✅ 已完成 2026-09-19] 本仓支:renderer.rs 五消费点(build_floating_layer
  定义 + Row/Column builder 臂 + Row/Column dynamic 臂 + Overlay 臂)
  opaque 下沉 content 级,Some 分支去外层包装,零偏移分支维持;
  top 偏移零宽 Space 改 Column padding(iced flex 零宽空间主轴
  失效,R8 实录)。SD-02(上游补丁策略)撤销。验证 = 探针行为
  翻转:R1 六击全达、R3 载具形状 msgs=[1,2,7,8]×2、R4 穿透
  [1,0,1]。code_commit: auto-lang plan-023-dev fc6ccc810。
- **T-03** [x] 测试转正 + 回归面(AC-02):依赖 T-02;验证 = CI 命令
  全绿。
  [✅ 已完成 2026-09-19] `p022_repeat_clicks_on_both_layers` 去
  ignore,断言改消息流 [A,B]×3(计数闭包构建期执行一次,原断言
  语义不可达,判决工件 §6);新增回归三钉:连击 5/5、反转序、
  spacer 空白区穿透(根修钉)。滚轮穿透用例按判决裁剪(opaque
  从不捕获滚轮,死亡链在按压/焦点,T-04 实车覆盖)。CI 面:
  `cargo test -p auto-lang --features ui-iced,iced-layout-tests
  --lib p022_stack` 5/5 绿;ui::iced 模块面 292 绿/8 失败,经
  stash 对照全部**存量于基线 e5ddf41ba**(含终端焦点门控为调度型
  flaky,基线 solo 同败),本计划零新增。同 fc6ccc810。
- **T-04** [~] 实车验证(AC-03):依赖 T-02(+022 §10.6 配方 + Cargo.toml
  重指 + 653 merge 状态核对);验证 = 探针日志 + 用户实点三动作。
  [自动化面 ✅ 2026-09-19] 载具复建全链完成:023 worktree codegen
  (fc6ccc810)→ `auto build -r rust` 重生成 main.rs → Cargo.toml
  重指 `../../../.wt/lang-023/auto-lang/crates/auto-lang`(untracked
  生成件,零 tracked WIP)→ 根 target 构建 auto-term.exe。烟测
  `evidence/023/t04-smoke.ps1`(AUTO_MA_DBG=1):**三动作探针全活**
  ——①右面板聚焦输入:TERM_PRESS/P22-FOCUS/P22-KEY key=pane-2 +
  shell 执行回显(s4 截图);②滚轮:P22-WHEEL key=pane-2(历史行
  造取受 SendKeys 转义所限,视觉滚动留实点);③分隔条拖拽:几何
  随动(pane-1 cols 86 / pane-2 cols 48)+ Press/Drop 投递。022
  "右面板死/终端收不到点击"症状面翻转。证据:t04-smoke.err +
  t04-s1..s6 截图。
  [用户实点门 R1 → 发现新缺陷 → 已根修 2026-09-19] 用户实测:
  滚轮/分隔条/右槽输入均活,但 **键盘输入双播**(右槽打字左槽
  同步、Enter 两边同跑)。根因:per-widget `state.focused` 在他端
  换焦后残留 true(iced 键盘事件广播全树,无人清旧持者标志),
  键盘门控双放行。根修 7d070317b:门控改查注册表 owner 单一事实
  源(`terminal_is_focused` 新增),`state.focused` 降级为逐拍同步
  缓存;terminal_input 老测试确定性化(原靠并行占位纯竞态,solo
  必挂,023 甄别在案)+ 注册表互斥锁串行化 +
  `p023_focus_switch_single_cast` 双播回归。烟测复测:pane-1 恰收
  13 键(=其持焦期键入),pane-2 收其余,字符直方图零翻倍——
  单播实证。载具已重建并交付用户复测。
  [用户实点门 R2 ✅ 2026-09-19] 用户复测通过:三动作(右面板聚焦
  输入/滚轮滚动/分隔条拖拽)+ 双播复测(右槽打字左槽静止)全部
  达标,AC-03 关门(验收人 = 用户)。裁定"转 review"。
- **T-05** [x] spec 沉淀(AC-05):依赖 T-01/T-02;验证 = spec 文件
  + delta 表定稿。
  [✅ 已完成 2026-09-19] SD-01 落册:`docs/specs/auto-lang/ui/design/
  overlay-interaction.md` 新篇(捕获边界=内容矩形/capture 每事件
  复位/top 偏移 padding 制/levitate 分层悬停/模拟器同轨 + 消息流
  断言纪律)+ 架构文 ADR-22 索引;与 022 SD-01 分层互引(本篇定
  层间归属,彼篇定层内归属)。SD-02 撤销(无上游归属)。worktree
  准备,随 merge 发布。code_commit: 6c8a4f3d0。
- **T-06** [x] 收口·022 交接:依赖 T-03/T-04/T-05;022 T-06 恢复条件
  达成记录入 022 复审记录;验证 = 022 状态推进可执行。
  [✅ 已完成 2026-09-19] 022 复审记录已备案:AC-03 交付(三动作 +
  单播),其 T-06/T-07 解锁;复现测试归属移交 023(已转正,022
  in-file 记录保留为历史);023 分支合流序 = 022 fold 前先并 023
  (023 基于其 plan-022-dev)。

## 9. 复审记录

- 2026-09-19 draft handoff:`stage: new`,PLAN-023 rev1。
  `outcome: pass`(起草授权 = 用户 2026-09-19 立项指令,§4.1
  在案;T-01 为首批可执行项,T-02+ 形态视判决工件,计划内演进,
  同 022 T-00 先例)。`next: work`(work 前按惯例
  `/auto-plan:review`;headless 复现 e5ddf41ba 已在册,首战即
  有靶)。
- 2026-09-19 stage: work | plan_id PLAN-023 | rev1 | outcome:
  **pass(T-01/T-02/T-03 完成,T-04 进行中)** | code_commit:
  auto-lang `plan-023-dev` fc6ccc810(基线 plan-022-dev e5ddf41ba;
  worktree D:/autostack/.wt/lang-023/auto-lang;依赖 worktree
  lang-023/auto-down detached @ a615d69 只读)| task_ids T-01/T-02/
  T-03 | evidence: docs/plans/evidence/023/{t01-verdict.md,
  t01-probe-output.txt} + 023 worktree p023_probe | blockers:
  无(§10.1 上游成本决策项因判本仓而撤销;§10.3 基线裁定 =
  023 基于 plan-022-dev,合流序 023 先落,已向 022 备案见 T-06)
  | next: T-04 实车验证(载具复建 §022 §10.6 配方 + AUTO_MA_DBG
  探针 + 用户实点三动作门)→ T-05 spec → T-06 收口。
- 2026-09-19 stage: work | plan_id PLAN-023 | rev1 | outcome:
  **blocked(T-04 自动化面 + T-05 完成,余 AC-03 用户实点门)** |
  code_commit: plan-023-dev {fc6ccc810, 6c8a4f3d0, 7d070317b} | task_ids
  T-04[自动化 ✅]/T-05 ✅;T-06 待门 | evidence:
  evidence/023/t04-smoke.{ps1,err} + t04-s1..s6 截图(三动作探针
  全活:双槽 TERM_PRESS/FOCUS/KEY、pane-2 WHEEL、拖拽几何 86/48)
  | blockers: **AC-03 用户实点三动作**(验收人 = 用户;运行命令
  见 T-04 用户实点门段)——gate 通过后:T-06 收口(022 复审记录
  备案 + 022 T-06/T-07 解锁记录)→ execution_done → review。
  若用户实点发现残余症状:回 work 续查(判决工件复跑指引在册)。
  | next: 用户实点门 → T-06 → review。

## 10. 待澄清事项

1. **上游判决时的成本决策**(若 T-01 判上游):fork 维护 vs 升级
   跨度 vs 应用层规避——T-01 工件列显式决策项,归用户裁定;
   **已撤销 2026-09-19**:判决 = 本仓映射误用,无上游归属,SD-02
   撤销(判决工件 §3/§6);
2. **与 022 SD-01 合流**:本案 SD-01(投递语义)是 022 SD-01
   (命中区几何归属)的下层——两 delta 合册或互引,T-05 定;
3. **测试归属仓与分支基线**:复现测试现居 auto-lang `plan-022-dev`,
   023 的 auto-lang worktree 基线选择(基于 plan-022-dev vs
   master + cherry-pick e5ddf41ba)——T-01 开工时定,避免两计划
   分支互锁;worktree/分支归属随裁定向 022 备案;
   **已裁定 2026-09-19**:023 worktree 基于 plan-022-dev
   (e5ddf41ba,复现面+Shrink 修复即判决对照现状;022 T-06/T-07
   门本等 023 AC-03,合流序天然 023 先落,无互锁);022 的复现
   测试归属移交 023(fc6ccc810 转正),随 T-06 向 022 备案;
4. **653 merge 时机**:与本计划无依赖已明,但 T-04 实车验证若在
   merge 后进行,需核对 vue 轨修复与 rust 载具配方无冲突
   (022 §10.6 重指步骤再核一遍)。
- 2026-09-19 stage: work | plan_id PLAN-023 | rev1 | outcome:
  **pass(全任务闭合,AC-01/02/04/05 工件面 + AC-03 用户实点门
  R2 通过)** | code_commit: plan-023-dev {fc6ccc810, 6c8a4f3d0,
  7d070317b}(基线 plan-022-dev e5ddf41ba)| task_ids T-01..T-06
  全闭合 | evidence: evidence/023/{t01-verdict.md,
  t01-probe-output.txt, t04-smoke.ps1/.err, t04-s1..s6.png} +
  auto-lang specs ADR-22/design-overlay-interaction + 用户实点
  R1(发现双播)/R2(通过)在案 | blockers: 无 | next: review
  (worktree D:/autostack/.wt/lang-023/auto-lang 留守;载具
  Cargo.toml 重指为运行态,merge 时按 §10.6 归位)。
- 2026-09-19 stage: review | plan_id PLAN-023 | rev1 | outcome:
  **pass** | reviewed_commit: plan-023-dev 7d070317b(HEAD,worktree
  零脏改)| base_commit: plan-022-dev e5ddf41ba |
  dependency_revisions: auto-down a615d69(detached 只读);iced
  crates.io 0.14.x(crates.io registry 只读,无 patch)| spec_inputs:
  docs/specs/auto-lang/ui/design/overlay-interaction.md(HEAD 62 行)
  + architecture.md ADR-22(HEAD)——冻结于 reviewed_commit | 
  acceptance_results: AC-01 **pass**(判决工件在册;复跑指引
  step1/step2 于被审 HEAD 重执行:基线轨迹伪影判定 + 探针矩阵
  R1 全达/R3 载具 [1,2,7,8]×2 复现,输出与 evidence/023/
  t01-probe-output.txt 根修后形态一致);AC-02 **pass**(无
  #[ignore],消息流断言 [A,B]×3,cargo test --lib p022_stack
  5/5 于 HEAD);AC-03 **pass**(用户实点 R2:三动作 + 双播
  复测通过,用户裁定转 review;探针日志佐证 TERM_PRESS/FOCUS/
  KEY 双槽各自到达 + pane-2 WHEEL);AC-04 **pass**(本仓支:
  根改 {fc6ccc810, 7d070317b} + 回归覆盖 = 转正套件 5 用例 +
  p023_focus_switch_single_cast 双播回归 + p023_probe 矩阵;
  SD-02 撤销因判决非上游);AC-05 **pass**(SD-01 落册
  worktree 分支,ledger 刷新归 merge 面——计划既定)| 
  findings: F1 info=ui::iced 8 失败存量于基线 e5ddf41ba
  (stash-diff 甄别,非本计划回归,归属各自主计划);F2 info=
  terminal_input 原竞态设计已确定性化(本计划内修复,原 solo
  必挂);F3 info=iced flex 零宽 Space 主轴失效机理未归因
  (KNOWN-DEBT 023-R8,padding 规则已绕开,spec 在册);F4 info=
  载具 Cargo.toml 重指为 untracked 运行态,merge 时按 022
  §10.6 归位 | evidence: 本记录所列命令均于 7d070317b 复跑
  (实现会话内 review,独立性限制已声明,verdict 由工件重构:
  测试输出 + 探针矩阵 + 用户实点记录,非执行摘要)| 门禁:
  仓规 Category B(局部 Rust 模块)= cargo check + 作用域模块
  测试,均过;cargo tf 不触发(未涉编译器/VM/核心协议)|
  next: merge(worktree 留守至 fold;merge 时刷新 ledger 并
  按 §10.6 归位载具指向)。

- 2026-09-19 stage: merge | PLAN-023:r1 | outcome: **pass** |
  收据 checkpoints——`prepared`: reviewed 7d070317b + 冻结 delta
  (overlay-interaction.md 62 行 + ADR-22)+ 账本投影 P023-1/2/3 |
  `landed`: auto-lang master 64841870b(no-ff 合并
  plan-023-dev,含 master 同步合并 84b15c626 零冲突,语义复核
  五处根修完好);master 冒烟 p022_stack 5/5 + terminal_input
  2/2 | `ledger_refreshed`: auto-lang .autoos/specs.json
  P023-1(designs)/P023-2(architecture)/P023-3(reviews,
  auto-term 归档路径跨仓显式标注),读回校验 593 items,经
  worktree 1275c27f8 随合并落 master | `archived`: 本文件
  auto-term docs/plans/archived/(untracked 文件系统迁移,
  status: archived)| `cleaned`: 见末条 |
  delivery_commit = auto-lang master 64841870b;载具 Cargo.toml
  指向已归位主检出(auto-term untracked 运行态;运行中实例
  8792 关闭后重 build 即用 landed 码)。| `cleaned`: wt-guard
  双 clean 复检;worktree lang-023/auto-lang + lang-023/auto-down
  (down 仓侧)均移除,branch plan-023-dev 删(was 1275c27f8,
  已并入 master),组目录 lang-023 空目录移除,双仓 worktree
  list 零残留——收据闭合。
