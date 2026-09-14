# PLAN-017 app 入口统一——at-app 三形态工程置换 app 桌面壳

```yaml
plan_id: PLAN-017
status: drafting
feature_name: app 入口统一（at-app → app/，桌面壳退役）
author: [agent]
created_at: 2026-09-14T07:07:16Z
updated_at: 2026-09-14T07:07:16Z
plan_revision: 1
current_step: 0
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

- **T-01 置换提交**：§5.1 表全量操作（删旧壳三件、git mv src/term.rs/
  README、重写 pac.at、gitignore 改径、terminal key 改名）单提交落
  auto-term main。前置：无。产出：可编译的 app/ 单入口工程。
  验证：`git status` 净、`ls at-app` 无。关联 AC-04。
- **T-02 三形态冒烟（本机）**：cwd=app/ 依次 `auto run -r vm`（快验
  banner/键入即退）与 `auto run -r rust`（build+run 即退）。前置 T-01。
  验证：命令成功退出码。关联 AC-05。
- **T-03 引用面同步**：auto-term README 根布局节、auto-os README/manifest
  核对（预期零或注释级）；`git grep at-app` 清零。前置 T-01。关联
  AC-04/06。
- **T-04 桌面实机门**：§6.1 全断言取证。前置 T-01（桌面重启用
  desktop.sh iced）。**失败即停**——携证据提请用户在分支 B/C 间裁决
  （§2 兜底）。关联 AC-01/02/03。
- **T-05 全量回归**：§6.2/6.4（rust/vm/vue 三轨 + cargo test）。前置
  T-04 过门。关联 AC-05。
- **T-06 遗留清偿 + 台账**：at-app/stdlib 运行态遗留处置（gitignore 或
  删）；`.autoos/specs.json` 入账；SD-01 spec 文档撰写。前置 T-05。关联
  AC-04、SD-01。
- **T-07 证据归档**：evidence/017/ 四类证据齐（桌面四面/rust 独立/三轨
  回归/历史可溯）；计划 frontmatter 更新。前置 T-06。关联全部 AC。

## 9. 复审记录

- 2026-09-14 stage:new rev1 起草交接——用户裁定置换方向（§4 授权记录）；
  主案 A 依据：装载链读码（entry_for_dir/build_dynamic_component/
  resolve_back_api 本地优先）+ 独立 vm merged 先例 + shim 面全覆盖；
  残余风险单点 = T-04 实机门（本地委托体 back 桌面动态编译无先例），
  兜底分支 B/C 预置待裁决。outcome: pass（授权范围内可开工）；
  next: work（T-01 起）。

## 10. 待澄清事项

1. **桌面 merged back 实机行为**（T-04 门）：若出桩失败，分支 B（cdylib
   工程新建，工作量约 os-config-back 量级）vs 分支 C（front store 双实
   现破坏单源）需用户裁决——两案均超出本轮授权范围。
2. `at-app/stdlib/` 遗留件处置（gitignore vs 删除）：T-06 就地按内容判
   定，若发现被运行期依赖则上报再动。
3. vue 轨 pac 端口：并合 pac 保留 17400/17401 占位——vue split 形态若
   未来需要真端口，另行计划（本计划不动 vue 行为）。
