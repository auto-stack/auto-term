# Mux Core 五层模型契约(Workspace / Tab / LayoutTree / Pane / TerminalRuntime)

> 来源:PLAN-018(SD-01,2026-09-15;外部分析 ChatGPT 八相方案的
> Phase 1 落地,rev2 增补配色方案面见
> [engine-ffi-color-encoding.md](engine-ffi-color-encoding.md))。
> 落位:`app/src/back/db.at`(MuxCore 状态机 + Action 单入口;a2r
> 吸收面为单一 db.at,见"模型落位")。消费方:`app/src/back/api.at`
> (/api/mux/* 契约面)、`app/src/front/app.at`(V1 视口槽)、
> `app/term.rs` 与 auto-lang `vm/ffi/term_engine.rs`(引擎 glue)。

## 五层对象模型

```
at-app front(app.at)                ← UI:render(state)/dispatch(action)
auto-lang terminal widget            ← Pane 的视图(Registry 按 key 多实例)
        │  Action(结构操作)           │  数据泵(per-key 定向)
┌───────▼────────────────────────────▼──────┐
│ MuxCore(db.at)                            │
│  Workspace / Tab / LayoutTree / Pane 表    │
│  Action:mux_split/close_pane/focus/…       │
└───────┬────────────────────────────────────┘
        │ 每 Pane 一个引擎句柄
┌───────▼────────────────────────────────────┐
│ TerminalRuntime = autoterm_core.dll(既有)  │
│  PtySession + alacritty Term + 快照面       │
│  spawn_ex(program/argv/cwd/几何)           │
└────────────────────────────────────────────┘
```

## 铁律(违反即契约漂移)

1. **Pane 不属于 UI**:Pane 是模型记录(id/handle/key/tab/program/
   cwd/几何/退出态);UI 组件只是渲染缓存。
2. **Domain 不等于 PTY**:引擎句柄生命周期 **owner = MuxCore pane 表**,
   由 pane 表唯一持有与释放(`engine_free`);widget Registry
   (key→TerminalCore)只是视图缓存,非生命周期 owner;关闭 Pane =
   模型删记录 + 引擎 free,widget 侧 key 自然失活。任何 UI 组件不得
   拥有 PTY/进程。
3. **终端输出逐步语义化**:styled 面保持 016 kind_color 语义编码
   (Default/Indexed/RGB 穿传,见 engine-ffi-color-encoding.md);
   配色解析在渲染端经 scheme 表(同前 spec),不在引擎侧烤色。

## 模型形态(执行期裁定,复审在案)

- **平行 List 表 + 线性查找**(计划 §10.2 预留降级路径):Auto 侧
  rec/Map 无在库先例(010 账内 Map 映射摩擦在案)。表:pane_ids/
  pane_handles/pane_keys/pane_tab_ids/pane_programs/pane_cwds/
  pane_cols/pane_rows/pane_exited;tab_ids/tab_root_node/
  tab_active_pane/tab_zoomed_pane;node_*(见下)。
- **LayoutTree = 平面节点表**(语义等价 wezterm `bintree::Tree` /
  kitty `splits.Pair`):node_ids/tab_ids/axis(0=纵向 1=横向)/
  ratio_permille(千分比 0-1000,默认 500,规避浮点跨轨)/
  first/second(枝)/pane(叶;0=枝)。树操作只有挂/摘/改比例三种。
- **模型落位 = db.at**:a2r back 吸收面硬编码单一 db.at(auto-man
  `merged_db_impl`),独立 mux.at 不可达;Action 命名 `mux_*` 前缀
  保持模型身份可辨。

## Action 单入口

全部结构操作收敛 `mux_*` 函数族(唯一变异入口;可观测性:
`mux_snapshot()` 返回结构 JSON 串,api 契约面可驱动、可结构断言):

| Action | 语义 | 返回 |
|---|---|---|
| `mux_split(axis)` | 活动焦点叶位替换为分支(原叶 first、新 Pane second) | 新 Pane id(0=失败) |
| `mux_close_pane(id)` | 兄弟收编父位(wezterm remove_pane/kitty collapse 同语义)+ 引擎 free | 1=关;0=末 Pane 拒绝;-1=未找到 |
| `mux_focus(id)` | 焦点切换 | 1/-1 |
| `mux_zoom(id)` | 活动 Tab 缩放开关 | 1=缩放 0=取消 |
| `mux_resize_pane(id,r)` | 父分支 ratio_permille(0-1000 钳位) | 新比例/负=未找到 |
| `mux_new_tab()` | 新 Tab(单 Pane,自动激活) | tab id |
| `mux_close_tab(id)` | 全 Pane 引擎 free + 表行摘除 | 1/0=末 Tab 拒绝/-1 |
| `mux_activate_tab(id)` | 激活 Tab | 1/-1 |
| `mux_init()` | V1 单 Workspace(id=1)+ 初始 Tab/Pane(幂等) | tab id/0 |

## SpawnSpec 引擎面(ffi 16→17)

`autoterm_engine_spawn_ex(program, argv, argc, cwd, cols, rows)`:
argv 为 `*const *const c_char`(argc 计数,遇 NULL 提前止);cwd
NULL/空 = 继承宿主;旧 `autoterm_engine_spawn` 语义零改动。
`PtySession::spawn_in(program, args, cwd, cols, rows)` 为 rust 侧
真身(旧 spawn 薄委托;portable-pty `CommandBuilder::cwd` 接线)。
domain/env 字段预留(Phase 5),V1 不实现;OSC 7/133 缺席,V1 的
split 只继承 program/静态 cwd,**不承诺**动态 cwd 继承。

## per-key / per-handle 泵契约

多 Pane 输入/几何/数据互不串线;广播/任意旧件退役为兼容薄委托:

| 面 | 定向新件 | 兼容旧件(语义不变) |
|---|---|---|
| 样式旁路投喂 | `terminal_feed_cells_for(key,row,cells)`(缺 key no-op) | `terminal_feed_cells_all` |
| 键入泵(VM) | `engine_pump_for(handle,key)` / `terminal_drain_inputs_for(core)` | `terminal_drain_all_inputs` |
| 键入泵(侧车) | `engine_pump_for(handle,key)` | `engine_pump_input` |
| 几何泵 | `engine_apply_resize_for(handle,key)` / `terminal_take_resize_for(core)` | `engine_take_any_resize` 系 |
| 快照 | `engine_rows_for(handle,key)` | `engine_rows` |
| 光标/视口/积压 | per-handle 存储(SessionState / CURSORS/VIEWPORTS 表) | 同名函数(签名不变,语义按柄) |

## V1 语义

1. **关末 Pane / 末 Tab = 拒绝 no-op**(产品语义留 UI 计划)。
2. **焦点切换新焦点 Pane 几何随动**(其渲染/泵由焦点门面驱动);
   隐藏期间保持旧几何。
3. **隐藏 Pane 每拍 drain-only**(收割引擎防 014 积压,不投喂
   widget;`get_lines` tick 内联)。
4. **恒单 Workspace(id=1)**,模型不设上限。
5. **V1 视口槽契约**:GUI 单一可见 terminal 组件 key = "auto-term"
   (静态);焦点 Pane 的快照旁路/键入泵/几何请求经该 key 定向。
   pane.key = "pane-<id>" 为模型身份(渲染 UI 计划启用多 widget)。
   UI 降格 `render(state)/dispatch(action)` 在 V1 体现为:GUI 仍只
   渲染焦点 Pane 全屏——视觉零变化,模型先行。

## 控制面雏形(非 Control API)

`/api/mux/*`(POST:split/close-pane/focus/zoom/new-tab/close-tab/
activate-tab;GET:snapshot/cols/rows/focus-key)为 Phase 4 Control
API 铺地基;**无** socket/CLI/权限分级,不是 Control API。

## face 记录与并存面

- ffi:16→19(+spawn_ex;+set_palette per-handle / palette_color
  纯函数,后者见 engine-ffi-color-encoding.md)。
- VM shim:2943-2982 存量;新增 2983 spawn_ex / 2984 rows_for /
  2985 pump_for / 2986 apply_resize_for(catalog 双表登记+完整性锁);
  2987 menu_take(PLAN-015:右键菜单载荷 0-3/-1,注册表任意端,
  catalog 双表登记,不进 bigvm 返回面)。
- `at/engine_face.at` 12 符号并存面**不**镜像 spawn_ex(012 并存态,
  at-gen 退役时收敛);契约文档记录分叉。
- at-gen/src/shell.rs 的 View::Terminal 构造自 auto-lang 014 起即
  stale(at-gen 非 workspace member,冻结 oracle 不动)。

## 回归

- 引擎:`cargo test -p autoterm-engine-ffi-tests --test
  engine_ffi_integration`(cwd/argv/双柄隔离/palette 四组)。
- 模型:`docs/plans/evidence/018/curl-mux-script.log`(axum back
  curl 剧本全断言)。
- 泵面:`cargo test -p auto-lang --features ui-iced,iced-layout-tests
  --lib terminal`(per-key 定向 + palette 解析 + 016 像素金样)。
- VM shim:SPAWNEX_OK 冒烟(auto-lang test/term_mux_pumps)。
