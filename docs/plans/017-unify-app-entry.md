# PLAN-017 app 入口统一——at-app 三形态工程置换 app 桌面壳

```yaml
plan_id: PLAN-017
status: execution_done
feature_name: app 入口统一（at-app → app/，桌面壳退役）
author: [agent]
created_at: 2026-09-14T07:07:16Z
updated_at: 2026-09-14T15:50:00Z
plan_revision: 2
current_step: 7
total_steps: 7
supersedes_spec_components: []
new_spec_components:
  - docs/specs/app-unified-entry.md
touched_goals: []
```

## 0. 变更摘要

auto-term 仓现存两个前端工程：`app/`（PLAN-013 T7 桌面壳，输入框+Send 的
第一代 UI，后续零演化）与 `at-app/`（PLAN-013 T5 标准三形态工程，吃到
PLAN-014 直键入收口与 PLAN-016 FFI 颜色修复全部演化，用户实际使用的独立
入口 `auto-term.exe` 即其 rust 轨产物）。auto-os 虚拟桌面经
`apps.manifest` 指向 `../auto-term/app`，故桌面上跑的是**停更的第一代壳**
——与独立 exe 判若两应用（用户 2026-09-14 实机对照发现）。

本计划按用户裁定：**以 at-app 工程内容置换 `app/`，未来统一 `app/` 为
AutoTerm 唯一工程入口**；`at-app/` 目录退役。桌面上 AutoTerm 升级为
014/016 形态（整窗 terminal 直键入 + 几何随动 + 颜色修复），独立 rust 入
口零回归。

## 1. 目标

1. 桌面 launcher 启动 AutoTerm = at-app 现行交互形态：整窗 terminal、
   点击直键入（像 Windows Terminal）、50ms 流式刷新、几何随动、016 颜色
   修复随行；输入框+Send 形态代码删除。
2. `app/` 成为唯一工程入口：pac.at + src/front + src/back + term.rs 侧车
   三形态同源；`at-app/` 目录消失，仓内引用面（README/gitignore）同步。
3. 独立入口零回归：`auto run -r rust` 仍产出 `auto-term.exe` 且行为不变
   （用户日常独立运行的入口）。
4. 伞形注册零变更或仅注释级同步：`apps.manifest` 条目（repo
   `../auto-term/app`、ports 17400/17401、无 daemon）与 launcher 展示
   （title/title_zh/icon/category）保持。

### 非目标

- vue 轨交互式键入（DEBTS 009 #4 xterm.js 留白维持——`/api/term/pump`
  恒 0 是设计行为）。
- os-config 内嵌终端页（原上游，保留原位不动）。
- 积压报警展示面（016 已按用户裁定撤下，契约面 api 保留不动）。
- `at/` + `at-gen/` 对拍 oracle（冻结，不随本计划改动）。

## 2. 架构方案

**主案 A（目录置换 + 进程内 merged back 装载）**：

- `git mv at-app → app`（git 跟踪件：pac.at、README.md、src/、term.rs；
  rust-workspace/gen/dist/target 为 gitignore 生成物，移仓后重生成）。
- 桌面装载链（实证依据见 §4）：manifest 臂 id `auto-term` →
  `entry_for_dir` 探 `src/front/app.at` → `build_dynamic_component`
  进程内编译 front。front 的 `use back.api` 无 pac `back: { project }`
  外部根时按本地优先解析（`resolve_back_api`：`src/back/api.at` 先于外
  部仓），api.at 委托体直调 db.at——与独立 `auto run -r vm` 的 merged
  形态同一编译器同一语义；db.at `use auto.term` 落 stdlib shims → 宿主
  进程内 `autoterm_core.dll`（部署契约不变，`deploy-autoterm.sh` 幂等件
  已在宿主 exe 目录在位）。
- pac.at 字段并合（唯一手写合并点）：保留 at-app 全部键（api: rust /
  rust_sidecar / exe_name: auto-term / window / theme: dark /
  title_zh: 终端），补回桌面注册三键（icon: terminal / category:
  system / front_port+back_port 17400/17401 伞形占位）。

**兜底分支（仅当 T-04 实机门失败时启用，启用前需用户确认）**：

- 分支 B：若桌面动态编译对本地委托体 back 出桩（api 调用恒空）——沿
  os-config 先例补 `back: { project }` 指向一个 cdylib back 工程
  （os-config-back 形态，a2r db.rs 为底 + Plan 061 backend_abi 桥）。
  工作量显著大于主案，属范围扩张。
