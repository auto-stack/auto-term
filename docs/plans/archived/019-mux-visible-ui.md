---
plan_id: PLAN-019
status: archived
feature_name: 可见多终端 UI——Tab 条 + 分屏渲染 + 快捷键(006 蓝图 ①)
author: [zcode-session]
created_at: 2026-09-15T00:00:00Z
updated_at: 2026-09-17T00:00:00Z
plan_revision: 1
current_step: 9
total_steps: 9
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
completion_kind: delivered
---

# PLAN-019 · 可见多终端 UI——Tab 条 + 分屏渲染 + 快捷键(006 蓝图 ①)

## 0. 变更摘要

006 蓝图(docs/designs/006-mux-terminal-roadmap.md)① 号计划。
PLAN-018 已把多终端做成"模型与 API 层的真实"(五层模型/Action 单入口/
per-key 泵/`/api/mux/*`),但 GUI 仍渲染单焦点 Pane 全屏——多终端在
界面上不可见。本计划把它做成**看得见、摸得着**:

1. **Tab 条**:app 视图顶部 Tab 条(Row/Button 既有组件,零新
   widget),新建/切换/关闭 Tab,与 `mux_new_tab/activate_tab/
   close_tab` 接线;单 Tab 时亦常显(Windows Terminal 同款形态)。
2. **分屏渲染**:View 消费 LayoutTree——同屏多 Terminal 组件实例
   (per-key 注册表 018 已备),嵌套 Row/Column 按 `axis/ratio_permille`
   布局;焦点 Pane 键入/几何随动,可见非焦点 Pane 持续上屏。
   **布局构建三案**(§5 D3)按 T-00 开工门勘定取舍(动态视图构建
   是本计划最大技术不确定点)。
3. **泵策略可见性化**(018 口径升级):焦点全量 → **可见全量**
   (焦点+同屏可见非焦点 Pane 各自投喂),仅不可见(被 zoom 掩盖/
   他 Tab)维持 drain-only——分屏后非焦点 Pane 不再冻结画面。
4. **末 Pane 关闭语义落定**(018 §10.4 遗留,归本计划裁定):关末
   Pane = 关其 Tab;关末 Tab = 拒绝 no-op(app 常驻)。
5. **快捷键**(auto-lang widget 侧新面):Terminal 组件键盘路径前置
   应用级捷径表(命中发消息,未命中落 VT 队列),Windows Terminal
   风格组合键:新 Tab/关闭 Pane/分屏横竖/zoom/切 Tab/scheme 循环。
6. **scheme 切换按钮**:Tab 条右侧 Dark/Light 切换(纯 app.at,
   载荷面零改动;widget 菜单扩展列为顺手项不做基准)。

非目标:快捷键 prefix 模式(tmux C-b 状态机)、Tab 拖拽重排、
Pane 边界鼠标拖拽调比例(`mux_resize_pane` API 已备,UI 拖拽后续)、
layout 策略(Tall/Grid/Stack,Phase 6 另半边)、Workspace 持久化
(③/④ 计划)、OSC 语义 cwd 继承(③)、vue 臂分屏视觉(vue 终端
本为只读最小视口臂,V1 = Tab 条 + 焦点 Pane 数据、分屏视觉不承诺)。

## 1. 目标

- G1 Tab 条可用:新建/切换/关闭 Tab 全链视觉可见可点,rust/vm
  双轨实机取证;单 Pane 形态交互零回归(内容区)。
- G2 分屏可见:水平/垂直分屏多 Pane 同时可见、各自独立 shell 会话、
  键入路由焦点 Pane、内容互不串线(018 per-key 契约的视觉面)。
