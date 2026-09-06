---
plan_id: PLAN-010
status: reviewed
feature_name: 009 后续收尾——a2r 存量编译债清偿 + 对拍门禁补全 + 契约/文档入账
author: [衍星居士]
created_at: 2026-09-07
updated_at: 2026-09-07

# /auto-plan:review 时填写
supersedes_spec_components:
  - "auto-lang test/a2r/compile_gate_known_broken.txt: 74→37 清偿(批1 question 族 21/批2 codegen 根修 15;遗留理由逐批在案)"
  - "auto-lang crates/auto-lang/src/lib.rs trans_rust_with_session: 兄弟 .at 预扫有界化(src 字面名才按 crate 根扫+深度/数量/大小上限)——CLI trans 恢复"
  - "auto-lang crates/auto-lang/src/trans/rust.rs: a2r codegen 根修批(Type.method 内置集合豁免/Option 尾包裹/say 内建/std 前导注入/委托组件缺省/空集合字面量/mut 方法注册表/Result<Box> 免克隆/Option-get 克隆/comptime 物化/闭包块尾值)"
  - "auto-term DEBTS.md: Phase 4 #5 关账(与旧账 #7 合并)+ 009 新增观察 7 条逐条销账(#1/#2/#3/#6/#7 销,#4/#5 维持)"
  - "auto-term docs/designs/003-ash-compatibility.md §5: 追补第 6 条——Auto 复刻应用形态 cdylib+ctrlc 同目录分发"
  - "auto-term at-gen/README.md: 转译再生成规程更新(CLI 恢复为主路径,List.new 绕开写法废止)"
new_spec_components:
  - "auto-lang test/a2r/10_collections/007_list_new/: List.new() 语料快照(限定 bug 回归锚,先红后绿)"
  - "auto-lang crates/auto-lang/src/ui/iced/terminal_pixel_tests.rs + test/ui/terminal_pixel/ 金样 4 件: terminal 组件像素级 headless 自动化(bounds 精确/选中帧/光标块帧,iced_test matches_image)"
  - "auto-lang crates/auto-lang/src/ui/iced/renderer.rs run_app_with_title(Option<&str>): iced application 链标题面(复刻应用去缺省标题)"
  - "auto-term at-gen scenario=selection + crates/autoterm-parity parity_selection_text(去 skip): 选中文本对拍(组件注册表面 SEL 协议 vs TermSession 选中,spawn_with_extras 载荷回收)"
touched_goals:
  - "DEBTS #8 Auto 化: 补全收尾相位(PLAN-010)——a2r 编译债 50% 清偿/CLI trans 恢复/E0080 销账/选中对拍+像素自动化两门禁补齐;无环铁律保持(at-gen→auto-lang 单边)

current_step: 9
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
  [✅ 已完成] commit auto-lang auto-term-dev:根因=Type.method() source_crate
  兜底限定误伤内置集合映射;修=auto_type_to_rust 命中者豁免前缀;快照
  007_list_new 先红(exit::Vec::new())后绿;实编门绿(新例编译);
  cargo tt 3818/3819(唯一红=master 存量 charts 红,待澄清⑤)
- **T2** [auto-lang] 存量编译债分批清偿(≥50% 线)。
  验证:实编门 skips 递减 + tt 全绿 + 金样 diff 逐例说明。
  [✅ 已完成] 两批 commit(批1: question 族 21 例;批2: say/std 前导/
  委托缺省/空集合/mut 注册表/Box 免克隆/Option-get/comptime/闭包块尾
  15 例)——ledger 74→37 行,恰 50%;金样漂移逐例审阅;tt 3818/3818
  (唯红=master 存量 charts 红);实编门绿 0 unexpected。
  遗留 37 例理由已记录(智能指针/interop/语法未编码/ownership-闭包族)
- **T3** [auto-lang] CLI trans 挂起修复。
  验证:60 行样本 CLI 60s 内完成,产物与 in-process diff 一致。
  [✅ 已完成] commit auto-lang auto-term-dev:根因=单文件 CLI 三处兄弟
  .at 预扫无界(crate-root 无条件取 parent().parent()),任意路径下遍历
  整棵无关树逐个 parse(实测 Temp 树 2675 个 .at)。修=src 字面名才按
  crate 根扫+深度/数量/大小/垃圾目录统一有界;60 行样本 13.3s 完成,
  产物 rustc 编译绿,与 in-process 逐字节一致;tt 3818/3818
