---
plan_id: PLAN-012
status: archived                # drafting → executing → execution_done → reviewed → archived
feature_name: engine-face-auto-adoption
author: [ZCode]
created_at: 2026-09-10
updated_at: 2026-09-10
plan_revision: 1
current_step: 5
total_steps: 5

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

affects: [at/, at-engine-face(新), scripts, docs]

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/designs/004-engine-autoization-roadmap.md §5⑤: 修改——增 auto-term 侧转正回执(并存态:真身+产物 crate+parity 级验证;dist 不替换留 P-S)"
  - "DEBTS.md #10: 修改——增 012 条(⑤ auto-term 侧转正账)"
  - "README.md 布局节: 修改——增 at-engine-face 行(可发现性)"
new_spec_components: []
touched_goals:
  - "004 路线图 §5⑤(引擎 Auto 化,C 通道导出面)"

---

# [PLAN-012] engine-face-auto-adoption：引擎 12 符号面 Auto 版转正（并存构建 + 置换验证）

> **来源**：auto-lang PLAN-610（已归档，landed 2038e1370）⑤ capstone 的
> auto-term 侧落地——用户 2026-09-10 裁定"把 610 落地的工作在 autoterm
> 这一边补齐"。**形态裁定（本轮）**：**并存验证**——Auto 面作为可构建、
> 可验收的替身在仓转正；**不替换** dist 生产物（替换属 P-S 终局决策，
> 留用户裁定）。② at-gen engine.rs 胶水 Auto 化（⑥ 应用）为后续独立计划。

## 0. 变更摘要

1. `at/engine_face.at` 真身落位（自 auto-lang `27_c_abi/005_engine_face_auto`
   语料适配迁移：出处注记 + 转译规程）；
2. `at-engine-face/` 产物 crate（仿 at-gen 模式：转译产物入库
   `src/lib.rs` + Cargo.toml cdylib + 再生成规程 README；dep
   autoterm-core path，独立 workspace 不入主树）；
3. 置换验证：parity 六场景门禁跑在 **Auto 面 DLL** 驱动之下（at-gen
   引擎解析序 exe 同目录候选，597 置换法先例；产物备份恢复）；
4. 回执：004 §5⑤ 补账（Auto 面转正=并存态）+ DEBTS #10 增 012 条 +
   README 打包节注记。

## 1. 目标

- **G1**：`at-engine-face` 一键构建产出 Auto 版引擎 cdylib，导出符号集
  与 Rust 版 `ffi.rs` 12 符号**恒等**（dumpbin 对照在案）；
- **G2**：`cargo test -p autoterm-parity`（六场景）在 Auto 面驱动下
  **全绿**——启动/echo/resize/中断/色彩/选中行为等价；
- **G3**：账实同步（004/DEBTS/README），替换决策的材料齐备。

### 非目标

- 不替换 `scripts/package-at.cmd` 的 dist 生产物（Rust 版
  autoterm_core.dll 仍是分发引擎；替换=P-S 用户终裁）；
- 不动 at-gen `engine.rs`（② 胶水 Auto 化，后续计划）；
- 不动 auto-lang 仓（只读消费 master 的 auto.exe 转译与 610 产物）；
- 不动 `crates/`（ffi.rs 契约冻结，oracle 身份不变）。

## 2. 架构方案

- **真身链**：`at/engine_face.at`（Auto 源真身）→ a2r 转译（auto-lang
  master `auto trans`，只读工具链）→ `at-engine-face/src/lib.rs`
  （产物入库，再生成规程入 README——at-gen `app_logic.rs` 同款纪律）；
- **构建**：`at-engine-face` 独立 workspace（at-gen 先例，无环铁律下
  仅 at-gen 依赖 auto-lang 运行时的格局不变——本 crate 零 auto-lang
  运行时依赖，产物是纯 Rust + autoterm-core）；cdylib 产物名
  `autoterm_engine_face_auto.dll`（独立名，防覆盖）；
- **置换验证**：at-gen 引擎解析序（env `AUTOTERM_ENGINE_DLL` → exe
  同目录 → 向上 4 级 target/debug）——把 Auto 面 dll 以
  `autoterm_core.dll` 之名置入 at-gen/target/debug（该目录原无同名
  Rust dll，不覆盖任何既有物），跑 parity 六场景，完毕清理。

## 3. 技术栈

auto-lang master `auto trans`（只读）、cargo cdylib、dumpbin（MSVC）、
autoterm-parity 门禁、at-gen 构建链。

