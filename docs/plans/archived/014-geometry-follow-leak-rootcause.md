---
plan_id: PLAN-014
status: archived
feature_name: 几何随动退化几何内存爆炸——根因修复与取证闭环(回溯收编)
author: [zcode-session]
created_at: 2026-09-13T12:00:00Z
updated_at: 2026-09-13T12:30:00Z
plan_revision: 1
current_step: 6
total_steps: 6
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
---

# PLAN-014 · 几何随动退化几何内存爆炸——根因修复与取证闭环

> 回溯收编计划:本变更在用户授权的事故排查会话中已实施完毕(含回归验证),
> 本文件补建契约、固化证据,使已交付部分可进入 review/merge 流程。
> 与"014 直键入"在途工作(VM shim、rust codegen)的边界见 §10。

## 0. 变更摘要

014 几何随动(窗口 resize → 引擎 resize)把**最小化窗口的退化可用空间(0px)
换算成 1×1 灌给 `Term::resize`**;alacritty `Grid::shrink_columns` 将满滚动
历史(ash+ping 场景 ~10k 行×136 列)折叠重排为 ~1.36M 行、逐格
`Vec::insert(0,..)` O(n²) 前插,**GB 级瞬态分配冲死系统**(分配位于
autoterm_core.dll 自有堆,宿主 GuardAlloc 盲区)。三案取证(09-13
17:09/18:05/19:13),末案经 at-suspend 挂起 + cdb 栈分析坐实:
`Tick→term_apply_resize→autoterm_engine_resize→Term::resize→
shrink_columns→RtlReAllocateHeap`。修复为三层护栏 + 顺带清账 + 取证
工具链沉淀,回归验证 PASS。

## 1. 目标

- G1 消灭"最小化/退化几何 → 引擎重排爆炸"事故路径(结构性不可达)。
- G2 三案证据链固化入库(dump/栈/曲线/报告),责任划分入 DEBTS。
- G3 沉淀可复用取证工具链(at-suspend/at-dump/at-run-job)与自动化
  复现配方,供后续任何内存事故复用。
- G4 顺带修复 ring 逐出记账 bug(反压永久假死)。

## 非目标

- 修 alacritty_terminal 上游实现(瞬态内存无上界/前插重排/无最小值
  文档)——记录 DEBTS #15,上游 issue 另行可选。
- VM 轨 014 shim(engine_backlog_* 等)与 rust codegen 回归——
  现场agent在途,见 §10 交接。
- wgpu/原生层曾列为嫌疑的方向已被证据排除,不再追查。

## 2. 架构方案

三道纵深防御闸(任何一道独立生效即阻断事故):

```
widget layout(auto-lang) ──退化空间( cols<2 )不发请求    闸1 语义修正
terminal_request_resize(注册表) ──MIN_RESIZE_COLS=2 钳位  闸2 通道下限
TermSession::resize(引擎) ──拒绝 cols<2/rows<1;
  列数减半及以上收缩先 grid_mut().clear_history()          闸3 边界不变量
```

闸3 同时是**嵌入契约**(DEBTS #15):任何宿主/外来实现不可依赖上游
钳制绕过;最小化/隐匿/零尺寸是可见性事件,永不进入几何通道。

ring 记账修复:`RingBuffer::push` 返回被挤条目,reader 对
`pending_bytes` 销账,消除"累计逐出越限 → reader 永久反压假死"。

取证工具链(crates/autoterm-core/src/bin/):at-suspend
(NtSuspendProcess 全线程暂停键)、at-dump(MiniDumpWriteDump,
0x2|0x20000——0x20000 单用是 IgnoreInaccessibleMemory,实测纠错)、
at-run-job(Job Object commit 上限 + KILL_ON_JOB_CLOSE)。

## 3. 技术栈

Rust(auto-lang iced UI / autoterm-core 引擎 / Win32 FFI);
alacritty_terminal 0.26(第三方,不修改);证据分析 cdb(Windows Kits)。

## 4. 需求分析与背景调查

- 授权:用户在 2026-09-13 会话中逐步授权排查→修复→回归(本会话记录);
  仓库范围 auto-term 与 auto-lang(双仓改动均在该授权内)。
- 三案时间线/曲线/冻结报告:见 evidence/014/(固化副本)与 %TEMP% 原件。
- 事故机制与责任划分全文:DEBTS.md #15(触发在我们=可见性事件误入几何
  通道+边界无不变量;烈度在第三方=退化输入下瞬态无上界)。
- 曾排除方向(证据在案):reader→drain 通道(三案 pending_bytes=0)、
  exe Rust 堆(冻结报告 live=0.0MB/7MB vs commit 4-10GB)、wgpu(WS/commit
  形态曾疑,终案栈定案引擎重排)。