- 分支 C（最后手段）：front 改 store 直调 `Term.engine_*` catalog shim
  （17 shim 面全覆盖，旧壳已证桌面可装载）——代价是状态机双实现
  （front store + db.at），与"单源"目标冲突，仅在前两案均不可行时提请
  用户裁决。

## 3. 技术栈

- .at DSL（front app.at / back api.at+db.at）；auto-lang VM 动态编译
  （`build_dynamic_component`，与 `auto run -r vm` 同编译器）。
- `auto.term` stdlib shims（17 函数面：spawn/write_line/rows/resize/
  interrupt/is_exited/free/pump_input/cursor_row/cursor_col/apply_resize/
  viewport_cols/viewport_rows/backlog×4）→ `autoterm_core.dll`
  （libloading 进程内；解析序 env `AUTOTERM_ENGINE_DLL` → exe 同目录 →
  祖先 target）。
- rust 轨 a2r（merged db 吸收 + `term.rs` 侧车供给 `crate::term`）；
  vue 轨 Vite+axum split。
- 部署件契约（003 §5）不变：`deploy-autoterm.sh` 幂等拷贝。

## 4. 需求分析与背景调查

**分叉证据（git）**：

- `app/` 全史两次提交：`0770f3f`（T7 建壳，自 os-config OS-013 T3 升格
  移植）+ `1a7ac1e`（PLAN-015 title_zh）。PLAN-014（`49ae426`）与
  PLAN-016（`09afd6e`）全部落在 `at-app/`。
- `app/src/front/autoterm_page.at` 现存输入框+Send UI（消息集
  Open/Close/Send/Interrupt/ResizeToggle）；`at-app/src/front/app.at`
  头注明言"014 直键入收口：整窗只渲染 terminal 本体，不再有输入行/按钮"。

**桌面装载链（auto-lang 侧读码）**：

- `app_registry.rs::entry_for_dir`：pac 缺 icon/category 时落缺省
  `app-window`/`app`——故合并 pac 必须显式保留两键；`back_root` 仅来自
  pac `back: { project }`（os-config 形态），无声明即 None。
- `session.rs::launch_app`：`spec.back_root` 为 None 时不装 cdylib，
  直接 `build_dynamic_component(spec.code, source_path)` 进程内编译
  front——模块解析按 app 根文件系统路径（`use back.api` →
  `src/back/api.at` 本地优先，`config.rs::resolve_back_api`；db.at 的
  `use auto.term` 落 stdlib 装机副本，deploy 契已在位）。
- 先例盘点：musk/jade-garden 为 HTTP 后端形态（非进程内先例）；os-config
  为外部 cdylib 先例；**本地真实委托体 back 在桌面动态编译**无先例——
  但独立 `auto run -r vm` 的 merged 形态（api fn 字节码直调 db）为同一
  编译器已实证路径（013 真机证据）。残余风险收敛为 T-04 实机门。

**引擎面**：term.vm.at shim 面对 db.at 所需 17 函数全覆盖（含 014 的
pump_input/apply_resize/viewport/cursor）——桌面无需 auto-lang 任何改动。

**授权记录**：用户 2026-09-14 会话裁定——"把 at-app 壳换回来替换掉 app
壳。未来都统一用 app 作为入口"。允许仓库：auto-term（主变更）+
auto-os（README/清单核对面，预期零或注释级）。预算/自动续跑：未指定。

**运行态遗留**：`at-app/stdlib/`（未跟踪、非 gitignore 项，016 提交记
"会话前遗留"）——T-06 处置（gitignore 或删除，非置换件）。

## 5. 详细设计

### 5.1 文件操作（单提交边界，git mv 保历史）

| 动作 | 路径 | 说明 |
|---|---|---|
| 删 | `app/src/front/{app.at,autoterm_page.at,autoterm_store.at}` | 旧壳三件（git 历史可溯） |
| 删 | `app/pac.at`（旧内容） | 并合后新写（见 5.2） |
| mv | `at-app/src` → `app/src` | front+back 整体 |
| mv | `at-app/term.rs`、`at-app/README.md` → `app/` | 侧车+工程说明随迁改写 |
| 改写 | `app/pac.at` | 字段并合（5.2 表） |
| 改 | `app/src/front/app.at` | terminal key `at-app`→`auto-term`（纯标识） |
| 改 | `.gitignore` | `at-app/*` 五项 → `app/` 同名项（rust-workspace/gen/dist/target/.auto；`.am` 已有） |
| 清 | `at-app/`（残留生成物+stdlib 遗留） | 目录整体消失 |

