---
plan_id: PLAN-018
status: drafting
feature_name: Mux Core 五层模型——Workspace/Tab/LayoutTree/Pane/TerminalRuntime 解耦(外部分析 Phase 1)
author: [zcode-session]
created_at: 2026-09-14T08:40:00Z
updated_at: 2026-09-14T08:40:00Z
plan_revision: 1
current_step: 0
total_steps: 9
supersedes_spec_components: []
new_spec_components:
  - docs/specs/terminal-mux-model.md
touched_goals: []
---

# PLAN-018 · Mux Core 五层模型——Workspace / Tab / LayoutTree / Pane / TerminalRuntime 解耦

## 0. 变更摘要

用户 2026-09-14 提交了一份外部分析(ChatGPT 对 AutoTerm 的架构建议),
核心主张:不要把下一步做成"给 TerminalView 套 TabView/SplitView",而是
先把 **Workspace → Tab → LayoutTree → Pane → TerminalRuntime** 五层
对象模型立起来,UI 降格为 `render(state) / dispatch(action)`,三条铁律
——①Pane 不属于 UI;②Domain 不等于 PTY;③终端输出逐步升级为语义数据。
本计划 = 该方案的 **Phase 1(Mux Core)** 落地:

1. **MuxCore 状态机**落应用 back 层(新 `mux.at`):Workspace/Tab/
   LayoutTree/Pane 四类对象 + 全部结构操作(建/拆/关/焦点/缩放/比例)
   收敛为唯一 Action 函数族;GUI(V1)仍只渲染焦点 Pane——视觉零变化,
   模型先行(与外部分析 §十七 建议一致)。
2. **SpawnSpec 引擎面**:`autoterm_engine_spawn_ex`(program/argv/cwd/
   cols/rows)新增符号;`PtySession` 补 cwd;多会话并存隔离实证。
3. **泵面 per-key / per-handle 化**:消除三处"单终端广播"假设
   (sidecar 全局 CURSOR/VIEWPORT、widget `drain_all/take_any/feed_all`
   、VM shim 单例静态)——多 Pane 输入/几何/数据互不串线,这是今天
   就埋着、开 Tab 必炸的暗雷。
4. **控制面**:结构操作经既有 api.at 契约面暴露(`/api/mux/*`),
   `mux_snapshot()` 可结构断言——为 Phase 4 Control API 铺地基但**不是**
   Control API(无 socket/CLI/权限分级)。

外部分析的 Phase 2–8(OSC 语义层、Workspace 持久化、Control API/CLI、
Domain、Layout 策略、Mux Daemon、Agent API)全部非目标,路线图见 §4
映射表;分屏/Tab 条**渲染 UI** 是紧随其后的下一计划候选。

## 1. 目标

- G1 五层模型落位:MuxCore 状态机在 back 层(`src/back/mux.at`)持有
  Workspace/Tab/LayoutTree/Pane;引擎句柄生命周期 owner = 模型
  (pane 表),任何 UI 组件不拥有 PTY/进程。
- G2 SpawnSpec 引擎面:`autoterm_engine_spawn_ex` 新符号生效
  (program/argv/cwd/几何);`autoterm_engine_spawn` 旧符号原样保留;
  双会话并存互不干扰。
- G3 泵面隔离:输入泵/resize 请求/行数据投喂全部带 key(或 handle)
  定向;焦点 Pane 全量泵,隐藏 Pane drain-only(只收割引擎防积压)。
- G4 单 Pane GUI 等价:rust/vm 两形态现有交互(直键入、几何随动、
  选中/菜单/中断、016 颜色)零回归;014 最小化护栏不破。
- G5 Action 单入口:全部结构操作走 mux Action 函数族;api 契约面可
  驱动、`mux_snapshot` 可断言(结构操作的可观测性)。

### 非目标

- Tab 条 / 分屏**渲染** UI(View 布局消费 LayoutTree)——下一计划;
  本计划 GUI 仍焦点 Pane 全屏。
- OSC 7 / OSC 133 语义层(cwd 动态继承、CommandBlock)——因此 V1 的
  SplitPane 只继承 program/静态 cwd,**不承诺**动态 cwd 继承。
- Control API(socket/CLI/权限分级)、Mux Daemon(detach/attach)、
  Workspace 持久化/会话文件、Domain 抽象(WSL/SSH/Container)、
  Layout 策略(Tall/Grid/Stack)。