## 4. 需求分析与背景调查

- **授权**：用户 2026-09-10 指示"尝试把 610 落地的工作在 autoterm
  这边补齐"——本计划立项+执行；
- 610 复审证据（auto-lang 归档件）：AC-03 dumpbin **12/12 符号集恒等**
  + 597 a2c 驱动器链接 Auto 面 CFACE_OK——本计划 G1/G2 是同一契约在
  auto-term 仓内的复刻与升级（parity 级行为验收）；
- 610 语料头注已载语义对齐说明（快照仅在 take_dirty_rows 刷新/
  kind_color 标量编码/null 哨兵统一化差异——驱动方从不传 null，六
  场景不触及该差异面）；
- at-gen 解析序与置换法先例：597 计划 T-05/归档件 §9。

## 5. 详细设计

### 转译再生成规程（at-engine-face/README 要点）

```bash
# auto-lang 主检出（只读工具链）
auto trans -p <auto-term>/at/engine_face.at rust
# 产物 <auto-term>/at/engine_face.a2r.rs → 复制为 at-engine-face/src/lib.rs
# 手写补充：无（610 产物含 auto_cabi_kit 生成桥，整文件自足）
```

### 置换验证规程（T-03）

1. `cargo build`（at-engine-face）→ 拷 dll 至
   `at-gen/target/debug/autoterm_core.dll`；
2. at-gen 下 `cargo build`（产出 autoterm-at.exe）+
   `cargo test -p autoterm-parity`（六场景，a2r 侧消费置换 dll）；
3. 对照跑：Rust 版 dll 同法（主 target 直驱）基线已知绿（011 终态）；
4. 完毕删除置换 dll（该文件为本计划新建，无备份恢复负担）。

### 规范增量

| delta_id | 操作 | target | before/after | rationale | acceptance |
|---|---|---|---|---|---|
| SD-01 | modify | docs/designs/004 §5⑤ | 立项回执→并存转正回执（Auto 面在仓可构建+parity 级验证） | 账实同步 | AC-3 |
| SD-02 | modify | DEBTS.md #10 | 增 012 条 | 同上 | AC-3 |
| SD-03 | modify | README.md 布局节 | 布局表增 at-engine-face 行 | 可发现性 | AC-3 |

## 6. 测试设计

- dumpbin 符号集对照（Rust 版 vs Auto 版，12/12 恒等断言）；
- parity 六场景（Auto 面驱动）全绿；
- 主 workspace `cargo test --workspace` 零回归（at-engine-face 独立
  workspace 不入主树，主树零改动除文档）。

## 7. 验收标准

- **AC-1**：at-engine-face 构建绿 + dumpbin 导出符号集与 Rust 版恒等
  （对照输出在案）；
- **AC-2**：parity 六场景在 Auto 面 DLL 驱动下全绿（运行证据在案）；
- **AC-3**：004 §5⑤/DEBTS #10/README 三处回执落账。

## 8. 执行步骤

- [x] **T-01** `at/engine_face.at` 落位 + 转译产物入库 +
  at-engine-face crate（Cargo.toml/README）；
- [x] **T-02** 构建 + dumpbin 符号对照（AC-1）；
- [x] **T-03** 置换验证：parity 六场景 Auto 面驱动全绿（AC-2）；
- [x] **T-04** 回执三处（AC-3）；
- [x] **T-05** 收口：主 workspace 零回归 + worktree clean 提交。

## 9. 复审记录

stage: new | PLAN-012 | rev 1 | outcome: pass | next: work
（立项+执行同轮授权；前置 610 已归档，材料齐备。）

## 10. 待澄清事项

（无——替换 dist 生产物的终局决策显式留用户，本轮并存态不触发。）
---
stage: work | PLAN-012 | rev 1 | **pass** | code_commit=ad7eae9+lock 补账 |
task_ids=T-01..T-05 | evidence=见下 | blockers=无 | next=review

### 执行证据(2026-09-11,worktree .wt/auto-012/auto-term)

- **T-01**:at/engine_face.at 真身(头注适配自 610 语料,语义对齐注记
  完整);转译 364 行=语料 expected.rs 同源(auto-lang master auto.exe
  **重建后**——首转 163 行系陈品工具链丢 #[export],时间戳对照实证
  11:24<14:05);at-engine-face crate(edition 2024,cdylib,dep
  autoterm-core path,零 auto-lang 运行时依赖)+README 再生成规程。