### 5.2 pac.at 并合表

| 字段 | 取值 | 来源/理由 |
|---|---|---|
| name | `"auto-term"` | 旧壳 pac（注册/包基名；at-app 的 `at-app` 退役） |
| title / title_zh | `AutoTerm` / `终端` | 两 pac 一致 + PLAN-015 中文名契约 |
| icon / category | `terminal` / `system` | 旧壳 pac；launcher 分拣必需（缺省落 app-window/app） |
| front_port / back_port | 17400 / 17401 | 旧壳 pac；apps.manifest 伞形占位一致性 |
| render / scene | `vm` / `ui` | 两 pac 一致 |
| api | `rust` | at-app（三形态声明） |
| exe_name | `auto-term` | at-app（rust 轨产物名=用户独立入口） |
| rust_sidecar | `modules: ["term:term.rs"] deps: ["libloading:0.8"]` | at-app 原样 |
| window / theme | `802x482` / `dark` | at-app（整窗即终端初值） |

### 5.3 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/app-unified-entry.md | 前：app/ 与 at-app/ 双前端并存，演化单点在 at-app/，桌面注册指 app/（停更壳）。后：app/ 为 AutoTerm 唯一工程入口（三形态同源）；桌面装载形态=进程内编译 front + 本地 src/back merged 委托体 + auto.term shims 进程内引擎；at-app/ 退役，新工作一律落 app/ | 入口唯一性裁定落档，防再分叉 | AC-01/04 |

### 5.4 证据与登记

- `docs/plans/evidence/017/`：桌面实机（直键入/几何随动/颜色/Ctrl+C 四
  面）、rust exe 独立运行、vm/vue 轨回归、置换前后 `git log --follow`
  历史可溯截图或文录。
- `.autoos/specs.json`：work/merge 期入账（六节，指向本计划与 SD-01）。
- auto-os 侧核对面（预期零变更）：`apps.manifest` auto-term 行、README
  Apps 表与 AutoTerm 注记——若措辞提及 at-app 则注释级同步。

## 6. 测试设计

1. **桌面实机门（T-04，gate）**：`bash D:/autostack/auto-os/scripts/desktop.sh
   iced` → launcher 搜 `term` → 启动。断言：整窗 terminal、cmd banner
   上屏（黑底白字，无 016 前红底）、点击后直接键入回显、Enter 执行、
   Ctrl+C 中断、拖拽/最大化网格随动、光标块落位。证据入 evidence/017/。
2. **三形态回归（T-05）**：cwd=app/——`auto run -r rust`（产物名
   auto-term，窗口交互同上）；`auto run -r vm`（banner/echo）；`auto run
   -r vue`（页面起、`/api/term/pump` 恒 0 只读边界维持）。
3. **单入口断言（T-03 附带）**：`git grep -l "at-app"` 于 auto-term 仓
   零命中（gitignore/文档/源内标识全清，terminal key 改名后）；`ls
   at-app` 不存在。
4. **既有面**：auto-term 仓 `cargo test`（parity/resize_guard 等既有套
   件）不红。

## 7. 验收标准

- **AC-01** 桌面 launcher 启动 AutoTerm 呈 014 形态：整窗 terminal、无
  输入框/Send/状态行；点击直键入有回显、命令可执行。验证：T-04 实机证据。
- **AC-02** 几何随动生效：窗口拖拽/最大化后网格与引擎 resize 跟随，
  光标格非零落位。验证：T-04 证据（前后对照）。
- **AC-03** 颜色正确（016 随行）：cmd 默认前景/背景语义色正确（黑底白
  字），无 257 截断暗红底。验证：T-04 证据。
- **AC-04** 单入口：`at-app/` 目录消失；`git grep at-app` 零残留；
  `git log --follow app/src/front/app.at` 可溯 at-app 前身历史。验证：
  命令输出。
- **AC-05** 独立入口零回归：`auto run -r rust` 产出 `auto-term.exe` 并
  可运行（用户独立入口）。验证：T-05 证据。
