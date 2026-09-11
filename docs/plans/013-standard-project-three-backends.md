---
plan_id: PLAN-013
status: executing            # drafting → executing → execution_done → reviewed → archived
feature_name: standard-project-three-backends
author: [ZCode]
created_at: 2026-09-11
updated_at: 2026-09-11
plan_revision: 1
current_step: 0
total_steps: 7

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

affects: [at-app/(新), at-gen(冻结不改), docs, auto-lang(兄弟仓 4 项)]

---

# [PLAN-013] standard-project-three-backends：标准 Auto 工程三形态运行（`auto run -r rust|vm|vue`）

## 0. 变更摘要

把 AutoTerm 复刻应用从"手工 curated 产物 crate（at-gen）"扩展出**标准
Auto（automan）工程形态**：新工程 `at-app/`（pac.at + src/front + src/back
+ 引擎侧车），使三种标准命令可直接构建运行：

| 命令 | 形态 | 引擎链 |
|---|---|---|
| `auto run -r rust` | a2r 合并进程内（iced，AutoUI rust 后端） | 生成工程内 `crate::term` 侧车 → autoterm_core.dll |
| `auto run -r vm` | VM 解释执行（动态轨，OS-013 两臂已在 master） | `auto.term.*` shims → 同一 DLL |
| `auto run -r vue` | Vite web 前端 + a2r axum back（HTTP） | back crate 内 `crate::term` 侧车 → 同一 DLL |

为闭合三形态，auto-lang 侧补四件小特性（详见 §5）：VM 裸名 alias、
生成工程 .rs 侧车机制、ui_gen rust/vue 两枚 terminal 臂。

**非目标**：不替换 at-gen 与 dist 生产物（冻结为 oracle，退役留后续
计划）；不做 web 交互级 terminal（选中/回滚 UI 等 xterm.js 类交互，
DEBTS 009 #4 留白维持）；不改引擎 12 符号面契约；不动 Unix 基座。

## 1. 目标

1. `at-app/` 标准工程三命令可跑，echo/resize/interrupt 三场景各自
   实测通过（AC-01/02/03）。
2. 调用面统一：应用源以 `use auto.term` + 裸函数调用单一写法跨三
   后端（AC-04）。
3. 侧车机制在库并可断言（AC-05）；本仓既有门禁/at-gen/dist 零回归
   （AC-06）。
4. Spec 增量落档（三形态运行契约 + 侧车机制 + 调用面裁定）。

**成功样态**：用户在 `at-app/` 目录下依次敲三条 `auto run -r …`，
分别得到 iced 窗口、VM 窗口、浏览器页面，三者驱动同一个
`autoterm_core.dll`，行为与 at-gen 对拍一致（echo 锚点/resize/中断）。

## 2. 架构方案

### 2.1 关键裁定 D1：统一导入面 = `use auto.term`（VM 原生命名空间）

现状两套导入面互不相通（探查证据，2026-09-11）：

| 面 | a2r（`auto run -r rust/vue`） | VM（`auto run -r vm`） |
|---|---|---|
| `use.rs crate::engine`（现 autoterm.at 写法） | 逐字 emit `use crate::engine::{…}`，但生成工程**无供给机制**（单 main.rs + 固定 Cargo.toml，rust_ui.rs:430-442/1734-1758） | 动态 UI 路径显式跳过（lib.rs:3654-3658），符号运行期未定义 |
| `use auto.term`（VM shims，native_catalog 2943-2949） | emit `use crate::term::{…}`（trans/rust.rs:16192-16202），生成工程含 `term` 模块即闭合 | **原生支持**（stdlib.rs:10361 实测 in-library） |

**裁定**：应用源统一 `use auto.term: engine_spawn, engine_write_line,
engine_rows, engine_resize, engine_interrupt, engine_is_exited,
engine_free`（7 规范名与现 at-gen/src/engine.rs 胶水完全同形）+
**裸函数调用**（与现 autoterm.at 调用面零改）。闭合条件：

- VM 侧：native_catalog 增 7 个**裸名 alias**（hardcoded extra aliases
  先例：vm/native.rs:279-287；否则裸调用 canonicalize 成
  auto.engine_spawn 落空，vm/native.rs:140-152）——T-01；
- a2r 侧：生成工程含 `crate::term` 模块（侧车供给）——T-02/T-05。

### 2.2 关键裁定 D2：引擎胶水供给 = pac.at 声明的 .rs 侧车机制