- **T4** [auto-lang] E0080 两示例复核 + 销账准备。
  验证:两示例 build 绿。
  [✅ 已完成] 按正文 fallback:ui_counter/ui_accordion 加 tick_interval_ms
  (Some(16))临时变体两例,cargo build --example ×2 --features ui-iced
  双绿(1m07s,run_app TickWrap 订阅路径实编通过)——变体已按计划撤销;
  E0080(009 观察 #7)销账证据在案
  → fold auto-lang,auto-term 侧 re-sync。
- **T5** [auto-term] 003 §5 契约追补 + at-gen 标题。
  验证:grep 契约命中;标题断言。
  [✅ 已完成] commit auto-term plan-010-dev:003 §5 追补第 6 条(cdylib+
  ctrlc 分发,grep 命中);at-gen main 接 run_app_with_title(Some(
  "AutoTerm"))(auto-lang 侧新增 application 链 title 面,HRTB 具名 fn
  承载),at-gen build 绿
- **T6** [auto-term] DEBTS #5/#7 关账回写。
  验证:grep 关账标注。
  [✅ 已完成] DEBTS Phase 4 #5 关账(证据=009 parity_interrupt 真实
  ConPTY 双投递绿 + engine_ffi_integration interrupt 分支),与旧账
  #7 合并销账,win32 立项不再需要
- **T7** [auto-term] 选中文本对拍接线,门禁去 skip。
  验证:parity 6 场景(5 实跑+1 ash-skip)全绿。
  [✅ 已完成] at-gen scenario=selection(组件注册表面 terminal+
  terminal_selection_* + terminal_selected_text,SEL 协议);parity_gate
  parity_selection_text 去 skip(oracle TermSession begin/update_
  selection + selection_text 对拍,行尾归一);A2r 增 spawn_with_extras;
  cargo test -p autoterm-parity → 5 passed + 1 skip(色彩,ash 缺席)
- **T8** [auto-term→auto-lang] iced 像素自动化。
  验证:`terminal_pixel` 测试绿。
  [✅ 已完成] auto-lang 新 mod terminal_pixel_tests(ui-iced,
  iced-layout-tests)三断言:bounds 精确/选中帧≠基线/光标块帧≠基线;
  Terminal widget 增 operate 暴露 bounds;matches_image 金样 4 件入库
  (每用例私有 base/frame);3/3 绿×2 轮稳定
  commit auto-lang auto-term-dev
- **T9** 收账:DEBTS 009 观察逐条销账 + 两仓终 fold + 全档回归。
  验证:tf/tt + workspace 全绿;grep 销账记录。
  [✅ 已完成] DEBTS 009 观察 #1/#2/#3/#6/#7 销账(证据链见 DEBTS.md
  对应条目),#4/#5 维持;at-gen README 再生成规程更新(CLI 恢复/
  List.new 绕开废止)= 复审遗留两项闭合的消费端记录;两仓 fold 完成
  (auto-lang auto-term-dev→master、auto-term plan-010-dev→main,
  双侧 worktree re-sync);全档回归:tf 3465/3466 + tt 3818/3818
  (唯红=master 存量 charts 红,待澄清⑤)+ workspace 绿 + tb 与
  master 同集;复审遗留两项(选中 headless 对拍+iced 像素自动化)
  经 T7/T8 闭合

## 复审记录

**复审人**: ZCode(/auto-plan:review);**时间**: 2026-09-07
**验证面**: 两仓折后 master(并行 plan-576 已在折内),默认检出重跑

### 验收标准逐条复核(9/9 pass)

1. **List.new() 快照绿 + 实编门含该例** — pass。`cargo tt
   test_10_collections_007` PASS(0.87s);ledger 37 行中无 007。