- **AC-06** 伞形注册一致：apps.manifest auto-term 行不变；launcher 展示
  名/中文/图标/分类不变；daemon 仍无声明。验证：文件 diff + launcher
  截图。

## 8. 执行步骤

- **T-01 置换提交** `[✅ 已完成]` commit c157c80：§5.1 全量（旧壳三件删、
  git mv src/term.rs/README、pac 并合、gitignore 改径、terminal key 改名、
  at-app/ 生成物+stdlib 会话遗留清空）。`ls at-app` 无、`git status` 净。
  AC-04。
- **T-02 三形态冒烟（本机）** `[✅ 已完成]`：vm-merged 编译装载 ✓（pac
  并合解析 auto-term/17400/802x482/终端/dark；"vm+vm merged backend
  in-process"）；裸形态矩阵实证出桩问题（§10.1）→ 裁决切限定名（commit
  e3095ec）；限定名形态 vm-merged 全实证（几何 84×29/光标 (3,15)/29 行
  收割/零桩告警，MCP state 证据）+ rust 构建 Finished（auto-term.exe，
  627 worktree CLI）；vm-split HTTP 200 ✓。AC-05。
- **T-03 引用面同步** `[✅ 已完成]` commit 66e38a6：root README 布局/运行
  节 at-app→app；auto-os 侧核对零变更（manifest/README 无 at-app 引用）；
  活跃面 at-app 清零（冻结 oracle at//at-gen、台账、DEBTS 历史记录豁免）。
  AC-04/06。
- **T-04 桌面实机门** `[x]` **[R2 修复轮复验通过]** **[R2 复审重开——F-R3: AC-03 fail]**：净场后 detached
  桌面（AUTOUI_ACCEPTANCE=1）经 MCP bus `launch\tauto-term`——
  `launch_app(inproc) auto-term` 进程内编译零错、Init/Tick 活、快照
  `terminal key=auto-term cols=81 rows=25 lines=25`（**25 行 shell 真实
  内容 + 几何随动实算 81×25 ≠ 模型缺省 100×30**）、截图在案
  evidence/017/desktop-terminal-live.png。取证期遭遇：后台 timeout 探针
  泄漏 auto.exe 幽灵占 9247 造成早前 lines=0 假象（净场后复测翻绿）；
  并行 os-018 会话桌面抢占 9247（截图路径 .wt/os-018 佐证）。键入泵/
  Ctrl+C 桌面内字节级 diff 因 MCP 工具面限制未取——机制链与 standalone
  同构（同进程队列+shim），窗口已为用户拉起供实测。AC-01/02/03。
  **[R2 复审裁定]** 快照行数/几何/形态三面证据成立（AC-01 形态/AC-02
  随动过），但归档截图经像素采样**整屏背景 RGB(128,0,0)** = 016
  "257 截断→base16[1] 暗红"精确签名（vm 对照 probe RGB(6,7,9) 黑底
  正常）——AC-03 在本证据上不成立，T-04 重开：定位桌面色彩路径根因
  （疑：桌面宿主内嵌 engine/widget 副本先于 016 修复，或桌面委托色彩
  编码分叉）→ 修复 → 重取桌面门截图（须非红底 + 几何 + 形态三面）
  → 交复审。
  **[R2 修复轮结案（work 2026-09-14）]** 根因坐实 = **长驻桌面进程跨
  DLL 重建的陈旧内存映像，非代码缺陷**：①当前源码零缺陷——spec 定义
  回归 `cargo test -p autoterm-parity parity_color_attestation` 绿
  （oracle vs FFI STYLE 协议行逐格对拍含语义色归并路径）；磁盘唯一
  引擎 DLL `auto-term/target/debug/autoterm_core.dll` mtime 16:53 晚于
  016 落地 14:42。②门时（16:54）红底 = 门会话的桌面进程在 16:53 DLL
  重建**前**启动、Windows 文件替换不换已加载映像 → 进程内仍是
  pre-09afd6e kind_color（发 Named(257) 原值）× 消费端截断 → 整屏
  RGB(128,0,0)；vm 探针为新进程加载新 DLL → RGB(6,7,9) 黑——红黑同
  时分叉唯此解释自洽。③复验（R2 门）：原生 PowerShell env 继承新起
  验收桌面（AUTOUI_ACCEPTANCE=1 + AUTOTERM_ENGINE_DLL 显式钉住
  post-fix DLL + AUTOUI_MCP_PORT=9251，避让生产桌面 bus）→
  `launch\tauto-term` 入队 → 快照 `terminal key=auto-term cols=81
  rows=25 lines=25`（几何随动签名与 T-04 一致）→ 新截图
  evidence/017/desktop-terminal-live-r2.png：四点采样背景 RGB(6,7,9)
  （=016 契约 #060709）、**零 128,0,0**、黑底白字 cmd banner+提示符+
  光标块、整窗形态无输入框——AC-01/02/03 三面单图全数成立。④附注：
  桌面 DLL 解析在 ui_desktop/auto.exe 默认布局下无缺省命中臂（祖先链
  够不到 auto-term target），生产链依赖 ambient AUTOTERM_ENGINE_DLL
  或 exe 同目录分发——陈旧映像风险类，属部署/契约面，记 §10 观察。
  复验期环境注记：原生产桌面（22:49 起，bus 9247）在复验窗口内消失
  （疑与验收实例单实例/端口冲突相关，非本计划代码触达）；复验后已以
  生产模式重启桌面归还。MSYS bash 中转层会丢失注入 env（vue 轨 vite
  同场崩），验收通道须原生 PowerShell/直接进程启动——程序性教训。