`auto run` 的两个生成点（rust front 工程、back crate）均无用户 .rs
供给机制（探查 §2 结论；back crate 仅吸收 db.at→db.rs 转译，
api_gen.rs:652-688）。新增最小机制：pac.at 声明侧车模块清单 +
Cargo 依赖，生成时复制入位并对两侧生成点生效（字段 schema 执行期
T-02 定，倾向 `rust_sidecar: { modules: [...], deps: [...] }`）。
侧车内容 `term.rs`：源自 `at-gen/src/engine.rs` 手写胶水（libloading
包装 autoterm_core.dll，解析顺序契约 003 §5），模块路径改
`crate::term`。

### 2.3 关键裁定 D3：terminal 组件两枚生成器臂（最小实现）

- `ui_gen/rust.rs`（map_tag 现 无 terminal 臂，探查 §4）：臂 →
  auto_lang::ui::terminal::iced::Terminal 真身组件（renderer.rs:4049
  同款接线），props cols/rows/lines + 基础事件；
- `ui_gen/vue.rs`（现 unknown-tag 兜底 `<div>`，ui_gen/vue.rs:8100）：
  最小只读视口 `<pre>` 等宽渲染 lines + 滚动；输入走 app 自带输入行
  经 back api。xterm.js 类交互**不属本计划**（留白维持）。

### 2.4 工程布局（auto-term 仓新增 `at-app/`）

```
at-app/
  pac.at                  # name "at-app"; scene "ui"; api "rust"; 侧车声明
  src/front/
    app.at                # AutoUI:terminal 组件 + 输入行 + 键/拍驱动(纯 View 面)
  src/back/
    api.at                # #[api] 契约面: tick/send_line/resize/interrupt/is_exited
    db.at                 # TermApp 状态机移植自 at/autoterm.at
                          # (back crate 用户逻辑唯一吸收面=db.at→db.rs,api_gen.rs:666;
                          #  文件名语义违和属机制现状,注记不改)
    term.rs               # 侧车:at-gen/src/engine.rs 改 crate::term(唯一新 .rs)
```

`at/autoterm.at` 与 at-gen **冻结不动**（对拍 oracle + dist 生产物）；
TermApp 状态机在新工程内为移植副本（~百行级，临时重复，at-gen 退役
时收敛，注记入 DEBTS）。

### 2.5 三形态运行链（同一 DLL 三种宿主）

```
                    ┌─ rust: iced(AutoUI rust 后端) ── in-process api ──┐
at-app/front ───────┼─ vm:   AutoVM 动态轨(aura_view_builder) ─ auto.term┤→ autoterm_core.dll
                    └─ vue:  Vite 页面 ── HTTP axum back ── crate::term ┘
```

VM 形态的渲染两臂与 shims 已在 auto-lang master（731fa57a3；
OS-013 桌面冒烟 ALL PASS 在案），本计划只补裸名 alias 与工程化。

## 3. 技术栈

- auto-term：Rust workspace（不动）、automan 标准工程（pac.at/.at）。
- auto-lang（兄弟仓 master + worktree）：CLI/automan/ui_gen/vm。
- 运行环境：Windows + ConPTY（autoterm_core.dll）、npm/vite（vue 形态，
  015-notes 先例）、MSVC 工具链（a2r 构建已有先例）。

## 4. 需求分析与背景调查

**授权（用户，2026-09-11 会话）**："我建议做一个小计划。用标准的
Auto 工程和对应命令来跑是合理的。另外 Vue 版也应该用 `auto run -r
vue` 来跑；VM 版也应该用 `auto run -r vm` 跑。" 范围：本仓工程新增 +
auto-lang 兄弟仓小特性（本仓双仓计划先例：PLAN-009/011）；预算：
小计划量级（7 任务）。无自动续跑预算；跨仓任务在执行计划内直接
授权（009/011 惯例），auto-lang 侧走其 worktree/分支惯例。

**背景调查证据**（探查 + 源码核对，2026-09-11，D:/autostack/auto-lang）：

| # | 事实 | 证据 |
|---|---|---|
| B1 | pac.at 必需；src/front(app.at 入口) + src/back/api.at 契约；#[api] 提取 + `use back.api` 客户端生成 | automan.rs:146-149;rust_ui.rs:30-40/2606;config.rs:345-355;api_gen.rs:2353 |
| B2 | rust 生成工程单 main.rs + 固定 Cargo.toml，无 .rs 供给；back crate 仅 db.at→db.rs | rust_ui.rs:430-442/1734-1758;api_gen.rs:652-688 |
| B3 | `use.rs` a2r 逐字 emit；VM 动态路径跳过、符号失联 | trans/rust.rs:16299+;lib.rs:3654-3658;compile.rs:603-611 |
| B4 | `use auto.term` VM 原生可用（in-library 实测）；a2r emit `use crate::term::` | stdlib.rs:10361;trans/rust.rs:16192-16202 |
| B5 | VM terminal 渲染两臂已折 master；shims 语义对齐 at-gen 胶水 | 731fa57a3;term_engine.rs:1-14;renderer.rs:4049;aura_view_builder.rs:8684 |
| B6 | ui_gen rust/vue 均无 terminal 臂（vue 兜底 `<div>`） | ui_gen/vue.rs:7879/8100;ui_gen/ 全目录零 "terminal" |
| B7 | VM 裸名 alias 有 hardcoded 先例；native 名 canonicalize 首段小写 | vm/native.rs:279-287;vm/native_registry.rs:270-300 |