- G3 泵可见性:可见非焦点 Pane 内容持续更新;zoom/切 Tab 掩盖的
  Pane drain-only 且积压不涨(DEBTS #12 面口径维持)。
- G4 快捷键全组生效:WT 风格组合键 rust/vm 双轨实测;未命中键
  原样落 VT(终端内程序不受扰)。
- G5 关闭语义:关末 Pane → Tab 消失;关末 Tab → 拒绝,app 存活;
  spec V1 语义节同步修订。
- G6 scheme 按钮:双方案实机切换生效(classic-dark/light),016
  像素金样与 018 双臂截图基线零回归。

### 非目标

见 §0;另:autoterm-core 引擎零改动(018 已备齐 per-handle/scheme
面);at/engine_face.at 并存面不动(012/018 口径);Unix 基座
(DEBTS #8 维持)。

## 2. 架构方案

```
        app.at(前)                                 ← Tab 条 + 布局消费视图
        │ .TabActivate/.NewTab/.Split/.Shortcut 消息      │ 多 terminal 实例(per-key)
        ▼                                          ▼
┌───────db.at(MuxCore,018)──────────────────────────────┐
│ Workspace/Tab/LayoutTree/Pane 表 + Action 族           │
│ 泵策略:可见全量/不可见 drain-only(本计划升级)        │
└───────┬───────────────────────────────────────────────┘
        │ 每 Pane 一引擎句柄(spawn_ex,018)
        ▼
   autoterm_core.dll(PtySession+Term+palette,零改动)
```

- **Tab 条 = 既有组件**:Row + Button(onclick 直接发消息)——
  view.rs 既有变体,零 auto-lang 新组件;数据从新 api getter
  (`mux_tabs()`)拉取,timer 50ms 轮询既有。
- **分屏 = 多 Terminal 实例 + 布局消费**:TERMINALS 注册表按 key
  多实例 018 已就绪;布局按 LayoutTree 平面节点表(`axis/
  ratio_permille/first/second/pane`)构建嵌套 Row/Column。构建方式
  三案:T-A view 内动态构建(a2r/VM 对 if/for 内嵌 row/col 与
  递归函数支持,开工门实证)/ T-B 有限布局枚举(V1 仅树深 1:
  一横一纵两分,view if 分支静态枚举)/ T-C auto-lang 新增
  View::Split{axis,ratio,first,second} 组件(widget 侧真分屏)。
  **基准 = T-A;T-A 不可行落 T-B(V1),T-C 留后续计划**。
- **泵可见性**:db tick 现为"焦点全量+隐藏 drain-only"(018 D7
  执行口径);升级为三档——焦点(全量+键入泵)/可见非焦点(全量
  投喂,不接管键入)/不可见(drain-only)。判定源 = 当前 Tab +
  zoom 掩盖关系(模型内可算,零引擎改动)。
- **快捷键 = Terminal 键盘路径前置拦截**:widget 键盘入口先查
  `shortcuts` 表(规范化键名→消息,Textarea keydown/Binding::Custom
  先例,renderer.rs:292 既有机制),命中发消息不落 VT,未命中
  原样进 TerminalCore 队列;View::Terminal 增可选 prop(a2r 7 处
  全字段枚举点 + ui_gen + VM convert/shim + stdlib 声明,018 D10
  scheme prop 同款扩建路径)。
- **所有权铁律不动**(SD-01):句柄 owner 仍 mux pane 表;Tab 条/
  布局视图只是 Action 的发起者与状态的消费者。

### 外部先例锚点(设计证据,SHA 沿 018 §4.1 浅克隆)

- wezterm:Tab 条渲染(resizable tab bar)与 pane tree 布局消费
  (bintree 尺寸下行)分离;Tab Bar 是渲染层对 Tab 表的投影。
- kitty:tab_bar.py 独立渲染条;layouts 按 splits 树分配矩形——
  布局消费与模型分离同构。

## 3. 技术栈

- Auto(.at):`app/` front/back(app.at 视图扩建、db.at 泵与关闭
  语义、api.at 新路由),三形态同源。
- auto-lang:View::Terminal `shortcuts` prop + widget 键盘拦截 +
  VM shim/stdlib 声明(依赖仓,master 直落,双仓锚定惯例)。
- 零新外部依赖;无环铁律不动;autoterm-core 零改动。

## 4. 需求分析与背景调查

### 4.1 授权记录

- 用户 2026-09-15 裁定(006 蓝图 + 本会话):"把蓝图更新到设计
  文件……按照蓝图开始立项下一个计划"——① 号计划立项授权在案;
  范围 = §0 所列六项,非目标之外的扩张须另行请求。
- 允许仓库:auto-term(主)+ auto-lang(widget/shim/stdlib,
  016/018 双仓锚定惯例)。预算/自动续跑:未指定。
- 依赖前置:PLAN-018 已 delivered 归档(main@d39c1f1 祖先链);
  无其他计划在途(017 收据"018 前置解除"后无新并行计划)。

### 4.2 外部证据

| 主张 | wezterm 锚点 | kitty 锚点 |
|---|---|---|
| Tab 条=模型投影,可独立于 pane 树渲染 | tab_bar 标签行渲染与 Tab 状态分离 | tab_bar.py 独立模块 |
| 布局消费=树→矩形下行 | bintree 节点带 SplitDirectionAndSize | layouts/ 按 Pair 树分配 |
| 键位表驱动结构操作 | keyassignment(建 tab/分屏/zoom 皆 action) | key_table/action 面 |

### 4.3 本地证据(读码,2026-09-15)

1. `app/src/front/app.at:88-95`——view 现 = 单 `col{terminal}`,
   key 静态 "auto-term";msg 族 {Init,Tick,KeyIn,Menu};timer 50ms;
   `.Menu` 载荷 3→term_interrupt 先例(载荷注册表任意端)。
2. `app/src/back/db.at:366-614`——MuxCore Action 族在库
   (mux_init/split/close_pane/focus/zoom/resize_pane/new_tab/
   close_tab/activate_tab/snapshot);`:410` mux_close_pane 现语义
   = 关末 Pane 拒绝(018 V1);tick 泵策略 = 焦点全量+隐藏
   drain-only(018 D7 执行口径)。
3. `app/src/back/api.at:110-177`——`/api/mux/*` 十一路由在库;
   新增路由沿同款 `#[api]` 注记。
4. auto-lang `ui/view.rs:428-700`——View 变体:Row/Column/
   Button(onclick)/Container/Select 等齐备;`:562` Terminal 变体
   12 字段(scheme 为 018 D10 末次扩建);Textarea `keydown`
   HashMap 先例(`:520`)。
5. auto-lang `ui/iced/renderer.rs:292-299,555`——Binding::Custom
   派发机制在库(Textarea 消费);terminal 臂 `:4144,6721`(iced/
   vm 双 convert 点,018 scheme 7 处全字段枚举点先例)。
6. auto-lang `ui/terminal/mod.rs:636-654`——menu payload 注册表
   per-core/any 双出口(015 shim 2987 spec-sync 在案)。
7. **已知摩擦先例**:018 T-05 执行期裁定"rec/Map 在 a2r/VM 有
   摩擦,模型降级平行 List"——动态结构(递归布局构建)在同族
   摩擦面上,故 T-00 开工门必勘。

## 5. 详细设计

| # | 改动 | 文件:符号(仓) | 说明 |
|---|---|---|---|
| D1 | 泵可见性化 + 关闭语义 | app `src/back/db.at`:tick 三档泵(焦点全量/可见非焦点全量投喂/不可见 drain-only;可见性 = 当前 Tab 内且非 zoom 掩盖)+ `mux_close_pane` 末 Pane 臂改关 Tab(内部转 `mux_close_tab` 路径) | 末 Tab 关闭拒绝维持;`mux_exited()` 语义不变(焦点 Pane 门面);014 积压护栏(隐藏 drain-only)不破 |
| D2 | api 面扩建 | app `src/back/api.at`:`GET /api/mux/tabs`(id/active/title/序)、`GET /api/mux/pane-lines?pane_id=`(每 Pane 行快照)、`GET /api/mux/layout`(树/可见性投影 DTO)、`POST /api/mux/scheme`(per-pane 或全局 set) | db 侧配 `mux_tabs():: List<rec-降级平行 List>`/`mux_pane_lines(pane_id)`/`mux_layout()`;snapshot JSON 保留(既有消费面) |
| D3 | Tab 条 + 布局消费视图 | app `src/front/app.at`:msg 族扩 {TabActivate(n),TabClose(n),NewTab,Split(axis),Zoom,Shortcut(n),Scheme(n)};view = `col{ tabbar_row, layout_root }`;tabbar = Row(Button×tabs + "+" Button + scheme Button);layout_root 按 D3 案构建多 `terminal{key: pane.key}` 实例 | 布局构建三案:T-A 动态(view 内函数按树递归/循环构建 row/col;T-00 实证 a2r+VM 可行则取)/ T-B 枚举(V1 树深 1 两分固定四形态 if 分支;T-A 不可行时 V1 落此)/ T-C widget Split 组件(留后续计划,不在本计划) |
| D4 | Terminal shortcuts prop | auto-lang `ui/view.rs`(Terminal 增 `shortcuts: Map/平行对 List<(str,M)>` 可选字段)+ `ui/terminal`(键盘入口先查捷径表命中发消息未命中落 VT)+ `ui/iced/renderer.rs` iced 臂 + `vm/ffi` convert/shim + stdlib `auto/*.at` 声明 | 键名规范化沿 Textarea("ctrl.shift.e" 族);Binding::Custom 机制既有;7 处全字段枚举点扩建(018 scheme 先例);**仅 rust/vm 轨承诺**(vue 臂声明透传不实现,与 vue terminal 最小臂口径一致) |
| D5 | 快捷键表 + scheme 接线 | app `src/front/app.at`:terminal `shortcuts:` 绑定(见下表);`.Shortcut(n)` 分派 api 调用;scheme Button → `api.mux_scheme(n)` | 快捷键表(WT 风格,V1):Ctrl+Shift+T 新 Tab / Ctrl+Shift+W 关焦点 Pane / Ctrl+Shift+E 水平分 / Ctrl+Shift+O 垂直分 / Ctrl+Shift+Z zoom / Ctrl+Shift+Tab 与 Ctrl+Shift+←→ 切 Tab / Ctrl+Shift+K scheme 循环;冲突键逐个勘定(终端内程序占用面,如 Ctrl+Shift+E 非 VT 惯用,在案核) |
| D6 | vue 臂适配 | app `src/front`(vue 发射面)/ back 契约零改 | vue:Tab 条渲染 + 焦点 Pane 数据(get_lines 焦点门面不变);分屏视觉/快捷键不承诺(§0 非目标) |
| D7 | 文档 | `docs/specs/terminal-mux-model.md`(V1 语义节修订:关末 Pane=关 Tab、泵三档可见性契约、快捷键 prop 契约)+ `app/README.md` + DEBTS 观察条(若有) | SD-01 modify;016 像素金样/018 双臂基线注记维持 |

### 快捷键分派契约(D4/D5 合并面)

```
按键 → widget 捷径表命中? ─是→ 发消息 M → app .Shortcut(n) → api.mux_*
                    └否→ TerminalCore 队列(原路径,VT 语义零变)
```

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/terminal-mux-model.md | 前:V1 语义 = GUI 单焦点 Pane 全屏、tick 焦点全量+隐藏 drain-only、关末 Pane 拒绝(018)。后:GUI 多 Terminal 实例按 LayoutTree 布局(Tab 条常显);泵三档(焦点/可见非焦点/不可见);关末 Pane=关 Tab、关末 Tab 拒绝;Terminal `shortcuts` prop 契约(命中拦截/未命中透传) | 006 蓝图 ①;018 §10.4 关闭语义裁定的承接 | AC-01..09 |
| SD-02 | modify | DEBTS.md | 新增观察条:布局构建案取舍记录(T-A/B/C 与判据);vue 臂分屏视觉边界 | 决策可追溯 | AC-09 |

## 6. 测试设计

1. **back 模型**(vue 形态 headless,018 curl 剧本同款):`auto run
   -r vue` 起 back,curl 剧本——`/api/mux/split`(横/竖)→
   `/api/mux/tabs`(结构断言)→ `/api/mux/pane-lines`(两 Pane
   各自内容隔离:一 Pane 写 `echo A` 另 Pane 行文本不变)→
   close 末 Pane(Tab 消失断言)→ 末 Tab 拒绝(结构不变)→
   `/api/mux/scheme` 三态;全程留 evidence/019/。
2. **widget 捷径单测**(auto-lang):捷径命中发消息/未命中落 VT
   队列/空表全透传;`cargo test -p auto-lang --features
   ui-iced,iced-layout-tests --lib terminal`(016 像素金样三件
   回归在列)。
3. **rust 形态 GUI**:`auto run -r rust` 实机——Tab 条可见(单
   Pane 形态)、新建/切换/关闭 Tab 点击、分屏快捷键两 Pane 同屏
   独立会话、焦点点击切换、zoom 掩盖、scheme 按钮;PrintWindow
   取证链截图入 evidence/019/。
4. **vm 形态 GUI**:`auto run -r vm` 同表抽查(快捷键 + Tab 条 +
   两分屏帧)。
5. **014 护栏抽查**:ash+ping 静置→最小化/恢复,无重排爆炸;
   积压采样(隐藏 drain-only 面不涨)。
6. **parity**:cargo test -p autoterm-parity(引擎零改动应全绿);
   **全量**:cargo test --workspace 0 error。
7. **vue 臂**:Tab 条 + 焦点 Pane 数据冒烟(浏览器截图)。

## 7. 验收标准

- **AC-01** Tab 条:rust 实机单 Pane 形态 Tab 条可见,新建/切换/
  关闭 Tab 全链可用(截图+trace);关末 Tab 拒绝且 app 存活。
  验证:测试 3/1。
- **AC-02** 分屏:横/竖分屏多 Pane 同屏可见、各自独立 shell
  (pane-lines 隔离断言+实机截图);键入路由焦点 Pane。
  验证:测试 1/3。
- **AC-03** 泵可见性:可见非焦点 Pane 内容持续上屏(实机:非焦点
  Pane 跑 `ping -t` 持续输出帧差);zoom/切 Tab 掩盖 Pane
  drain-only 且 backlog 采样不涨。验证:测试 3/5。
- **AC-04** 快捷键:§5 D5 全组键 rust/vm 双轨实测生效;未命中键
  (如普通字符/Ctrl+C)行为零变。验证:测试 2/3/4。
- **AC-05** 关闭语义:关末 Pane → 其 Tab 消失(focus 回落兄弟
  Tab);关末 Tab → 拒绝 no-op(spec 修订在库)。验证:测试 1。
- **AC-06** scheme:按钮切换 Dark/Light 实机生效,016 像素金样
  三件零回归。验证:测试 2/3。
- **AC-07** 单 Pane 内容区交互零回归:直键入/选中/菜单/中断/IME/
  几何随动(018 基线行为表);Tab 条为唯一新增可见面。
  验证:测试 3/5。
- **AC-08** 三形态构建绿:rust/vm/vue `auto run` 冒烟 + `cargo
  test --workspace` 0 error + parity 全绿。验证:测试 6/7。
- **AC-09** 文档在库:terminal-mux-model.md SD-01 修订、DEBTS
  SD-02 观察条、evidence/019/ 齐备(测试日志/curl 剧本/双轨截图/
  双仓 SHA 锚定)。验证:文件检查。

## 8. 执行步骤

- **T-00 [x] 开工门勘定**:①布局构建三案判据——a2r/VM 对 view 内
  动态构建(if/for 内嵌 row/col、函数递归、平行 List 驱动循环)
  的支持实证(最小语料编译+运行,记 §9);案决 T-A/T-B。②auto-lang
  master 锚点勘定(018 后新提交面;shortcut 扩建点 7 处枚举位)。
  ③快捷键冲突表勘定(候选键 × 常见终端内程序占用)。前置:无。
  产出:§9 勘定附记。关联全部。
  [✅ 已完成] 案决 **T-B(V1 树深 1)**——探针工程
  `spikes/019-layout-probe`(auto build 全绿 1m48s,生成码
  rust-workspace/019-layout-probe/src/main.rs:85 勘读):view if/else
  枚举 row/col 臂 + 臂内多 terminal 实例 a2r 生成逐字正确;row 直属
  for 摊平缺口在案(a2r 包 col 致纵向堆叠;VM 臂 Plan 047 既有摊平)
  → Tab 条使能 = a2r row×for 摊平小修(Plan 407 grid 先例推广,随
  T-03 落 auto-lang);terminal 动态键 a2r 仅认字面量(T-A 迭代形态
  不可行,AC-02 依 §10.3 收窄两 Pane、rev 不增)。详证 §9 附记。
- **T-01 [D1+D2] back 泵可见性化 + 关闭语义 + api 面**(db.at/
  api.at)。前置 T-00。关联 AC-02/03/05。
- **T-02 [D3 案按 T-00] Tab 条 + 布局消费视图**(app.at;单 Pane
  回归优先,分屏臂随后)。前置 T-01。关联 AC-01/02/03/07。
- **T-03 [D4] Terminal shortcuts prop**(auto-lang view/terminal/
  renderer/vm/stdlib 五面 + 单测;018 D10 扩建路径)。前置 T-00。
  关联 AC-04。
- **T-04 [D5] 快捷键表 + scheme 接线**(app.at;含冲突键实测
  勘定回填)。前置 T-02/T-03。关联 AC-04/06。
- **T-05 [D6] vue 臂适配**(Tab 条 + 焦点 Pane 冒烟)。前置 T-02。
  关联 AC-08。
- **T-06 集成取证**:测试 1 curl 剧本(vue)+ 测试 3/4/5/7(rust/
  vm 实机 + 014 护栏 + parity/全量);证据入 evidence/019/。
  前置 T-04/T-05。关联 AC-01..08。
- **T-07 [D7] 文档收口**:SD-01 spec 修订、SD-02 DEBTS、app/README、
  frontmatter 步数对账。前置 T-06。关联 AC-09。
- **T-08 双仓锚定与复审准备**:auto-lang 提交 SHA/auto-term 提交
  SHA 回填 §9;status execution_done。前置 T-07。关联全部。

## 9. 复审记录

- 2026-09-15 stage:new · PLAN-019 · rev1 · outcome:**pass** ·
  授权:006 蓝图 ① 立项(用户 2026-09-15,§4.1)· 设计依据:
  双仓读码(§4.3 七实体)+ wezterm/kitty 锚点(SHA 沿 018 §4.1
  浅克隆)+ 018 归档计划与 terminal-mux-model.md 契约 · 核心风险
  预判与处置:布局动态构建 = T-00 开工门三案判据(T-A 不可行落
  T-B,视觉受限但不阻交付);快捷键拦截 = 既有 Binding::Custom
  机制 + 018 D10 prop 扩建路径,均为已验证模式 · next:work
  (T-00 起)。

- 2026-09-15 stage:work · PLAN-019 · rev1 · **T-00 勘定附记**
  (outcome:pass,进入 executing):
  - **工作树勘定**:主检出直落(014/016/017/018 四例"无 worktree
    主检出交付"惯例;auto-term 无 AGENTS.md 反向约束);auto-lang
    master 直落(双仓锚定惯例)。auto-term 基点 c7b2df4(019 立项
    提交);工作区唯一遗留 tmp_ash_colors.png(016 收据在案非本计划)。
  - **①布局构建三案判据**:探针工程 `spikes/019-layout-probe/`
    (pac.at render rust + 纯本地模型 app.at,零 back 依赖;
    `auto build` 全绿 1m48s;生成码
    `spikes/019-layout-probe/rust-workspace/019-layout-probe/src/
    main.rs:85` 勘读)。实证四条:
    (a) **T-B 可行**:view `if`/嵌套 `if` 静态枚举 row/col 臂 +
    臂内多 terminal 实例(literal key/`.field` 绑定/oninput)→ a2r
    生成码逐字正确(`if self.split == 1 { if self.axis == 1 {
    View::row().child(Terminal pane-1).child(Terminal pane-2) } ...
    }`)——深度 1 两分(横/竖)视图消费在案;
    (b) **row×for 摊平缺口**:a2r 通用臂把 row 直属 for 包成单 col
    子(`.child(View::col().children(map))`,按钮纵向堆叠,
    ui_gen/rust.rs:3247 "always use .child()" 注记位);VM 臂无此
    缺口(aura_view_builder.rs convert_row:6049 for_loop_iterations
    横向摊平,Plan 047 在库);grid 有 a2r 摊平先例(Plan 407
    generate_for_loop_cells)。**裁定:随本计划落 auto-lang 小修**
    ——row 直属 ForLoop 改 `.children()` 批量加(Plan 407 先例推广
    至 row;col 不动防扰存量),Tab 条(G1/AC-01)两轨同源 .at 所
    必需,执行点 = T-03;
    (c) **terminal 动态键缺口**:a2r terminal 臂 key 仅认
    `Expr::Str` 字面量(ui_gen/rust.rs:2474),动态表达式回落
    "main";VM 臂 extract_string_with(bindings) 动态键可用——
    迭代驱动多实例(T-A 形态)rust 轨不可行;
    (d) component fn 递归 + 平行 List 全表传参未验证,且视图内拉
    模型数据违反 tick-pull 数据流(013 架构:tick 拉取入 model,
    view 纯消费)。
    **案决:T-B(V1 树深 1)**,§10.3 预授权降级生效——AC-02 口径
    收窄"两 Pane 分屏",rev 不增;更深嵌套归 T-C(View::Split
    组件,后续计划)。探测语料保留于 spikes/(不入 CASA 面)。
  - **②auto-lang 锚点**:master de86e1d8e(PLAN-635 文档提交链顶;
    并行工作区遗留:iced/renderer.rs 12 行 eprintln 调试 + 计划文档
    632/634 修改——非本计划,保留不提交,renderer.rs 提交时选择性
    staging(git apply --cached 过滤 hunk))。D4 shortcuts 扩建点
    枚举(018 T-10 scheme 先例 + Textarea keydown 先例双轨对齐):
    1. view.rs View::Terminal 变体 + 全字段映射(:562/:2091)增
       `shortcuts: Vec<(String, M)>`;
    2. aura_view_builder.rs convert_terminal(:9429)——`onkeydown.*`
       事件收集 → shortcuts(Textarea :9386-9412 同款收集器);
    3. renderer.rs iced 臂 1(:4144 视图构建)传穿;
    4. renderer.rs iced 臂 2(:6727 VM 消息转换)from_dynamic;
    5. ui_gen/rust.rs terminal 发射臂(:2471)发射 shortcuts vec;
    6. ui/terminal/iced/widget.rs 键入捕获块(:372)前置拦截 +
       Terminal struct(:137)增 shortcuts 字段——命中发消息不落
       VT 队列不触发 on_input;未命中原样 key_event_to_vt;
    7. 键名规范化新写 widget 本地 `terminal_key_binding_name`
       (命名沿 Textarea "ctrl.r" 族;差异:字符键 shift 恒前缀 +
       小写化——修 Ctrl+Shift+E/E 大小写平台漂移);
    8. schema/aura.at 文档条可选(Textarea keydown 未声明的先例;
       校验仅 S001 warning 非硬门)。vue 臂声明透传不实现(D4 承诺
       面,rust/vm 轨)。
  - **③快捷键冲突表**:全组(§5 D5 WT 风格)只命中带 Shift 组合;
    裸控制码路径零扰——widget 拦截仅吃精确表项(含修饰键全匹配)。
    逐键:Ctrl+Shift+T/W/E/O/K/Z 均无终端内程序默认占用(readline
    C-t/C-w/C-e/C-o/C-k/C-z 全为裸 Ctrl;vim C-w 窗口族同);浏览器
    同名键属宿主 chrome 不入终端。特勘两条:Ctrl+Shift+Z——自身 VT
    翻译 ctrl 分支 to_ascii_lowercase,本会译成 0x1A(同 Ctrl+Z),
    拦截先行即吞无损;Ctrl+Shift+←→——现 VT 翻译对 Named+ctrl 返
    None(按键丢弃),改绑切 Tab 净增功能。命中后不落队列不触发
    on_input(双通道都断,AC-04"未命中零变"由未命中路径零改动保证)。
  - **④D2 api 形状裁定(执行期,rec/Map 摩擦面规避)**:布局/Tab
    前端消费面用标量 getter 族(018 §10.2 平行 List 先例同款):
    `mux_tab_count/mux_tab_id_at/mux_tab_is_active_at/mux_tab_title_at/
    mux_active_pane_count/mux_split_axis/mux_slot_pane_id/mux_zoom_active/
    mux_pane_cols/mux_pane_rows/mux_pane_cursor_row/mux_pane_cursor_col`
    + `mux_pane_lines(pane_id) []str`;D2 契约路由 `mux_tabs() []str`
    ("id|active|title" 记录)与 `mux_layout() str`(DTO)保留
    (vue/HTTP 断言面)。前端 tick 走标量族,零字符串解析依赖。
  - next:T-01(back 泵可见性化+关闭语义+api 面)。

- 2026-09-15 stage:work · PLAN-019 · rev1 · **T-01..T-04 执行记录**
  (outcome:executing 进行中):
  - **T-01 已交付**:back 泵三档(db.at)+ 关闭语义(关末 Pane=关
    Tab;末 Tab 拒绝维持)+ 分屏 V1 深度上限 + D2 api 面 19 路;
    curl 剧本 12 组断言全绿(evidence/019/t01-model-assertions.log);
    auto-term 提交于 T-00+T-01 同笔。
  - **T-03 已交付(auto-lang 028c53b5f + 752c9fc49)**:shortcuts
    prop 五面 + row×for 摊平 + terminal 动态 key(.field 绑定)+
    merged_db_delegate 参数面 i32/i64 同型校正(014 返回面镜像);
    terminal 门 36/36 绿(4 件新单测在列)。
  - **T-02/T-04 已落地(app.at 全量重写)**:Tab 条(for 摊平按钮,
    点击激活/右键关闭/"+"新建/◐scheme)+ 布局槽位 if 枚举(zoom
    全屏/横 row/纵 col/单叶)+ WT 风格快捷键表 9 键(terminal
    onkeydown.* → .Shortcut(n) 分派 mux_*)。rust 形态全链编译绿。
  - **实机取证(进行中,发现 1 开放缺陷)**:GUI 剧本 12 帧入
    evidence/019(rust-gui-*.png,t02_seq.ps1 单进程驱动)+ MCP
    state 探针。已实证:Tab 条初始渲染、直键入(pump keys trace)、
    Ctrl+Shift+E 分屏模型态(axis=1/slot2=2)、泵三档、光标/几何
    同步。**开放缺陷 V-1:rust 轨视图结构不随模型重建**——split 后
    state axis=1 但屏面仍单 Pane(018 前 rust 轨从未行使过结构性
    view 变更;疑 run_app_devtools 重建路径/iced diff 对 into_iced
    自定义 widget 树的结构变更失效;Tab 条增删按钮同受累)。归
    auto-lang 侧修复,按 87eba67ab 裁定走 lang-019 worktree 流程。
  - **测试门观察**:terminal_pixel_preedit 全 gate 下偶发红(单测
    过;015 F-2 同族环境漂移,非本计划代码面)——归 016 门跟踪,
    复审时以单测/重跑为准。
  - **协同事件(634 会话反馈,已处置)**:019 的 auto-lang D4 曾以
    未提交 WIP 形态在 master 工作树(014-018 "master 直落" 惯例),
    634 会话按 87eba67ab 裁定做 stash 保真处置并反馈。处置:①D4
    已提交(028c53b5f),632 merge 已在其上构建,回撤重写会扰第
    三方落地——维持;②剩余 WIP(rust_ui.rs 参数面校正)已补提交
    (752c9fc49,提交语注明流程偏差与裁定张力);③master 工作区
    余留 renderer.rs eprintln/vue.rs/633 文档均他方遗留,不处置;
    ④本计划后续 auto-lang 改动(V-1 修复)改走 lang-019 worktree。
  - **方法论调整(用户 2026-09-15 指示)**:自动 GUI 驱动期用户同时
    在操作电脑,存在干扰;且用户实机目击**分屏成功、两侧均有 echo
    输出**——V-1"视图冻结"结论存疑,主嫌 = PrintWindow 对遮挡/后台
    窗口抓到陈旧帧(伪证)。处置:①测试 app Tab 条新增手动按钮面
    (横分/竖分/关Pane/Zoom,纯 app.at,消息分派复用既有
    .Split/.CloseFocusPane/.Zoom);②后续取证以用户手动操作 + 截图
    为准,自动驱动(前台抖动/键入注入)停用;③V-1 改记"待手动取证
    复核",暂不归缺陷。
  - next:用户手动分屏取证 → T-05/T-06 收口。

- 2026-09-15 stage:work · PLAN-019 · rev1 · **手动测试期缺陷双杀记录**
  (用户实机操作驱动,自动驱动停用):
  - **缺陷 A(已修,app.at)**:`.Split(axis)` 处理器参数名与模型字段
    `axis` 同名,a2r 处理器体把它译成 `self.axis`(恒 -1)遮蔽载荷——
    点分屏按钮实际分出 axis=-1 的屏,视图 if 链不匹配渲染回单 Pane。
    修法:参数改名 `ax`;生成码核验 `mux_split(ax)`。同型排查
    TabActivate/TabClose/Shortcut 无遮蔽。**codegen 参数遮蔽缺陷记
    auto-lang 侧待修(T-07 DEBTS)**。
  - **缺陷 B(已修,app 架构面)**:"+" 新 Tab 后再分屏 → **AppHangB1
    挂起**(用户实测 + 复现)。根因链:①按钮处理器在 UI 线程直接调
    mux_*(含 engine_spawn),而生成的 db 代码在引擎调用语句期间持有
    List 互斥锁守卫(临时变量存活至语句末),与 tick 执行器构成
    db 锁 × 引擎内部锁 ABBA 死锁;②trace 取证 resize 风暴
    (135x47↔67x47↔135x23 振荡)。修法:**用户动作队列**——UI 线程
    处理器只 `mux_enqueue(code,arg)`(纯 push),get_lines 每拍
    `mux_drain_actions()` 在 tick 线程排水执行;引擎操作回归单线程
    序列化(013 Action 单入口/所有权铁律的执行面)。headless 验证:
    队列 split/newtab 语义 ✓。
  - **缺陷 C(真凶,auto-lang lang-019 worktree 修复中)**:挂起前置
    条件的更深根因——**a2r 把 List 接收者的 `.set(idx,v)` 译成
    `Vec::insert(idx,v)`(插入语义,右移后续元素)**。split 后
    tab_root_node=[3,1](期望 [3,4],replace_child 的 set 变 insert);
    多 Tab 后表错位 → slot/focus 指向错误 Pane → 孤儿节点 + 几何
    风暴。018 未暴露:单 Tab 时 ti=0 的 insert 恰好读值正确,腐坏
    静默右移;Plan 514 W1 守卫只区分了 struct/List 未区分 List/Map。
    **最小 Rust 直调复现**(stub 引擎):`after split: root=[3, 1]`
    铁证。修法(镜像 Plan 514 W3 get 索引形特化):List 接收者的
    .set 走索引赋值特化臂 `recv[(idx) as usize] = v;`;Map/结构体
    语义不变。**按 87eba67ab 裁定在 lang-019 worktree(plan-019-dev)
    实施并验证**,金样回归 test 10_collections/008_list_set_index。
  - **V-1 正式撤销**:"视图结构不随模型重建"从未成立——系缺陷 A/B/C
    复合误导 + PrintWindow 遮挡抓帧陈旧帧伪证。用户实机目击与
    headless 模型断言均证明视图消费与泵正常。
  - **缺陷 C 修复落地(auto-lang f94558d36,plan-019-dev → master FF)**:
    最小根因修复 = `is_auto_list_expr` 补全局 var List 识别(镜像
    recv_is_list_like 的 PLAN-018 全局覆盖);另两处方法臂特化拦截
    (trans/rust.rs 8325/6563)为冗余保险。金样两件(参数形态 008 +
    全局形态 009),10_collections 9/9 绿;a2r 全套 375 绿 + 4 预存
    环境失败(基线同款)。生成码核验:`replace_child` 现发射索引赋值
    `[(ti) as usize] = new_child`,腐坏 insert 归零。
    **headless 全序列验证(修复后)**:队列 split → newtab → tab2 再
    分屏(原挂起路径)→ 关 Pane 兄弟收编 → 关末 Pane 消 Tab——每步
    snapshot 全一致(tab2 root=4→6→5,无孤儿无错位)。流程合规:
    lang-019 worktree(plan-019-dev)实施验证 → FF 落 master
    (87eba67ab 流程);worktree 留存供复审。
  - **用户手动验证(2026-09-15,AC-01/02 面通过)**:"横分竖分都可以
    了。开新的 tab 里每个 tab 自己也可以横分或竖分"——分屏正确性、
    每 Tab 独立布局、多 Tab 并存全部实机确认。修复前三态(横分无响应
    /新 Tab 分屏挂起)均消除。余项待用户顺手验证:Zoom/scheme/快捷键
    组/关末 Pane 消 Tab(模型面已 headless 断言)。
  - **T-05 vue 臂冒烟(数据面绿,视口待用户目验)**:vue arm 起服
    (vite :17400 200 / axum back :17401 200,DLL 经启动脚本注入);
    018 的 App.vue onUnmounted 重复导入缺陷已消失(单导入双使用);
    生成 App.vue 含 Tab 条(横分/竖分/关Pane 按钮)+ 每拍 tick;
    代理链冒烟:页面 200 → proxy tick 200 → mux_init 落账 →
    /api/mux/layout slots[1,0] / mux_tabs 记录正确。VM 轨 List.set
    原生(vm/list.rs list_set + shim_list_set)核验为替换语义,无
    缺陷 C 同款问题。浏览器目验留用户(本会话无浏览器驱动工具):
    http://localhost:17400/。
  - **T-06 全量门(auto-term workspace)**:`cargo test --workspace`
    exit 0 全绿(含 parity;引擎零改动回归)。014 护栏最小化/恢复
    与 vm 实机帧归 T-06 余项(可并入复审前抽查)。
  - **T-07 文档已落**:SD-01 spec 修订(terminal-mux-model.md V1
    语义节九条:关闭语义/打字随动焦点/泵三档/槽位投影+axis 语义/
    深度上限/shortcuts 契约/动作队列/D2 观测面/i32-i64 桥接)+
    DEBTS #19(已根修)/#20(参数遮蔽待修)/#21(锁守卫跨引擎调用,
    队列规避,键入泵残余窗口在案)/#22(观察条)+ app/README 可见
    多终端节。
  - **T-06 全量门**:cargo test --workspace exit 0 全绿(含 parity;
    引擎零改动)。vm 形态冒烟 = §10.6 阻塞项(保持 executing)。
  - **用户反馈双修 + 补金样(auto-lang 9980c6e4f + 9129760bd,lang-019
    worktree → master,期间并入并行会话 auto-musk/635 落地)**:
    ①terminal 臂 scheme 支持 `.field` FieldAccess 绑定(此前仅认
    Int 字面量/Ident,动态绑定静默回落 -1 → ◐ 按钮失效;用户实测
    确认后修复);②捷径拦截摘出 pane 聚焦门 → 窗口全局语义(用户
    期望:开窗即生效;app 面配套 = 捷径表只挂主槽一份去重);
    ③scroll_offset 同类缺口一并补;④回归金样
    terminal_scheme_and_scroll_fieldaccess_bindings_emit。terminal
    门 36 绿(preedit 漂移已记账)。master 频繁并行推进,每次落地
    前先 merge master 入分支再 FF(本轮两度)。
- 2026-09-17 stage:work · PLAN-019 · rev1 · **T-06 vm 链接阻塞解除**
  (outcome:pass,lang-019 worktree → master FF f6a40d20a):
  - **§10.6 归因翻案 + 根因定位**:真机探针 dump(loader.rs 临时插桩,
    已撤;t06-vm-link-probe-dump.log)证实 App 模块导出齐全(全部带点
    限定名),失败 reloc 为**裸名**——db.at 函数体内七处模块内裸调用
    (mux_resize_branch/mux_tab_id_at/mux_rect_kind×2/mux_split_axis/
    mux_slot_pane_id×2/mux_visible_pane_count)在 api.+db. 双同名导出
    下歧义,unique-suffix 回退拒绝猜 → 裸 reloc 无法绑定(入口导出全
    带点,Plan 545 own-module 回退只认 mod#sym 形)。原记录"GET 带参
    路由未合成"系巧合归因(裸调用 fn 恰多带参;018 面未暴露是因前端
    从不调用含裸调用的 fn;db.at 独立编译全通因独立编译不 qualify)。
  - **修法(auto-lang f6a40d20a)**:Codegen.current_fn_module
    (Stmt::Fn 编译期设置/恢复)+ resolved_func/resolve_call_symbol
    own-module 绑定两臂(镜像 loader Plan 322/545 语义到 codegen 层;
    resolve_call_symbol 臂配 fn_return_types 覆盖前向引用)。
  - **验证五链**:①真机 019 app 探针前后对比(7 裸 reloc 全部转
    own-module 带点解析);②最小语料红绿(master exe 红:Undefined
    symbol: bump;修复 exe 绿:过链开窗,t06-vm-delegate-*.log);
    ③金样 plan019_vm_own_module_link(语料 api.bump 语义差异化
    x+100,v==4 钉死 own-module 绑定非误绑包装);④日常档 5006 跑
    26 败=stash 基线逐名一致(预存漂移,零新增);⑤cargo tv
    3743/3743 全绿。落地流程:plan-019-dev 提交 → merge master
    (5de39dd55)→ FF 落 master,合并树金样复跑绿。
  - **019 提交态真机 vm 过链**:临时 auto-term worktree(a47a18f,
    已清)vm 形态开窗存活、0 Undefined symbol
    (t06-vm-link-fixed-019state.log)。
  - **残余面(归 020)**:主检出 020 WIP 新增 `use auto.term:
    window_width/window_height` 无 VM 原生目录项,vm 全链在其 WIP 上
    仍链接失败(term.window_width)——归 020 补原生注册;vm GUI 交互
    冒烟(快捷键/Tab 条/两分屏帧)待其落定后按既定方法论(用户手动
    +截图)补证。
- 2026-09-17 stage:work · PLAN-019 · rev1 · **vm GUI 冒烟(用户手动
  方法论)+ 浅色配色修订**(outcome:pass 冒烟面;配色面随修):
  - **冒烟结果(用户实机,浅色态截图为证)**:Tab 条(● 活动标记/
    新建/切换/关闭)、横竖分屏、快捷键组、Zoom、关 Pane、scheme
    切换全链正常——**AC-01/02/04/05 vm 形态实机通过**(用户原话
    "其他操作都正常");014 护栏抽查待用户顺手(非阻塞)。
  - **浅色配色修订(用户反馈:背景太亮、字体太浅对比不足)**:根因
    = LIGHT 盘沿用 WT Solarized Light 官方值——base16[15](亮白)
    =0xFDF6E3 与 def_bg 同色,cmd 文件名族走亮白 → 整列隐身(截图
    实证);def_fg=0x586E75 偏浅;def_bg 高饱和奶油刺眼。三修:白/
    亮白族反转深色(7=base01/8=base00/15=base02)+ 亮色族 vivid 化
    (10/12/14)+ def_fg→0x073642(base02)/def_bg→0xEEE8D5(base2)。
    落地:引擎单源 auto-term palette.rs(新护栏测
    light_scheme_white_family_readable_on_bg:白族与 bg 亮度差>60
    防回退)+ auto-lang widget 回退副本表同值同修(7af29ce8e,FF
    d2a0032a8);palette_color 轴测试/widget fallback 测试随值更新;
    016 像素金样 5/5 绿(dark 臂零涉)、widget terminal 门 27 绿、
    autoterm-core 6 绿。备注:019 非目标"autoterm-core 零改动"指
    机制面(per-handle/scheme 面);本次为 016 配色数据面修订,用户
    授权,金样无 light 钉值故零重生成。
  - **浅色切换旧行重着色修复(第二段,auto-lang eaf237f11)**:用户
    复验发现切方案后**旧输出文字不重着色**(残留深底浅字不可读),
    新输出才正确。插桩实证(cmd 默认文本 kind=0 语义 Default,引擎
    语义穿传无恙)→ 病灶 = widget 段落缓存键:行 digest 只含语义值,
    切方案语义不变 → 复用旧 palette 烤入的 Paragraph(构建时
    to_iced_color 已定色)。修法 = draw 每帧把生效 [18] 表 hash 为
    pal_key XOR 进 digest,任何换表即全行失效重建。terminal 门 32
    绿+像素金样绿;自证截图 t06-vm-light-toggle-fixed2.png(切浅色
    后旧行即深字,无需新输出)。
  - **终端滚动回看接线(冒烟期用户追加授权)**:用户实测无法翻看
    历史输出。引擎 TermSession scroll/display_offset/history 016
    既有,断点 = FFI 无 scroll 出口 + glue 无排水( widget 滚轮仅
    累 badge 计数)。接线:①引擎两导出 autoterm_engine_scroll(正=
    上翻历史)/autoterm_engine_scroll_offset(回读)+ write_input
    键入即贴底(scroll_to_bottom,终端惯例);②widget 滚轮增量入
    core 队列(正=上翻,iced y>0=上);③双 glue(rust 侧车 term.rs/
    vm term_engine.rs)泵内 Key 侧带排水先于损伤重采(同拍快照即
    滚动视图)+display_offset 回写 badge。rust/vm 双形态同享(侧车
    与 vm glue 同载一 dll)。vue 臂只读视口臂不承诺(§0 边界)。
    后续演进:虚拟滚动容器已记入 auto-lang 蓝图 roadmap Desktop生态
    (837e3420e),官方落地后终端迁移复用、删除自绘滚动条。
    **滚动条暂缓裁定(2026-09-17 用户裁定)**:自绘滚动条存在拇指比例
    不准、拖拽后位置/比例漂移、拖拽表现生硬、缺 hover 加宽与 pointer
    联动、不支持 theme 等问题——逐项修复不现实,用户裁定**暂缓**,已
    知问题清单记入 auto-lang roadmap 同条目,待官方虚拟滚动组件统一
    解决。滚动基础能力(滚轮翻历史/贴底/开窗焦点)保留可用,**不作
    为 019 阻塞项**。
- 2026-09-17 stage:work · PLAN-019 · rev1 · **T-08 双仓锚定+收口**
  (outcome:pass,step 9/9,status → execution_done):
  - **用户终验三项全过**:①开窗即焦点(启动直接打字);②浅色配色
    (背景柔和、文字对比清晰);③014 护栏(ping -t 挂载 → 最小化/
    恢复布局无爆炸、内容跟上)。滚动条自绘形态按用户裁定暂缓
    (已知问题清单见 auto-lang roadmap,非阻塞)。
  - **双仓锚定**:auto-lang master = 4579d59e3(019 相关:f6a40d20a
    vm 委托合成根因修复、7af29ce8e 配色副本表、7363111d1 滚动条交互
    与 eaf237f11 前 palette cache 键、837e3420e/4579d59e3 roadmap);
    auto-term 019 链末笔 = 9966a54(链:33a9649 vm 链接记账 → c23a8ce
    浅色配色 → 8f0f35e 冒烟记账 → 5ab0381/5f34ac7/337f612 滚动接线 →
    c422796/9966a54 蓝图交叉与暂缓裁定;main 当前 tip c3b8d50 为
    021 的提交,叠于其上)。临时 worktree
    term-019-smoke/term-019-verify 与隔离 target 均已清理
    (wt-guard clean);lang-019 worktree 留存供复审。
  - next:/auto-plan:review 复审。

- 2026-09-17 stage:review · PLAN-019 · rev1 · outcome:**pass**
  (在实现会话内复审,结论依据工件与可复现证据重建,非执行摘要)。
  reviewed_commit: auto-term 9966a54(019 末笔;main tip c3b8d50 为
  021 叠加)· base_commit: c7b2df4(019 立项点,40 提交)·
  dependency_revisions: auto-lang master 788a3369b(含复审 F1 修复;
  019 系 5de39dd55/7363111d1/4579d59e3)· spec_inputs:
  docs/specs/terminal-mux-model.md @ a55b454(SD-01 九条在库;第 5 条
  已被 020 rev2 按序演进为矩形投影契约,谱系一致)。
  **AC 映射**:AC-01/02/04/05 = 用户 vm 形态实机全链通过("其他操作
  都正常")+ rust 实机截图 evidence/019/rust-gui-*.png + 模型断言
  t01-model-assertions.log;AC-03 = 泵三档断言 + 014 护栏用户通过
  (最小化/恢复无爆炸、内容跟上);AC-06 = scheme 切换生效 + 016 像素
  金样 5/5 绿(light 盘修订经用户授权,护栏测钉亮度差,light 不入
  金样);AC-07 = 单 Pane 交互零回归(用户实机 + terminal 门 32 绿);
  AC-08 = rust/vm 实机构建绿(自证多次)+ auto-term workspace exit 0
  + parity 绿;AC-09 = SD-01 spec 九条 + DEBTS #19-#22 + evidence/
  019 齐备(t06-* 五件 + 自证截图 + 复验脚本)。
  **复审门禁**:auto-lang tv 3743 档 1 失败(见 F2)+ tf 全量
  3596/3597(--no-fail-fast,唯一失败同 F2)+ terminal 门 32 绿 +
  像素金样 5/5 绿;auto-term workspace 全绿。
  **findings**:F1(019 引入,no-ui 编译回归)——滚动排水块缺 ui
  特性门控,tv 档 E0433×4;已修复(apply_scroll_queue cfg ui/not(ui)
  双臂,788a3369b)并复验。F2(master 基线,非 019)——plan367
  real_sidebar 语料解析失败,归因 PLAN-637 B1 落地 415484a65 改
  015-notes/sidebar.at 引入,路由 637 会话处置。F3(非阻塞文档缺口)
  ——滚动回看面(scroll FFI+键入贴底)与浅色盘可读性修订尚未写入
  terminal-mux-model.md,merge 折叠账本时补记。
  evidence:t06-* 七件 + light-selfcheck.ps1 + /tmp/tf_review.log
  摘录(3596/3597)。next:merge。

- 2026-09-17 stage:merge · PLAN-019:r1 · 收据(逐检查点记入,2/3、3/3
  提交补 archived/cleaned):
  - **prepared**:复审基线 rev1 / base c7b2df4 / 019 末笔 9966a54 /
    auto-lang 788a3369b;canonical diff = F3 补记
    (terminal-mux-model.md V1 增第 10 条滚动回看面、第 11 条配色盘
    修订);账本投影目标 .autoos/specs.json 六节 P019-1..6。
  - **landed**:019 实现按主检出直落惯例随工作逐笔落于 main
    (019 末笔 9966a54 在 main 祖先链;auto-lang 侧经 lang-019
    worktree FF 落 master,788a3369b 在祖先链)。
  - **ledger_refreshed**:.autoos/specs.json P019-1..6 六节入账
    (architecture file → docs/specs/terminal-mux-model.md,其余 →
    archived 路径;round-trip 校验+回读 OK)。
  - **archived**(2/3):git mv docs/plans/archived/019-mux-visible-ui.md
    + status archived + completion_kind delivered(0c6533a 复审 pass
    之后的交付终态;前置检查点 1/3 2ca034f)。

## 10. 待澄清事项

1. **快捷键风格**:V1 默认 Windows Terminal 风格组合键(§5 D5 表;
   与现输入语义"像 Windows Terminal"一致);tmux prefix 模式
   (C-b 状态机)留后续,若用户裁定 prefix 为必要条件,提请
   翻案(D4/D5 重设计)。
2. **末 Tab 关闭语义**:V1 = 拒绝 no-op(app 常驻,018 口径延续);
   产品级"关末 Tab = 退出 app"留用户后续裁定,不阻 V1。
3. **T-B 降级的视觉边界**:若 T-00 勘定 T-A 不可行,V1 分屏 = 树深
   1(两 Pane 横/竖),更深嵌套留 T-C/后续计划——届时 AC-02 口径
   收窄为"两 Pane 分屏",rev 不增(等价实现路径,§5 D3 已预埋)。
4. **vue 臂分屏视觉**:非目标(§0);若用户裁定 vue 全功能为必要
   条件,需独立评估 vue terminal 臂扩建(a2vue 最小视口臂现状),
   另行立项。
5. **Tab title 来源**:V1 = 序号 + shell 名(静态派生);动态
   title(OSC 转义/进程名感知)归 ③ OSC 计划。
6. **【T-06 发现 → 2026-09-17 已解除,归因修正】vm 形态链接失败**:
   原记录"App 侧 api 委托合成对 GET 带参路由未合成"系巧合归因——
   真机探针 dump 证实根因 = db.at 函数体内**模块内裸调用**(七处,
   mux_resize_branch 族)在 api.+db. 双同名导出下歧义,unique-suffix
   回退拒绝猜 → 裸名 reloc 无法绑定(与带参/GET 无关)。已根修:
   auto-lang f6a40d20a(Codegen.current_fn_module + own-module 绑定
   两臂,lang-019 worktree → master FF;五链验证见 §9 2026-09-17
   记录)。019 提交态真机 vm 过链验证在案。残余:主检出 020 WIP 的
   `use auto.term: window_width/window_height` 缺 VM 原生注册,其
   WIP 上 vm 全链仍失败——归 020;vm GUI 交互冒烟待补证(用户手动
   方法论)。