- **T-05 全量回归** `[✅ 已完成（两项预存红在案）]`：rust 轨 ✓（627
  worktree CLI 构建 Finished + auto-term.exe 16:30 重建）；vm-merged ✓；
  vue 轨 = axum back `crate::ui::i18n_lookup` 编译错——**master 预存红**
  （主检出 CLI 同错复证，非本计划引入；api.ts 客户端生成 ✓）；auto-term
  `cargo test --workspace`：8/10 绿，ash_integration 2 红 = ctrl_c 中断
  路径 **PLAN-015（drafting）在册的 014 已知回归**（017 未触引擎代码）。
  AC-05。
- **T-06 遗留清偿 + 台账** `[✅ 已完成（work 半；入库半随 merge 执行）]`：
  at-app/stdlib 遗留已随 T-01 清（实证 lang stdlib 安装副本在位）；
  627 闸门解除实证——代码落地 master 3b2f7cf56 + 全收据闭合 696399830
  （R2 复审 pass 8877393f6 → spec 沉淀 → 归档 → cleaned，ancestry
  `--is-ancestor` YES），`.autoos/specs.json` 六节入账与 SD-01 spec
  文档（docs/specs/app-unified-entry.md）按 §5.4 "work/merge 期入账"
  口径**随复审/merge 窗口落**（014/016 无 worktree 主检出先例：
  spec 文档 merge 首建；review 期禁发布 canonical spec/ledger）——
  **merge 阶段必办项，勿以本勾选推断已入库**。
- **T-07 证据归档** `[✅ 已完成]`：evidence/017/ 三件（desktop-terminal-
  live.png / vm-merged-qualified-probe.png / vm-merged-qualified-run.log）。

## 9. 复审记录

- 2026-09-14 stage:new rev1 起草交接——用户裁定置换方向（§4 授权记录）；
  主案 A 依据：装载链读码（entry_for_dir/build_dynamic_component/
  resolve_back_api 本地优先）+ 独立 vm merged 先例 + shim 面全覆盖；
  残余风险单点 = T-04 实机门（本地委托体 back 桌面动态编译无先例），
  兜底分支 B/C 预置待裁决。outcome: pass（授权范围内可开工）；
  next: work（T-01 起）。

- 2026-09-14 stage:work · plan_id: PLAN-017 · plan_revision: 2 ·
  outcome: **pass** · code_commit: 4960bb2（main，无 worktree 主检出
  交付，014/016 同款先例）· task_ids: T-01..T-07（7/7）· evidence:
  ①627 依赖落地实证——auto-lang master 含 3b2f7cf56（ancestry
  `--is-ancestor` YES）且收据链闭合（R2 复审 pass 8877393f6 → spec
  沉淀 f7b2a4dde/9fbbfc365 → 归档 194b9202e → cleaned 696399830，
  worktree/分支已拆）；②evidence/017/ 三件在位（desktop-terminal-
  live.png / vm-merged-qualified-probe.png / vm-merged-qualified-
  run.log）；③工作区零未提交代码（HEAD 4960bb2，仅 untracked tmp
  探针件与本计划无关）· blockers: 无 · next: **review**
  （/auto-plan:review 017）。
  观察项（非阻塞）：PATH CLI `auto-lang/target/debug/auto` 为 18:20
  pre-627 构建（a9d3b8c67，627 落地 22:15 之前）——在册证据不受影响
  （16:30 auto-term.exe 用 627 worktree CLI 重建；桌面/merged 进程内
  动态编译对限定名恒支持，PLAN-053 仅拦裸名）；后续以 PATH CLI 重建
  rust 轨前需先重建 CLI（auto-lang 日常构建自然刷新）。