- docs/specs/ 目录不存在,Module Specs 体系未建立——规范增量以 DEBTS
  账本为权威落点(见 §5 说明)。

## 5. 详细设计

| # | 改动 | 文件:符号 | 说明 |
|---|---|---|---|
| D1 | 退化空间不发请求 | auto-lang `ui/terminal/iced/widget.rs` layout() | cols≥2 才 request_resize |
| D2 | 注册表列数下限 | auto-lang `ui/terminal/mod.rs` MIN_RESIZE_COLS=2 | clamp 下限 1→2,注释指向 DEBTS #15 |
| D3 | 引擎边界不变量 | auto-term `term.rs` TermSession::resize | 拒绝退化几何;≥2× 收缩先 clear_history() |
| D4 | ring 逐出销账 | auto-term `ring.rs` push→Option<T>;`pty.rs` reader fetch_sub | 假死修复 |
| D5 | 取证三件套 | auto-term `src/bin/at-{suspend,dump,run-job}.rs` | 见 §2;含 \\?\ 前缀剥离等实测修正 |
| D6 | 责任/警示入账 | auto-term `DEBTS.md` #15 | 含嵌入契约警示 |

### 规范增量

| delta_id | add/modify/retire | target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify | DEBTS.md #15(代行 specs 职责) | 无此账目 → 机制/责任/嵌入契约全录 | 本仓 docs/specs/ 不存在;DEBTS 为既有权威账本 | AC-01..03 |

## 6. 测试设计

- 单测:auto-lang terminal clamp(ring/resize 请求钳位,已更新断言
  MIN_RESIZE_COLS);autoterm-core ring 逐出返回被挤条目(已更新)。
- 回归(已执行):完整炸弹配方自动化——武装梯子(at-run-job 6144MB+
  AUTO_MEM_LIMIT_MB=3000+观察哨 WS≥3.5GB 自动挂起+dump)下,ash 会话+
  52 键入+两轮最小化+5 分钟最小化保持:resize 计数恒 1、WS 212MB 平稳、
  恢复无缝(零 resize,几何从未离开 136×48)。证据 evidence/014/run4。
- 引擎护栏单测(T-06,已落地):tests/resize_guard.rs 三测钉死语义
  (退化惰性拒绝/大幅收缩裁史/同尺寸 no-op),实测 3 passed。

## 7. 验收标准

- AC-01 最小化零退化几何:修复版完整配方下最小化,引擎侧无 1×1 resize
  (trace resize 计数不变),WS 稳定,恢复后画面/会话如常。
  验证:evidence/014/run4 回归日志 + watch 曲线(PASS,2026-09-13)。
- AC-02 嵌入契约三道闸在码:D1/D2/D3 三处代码+注释在库;registry 钳位
  单测绿。验证:`cargo test -p auto-lang --lib --features ui-iced terminal::`
  (21 passed,实测);`cargo check` 双仓 0 error(实测)。
- AC-03 责任与警示入账:DEBTS.md #15 存在且含双仓责任划分+外来实现
  警示。验证:grep DEBTS.md(在库)。
- AC-04 ring 记账:逐出后 pending_bytes 回落,累计逐出不再造成永久
  反压。验证:ring 单测(绿)+ pty.rs 销账代码在库。
- AC-05 取证工具链:at-suspend/at-dump/at-run-job 可用且 at-dump full
  模式产出 ≥ 数百 MB 真全量(实测 530MB)。验证:evidence/014 工具
  自测记录。
- AC-06 引擎护栏单测绿:resize_guard 三测(退化惰性拒绝/大幅收缩裁史/同尺寸
  no-op)通过(实测 3 passed)。

## 8. 执行步骤

- T-01 ✅ D1 widget 闸(2026-09-13)
- T-02 ✅ D2 注册表钳位+测试更新(同日)
- T-03 ✅ D3 引擎边界不变量+裁历史(同日)
- T-04 ✅ D4 ring 记账修复+测试(同日)
- T-05 ✅ D5/D6 工具链+DEBTS+证据固化(evidence/014)+回归(同日)
- T-06 ✅ 引擎护栏单测(crates/autoterm-core/tests/resize_guard.rs:三测全绿,
  2026-09-13 `cargo test -p autoterm-core --test resize_guard` 3 passed)。

## 9. 复审记录

- 2026-09-13 stage:new/收编 · outcome:pass(T-01..T-06 全部完成,AC-01..06
  证据齐备)· next:/auto-plan:review(可复审)。

## 10. 待澄清事项