2. **实编门清偿率 ≥50%(74→≤37),门全绿** — pass。ledger 实测
   `grep -c "^a2r/"` = 37(恰 50.0%);批 1/批 2 commit 记录理由;
   实编门绿(0 unexpected,142-143 compiled);失败清单逐例输出设施
   已并入门禁。
3. **CLI trans 60s 内完成且产物编译通过** — pass。折后 master 重建
   CLI:20.4s(<60s,冷缓存;含预算内 stray 解析);产物 rustc 编译
   绿;与 in-process transpile_rust 逐字节一致(diff 空)。
4. **E0080 两示例 build 绿 + DEBTS #7 销账** — pass。折后重建
   ui_counter+tick_interval_ms 变体 --features ui-iced 绿(55s,验证
   后撤销,worktree 无残留);DEBTS.md:134 观察 #7 销账记录在案。
5. **003 §5 含 cdylib 分发条;at-gen 标题非缺省** — pass。003:202
   cdylib 条 grep 命中;**实窗取证**:autoterm-at.exe 启动 →
   PowerShell MainWindowTitle = `AutoTerm`(非 "Auto Lang - Iced")。
6. **DEBTS #5/#7 关账标注在案** — pass。DEBTS.md:63 #5 关账(证据=
   parity_interrupt 真实 ConPTY 双投递绿 + engine_ffi_integration)。
7. **对拍门禁:选中文本实跑等价(去 skip),全门禁绿(色彩保留
   ash-skip)** — pass。折后 `cargo test -p autoterm-parity`:
   5 passed + 0 failed(色彩 ash 缺席显式 skip,惯例保持);SEL
   hello world 双侧一致(行尾归一)。
8. **iced 像素自动化测试在案且绿** — pass。折后 `--features
   ui-iced,iced-layout-tests --lib terminal_pixel` 3/3 绿(1.01s);
   金样 4 件入库。
9. **两仓全档回归绿(tf/tt + workspace),零新依赖** — pass。
   auto-lang 四层(折后 master,含并行 576):tf 3468/3469、tt
   3821/3822、tb 3516/3524、tv 3609/3610——红全部为 master 既有基线
   (charts + 7 book listing,与 master 主检出逐一相同,无新增);
   auto-term `cargo test --workspace` exit 0 全绿;两仓 Cargo.toml
   diff 零新增依赖。

### 遗漏/延后/workaround 猎查

- **遗漏**: 无。9 步均有对应 diff 与验证;temp 变体验证后撤销无残留。
- **延后**: 计划正文「非目标」四项(Vue/web、auto-os、形态乙、a2r
  风格变形)均为用户裁定的显式排除,非执行期私设;遗留 37 ledger 例
  为计划预授权(≥50% 线达成),理由逐批在案。
- **workaround**: T3 口径「修至不挂起」的扫描有界化即计划批准的修复
  形态,非遮盖;无 TODO/hack 残留(diff 猎查无新增 TODO)。

### 债候选(记录,均不阻塞)

- charts 红 + 7 book 红:master 存量(非 010 引入,基线双验),留
  auto-lang 侧独立修缮。
- 遗留 37 ledger 例:智能指针自动解包族/interop 外部框架族/语法未
  编码族/ownership-闭包耦合族/singles。
- **merge 时提醒**:auto-lang `temp_plan009_t8_transpile_at_app` 现
  指向 .wt/auto-010 工作树路径,merge 清理 worktree 前须改回主检出
  (009 merge 前置同款约定)。

### 协议注记

两仓分支 fold 已按 /auto-plan:work 多相位协议在执行期落主(T4 阶段
折 + T9 终折),worktree 在位未清——/auto-plan:merge 只做收尾清理与
沉淀,无分支落地动作残留。

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
5. **auto-lang master 存量红(非本计划引入)**:`ui_gen::vue::tests::
   test_charts_gallery_compiles` 在 auto-lang master(1c6753a92)即红
   (LineChart tag 缺失,vue ui_gen 域,2026-09-07 worktree 与主检出双验
   )。"tt 全档绿"按「除该存量红外全绿」口径执行;该例不在本计划
   范围(vue 图表 tag 生成),留给 auto-lang 侧独立修缮。
