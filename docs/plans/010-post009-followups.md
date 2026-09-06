---
plan_id: PLAN-010
status: drafting
feature_name: 009 后续收尾——a2r 存量编译债清偿 + 对拍门禁补全 + 契约/文档入账
author: [衍星居士]
created_at: 2026-09-07
updated_at: 2026-09-07

# /auto-plan:review 时填写
supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 0
total_steps: 9
---

# [PLAN-010] 009 后续收尾(a2r 存量编译债清偿 + 对拍门禁补全 + 契约/文档入账)

## 变更摘要

兑现用户 2026-09-07 裁定:先立项清偿 PLAN-009 分析出的后续补充项,
**虚拟桌面集成(auto-os 侧)单独立项,不在本计划**。本计划是纯
修缮/接线/入账相位,无新架构,两仓(同 009 布局)作业:

- **auto-lang 侧**:
  - a2r 快照 74 例存量编译债清偿(009 实编门 ledger 显式豁免的存量,
    按错误码分批修复,ledger 逐例摘除;预授权:个别例超出本计划可留
    ledger 并记录理由,但清偿率须过半);
  - a2r `List.new()` 路径限定 bug 修复(009 T8 实证:`List.new()` 被
    错误限定为 `<最后 use.rs 符号>::Vec::new()`);
  - CLI `auto trans` 挂起修复(master 存量回归,f3185a7 时代可用);
  - E0080 销账复核:复跑 002 §4 所指两 rust-mode 示例(TickWrap 根修后
    应已解除),复核通过即销 009 新增观察第 7 条;
- **auto-term 侧**:
  - 003 §5 部署契约追补一条(`autoterm_engine` cdylib 随包分发;
    009 待澄清③授权、T11 漏项);
  - at-gen 窗口标题 cosmetic(iced `.title()`,去缺省
    "Auto Lang - Iced");
  - DEBTS #5/#7 关账回写(Ctrl+C 双投递已经 009 对拍门禁
    `parity_interrupt` 真实 ConPTY 实测有效,关账条件满足);
  - **对拍门禁补全**:选中文本场景去 skip(组件选中面经 scenario
    协议接线,oracle 侧 `TermSession::selection_text` 对比);
  - **iced 像素级自动化**:terminal 组件进 selectable_text 同款
    iced_test 管线(bounds/绘制证据),补齐 009 复审记的两项债候选。

明确不做(边界):Vue/web 后端留白(另行调查)、数据面形态乙
(未触发)、虚拟桌面集成(auto-os 侧单独立项)。

## 目标

1. a2r 实编门 ledger 清偿率 ≥50%,清零为理想;每批修复附快照回归
   (现有金样文本不得无故漂移,漂移须逐例说明);
2. a2r `List.new()` 转译正确(at 语料新增快照:构造含 `List.new()`
   的 .at,产物含 `Vec::new()` 且无错误路径限定);
3. `auto trans -p <at> rust` 恢复可用(60 行样本秒级完成,产物与
   in-process 一致);
4. E0080 两示例复跑通过,009 新增观察 #7 销账;
5. 003 §5 含 cdylib 分发条目;at-gen 窗口标题非缺省;
6. DEBTS #5/#7 关账回写;
7. 对拍门禁 7 场景全实跑(去全部 skip):既有 3 实跑 + 选中文本新场景
   + 色彩自证(仍 ash 缺席 skip 则保留其 skip 语义,但注明与选中的
   区别);iced 像素自动化在案;
8. 两仓回归底线:auto-lang `cargo tf`+`cargo tt`、auto-term
   `cargo test --workspace` 全绿。

### 非目标(明确排除)

- Vue/web terminal 后端(另行调查计划);
- 虚拟桌面加载 AutoTerm(auto-os 侧单独立项,用户 2026-09-07 裁定);
- 数据面形态乙降级(基准未触发);
- a2r 其余变形类(F2 int 宽度/F3 可见性/F4 冗余出口等风格变形,
  非编译炸裂,不在本计划)。

## 架构方案

无新架构。三条作业原则:

1. **实编门为准**:存量债修复以 ledger 摘除为进度单位;修复不得
   借机重生成金样(`SCHEMA_DRIFT_GENERATE_AT` 式全量再生成的教训
   见 009 T5——文本金样漂移须逐例人工确认);