- VM 轨 014 shim(engine_backlog_* 缺失致 `auto run -r vm` 链接失败)
  与 rust codegen 回归(terminal 臂属性退化/i64 不匹配):均为现场
  agent 在途工作,不属本计划验收;建议其完成后再以同配方复测 VM 轨
  (自动化脚本 %TEMP%/at-ritual2.ps1 可复用)。
- 上游 issue(alacritty_terminal 退化 resize 瞬态无上界):可选,非阻塞。

- 2026-09-13 stage:review · PLAN-014 · plan_revision 1 · outcome:**blocked**
  · reviewed_commit: 工作树未提交(auto-term @1032158 脏,auto-lang @c9f4d5a 脏)
  · base_commit: 同 HEAD(实现全部未锚定) · dependency_revisions: auto-lang c9f4d5a(脏)
  · spec_inputs: DEBTS.md #15(行哈希 10cc014ea1c61f82;docs/specs/ 不存在,§4 已说明)
  · acceptance_results: AC-01 pass(evidence/014/run4:resize 计数 1、末态 WS 212,984K、
    feeds 健康;方法=证据文件复核) · AC-02 pass(三道闸 grep 在码 + terminal:: 21/21 重跑)
    · AC-03 pass(DEBTS #15 在库) · AC-04 pass(ring 1/1 重跑 + pty.rs evicted.len() 销账)
    · AC-05 pass(运行时复证:100MB 牺牲进程 → 346MB 全量 dump) · AC-06 pass
    (resize_guard 3/3 重跑)
  · findings: F-01(阻断合并,结构性):双仓实现未提交,且计划范围文件(pty.rs/ring.rs/
    widget.rs/mod.rs)与现场 agent 014 在途 WIP 交织,无法做无污染的范围提交——pass 无
    SHA 可锚定,按技能契约不予发放。F-02(基线,非本计划回归):auto-lang `cargo tf`
    (无 ui 特征构建)12 错,全部位于 vm/ffi/term_engine.rs(agent 在途,unguarded
    crate::ui 引用);带 ui-iced 的全部相关测试绿。
  · evidence: evidence/014/(dmp/栈/四轮日志/曲线/报告)+ 本记录内命令结果
  · next: 解阻二选一——①agent 落地其 014 WIP 提交后,本计划变更随之或另行提交,
  即刻重审(代码未变,证据可复用,理由已录);②用户授权对交织文件做整体快照提交
  (含 agent 在途 hunk)。F-02 转 agent:term_engine.rs 补 cfg 门。

- 2026-09-13 stage:review(重审) · PLAN-014 · plan_revision 1 · outcome:**pass**
  · reviewed_commit: auto-term 49ae426 / auto-lang ee2fafa(双仓锚定,F-01 解除)
  · base_commit: 1032158(auto-term)/ c9f4d5a7(auto-lang)
  · dependency_revisions: auto-lang ee2fafa(含 F-02 修复+VM shim 三表登记)
  · spec_inputs: DEBTS.md #15(随 49ae426 入库)
  · acceptance_results: AC-01..06 全 pass——重审复用前次工件复核结果,
    理由:代码与证据未变,仅新增提交锚定;F-02(前次基线阻断)已随
  ee2fafa 修复验证(无 ui 特征编译通过);新增 VM 轨同配方回归 PASS
  (20:4x,3 分钟最小化 220MB 平稳零错误,恢复无缝,见会话记录)。
  · findings: F-01 已解除(双仓提交);F-02 已修复(term_engine ui 适配层
  cfg 门+测试模块同门)。新基线注记:tf 的 ffi_dep_parity 五测缺 oracle
  夹具二进制(环境性,非本计划范围)。
  · evidence: evidence/014/ + 提交 49ae426/ee2fafa 的树内全部工件
  · next: /auto-plan:merge

- 2026-09-13 stage:merge · PLAN-014:r1 · outcome:pass
  · prepared: 复审 pass 基线(§9 前条),SD-01 规范落点 DEBTS.md #15 已随
  49ae426 入库(本仓无 docs/specs/,复审已认定)
  · landed: auto-term 49ae426(实现+DEBTS+evidence)→ 8f17c48(复审记录)→
  本合并提交;依赖仓 auto-lang ee2fafa(祖先核验通过)
  · ledger_refreshed: .autoos/specs.json 新增 P014-1..4(reports/architecture/
  tests/reviews,file 指向 archived/014;回读验证;无关条目保全)
  · archived: docs/plans/archived/014-geometry-follow-leak-rootcause.md
  (git mv,status: archived),completion_kind: delivered
  · cleaned: 无需清理——本计划为主检出应急处置交付,无专用 worktree/
  分支(wt-guard 不适用;junction at-app/stdlib 为 VM 运行依赖的现场
  workaround,保持未跟踪,随 auto-lang CLI 的 CWD 缺陷修复移除)