**本仓现状**：at/autoterm.at（单一 .at,use.rs crate::engine,裸调用）；
at-gen（app_logic.rs 转译入库 + engine.rs/shell.rs 手写,独立 crate,
dist 三件套流程在库）；parity 门禁六场景（crates/autoterm-parity）。

**关联活动计划**：无（docs/plans/ 仅 archived,最大 012）。
**关联债**：DEBTS 009 #4（Vue/web terminal 留白——本计划收窄为
最小视口,xterm.js 交互仍留白）；F-1 ash 基线红（与本计划无关,维持
移交）；PLAN-012 遗留"at-gen engine.rs 胶水 Auto 化"（本计划的
term.rs 侧车是其前置形态,Auto 化仍留后续）。

## 5. 详细设计

### 5.1 auto-lang 侧四件（T-01..T-04,全部向后兼容 additive）

1. **T-01 裸名 alias**：register_std_shims/native 注册处为
   auto.term 的 7 规范名增裸名 alias（native.rs:279-287 同款表）。
   验证:term_engine_shims_echo_roundtrip 增裸调用变体（或新测试）。
2. **T-02 .rs 侧车机制**：pac.at 新字段（schema 执行期定）;两个
   生成点各一段:复制模块文件入 src/ + Cargo.toml deps 注入 +
   main.rs/lib.rs `mod` 声明。幂等（重复生成不重复注入）。验证:
   单测——临时工程带侧车生成后断言文件在位、Cargo.toml 含 dep。
3. **T-03 ui_gen rust terminal 臂**：tag "terminal" → 真身组件
   接线（cols/rows/lines props;事件经既有消息机制）。验证:生成
   快照 + 生成工程 cargo check 绿。
4. **T-04 ui_gen vue terminal 臂**：`<pre>` 等宽只读视口（lines
   逐行 + overflow 滚动）;a2vue 金样再生。验证:金样 diff 入库。

### 5.2 at-app 工程（T-05）

- front/app.at：terminal 组件（bind cols/rows/lines 到 state）+
  输入行（回车 → send_line）+ Ctrl+C 臂（→ interrupt）+ tick 拍
  （tick() 收割快照回填 lines）。
- back/db.at：TermApp（handle/cols/rows/lines/exited）+ new/tick/
  send_line/resize/interrupt/is_exited/free,`use auto.term` 裸调用。
- back/api.at：#[api] 七面（薄代理到 TermApp 实例;实例宿主 lazy_static,
  015-notes db 惯例）。
- sidecar term.rs：at-gen/src/engine.rs 复制改 `crate::term`
  （解析顺序/降级语义逐行保留）。
- pac.at：scene "ui";api "rust";侧车声明;render 缺省（命令行 -r 覆盖）。

### 5.3 执行期核查点（有界,不扩范围）

- C1:a2r 对 `use auto.term` 的 companion-import/裸调用转译行为
  （trans/rust.rs:16304 表）;若有出入 → fallback 调用面 `Term.` 限定,
  两臂对齐（改动限 at-app 源,不改裁定 D1 的命名空间统一）。
- C2:merged rust 形态 api 直调对象（generate_merged_api_client
  rust_ui.rs:630-636/1032）——确认侧车注入点覆盖 merged 路径。
- C3:vue 形态 back crate 侧车注入点（api_gen 生成流程内）。

### 5.4 规范增量

| delta_id | add/modify/retire | target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/designs/005-standard-project-three-backends.md | 无→三形态运行契约+侧车机制+调用面裁定(D1..D3)全文 | 标准命令形态首次入账 | AC-01..05 |
| SD-02 | modify | DEBTS.md 009 观察 #4 | "Vue/web terminal 留白"→"最小只读视口已通(013);xterm.js 类交互留白维持" | 口径随事实收窄 | AC-03 |
| SD-03 | modify | DEBTS.md #10 / README.md 布局节 | 增 at-app 工程行与三命令运行节;PLAN-012 遗留"胶水 Auto 化"注记侧车前置 | 可发现性 | AC-06 |

## 6. 测试设计

- **单测（auto-lang）**：裸调用 alias（T-01）;侧车生成断言（T-02）;
  rust 臂生成快照+cargo check（T-03）;a2vue 金样（T-04）。