2. **对拍口径不变**:语义等价(行尾归一/空尾行裁剪/易变行屏蔽
   既有三类)+ 提示符回归时序点;选中文本场景沿用同协议,仅增
   「选中→抽取→输出」一跳;
3. **像素证据管线复用**:iced-layout-tests feature 的
   `iced_test::simulator`(selectable_text/414 先例),terminal
   组件按同款写 bounds/绘制断言,不新建管线。

## 技术栈

两仓零新依赖(iced_test 已是 auto-lang 既有 optional dep;
libloading 已在 lock)。修复均在现有转译器/门禁/组件代码上进行。

## 需求分析与背景调查

### spec 依据(总览)

- P009-1..7(009 全链沉淀):本计划是其复审记录两项债候选 + 待澄清
  遗留 + DEBTS 009 新增观察的清偿载体;
- DEBTS #5/#7(Ctrl+C 稳定版复测):009 `parity_interrupt` 已在
  26200 真实 ConPTY 上实测双投递有效(门禁绿),关账证据在案;
- 003 §5 虚拟桌面接入契约:现有部署条仅含 autoterm.exe + helper,
  缺 cdylib 分发条(009 待澄清③授权追补)。

### 现状证据(file:line)

| 事实 | 证据 |
|---|---|
| 74 例存量债清单(错误码逐例) | `auto-lang crates/auto-lang/test/a2r/compile_gate_known_broken.txt`(67 行=4 头+63+11 批注,门测试 `a2r_rustc_real_compile_gate` 消费) |
| List.new() 限定 bug 实证 | 009 T8:`List.new()` → `engine_write_line::Vec::new()`(auto-term at-gen/README.md 记录);根因在 `auto-lang src/trans/rust.rs` use.rs 符号表与表达式限定交互(调查点) |
| CLI trans 挂起 | DEBTS 009 新增观察 #3;`crates/auto/src/main.rs:1853` TransTarget::Rust → `auto_lang::trans_rust(path)`(lib.rs `trans_rust_with_session`);60 行样本 >3min,新旧 exe 一致 |
| E0080 待销账 | DEBTS 009 新增观察 #7;002 §4 两示例所指 + auto-lang Plan 435 台账 `renderer.rs:11058` 记录;根修=009 T8 TickWrap(auto-lang `src/ui/iced/renderer.rs` run_app) |
| 选中面已有件 | 组件:`terminal_selection_*`/`terminal_selected_text`(auto-lang `src/ui/terminal/mod.rs`);引擎:`TermSession::selection_text`(auto-term `crates/autoterm-core/src/term.rs:181`);门禁 skip 位:`crates/autoterm-parity/tests/parity_gate.rs` `parity_selection_text` |
| iced_test 管线先例 | auto-lang `src/ui/iced/layout_tests.rs`(Plan 414,`--features ui-iced,iced-layout-tests`)+ `src/ui/iced/selectable_text.rs` |
| 契约现状 | auto-term `docs/designs/003-ash-compatibility.md:187-198`(§5,部署条仅 exe+helper) |

## 详细设计

### T1 [auto-lang] List.new() 限定 bug 修复

调查 `trans/rust.rs` 表达式限定:`List` 走 common_collection_types
(parser.rs:1164)→ `Type::List(Unknown)`;产物却出现
`engine_write_line::Vec::new()` —— 疑 `Vec::new()` 生成后又被
「未知符号 → use.rs 符号补前缀」逻辑改写(009 实证恰好选中最后注册的
use.rs 符号)。修复口径:集合字面量构造(`Vec::new()`/`vec![]`)列入
豁免名单,任何 use.rs 符号不得前缀之。新增语料快照:
`test/a2r/10_collections/007_list_new/`(含 `List.new()` 赋值字段)。
验证:新快照绿 + 全 tt 档绿 + 实编门该用例(新增)绿。

### T2 [auto-lang] 存量编译债分批清偿(计划主体)

按错误码分组推进(每批一个 commit,批间 `cargo tt` + 实编门全绿):

- 批次预估(ledger 分布):question 族 ~17 例(E0308 类)→ interop
  ~10 例 → methods/collections/delegation ~15 例 → 其余;
