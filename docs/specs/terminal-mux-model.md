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

## V1 语义(PLAN-019 修订)

1. **关末 Pane = 关其 Tab**(018 §10.4 裁定承接;`mux_close_pane`
   内部转 `mux_close_tab` 路径,焦点回落兄弟 Tab);**关末 Tab =
   拒绝 no-op**(app 常驻;产品级"关末 Tab 退出"留后续裁定)。
2. **焦点切换新焦点 Pane 几何随动**;**打字随动焦点**:逐可见 Pane
   泵各自队列,有键入者记为其 Tab 焦点(V1 近似:纯点击不打字不
   迁移焦点)。
3. **泵三档可见性**(018"焦点全量+隐藏 drain-only"升级):焦点 Pane
   = 全量泵 + 键入泵目标;可见非焦点 Pane = 全量投喂(各自 pane
   key,分屏后画面不冻结);不可见 Pane(zoom 掩盖/他 Tab)=
   drain-only(014 积压护栏维持)。
4. **恒单 Workspace(id=1)**,模型不设上限。
5. **可见多终端视图契约(PLAN-019 D3,T-B 形态)**:GUI 消费槽位
   投影——`mux_slot_pane_id(slot)` / `mux_slot_pane_key(slot)`:
   zoom → slot1=zoomed;无分屏 → slot1=唯一叶;树深 1 分屏 →
   slot1/2 = first/second 叶。**axis 语义:0=纵向堆叠(col)、
   1=横向并排(row)**。视图 = Tab 条(常显,for 摊平按钮;点击
   激活/右键关闭/"+ "新建/scheme 切换)+ 布局槽位 if 枚举;每
   terminal 实例按 pane key 动态绑定(`key: .field`)。**V1 分屏
   UI 上限 = 树深 1**(已分屏 Tab 再 split 拒绝 0,防不可见
   Pane;更深嵌套归 View::Split 组件后续计划)。
6. **Terminal shortcuts 契约(PLAN-019 D4)**:widget 键盘路径前置
   查捷径表(规范化键名 "ctrl.shift.e" 族;字符键 shift 恒前缀 +
   小写化),命中发消息不落 VT 队列不触发 on_input(双通道都断),
   未命中原样 `key_event_to_vt`(终端内程序不受扰;裸控制码零变)。
   .at 面 = `onkeydown.<键名>: .Msg` 事件;rust/vm 轨承诺。WT 风格
   V1 表:C-S-T 新Tab / C-S-W 关焦点Pane / C-S-E 横分 / C-S-O 纵分
   / C-S-Z zoom / C-S-Tab、C-S-← 前 Tab / C-S-→ 后 Tab / C-S-K
   scheme 循环(全组只命中带 Shift 组合,裸控制码路径零扰)。
7. **用户动作队列**:UI 线程处理器只 `mux_enqueue(code,arg)` 入队
   (纯 push),`get_lines` 每拍 `mux_drain_actions()` 在 tick 线程
   排水执行——引擎操作保持单线程序列化(所有权铁律执行面;UI 线程
   直调引擎与 tick 并发会产生 db 锁 × 引擎锁 ABBA 死锁,实测
   AppHangB1)。API 面:`POST /api/mux/enqueue`。
8. **D2 观测/消费面**:`GET /api/mux/tabs`(记录 "id|active|title")
   + 标量 getter 族(tab-count/id-at/is-active-at/title-at/
   pane-lines/pane-cols/pane-rows/pane-cursor-row/pane-cursor-col/
   visible-pane-count/split-axis/slot-pane-id/slot-pane-key/
   zoom-active/focus-id/enqueue)+ `GET /api/mux/layout`(JSON DTO)。
   V1 Tab 标题 = 序号 + shell 名(静态派生;动态 title 归 ③ OSC)。
9. **前端 int=i32 / db int=i64 桥接**:merged 垫片返回面降位、参数
   面升位(014 返回面裁定的镜像,auto-man merged_db_delegate)。

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
- 模型:`docs/plans/evidence/018/curl-mux-script.log`(018 剧本)+
  `docs/plans/evidence/019/t01-model-assertions.log`(019:泵三档/
  关末 Pane=关 Tab/深度上限/槽位投影,axum back curl 全断言)。
- 泵面:`cargo test -p auto-lang --features ui-iced,iced-layout-tests
  --lib terminal`(per-key 定向 + palette 解析 + 016 像素金样;
  019 增 shortcuts 命名/命中语义/发射金样/row×for 摊平)。
- VM shim:SPAWNEX_OK 冒烟(auto-lang test/term_mux_pumps)。
- 全量:`cargo test --workspace`(auto-term,019 起入门)。