- **T-02**:cargo build 绿(21 警告零错);dumpbin 对照 **12/12 符号集
  恒等**(auto 侧 findstr 多两行=文件名噪声,导出名集 diff 为空)。
- **T-03**:at-gen 构建(组内 auto-lang 兄弟补齐后 1m41s);Auto 面 dll
  以 autoterm_core.dll 之名置入 at-gen exe 同目录(解析序第一命中);
  主检出 AUTOTERM_AT_BIN 置换法跑 parity:**5 passed/6.29s 全绿**
  (中断=真 ConPTY+ctrlc helper 经 Auto 面;色彩=row_style+kind_color
  标量逐格;选中=组件表面);headless 冒烟 SCENARIO_OK/exit 0;完毕
  已删置换物。
- **T-04**:004 §5⑤ 转正补账+DEBTS #10 增 012 条+README 布局行
  (worktree 内,随交付合入)。
- **T-05**:主 workspace 零改动(新增物均在 at/ 与独立 workspace
  at-engine-face/,不入主树——构造性零回归;parity 已从主检出跑绿);
  worktree clean 双提交(ad7eae9+lock)。
- **边界执行**:计划 §2 预估"无 auto-lang worktree 需求"有误——at-gen
  路径依赖需组内 auto-lang 兄弟(011 双树同款),已补 .wt/auto-012/
  auto-lang(auto-term-dev 分支),merge 清理时一并处理。
---
stage: review | PLAN-012 | rev 1 | **pass** | reviewed_commit=69f2851 |
base=4accf44 | deps: auto-lang@fbfc145bc(组内兄弟,只读工具链+at-gen 路径依赖) |
spec_inputs: 004/DEBTS/README(交付内落稿,auto-term 无 docs/specs 体系) |
acceptance: AC-1 复跑=导出行过滤后 **12/12 恒等**(执行期 diff 的噪声行=文件名
误中,已换行式过滤);AC-2 复跑=置换+parity **5 passed/6.11s**(完毕清理);
AC-3=三回执随 ad7eae9 入库(diff 计件:004+5/DEBTS+4/README+3) |
再生成验证: master 工具链重转译 vs 入库 lib.rs **字节级一致**(可复现+零手改
双证) |
suites: 主检出 cargo test --workspace --no-fail-fast=**唯 ash_integration
6/2 红**;pariat 5/0、ctrl_event 3+1ignored、sim_regression 23、live_pty/
pty_lifecycle/engine-ffi/ui 全绿 |
findings: F-1(info/基线/移交):ash_integration ctrl_c 双测(ctrl_c_aborts_
input_line/ctrl_c_event_effect_on_ash)在 main 基线即红——012 diff(at/+
docs+at-gen lock)与其消费面零重叠;**重建 ash.exe(2026-09-11)后仍红**,
新旧 ash 构建一致失败=非陈品漂移,真存量红(DEBTS #12/#13 领地/跨仓协调
或环境态),建议独立调查;F-2(info):at-gen Cargo.lock 随 auto-lang master
刷新(+77 行,依赖树追平,预期内) |
evidence: 复审会话内重执行;命令/结果摘录见上 | next: merge

---
## 合并回执(PLAN-012:r1,2026-09-11)

- stage: merge | outcome: **pass** | delivery_commit=c67579e(merge --no-ff
  plan-012-dev;含实现 ad7eae9+lock 69f2851+复审后同步 b357810)
- prepared: 复审基线零漂移(受审面 5237eb4..HEAD 无触及);规范增量三处
  随交付内落稿(auto-term 无 docs/specs 体系,载体=004/DEBTS/README)
- landed: main c67579e,wt-guard 前置 clean;主检出冒烟=at-engine-face
  构建 5.68s 产出 dll+默认 parity 5/0(Rust 引擎基线)——main 已知绿
- ledger_refreshed: .autoos/specs.json(离线 RMW+回读)P012-1..6 六节
  各+1,file→archived/012-*.md;无关条目保全
- archived: docs/plans/archived/012-engine-face-auto-adoption.md(git mv)
  + status: archived + completion_kind: delivered
- cleaned: 双 wt-guard clean(auto-term + auto-lang 组内兄弟)→worktree
  remove×2+分支 plan-012-dev(已合)/auto-term-dev 删除+组目录 .wt/auto-012
  空删
- 移交: F-1(ash_integration ctrl_c 双测基线红,重建 ash 仍红——独立调查
  建议,DEBTS #12/#13 领地);P-S 两决策留用户(dist 替换/at-gen 胶水
  Auto 化后续计划)