- 每例流程:读 ledger 行 → 取 auto-lang worktree 重转译该 .at →
  rustc 复现 → 定位 codegen 缺陷修复 → 实编门跑绿(该例从 ledger
  摘除)→ 金样比对(文本漂移须逐例说明,不得静默重生成);
- 快照金样同步:修复若改变既有 `.expected.rs` 输出,逐例更新并在
  commit message 记录理由;
- **预授权**:个别例根因过深(如涉及 CTEE/类型推断重构)可留 ledger,
  commit 记录理由;验收线=清偿率 ≥50%(74 → ≤37)。

验证:`cargo tt`(全绿)+ `cargo t a2r_rustc_real_compile_gate`
(skips 单调递减)+ 金样 diff 审阅。

### T3 [auto-lang] CLI trans 挂起修复

流程:复现(60 行样本 timeout 60s)→ `cargo build -p auto` 后以
`RUST_LOG`/eprintln 分段定位(trans_rust_with_session vs main 前置
初始化)→ 若为回归,`git log f3185a7..master -- crates/auto/src/main.rs
crates/auto-lang/src/lib.rs` 二分候选提交 → 修复 → CLI 实跑
`auto trans -p <样本> rust` 秒级完成,产物 `diff` in-process 一致。
若根因在并行会话领域(如 daemon/初始化),修至「CLI 不挂起」即可,
深度重构记 DEBTS 不阻塞。

验证:CLI 命令 timeout 60s 内退出 + 产物编译通过。

### T4 [auto-lang] E0080 销账复核

复跑 002 §4 所指两 rust-mode 示例(定位:`examples/` 下含
`HostBackend::Iced.run` 且带 tick 的示例,009 已确认 run_app TickWrap
根修)→ `cargo build --example <x> --features ui-iced` 双示例绿 →
销 009 新增观察 #7(DEBTS 回写)。示例若缺 tick 分支,补最小
tick_interval_ms 验证路径。

验证:两示例 build 绿(如示例不存在则以 ui_counter 加
tick_interval_ms 的临时变体验证后撤销)。

### T5 [auto-term] 契约追补 + at-gen 标题

- `docs/designs/003-ash-compatibility.md` §5 部署条后追加:
  「Auto 复刻应用形态:随包分发 `autoterm_engine` cdylib
  (`autoterm_core.dll`,debug/release 后缀随目标)+ 与宿主同目录的
  `autoterm-ctrlc.exe`(interrupt 契约不变)」;
- at-gen `src/shell.rs`/`main.rs`:iced application 标题改
  "AutoTerm"(经 auto-lang run_app 侧若不支持自定义标题,则
  run_app 增可选 title 参数或组件侧 `app_title()` 面——按现状最小改)。

验证:grep 契约命中;GUI 起窗标题非缺省(smoke 截图复用)。

### T6 [auto-term] DEBTS #5/#7 关账回写

DEBTS.md Phase 4 清单 #5(Ctrl+C 稳定版复测/债务 #7)标注关账:
证据=009 `parity_interrupt`(真实 ConPTY,timeout 主体,双投递广播
成功路径)+ auto-term `engine_ffi_integration` interrupt 分支;
#5 与 #7 合并销账(同一事实)。

验证:grep DEBTS 关账标注在案。

### T7 [auto-term] 选中文本 headless 对拍接线(门禁去 skip)

- at-gen scenario 扩 `selection` 模式:headless 进程直接驱动组件
  注册表面(`terminal_selection_begin/extend/finish` +
  `terminal_selected_text`,auto-lang `src/ui/terminal/mod.rs` 公面;
  经 at-gen 已依赖的 auto-lang),对喂入网格编程选中("hello world"
  行 Simple 选中),ROW 协议输出选中文本;
- oracle 侧:`TermSession::begin_selection/update_selection/
  selection_text` 同区间选中(alacritty Point/Side 坐标换算);
- `parity_gate.rs`:`parity_selection_text` 去 skip,双侧选中文本
  assert 相等(语义等价归一沿用);
- 色彩自证保持 ash-缺席 skip 语义不变(注明与选中的差异:缺外部
  依赖 vs 待接线)。

验证:parity 6 场景实跑 5 + 1 skip(色彩)全绿。

### T8 [auto-term→auto-lang] iced 像素自动化