- env 自定义、profile、主题系统(SpawnSpec 预留字段,不实现)。
- vue 交互式视口(009 #4 留白维持);`at/` + `at-gen/` 冻结 oracle
  不动;at-engine-face 并存面不动(见 §10.3)。
- Unix 基座(DEBTS #8 维持)。

## 2. 架构方案

### 2.1 五层落位(外部分析 → 本仓映射)

```
            at-app front(app.at)                ← UI:render(state)/dispatch(action)
            auto-lang terminal widget            ← Pane 的视图(Registry 按 key 多实例已就绪)
                    │  Action(结构操作)           │  数据泵(per-key 定向)
            ┌───────▼────────────────────────────▼──────┐
            │ MuxCore = src/back/mux.at(新)             │
            │  Workspace / Tab / LayoutTree / Pane 表    │
            │  Action: create/split/close/focus/zoom/…   │
            └───────┬────────────────────────────────────┘
                    │ 每 Pane 一个引擎句柄
            ┌───────▼────────────────────────────────────┐
            │ Pane Runtime = autoterm_core.dll(既有)     │
            │  PtySession + alacritty Term + 快照面       │
            │  新增:spawn_ex(program/argv/cwd/几何)      │
            └────────────────────────────────────────────┘
```

- **TerminalRuntime 已存在**:autoterm-core 的 `PtySession`+`Term`,
  FFI opaque 句柄天然一柄一会话(ffi.rs 头注);多 Pane = 多句柄。
- **MuxCore 落 back 层而非引擎**:外部分析自己的分层图把 Mux Core 画在
  GUI 与 Pane Runtime 之间;本仓 back 层(db/api,三形态同源:rust
  merged / VM 字节码直调 / vue axum)正是这个位置。引擎保持
  "bytes → terminal state → 快照"的干净边界(004 路线裁定不动)。
- **LayoutTree 用平面节点表**而非递归 enum:Auto 侧 a2r/VM 对
  `Map<int, node>` 平面表表达确定性最高(016 同款保守取向),语义与
  wezterm `bintree::Tree`、kitty `splits.Pair` 等价(树操作只有
  挂/摘/改比例三种,平面表同样 O(小常数));比例用 int 千分比
  `ratio_permille`(0–1000,默认 500),规避浮点跨轨差异。
- **所有权铁律**(入 SD-01 spec):引擎句柄由 mux pane 表唯一持有与
  释放;widget Registry(key→TerminalCore)只是视图缓存,非生命周期
  owner;关闭 Pane = 模型删记录 + 引擎 free,widget 侧 key 自然失活。
- **V1 语义规则**:①关末 Pane = 拒绝(no-op,产品语义留 UI 计划);
  ②焦点切换时新焦点 Pane 几何随动(其 widget 发 resize 请求,引擎
  跟随)——隐藏期间保持旧几何;③隐藏 Pane 每拍 drain-only
  (`feed_ready`+`take_dirty_rows`,防 014 积压)不投喂 widget。

### 2.2 外部先例锚点(设计证据,详见 §4)

- wezterm:`Tab owns pane tree + zoom`(tab.rs:40),树节点带尺寸
  (bintree + SplitDirectionAndSize),Pane=运行时实体非 UI
  (pane.rs:170/localpane.rs:124),Workspace 只是 label(window.rs:14)
  ——我们取其树与所有权模型,**超愈**其 Workspace(一等记录,为
  Phase 3 铺路)。
- kitty:`Pair{horizontal,one,two,bias}` 单节点类二叉树(splits.py:34)
  与"swap layout 不动 children"的布局/子件分离;session 文件
  (定义)与运行态分离(session.py)——定义/运行分离留 Phase 3。

## 3. 技术栈

- Auto(.at):at-app(017 置换后为 `app/`)front/back、api 契约面。
- auto-lang:terminal 注册表 per-key 泵 API、VM `auto.term` shims、
  stdlib 声明面(双轨:rust a2r / VM 直调)。
- autoterm-core:portable-pty `CommandBuilder::cwd` + ffi.rs 新符号。
- 侧车 term.rs(libloading):per-handle 状态 + spawn_ex 包装。
- 零新外部依赖;无环铁律不动(auto-lang 零 Cargo 依赖本仓;at-gen
  仅有的 auto-lang 运行时依赖边不变)。

## 4. 需求分析与背景调查

### 4.1 授权记录

- 用户 2026-09-14 会话:提交外部分析全文并指定"根据他的分析做成详细
  实施计划"(auto-plan:new);明确授权 clone WezTerm/Kitty 至
  `D:/github/` 并扫描源码。**已完成**:wezterm `2afb836403838c3ed7e0
  9e5d570190adb054b607`、kitty `5cf63d9cc55cfe868fb92362445d2c7c18
  90f1a1`(浅克隆)。
- 允许仓库:auto-term(主)+ auto-lang(widget/VM shim/stdlib,016
  双仓锚定惯例)。预算/自动续跑:未指定。

### 4.2 外部证据(Phase 1 相关面)

| 主张 | wezterm 锚点 | kitty 锚点 |
|---|---|---|
| Mux 为无 GUI 运行时注册表 | mux/src/lib.rs:102 `Mux{tabs,panes,windows,domains,…}` | boss.py 持 os_window_map;registry 在 C child-monitor |
| Pane=运行时实体(Term+PTY+进程) | pane.rs:170 trait;localpane.rs:124 字段 | window.py:721,811-814(Window 持 child+screen) |
| Tab 持分屏树+zoom | tab.rs:40 `TabInner{pane: Tree,zoomed,…}`;bintree/src/lib.rs:16 | splits.py:34 `Pair{horizontal,one,two,bias}` |
| Domain 抽象(spawn/split/attach) | domain.rs:49-199 trait;LocalDomain :202 | (kitty 无 domain 层) |
| Workspace 仅 label | window.rs:14 `workspace: String` | — |
| 定义/运行分离 | codec SpawnV2(lib.rs 引) | session.py:43/60/81 WindowSpec/SessionTab/Session |

### 4.3 本地证据(单会话假设的四处实体)

1. `crates/autoterm-core/src/ffi.rs:54` `autoterm_engine_spawn(cols,rows,
   program)`——无 argv/cwd/env;`pty.rs:63` `PtySession::spawn(program,
   args,cols,rows)`(`CommandBuilder` :77,未接 cwd)。ffi.rs 全 16 符号
   逐柄操作,多柄并存结构上可行、未经实证。
2. `at-app/term.rs`:`HANDLES/SNAPSHOTS` 已按柄(:151),但
   `CURSOR`(:20)/`VIEWPORT`(:21)/`BACKLOG_STATE`(:40)进程级单例;
   `engine_pump_input`(:193)= `terminal_drain_all_inputs` 全键入写进
   **当前柄**;`engine_apply_resize`(:218)= `terminal_take_any_resize`
   任意 widget 的请求;`engine_rows`(:471)经 `terminal_feed_cells_all`
   (:321)广播投喂——多柄下三路全部串线。
3. auto-lang `ui/terminal/mod.rs`:Registry 按.key 多实例就绪
   (`TERMINALS` :247、`ROW_CACHES` widget.rs:118 按 key),但
   `terminal_feed_cells_all`(:362)/`drain_all_inputs`(:652)/
   `take_any_resize`(:689)均为广播/任意语义(头注自认"single-
   terminal apps are the current consumer shape");on_input/on_menu/
   on_select 是无载荷信号位,载荷须按 key 回读。
4. `at-app/src/back/db.at:14` 全局单 `handle int`;app.at 视图
   (:57-70)整窗单 terminal(key 静态);VM 轨
   `vm/ffi/term_engine.rs:33-34` `CURSOR/VIEWPORT` 单例静态。
   api.at(:1-5 契约注记)三形态同源声明,routes :10-:94。

### 4.4 八相路线映射(外部分析 §十六 → 本仓)

| 相位 | 内容 | 本仓落点 | 状态 |
|---|---|---|---|
| 1 | Mux Core 五层 | **本计划** | 本计划 |
| 2 | OSC 7/133 + Shell Integration | 引擎 face 语义通道 + widget 语义态 | 后续 |
| 3 | Workspace + Save/Restore | WorkspaceDefinition(文件)≠ Runtime;Restore≠Attach | 后续 |
| 4 | Control API + CLI | IPC socket + 权限分级(api.at 面为雏形) | 后续 |
| 5 | Domain 抽象 | SpawnSpec.domain;WSL/SSH/Container | 后续 |
| 6 | Layout Engine + 分屏渲染 UI | 策略生成/重排树 + View 布局消费 | 后续(渲染 UI 为下一计划候选) |
| 7 | Mux Daemon | 模型出进程,detach/attach | 后续 |
| 8 | Agent / Extension | 结构化 execute/事件流 | 后续 |

外部分析"Shell Integration 先于 SSH"(Phase 2 先于 5)的排序理由
成立,采纳为路线约束;本计划不动。

### 4.5 依赖:PLAN-017(app 入口统一)

017(2026-09-14 并行立项,`docs/plans/017-unify-app-entry.md`)裁定
`at-app/ → app/` 目录置换、terminal key `at-app→auto-term`。本计划
**路径基准 = `app/`**(017 的 §5.1 mv 结果);开工门 T-00 勘定仓库
状态:017 已 merge → 直用 `app/`;未 merge → 同径映射
`at-app/ ≡ app/` 在 work 期换算并记录。两计划同触面仅 app.at/db.at/
term.rs,不允许并行 work(先后串行,以先 merge 者为基)。

## 5. 详细设计

| # | 改动 | 文件:符号(仓) | 说明 |
|---|---|---|---|
| D1 | cwd + spawn_ex | autoterm-core `pty.rs`(PtySession::spawn_in(program,args,cwd,cols,rows);旧 spawn 薄委托)+ `ffi.rs` `autoterm_engine_spawn_ex(program,argv,argc,cwd,cols,rows)` | argv 为 `*const *const c_char`(argc 计数);cwd NULL/空 = 继承宿主;旧符号零改动(16 符号面 +1=17)。portable-pty `CommandBuilder::cwd` 既有能力接线 |
| D2 | 引擎测试 | autoterm-engine-ffi-tests `engine_ffi_integration.rs` | spawn_ex 三用例:cwd 生效(`cmd /c cd` 行文本含目录)、argv 生效、双柄隔离(一柄写 input 另柄 row_text 不变)+ 旧 spawn 回归 |
| D3 | 侧车 per-handle | at-app `term.rs`:`SessionState{raw,cursor,viewport,backlog}` 入 HANDLES;`engine_spawn_ex` 包装;`engine_rows_for(h,key)`/`engine_pump_input_for(h,key)`/`engine_apply_resize_for(h,key)`/`engine_cursor_row_for(h,…)` 定向变体 | rows_for 内部改调 auto-lang `terminal_feed_cells_for(key,…)`(D4);旧函数保留薄委托(ABI/冻结消费者不受扰);SNAPSHOTS 已按柄不动 |
| D4 | widget per-key 泵 | auto-lang `ui/terminal/mod.rs`:`terminal_feed_cells_for(key,row,cells)`(缺 key no-op)、`terminal_drain_inputs_for(core)`、`terminal_take_resize_for(core)` | 与既有 per-core `feed_cells`(:344)/`push_input`(:619)/`pending_resize` 同存储,只加定向出口;广播旧三件原样保留(其余消费者零扰) |
| D5 | VM 轨 per-handle | auto-lang `vm/ffi/term_engine.rs`:CURSOR/VIEWPORT 改 per-handle 表;`shim_term_spawn_ex`、`shim_term_rows_for/​pump_for/​apply_resize_for`;stdlib `auto/term.at`+`term.vm.at` 声明面加同名变体(旧声明保留) | registry 新 shim ID 沿 2943+ 段顺延;`feed_styled_sideband`(:231)带 key 定向;597 §9 两枚存量 shim 缺陷若撞上先修后接 |
| D6 | MuxCore 模型 | at-app `src/back/mux.at`(新):`rec Pane{id,handle,key,tab_id,program,cwd,cols,rows,exited}`、`rec Tab{id,active_pane,zoomed_pane,title}`、`rec WsNode{id,tab_id,axis,ratio_permille,first,second,pane}`、`rec Workspace{id,name,tabs,active_tab}`;全局表 + `mux_init/mux_create_pane/mux_split_pane/mux_close_pane/mux_focus_pane/mux_resize_pane/mux_zoom_pane/mux_new_tab/mux_close_tab/mux_activate_tab/mux_snapshot` | Action 函数族 = 唯一变异入口;close_pane 树再平衡(兄弟收编父位,wezterm remove_pane/kitty collapse 同语义);关末 Pane 拒绝;`mux_snapshot()` 返回结构 JSON 串;V1 恒单 Workspace(id=1),模型不设上限 |
| D7 | db/api 接线 | at-app `src/back/db.at`(pane 化:tick 驱动改焦点全量+隐藏 drain-only;term_* 保留为焦点 Pane 门面)+ `src/front/app.at`(view 的 terminal key/cols/rows 改绑焦点 Pane;`oninput` 泵焦点 Pane)+ `src/back/api.at`(`/api/mux/split|close-pane|focus|zoom|new-tab|close-tab|activate-tab` POST + `/api/mux/snapshot` GET + `/api/mux/cols|rows|focus-key` GET) | `get_lines()` 零参惯例(extract_init_api_func)不动——语义改为"焦点 Pane 快照";前键入 `.KeyIn` 路径不变;隐藏 Pane 泵在 tick 分支 |
| D8 | 文档 | `docs/specs/terminal-mux-model.md`(SD-01)+ DEBTS.md 新观察条 + at-app(→app)/README | 见规范增量 |

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/terminal-mux-model.md | 前:应用=单会话(单柄/单 widget/广播泵),无 Workspace/Tab/Pane 概念。后:五层对象模型 + 所有权铁律(引擎句柄 owner=mux pane 表,UI 组件永不拥有 PTY/进程)+ Action 单入口 + SpawnSpec(spawn_ex;domain/env 字段预留)+ per-key/per-handle 泵契约(广播三件退役为兼容薄委托)+ V1 语义(末 Pane 不关/焦点几何随动/隐藏 drain-only/比例千分比)+ face 记录(ffi 16→17;at/engine_face.at 12 符号并存面本计划不动,分叉在案) | 外部分析三铁律落档;Phase 2–8 的共同地基契约 | AC-01..09 |
| SD-02 | modify | DEBTS.md(新观察条) | 新增 #16:MuxCore V1 落位与已知边界(无 cwd 动态继承=Phase 2;关末 Pane 语义待 UI 计划;at-engine-face face 分叉) | 债务/边界持续可见 | AC-09 |

## 6. 测试设计

1. **引擎面**(autoterm-core):§5 D2 三新用例 + 既有 engine_ffi_
   integration 全量回归;命令 `cargo test -p autoterm-engine-ffi-tests
   --test engine_ffi_integration`,期望全绿(新用例先红后绿)。
2. **widget 泵面**(auto-lang):`terminal_feed_cells_for/drain_for/
   take_resize_for` 单测(定向性:多 key 互不取走;缺 key no-op);
   既有回归 `cargo test -p auto-lang --features ui-iced,iced-layout-
   tests --lib terminal`(016 pixel 金样在列)。
3. **VM 轨**:shim 冒烟(spawn_ex+定向泵路径;VFACE_OK 口径);
   stdlib 声明面编译绿。
4. **parity 回归**:`cargo test -p autoterm-parity`(六场景,face 旧
   符号零改动应全绿——016 色契约同验)。
5. **模型行为**(vue 形态,headless 权威验证):`auto run -r vue` 起
   back,curl 剧本——`/api/mux/snapshot`(初始单 Pane)→ split(axis=
   0)→ snapshot(树 3 节点 2 Pane,focus 不变)→ focus 新 Pane →
   close → snapshot(兄弟收编)→ close 末 Pane 拒绝断言(结构不变);
   snapshot 全程留 evidence。
6. **GUI 等价**(rust 形态):`auto run -r rust` 冒烟——直键入回显/
   几何随动/右键菜单/选中复制/中断(014/016 基线行为);014 配方抽查
   (ash+ping 静置→最小化,无重排爆炸);PrintWindow 取证链(004 定型)
   截图入 evidence。
7. **VM 形态 GUI**:`auto run -r vm` banner/键入/几何随动冒烟。
8. **全量**:`cargo test --workspace`(auto-term)0 error。

## 7. 验收标准

- **AC-01** spawn_ex 生效:cwd/argv 断言用例绿,旧 `autoterm_engine_
  spawn` 全量回归绿(ffi 16→17 符号,旧 16 零 diff)。验证:测试 1。
- **AC-02** 多会话隔离:双柄写读互不串(测试断言),引擎侧无全局态
  新增。验证:测试 1。
- **AC-03** sidecar 定向泵:cursor/viewport/backlog 按柄;rows/pump/
  resize 按 key;旧接口薄委托编译兼容。验证:测试 1 + 8 + rust 形态
  冒烟(测试 6)。
- **AC-04** widget per-key 泵单测绿;016 pixel 金样与 014 护栏回归绿。
  验证:测试 2。
- **AC-05** VM 轨 spawn_ex + 定向泵冒烟绿;rust 轨 parity 六场景绿。
  验证:测试 3/4。
- **AC-06** 模型行为:vue curl 剧本全断言过(split/close/focus/zoom/
  拒绝关闭;snapshot 结构留证)。验证:测试 5。
- **AC-07** GUI 单 Pane 等价:rust 形态交互零回归 + 014 最小化配方
  抽查无复燃 + 截图与 016 基线同构。验证:测试 6。
- **AC-08** 三形态构建绿:rust/vm/vue `auto run` 冒烟 + workspace
  cargo test 0 error。验证:测试 6/7/8。
- **AC-09** 文档在库:specs/terminal-mux-model.md(SD-01 全要素)、
  DEBTS #16、evidence/018/ 齐备(引擎测试日志/curl 剧本截图/GUI
  截图/双仓 SHA 锚定)。验证:文件检查。

## 8. 执行步骤

- **T-00 开工门**:勘定 PLAN-017 状态(§4.5);路径基准裁定与记录
  (`app/` 或 `at-app/` 映射)。前置:无。产出:§4.5 附记。关联全部。
- **T-01 D1+D2 引擎 SpawnSpec 面**(autoterm-core + ffi 测试)。前置
  T-00。关联 AC-01/02。
- **T-02 D4 widget per-key 泵**(auto-lang;与 T-03 并行,同仓异文件
  ——mod.rs vs term_engine.rs,注意 stdlib 声明面在 T-03)。前置
  T-00。关联 AC-04。
- **T-03 D5 VM 轨 per-handle + spawn_ex shim + stdlib 声明**(auto-
  lang)。前置 T-00(依赖 T-01 的符号语义定稿,可并行起草)。关联
  AC-05。
- **T-04 D3 侧车 per-handle + spawn_ex 包装**(at-app term.rs)。前置
  T-01。关联 AC-03。
- **T-05 D6+D7 MuxCore 模型 + db/api/front 接线**(mux.at 新建;db.at
  pane 化;api.at 路由;app.at 焦点绑定)。前置 T-02/T-04。关联
  AC-06/07。
- **T-06 集成取证**:测试 5 curl 剧本(vue)+ 测试 6/7(rust/vm GUI
  + 014 抽查)+ 测试 4 parity;证据入 evidence/018/。前置 T-05。关联
  AC-06/07/08。
- **T-07 D8 文档收口**:SD-01 spec 撰写、DEBTS #16、README、frontmatter
  步数对账。前置 T-06。关联 AC-09。
- **T-08 双仓锚定与复审准备**:auto-lang 侧提交 SHA、auto-term 侧提交
  SHA 回填 §9;工作收尾状态置 execution_done。前置 T-07。关联全部。

## 9. 复审记录

- 2026-09-14 stage:new · rev1 起草交接——外部分析(ChatGPT 八相方案)
  → 本计划切片 = Phase 1(Mux Core);设计依据:双仓读码(§4.3 四处
  单会话假设实体)+ wezterm/kitty 源码锚点(§4.2,浅克隆 SHA 在案);
  前置依赖 PLAN-017 已勘明(T-00 开工门);outcome: pass(授权范围
  内可开工,顺序约束见 §4.5);next: work(T-00 起)。

## 10. 待澄清事项

1. **PLAN-017 顺序**:两计划同触 app.at/db.at/term.rs——默认 017 先
   merge、本计划后开工(T-00 勘定);若 reversed,work 期路径换算,
   不构成语义变更(rev 不增)。
2. **Auto 表达边界**:`Map<int, rec>` + 平面节点表为设计基准;若 a2r/
   VM 对 rec 值类型有摩擦(010 账内 Map 映射已证存在),降级为平行
   `List` + 线性查找——执行期裁定,复审时回填 §9,rev 不增。
3. **at/engine_face.at 并存面**:12 符号 Auto 面默认**不**镜像
   spawn_ex(012 并存态,at-gen 退役时收敛);契约文档记录分叉。
   若 parity/对拍因 face 分叉报需要镜像,提请用户裁定。
4. **末 Pane 关闭语义**:V1 = 拒绝 no-op(§2.1);产品级语义(关应用?
   重生默认 Pane?)留 UI 计划期用户裁定。
5. **Phase 2–8 立项节奏**:路线图(§4.4)采纳外部分析排序(OSC 语义
   先于 SSH/Domain);启动顺序与优先级待用户裁定,本计划只交付
   Phase 1。