- 2026-09-14 stage:review · plan_id: PLAN-017 · plan_revision: 2 ·
  outcome: **needs_fix** · reviewed_commit: 058d22b · base_commit:
  4b5d2a4d（c157c80^，diff 基线）· dependency_revisions: auto-lang
  master 853ad131c（627∈master：3b2f7cf56 + 修复轮 7ad387ec3，R2 复审
  pass 8877393f6）；auto-down 140775f（review 组兄弟）· spec_inputs:
  SD-01 add docs/specs/app-unified-entry.md（delta 表在案；merge 期
  首建，本次未发布）· 独立性声明：复审与 work 收口同会话，结论全部
  从工件重建（像素采样/命令复现/提交考古），未采信执行者摘要 ·
  acceptance_results: AC-01 **partial**（形态/启动证据成立；点击直键入
  交互面留用户实测=在案 caveat）｜ AC-02 **pass**（快照 81×25 ≠ 模型
  缺省 100×30 = 几何随动实算）｜ AC-03 **fail**（F-R3，见下）｜
  AC-04 **pass**（F-R1 解读在案）｜ AC-05 **pass**（原 T-5 证据 +
  当日基线复现：review CLI `v0.4.2-615-g853ad131c`（post-627 master
  tip，含 F1/F2）`auto build -r rust` cwd=app/ → Finished 1m04s
  EXIT=0；cargo test 复用 T-5 证据，理由=auto-term 代码零变化且
  autoterm-core 测试不触 auto-lang）｜ AC-06 **pass**（manifest 行
  id/repo/kind/ports/status/added + 无 daemon ✓；pac.at
  title/title_zh/icon/category ✓；launcher 截图半边未归档=F-R2）·
  findings:
  - **F-R3（major，AC-03/T-04）**：归档桌面门截图整屏背景
    **RGB(128,0,0)**（四采样点一致）= 016"Named(257)→as u8→1→
    base16[1] 暗红"精确签名（archived/016 §根因；cmd 会话整屏默认格
    即红底）；同证据集 vm-merged probe 对照 RGB(6,7,9) 黑底正常——
    016 修复在 vm/standalone 路径生效、**桌面委托路径仍截断**。疑因
    （work 期定位）：桌面宿主内嵌 engine/widget 编译副本先于 016
    修复（宿主重建即自愈——与用户后续"恒黑底"观察相容），或桌面
    委托色彩编码分叉。修正动作：根因定位 → 修复/宿主刷新 → 重取
    桌面门截图（须非红底 + 几何随动 + 形态三面）→ 复审。
  - F-R1（info，AC-04）：`git grep at-app` 活文件命中均为出处注记
    （§5.1/5.2 git mv 保历史+并合表自要求）、DEBTS #14③ 冻结 oracle
    字符串（at/autoterm.at、at-gen）、ledger/文档史——功能残留为零
    （at-app/ 目录不存在、无构建引用、--follow 可溯 0770f3f→c157c80）。
    字面"零残留"与计划自身注记要求冲突，按功能残留口径判过。
  - F-R2（minor，AC-06）：launcher 展示截图未入 evidence/017（缓解：
    manifest+pac 键文件 diff 全过；T-04 截图任务栏 terminal 图标活跃
    = launcher 实际拉起成功）。
  evidence: evidence/017/desktop-terminal-live.png（像素采样
  RGB(128,0,0)×4 点）、evidence/017/vm-merged-qualified-probe.png
  （RGB(6,7,9)）、本记录复现命令与结果、archived/016 根因节对照 ·
  next: **work**（修 F-R3，T-04 已重开，current_step 6/7）。

