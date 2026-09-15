---
plan_id: PLAN-019
status: drafting
feature_name: 可见多终端 UI——Tab 条 + 分屏渲染 + 快捷键(006 蓝图 ①)
author: [zcode-session]
created_at: 2026-09-15T00:00:00Z
updated_at: 2026-09-15T00:00:00Z
plan_revision: 1
current_step: 0
total_steps: 9
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
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

- **T-00 开工门勘定**:①布局构建三案判据——a2r/VM 对 view 内
  动态构建(if/for 内嵌 row/col、函数递归、平行 List 驱动循环)
  的支持实证(最小语料编译+运行,记 §9);案决 T-A/T-B。②auto-lang
  master 锚点勘定(018 后新提交面;shortcut 扩建点 7 处枚举位)。
  ③快捷键冲突表勘定(候选键 × 常见终端内程序占用)。前置:无。
  产出:§9 勘定附记。关联全部。
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