- **冒烟（本仓）**：三形态各一 echo 锚点脚本/截图证据:
  rust=窗口进程 headless scenario（at-gen scenario echo 同款思路）;
  vm=run_vm_ui + headless 断言;vue=vite 起服 + HTTP 回路断言
  （Playwright 可选,冒烟最低限=HTTP echo roundtrip）。
- **对拍**：at-app 三形态 echo/resize/interrupt 结果与 at-gen
  parity 门禁 oracle 输出对齐（复用 crates/autoterm-parity 场景数据,
  不改门禁本体）。
- **回归**：`cargo test --workspace`（本仓）不新增红（F-1 基线红
  除外）;at-gen `package-at.cmd` dist 冒烟 SCENARIO_OK 复跑。

## 7. 验收标准

| ID | 验收标准 | 验证方法 |
|---|---|---|
| AC-01 | `cd at-app && auto run -r rust` 构建绿、iced 窗口起、echo 回显/resize 生效/Ctrl+C 中断实测 | 冒烟证据（截图/日志）入库 docs/plans/evidence/013/ |
| AC-02 | `auto run -r vm` 同上三场景,VM 解释执行（AutoVM 动态轨）,无 a2r 产物参与渲染 | 同上 + 进程观察（无生成工程构建步骤） |
| AC-03 | `auto run -r vue` Vite 页面起,`<pre>` 视口渲染回显行,输入行经 HTTP back→引擎→回显可见 | HTTP echo roundtrip 断言 + 截图 |
| AC-04 | at-app 源单一导入面 `use auto.term`+裸调用;VM 裸调用单测绿 | auto-lang 单测入库 |
| AC-05 | 侧车机制:生成的 rust 工程与 back crate 均含 term.rs + libloading 依赖,编译绿,幂等 | auto-lang 单测断言 |
| AC-06 | 零回归:本仓 workspace 套件无新增红（F-1 除外）;at-gen dist 冒烟 SCENARIO_OK;parity 门禁本体零改动 | 套件复跑 + git diff 断言 |

## 8. 执行步骤

| ID | 任务 | 仓 | 依赖 | 产出/验证 | AC |
|---|---|---|---|---|---|
| T-01 | VM 裸名 alias 7 枚（native alias 表）+ 裸调用 roundtrip 单测 | auto-lang | — | 单测绿（真 DLL 在场即跑,缺席 skip 惯例） | AC-04 |
| T-02 | pac.at .rs 侧车机制（两生成点 + 幂等 + deps 注入）+ 单测 | auto-lang | — | 生成断言单测绿;schema 注记入 pac 文档 | AC-05 |
| T-03 | ui_gen/rust.rs terminal 臂（真身组件接线）+ 生成快照 | auto-lang | — | 快照入库 + 生成工程 cargo check 绿 | AC-01 |
| T-04 | ui_gen/vue.rs terminal 最小 `<pre>` 臂 + a2vue 金样再生 | auto-lang | — | 金样 diff 入库 | AC-03 |
| T-05 | at-app/ 标准工程落位（pac.at/front/back/sidecar term.rs） | auto-term | T-01..T-04 | headless 冒烟脚本在库 | AC-01..04 |
| T-06 | 三形态验收跑 + 对拍 + 证据入库（evidence/013/） | auto-term | T-05 | AC-01/02/03 证据 + 对拍记录 | AC-01..03 |
| T-07 | 文档回执:README/DEBTS(SD-02/SD-03)/005 设计文档(SD-01) | auto-term | T-06 | 文档 diff | AC-06 |

worktree：双仓惯例 `.wt/auto-013/{auto-term,auto-lang}`（011 先例）,
auto-lang 侧分支随其仓惯例;T-01..T-04 可并行,T-05 串后。

## 9. 复审记录

- 2026-09-11 draft（ZCode,/auto-plan:new）:初稿,v1。背景探查 B1-B7
  七项事实入账;三裁定 D1(use auto.term 统一)/D2(侧车供给)/D3(两枚
  生成器最小臂);执行期核查点 C1-C3 有界在案。
  stage: new / outcome: pass（授权范围内可开工;C1-C3 属执行期核查,
  fallback 路径已预置,不构成 blocked）/ next: work。

## 10. 待澄清事项

1. pac.at 侧车字段名与 schema（T-02 执行期定,倾向最小三元:
   modules/deps/挂载点;不阻塞开工）。
2. 工程目录名 `at-app/`（本计划内暂定,用户可改;改名属 cosmetic,
   不增 revision）。
3. vue 形态运行环境依赖 npm/vite 首次安装（015-notes 先例;如环境
   缺失,T-06 vue 臂以生成+构建绿为最低验收,运行时回路降级注记）。