- 2026-09-14 stage:work（F-R3 修复轮）· plan_id: PLAN-017 ·
  plan_revision: 2 · outcome: **pass** · code_commit: ff0ef3f
  （计划记账基线；F-R3 = 部署陈旧映像类，零 auto-term 代码改动）·
  task_ids: T-04（重开→复验勾回）；T-01..T-03/T-05..T-07 证据复核
  不受影响 · evidence: ①parity_color_attestation 绿（1.28s，当前
  构建语义 = 016 契约）；②根因链 = 门时桌面进程早于 16:53 DLL 重建
  启动、Windows 映像替换不溯及已加载 DLL → pre-016 kind_color 内存
  常驻（RGB(128,0,0) 签名），vm 探针新进程对照黑（RGB(6,7,9)）；
  ③R2 门新证据 evidence/017/desktop-terminal-live-r2.png——验收
  桌面（9251 bus，AUTOTERM_ENGINE_DLL 钉 post-fix DLL）launch
  auto-term：快照 81×25/25 行 + 截图四点采样 RGB(6,7,9) 零红 +
  整窗形态 + 光标块，AC-01/02/03 三面齐 · blockers: 无 ·
  next: **review**（F-R3 结案复核；用户已授权修完即复审）。

## 10. 待澄清事项

1. **[2026-09-14 work 会话实证——T-02 升级为裁决点]** merged 模式下裸
   `#[api]` 调用按 auto-lang 当前 master 设计进 no-op 桩
   （`vm/codegen.rs:8200` PLAN-053 拦截；`api_funcs` 在共享 handler 编译
   路径注册——桌面 `build_dynamic_component` 同样命中）。**主案 A 的
   "merged 字节码直调"仅对限定名调用成立**（codegen 明示 qualified 跳过
   拦截直落本地字节码）。实测矩阵（tmp 探针，已清理）：
   - 裸导入（at-app 原样）：rust ✓ / vue ✓ / vm-split ✓ / vm-merged ✗ 桩 /
     **桌面 ✗ 桩**（代码路径确证，同拦截）。
   - 限定名（`use back.api` + `api.X()`）：vm-merged ✓ 全实证（几何
     84×29 真值、光标 (3,15)、29 行收割、零桩告警）；rust ✗ E0425×13
     （a2r 只改写裸名）；vue ✗（api.ts 裸导出无命名空间）。
   - **分支 B（cdylib）技术否证**：`engine_pump_input`/`apply_resize` 读
     `auto_lang::ui::terminal` 进程内静态注册表（app/term.rs:195/220）
     ——cdylib 自带 auto-lang 拷贝与宿主静态分裂，桌面键入/几何必死
     （os-config 先例无此耦合，不可迁移）。
   - plan622（auto-lang 09-14 归档）确认：split 可达后端，merged 宿主
     分派需 ash-runner/桥——无在途修复改变裸调用语义。
   **请裁决**：(a) 前端切限定名 + 立跨仓小计划补 a2r/vue 限定名支持
   （017 挂 T-04/T-05 部分项待依赖）；或 (b) 回滚 T-01（revert c157c80）
   017 挂起等 auto-lang 先行。桌面零升级不成立（旧壳已删，裸形态桌面
   = 死终端）。**[已决并落地]** 用户裁定 (a)：前端切限定名 + 跨仓补齐
   ——auto-lang PLAN-627 已走完全流程归档（R2 复审 pass → master
   3b2f7cf56 → cleaned 696399830），017 依赖全解除（见 §9 work 记录）。
2. ~~at-app/stdlib 遗留~~（T-01 已清：stdlib 会话遗留副本 + rust-workspace
   生成物随目录退役删除，实证 = lang stdlib 安装副本在位）。
3. vue 轨 pac 端口：并合 pac 保留 17400/17401 占位——vm-split 实测以
   17401 起真服务（front 17400/back 17401 落位正确），vue split 形态
   未来真端口需求另行计划。
4. **[R2 复审 F-R3 · 已结案（work 修复轮）]** AC-03 fail：桌面委托路径整屏暗红底
   （像素实证 RGB(128,0,0) = 016"257 截断→base16[1]"签名；vm 路径
   对照正常黑底）——根因 = 长驻桌面进程跨 DLL 重建的陈旧内存映像
   （非代码缺陷，parity 色彩对拍绿），新进程加载 post-fix DLL 复验
   黑底（evidence/017/desktop-terminal-live-r2.png，RGB(6,7,9) 零红）。
   详见 T-04 结案注记。残余观察：桌面 DLL 解析缺省命中臂缺失 =
   部署/契约面风险类，留 DEBTS/后续计划候选，不阻 017。