auto-lang `src/ui/iced/` 新增 terminal 像素测试(layout_tests 同款,
`--features ui-iced,iced-layout-tests`):构造 `View::Terminal`(固定
文本行/选中态/光标三形),simulator 渲染后断言:bounds 非零、
选中列带颜色采样、光标块像素区域。放 `layout_tests.rs` 同 cfg 门
(新 mod `terminal_pixel_tests`)。

验证:`cargo t --features ui-iced,iced-layout-tests terminal_pixel` 绿。

### T9 收账

两仓 fold(auto-lang T1-T4 毕 → fold → auto-term T5-T8 毕 → 终 fold);
DEBTS 009 新增观察逐条销账(#1 清偿率在案/#2 已修/#3 已修/#6 已修/
#7 已销;#4 留白维持;#5 维持观察);复审遗留两项闭合记录。

## 测试设计

- 自动(auto-lang):List.new 快照 + 74 例实编门(ledger 递减)+
  全 tt/tf 档;E0080 两示例 build;像素测试;
- 自动(auto-term):parity 6 场景(选中新场景)+ ffi 集成测试 +
  workspace 全量;
- 文档:契约/DEBTS grep 断言;
- 回归底线:两仓全档(验收时重跑)。

## 验收标准

1. List.new() 快照绿 + 实编门含该例;
2. 实编门 ledger 清偿率 ≥50%(74 → ≤37),每批 commit 记录理由,
   门全绿;
3. CLI trans 60s 内完成且产物编译通过;
4. E0080 两示例 build 绿 + DEBTS #7 条(009 观察)销账;
5. 003 §5 含 cdylib 分发条;at-gen 标题非缺省;
6. DEBTS #5/#7 关账标注在案;
7. 对拍门禁:选中文本实跑等价(去 skip),全门禁绿(色彩保留
   ash-缺席 skip);
8. iced 像素自动化测试在案且绿;
9. 两仓全档回归绿(tf/tt + workspace),零新依赖。

## 执行步骤

> 约定:两 worktree 同组 `.wt/auto-010/{auto-term,auto-lang}`
> (分支 `plan-010-dev` / `auto-term-dev`)。T1-T4 在 auto-lang,
> T5-T8 在 auto-term(T7 用到 T1 的 auto-lang 修复则先行 fold);
> 每段验收过即 fold + re-sync。

- **T1** [auto-lang] List.new() 限定 bug 修复 + 快照。
  验证:新快照绿 + tt 全档绿。
- **T2** [auto-lang] 存量编译债分批清偿(≥50% 线)。
  验证:实编门 skips 递减 + tt 全绿 + 金样 diff 逐例说明。
- **T3** [auto-lang] CLI trans 挂起修复。
  验证:60 行样本 CLI 60s 内完成,产物与 in-process diff 一致。
- **T4** [auto-lang] E0080 两示例复核 + 销账准备。
  验证:两示例 build 绿。
  → fold auto-lang,auto-term 侧 re-sync。
- **T5** [auto-term] 003 §5 契约追补 + at-gen 标题。
  验证:grep 契约命中;标题断言。
- **T6** [auto-term] DEBTS #5/#7 关账回写。
  验证:grep 关账标注。
- **T7** [auto-term] 选中文本对拍接线,门禁去 skip。
  验证:parity 6 场景(5 实跑+1 ash-skip)全绿。
- **T8** [auto-term→auto-lang] iced 像素自动化。
  验证:`terminal_pixel` 测试绿。
- **T9** 收账:DEBTS 009 观察逐条销账 + 两仓终 fold + 全档回归。
  验证:tf/tt + workspace 全绿;grep 销账记录。

## 复审记录

(留 /auto-plan:review 填写)

## 待澄清事项

1. **74 例清偿率红线**:**已预授权**(本计划正文)≥50%,清零理想;
   超深根因例可留 ledger 并记录理由——不阻塞验收。
2. **CLI trans 挂起若根因在并行领域**:修至「不挂起」即可,深度
   重构记 DEBTS 不阻塞(正文 T3 口径)。
3. **色彩自证场景**:ash 缺席维持 skip;若执行期 ash 可指路
   (AUTOTERM_ASH_BIN),升级为实跑并同样对选中语义归一——非本计划
   验收硬项。
4. **at-gen 标题改动落点**:run_app 若无 title 面,优先最小改
   (application 链 title 参数),不动 VM 轨。
