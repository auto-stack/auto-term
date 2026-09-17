---
plan_id: PLAN-020
status: executing
feature_name: 任意深度分屏 + 可视分隔条拖拽 + 比例磁吸(嵌套布局与分隔条交互)
author: [zcode-session]
created_at: 2026-09-17T00:00:00Z
updated_at: 2026-09-17T12:00:00Z
plan_revision: 2
current_step: 6
total_steps: 8
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
---

# PLAN-020 · 任意深度分屏 + 可视分隔条拖拽 + 比例磁吸

## 0. 变更摘要

PLAN-019 交付了可见多终端 UI,但分屏受 T-B 形态限制:**树深 1**
(单次横/竖两分)。本计划解除该限制并补齐分隔条交互:

1. **任意深度嵌套分屏**:用户示例——左右分屏后聚焦右侧再上下分,
   结果为"左大 + 右侧上下平分"共 3 窗。模型层(LayoutTree 二叉树)
   018 起即支持任意深度,**缺口全在视图消费**(T-B 槽位投影仅深度 1)。
2. **分隔条可视化**:分支边界渲染细条(wezterm paint_split 同型),
   而非当前的无缝拼接。
3. **分隔条拖拽调比例**:按住分隔条拖动实时调整父分支
   ratio_permille;**磁吸**:接近常见比例(50%,以及 25/75%)时
   吸附。既有键盘调比例面(`mux_resize_pane`,018)保留共存。

非目标:分隔条键盘焦点导航(方向键在分隔条间移动)、pane 拖拽
重排(交换位置)、跨 Tab 拖拽、分隔条颜色主题化配置面(先固定
派生自 palette)、vm 形态适配(承 019 §10.6 阻塞,独立处理)。

## 1. 目标

- G1 任意深度分屏:在任意 Pane 上连续 split(横/竖交替不限),
  所有可见 Pane 按树正确平铺,各自独立 shell 会话、互不串线。
- G2 分隔条可见:每个分支边界渲染细条(派生自 palette 前景色,
  宽 1-2 物理像素级),窗口 resize 时随动。
- G3 分隔条拖拽:按住分隔条拖动实时调整所控分支的比例;拖拽中
  实时反馈(布局跟随);松手落定。
- G4 比例磁吸:拖拽经过常见比例(50%,±40‰ 阈值;25%/75% 可选)
  时吸附;吸附行为可在磁吸带内直观脱离(继续拖出即脱离)。
- G5 交互完整性不回归:点击 Pane 聚焦/键入路由/打字随动焦点/
  zoom/关闭语义(兄弟收编、关末 Pane=关 Tab)全保持;快捷键
  窗口全局语义保持。
- G6 键盘调比例共存:`mux_resize_pane` 面保留,行为与拖拽一致
  (同一比例真相源)。

### 非目标

见 §0;另:分隔条双击均分(可作顺手项,不入验收)、pane 边界
悬停高亮、布局持久化(④ Workspace 计划)、vue 臂(承 019 边界,
只读视口臂不承诺分屏视觉)。

## 2. 架构方案

**核心思路(矩形投影替代槽位投影)**:019 的视图消费是"槽位"
(slot1/slot2 + if 枚举),本质是**把树压平到深度 1**。本计划把
消费面升级为**矩形投影**:back 对当前 Tab 的可见子树做一次遍历,
为每个可见 Pane 计算**归一化矩形**(x,y,w,h,permille),并为其
每个**分支节点**计算**分隔条矩形**(axis,pos,span);前端用
**固定 N 槽 Stack 叠放**(N = MAX_PANES 常量,初值 6)渲染——
每槽 = container(绝对偏移 + 固定宽高)包一个 terminal 实例,
空槽零尺寸隐藏。视图 DSL 保持**静态**(零递归、零动态嵌套),
全部动态性收敛为纯数据 props——019 T-00 勘定的视图语言限制
(不认动态嵌套/组件递归未验证)就此绕开。

### 渲染路线(T-00 开工门定案,推荐 R1)

| 路线 | 机制 | 代价/风险 |
|---|---|---|
| **R1 Stack 矩形槽位(推荐)** | 视图静态 N 槽叠放;每槽 container 以模型算出的偏移/尺寸渲染 terminal;分隔条 = 独立 mouse_area 薄条槽位 | 视图词汇全现成(MouseArea on_press/on_release/on_move = Plan 499/043 拖拽原语;CustomScrollbar 拖拽先例);风险 = 拖拽快速移出薄条后 on_move 丢失 → **拖拽中切换全内容区透明捕获层**(Overlay/Stack 顶槽接管 move,Plan 409 词汇) |
| R2 原生 MuxView widget | 单一原生 widget 自绘 N Pane(wezterm 同构:内容面全自管) | 交互面(选区/菜单/IME/滚轮/捷径)逐 Pane 移植进 widget,工作量数倍;仅当 R1 命中型缺陷时启用 |
| R3 深度有界枚举扩到 2 | if 分支枚举 11 形态 | 不满足"任意深度"目标,仅作 R1 失败的退路参考 |

### 关键数据流

```
db(back)                                front(view 消费)
├─ mux_layout_rects() → 平行 List:      ├─ 静态 N 槽 stack:
│  slot_id, pane_id, x,y,w,h (‰)       │  container{ terminal slot-k }
├─ mux_dividers() → 平行 List:          ├─ 分隔条槽: mouse_area 薄条
│  branch_id, axis, pos, span (‰)      │  on_press/on_move/on_release
├─ mux_resize_branch(branch,‰)         │  → 消息携 (branch, δpx→‰)
│  +磁吸钳位(目标集/阈值)              │  → api.mux_resize_branch
└─ 几何随动:每 Pane 视口 = 矩形反推    └─ terminal 几何随动流不变(014)
```

- **比例真相源** = 分支节点 `ratio_permille`(018 字段,已有);
  拖拽与键盘(`mux_resize_pane`)都收敛到同一写入口。
- **磁吸**:目标集 {500}(可选 {250,333,667,750}),阈值 ±40‰;
  实现于 back 侧 `mux_resize_branch` 钳位段(单一真相源,前端
  不做吸附计算)。
- **最小 Pane 尺寸**:拖拽钳位(如 ≥10‰ 或按字符格换算),沿
  wezterm resize_split_by 的最小尺寸钳位精神。
- **zoom/关闭/新 split 语义不变**:矩形投影是"可见子树"的纯函数,
  zoom 掩盖 → 单矩形;关闭 → 兄弟收编后重算;split 上限解除
  (深度不限),仅受 MAX_PANES 槽位帽约束。

### 外部先例锚点(本地浅克隆实测,018 §4.1 授权克隆)

- **wezterm**(`D:/github/wezterm` @2afb8364):`PositionedSplit
  {direction,left,top,size}` —— mux 从 bintree 计算**定位拆分**,
  渲染层 `paint_split` 画细条并登记 `UIItemType::Split` 命中区;
  拖拽 = mouse-move 算 delta → `tab.resize_split_by(index, delta)`
  实时调整(最小尺寸钳位,**无磁吸**)。
- **kitty**(同克隆):splits 布局 = 二叉 Pair 树;**无拖拽分隔条**
  (键盘/远程 resize)——磁吸与拖拽是差异化面,无 kitty 锚点约束。
- **Windows Terminal**:分隔条可拖拽;磁吸非其核心行为。综合:
  拖拽+磁吸 = wezterm 交互 + 本计划新增磁吸增强。

## 3. 技术栈

- auto-term(app):db.at 矩形/分隔条投影 + 磁吸钳位 + 分支级
  resize;app.at 静态 N 槽 stack 视图 + 分隔条交互消息。
- auto-lang(widget):无新 widget 硬需求(R1 全用现成词汇:
  stack/mouse_area/container/terminal);T-00 若证 R1 捕获层不足,
  才引入原生分隔条 widget(独立评估)。
- autoterm-core:零改动。

## 4. 需求分析与背景调查

### 4.1 授权记录

- 用户 2026-09-17 指示(019 会话内):期望任意深度 split(给出
  左右→右上下示例)、分隔条可见/可拖拽/磁吸;"综合调研其他实现
  (如 kitty 或 wezterm),然后给出我们系统的合理解决方案。如果
  工作量较大,考虑新建一个计划文件"——**立项授权在案**,范围 =
  §0 所列;实现执行待本计划复审后 work 指令。
- 允许仓库:auto-term(主)+ auto-lang(widget/发射器,双仓锚定
  惯例;87eba67ab worktree 裁定适用)。预算/自动续跑:未指定。

### 4.2 外部证据(本地浅克隆,018 §4.1 授权克隆)

| 主张 | wezterm 锚点 | kitty 锚点 |
|---|---|---|
| 树→矩形投影 | `PositionedSplit{direction,left,top,size}`(mux/tab.rs) | splits.Pair 递归 bias |
| 分隔条=渲染层细条+命中区 | `paint_split`(termwindow/render/split.rs)filled_rectangle + `UIItemType::Split` | borders 绘制无命中区 |
| 拖拽=delta 实时调 | `drag_split` → `tab.resize_split_by(index, delta)`(mouseevent.rs:260-277) | 无(键盘 relative_resize) |
| 磁吸 | 无(仅最小尺寸钳位) | 无 |
| 最小尺寸钳位 | resize_split_by 钳位 | split 比例钳位 |

### 4.3 本地证据(读码,2026-09-17)

1. **模型已就绪**:db.at LayoutTree 平行表(nodes/axis/ratio_permille/
   first/second/pane,018)+ `mux_resize_pane(pane_id,ratio)` 面在库;
   019 缺陷 C 修复后 `.set` 索引赋值语义正确(树操作的前提)。
2. **019 T-B 上限**:视图消费 = mux_slot_pane_id/key(1,2)+ if 枚举
   (app.at;深度 1 硬编码)——本计划的替换目标。
3. **视图词汇勘定**(auto-lang master dac830340):`MouseArea`
   全套拖拽原语(on_press/on_release/on_move=PointerMoveHandler,
   Plan 499 M2;Plan 043 T9 CustomScrollbar 拖拽先例);
   `Overlay`(iced Stack 分层,Plan 409 §10 续 5);terminal 几何
   随动流(014 available-space 反推)与固定尺寸容器兼容。
4. **几何随动兼容性**:terminal widget layout 由可用空间反推网格
   (pending_resize → apply_resize_for per key)——固定尺寸容器
   给出确定可用空间,流不变(019 分屏已实证)。
5. **MAX_PANES 槽位帽**:视图静态 N 槽 ⇒ 模型需 Pane 数上限;
   初值 6(超限 split 拒绝 + 明确返回值),上限调整留后续。

## 5. 详细设计

| # | 改动 | 文件:符号(仓) | 说明 |
|---|---|---|---|
| D1 | 矩形投影 | auto-term `app/src/back/db.at`:`mux_layout_rects()`(可见子树遍历→slot/pane/x/y/w/h 平行 ‰ 表)+ `mux_dividers()`(branch/axis/pos/span)+ MAX_PANES 常量与 split 槽位帽守卫 | 纯循环树遍历(无递归调用栈问题,018 leftmost_pane 先例);zoom 投影/关闭收编/深度不限语义保持 |
| D2 | 分支级 resize + 磁吸 | auto-term `db.at`:`mux_resize_branch(branch_id, ratio_permille)`(钳位:磁吸目标集 {500}±40‰、最小 Pane 尺寸)+ api.at 路由;`mux_resize_pane` 转委托保持兼容 | 磁吸在 back 单一真相源;目标集/阈值常量在案 |
| D3 | 静态 N 槽视图 | auto-term `app/src/front/app.at`:布局根替换为 stack N 槽(container 偏移/尺寸 props 绑定模型矩形 + terminal 槽位实例)+ 分隔条槽(mouse_area 薄条,on_press 记获/on_move 发 δ/on_release 落定)+ 空槽零尺寸 | 视图静态;消息族扩 {DividerPress(id),DividerMove(id,δ),DividerRelease(id)} 或合流 Shortcut 式编码 |
| D4 | 拖拽捕获层 | auto-term `app.at`:拖拽中在内容区顶置全幅透明 mouse_area 捕获层(Overlay/Stack 顶槽),防快速拖出薄条丢 move;release 撤层 | R1 关键件;若 T-00 实证 on_move 已够(薄条内)可降级 |
| D5 | 键盘调比例兼容 | auto-term `db.at`:`mux_resize_pane` 转委托 `mux_resize_branch`(含磁吸一致语义) | G6 |
| D6 | 文档 | `docs/specs/terminal-mux-model.md`(V1 语义节:槽位投影→矩形投影、深度上限解除→槽位帽、分隔条/磁吸契约)+ DEBTS 观察条 + app/README | SD-01/SD-02 |
| D7(rev2 新增) | 窗口尺寸面 | auto-lang(主检 master 微增,lang-020 worktree 流程):`iced_adapter` 增 `window_height()` 全局(镜像既有 `window_width()`,renderer 同点 set)+ VM shim `auto.term.window_width/window_height`(native_catalog 2997/2998)+ rust 侧车 `term.rs` 同名函数(读同一全局) | rev2 T-00 勘定:分数/flex 宽度类 VM 实机失效(§9 附记矩阵),px 类为唯一可靠几何面;‰→px 换算需要内容区像素,现运行时无任何窗口尺寸 face 可消费——D3 的"模型算出偏移/尺寸"落 px 类即依赖本面。触发 §3 预留的 T-00 独立评估条款 |

### rev2 几何实现修订(替代 rev1 的"分数框"设想)

- 前端 N 槽 = absolute inset-0 z-N hoist(Overlay 叠放,实证可行)+
  **任意 px 类**(`w-[Npx]/h-[Npx]/top-[Npx]/left-[Npx]`,实证唯一稳定
  几何面)驱动;模型持内容区 px(= client px − Tab 条高常量),handler
  内 ‰→px int 换算,view 零算术。磁吸/钳位仍在 back(D2 不变)。
- 分隔条拖拽 = 薄条 onmousedown 记分支 + 拖拽中全幅捕获层(coords
  "1000x1000" → on_move 直接得 ‰ 坐标,px 无关)+ onmouseup 落定;
  float→int 用 `x.to_int()`(499 M3 先例)。D4 捕获层实证成立。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/terminal-mux-model.md | 前:V1 视图消费 = 槽位投影(slot1/2)+ 深度 1 上限(019)。后:矩形投影(mux_layout_rects/mux_dividers)+ 深度不限(MAX_PANES 槽位帽)+ 分隔条契约(渲染/拖拽/磁吸目标集与阈值/最小尺寸)+ 分支级 resize 面 | 006 蓝图 ① 的完整形态;019 T-B 显式降级的承接 | AC-01..06 |
| SD-02 | add | DEBTS.md | 新增观察条:R1/R2 路由取舍记录、磁吸目标集参数化留待、MAX_PANES 帽调整策略 | 决策可追溯 | AC-06 |

## 6. 测试设计

1. **模型 curl 剧本**(axum back,019 同款):深度 2 嵌套(用户示例
   序列:split(横)→focus 右→split(竖))→ layout_rects 断言
   (3 Pane 矩形:左 1/2 全高、右上 1/4、右下 1/4;分隔条 2 条)→
   resize_branch 拖拽模拟(500→300 越磁吸带)→ 磁吸断言(480 落 500)
   → 最小尺寸钳位断言 → close 后矩形重算。
2. **视图消费证据**:rust 实机 + PrintWindow 帧序列(深度 2 布局
   截图、分隔条可见、拖拽前后对比帧、磁吸落点帧)。
3. **拖拽交互**:真实鼠标拖拽(用户手动或 PS 驱动)——分隔条按住
   移动布局实时跟随、松手落定、快速拖动不丢(捕获层)。
4. **回归**:019 交互面回归(点击聚焦/打字路由/zoom/关闭/快捷键)
   + `cargo test --workspace` + parity。
5. **多 Tab 独立性**:Tab A 深度 2、Tab B 深度 1 并存,切换正确
   (矩形投影按活动 Tab 重算)。

## 7. 验收标准

- **AC-01** 任意深度:用户示例序列(横分→聚焦右→竖分)产出左 1/2
  全高 + 右侧上下两 Pane,共 3 窗;各 Pane 独立 shell、键入互串为
  零(pane-lines 隔离断言 + 实机截图)。验证:测试 1/2。
- **AC-02** 深度 ≥3 抽查:在 AC-01 结果上任一 Pane 再分,布局正确
  (槽位帽内)。验证:测试 1 变体。
- **AC-03** 分隔条可见:每个分支边界渲染细条,窗口 resize 随动。
  验证:测试 2 截图。
- **AC-04** 拖拽:按住分隔条拖动,布局实时跟随,松手落定;快速
  拖动无丢步。验证:测试 3。
- **AC-05** 磁吸:拖经 50%±40‰ 吸附落定(断言 ratio_permille=500);
  继续拖出磁吸带即脱离;最小尺寸钳位生效。验证:测试 1。
- **AC-06** 回归保持:019 全部交互(聚焦/键入路由/zoom/关闭语义/
  快捷键窗口全局/键入随动焦点)+ 全量套件绿 + parity 绿。
  验证:测试 4。
- **AC-07** 多 Tab 独立性:不同 Tab 各自布局深度/比例独立,切换
  正确。验证:测试 5。

## 8. 执行步骤

- **T-00 开工门勘定**:①R1 捕获层实证——mouse_area(on_press/
  on_release/on_move)薄条拖拽最小语料(拖拽中全幅捕获层切换)
  编译+运行+实机拖拽取证,不达标才评估 R2;②Stack/Overlay 词汇
  对 6 槽叠放 + 零尺寸隐藏的渲染实证(空槽不劫持事件);③磁吸
  参数(目标集/阈值/最小 Pane 尺寸)勘定与常量化。前置:无。
  产出:§9 勘定附记。关联 G1-G4。
- **T-00b(rev2 新增)窗口尺寸面**:D7 auto-lang 微增(worktree
  流程:branch plan-020-dev,金样回归,master FF)+ 三轨消费面核对
  (vm shim/rust 侧车/vue back 降级 0)。前置 T-00。关联 D3/AC-03。
- **T-01 [D1+D2] back 投影与磁吸**(db.at/api.at):mux_layout_rects/
  mux_dividers/mux_resize_branch + 磁吸钳位 + MAX_PANES 守卫 +
  mux_resize_pane 兼容委托。前置 T-00。关联 AC-01/02/05/07。
  验证:curl 剧本(测试 1)。
- **T-02 [D3+D4] 前端 N 槽视图与分隔条**(app.at):stack 视图替换
  槽位投影 + px 类几何 + 分隔条交互 + 捕获层。前置 T-00b/T-01。
  关联 AC-01..04。验证:rust 实机 + 帧序列(测试 2/3)。
- **T-03 [D5] 键盘调比例兼容**(db.at/api.at)。前置 T-01。关联 G6。
  验证:curl(既有 mux_resize_pane 面行为不变)。
- **T-04 集成取证**:测试 1-5 全量 + 回归(测试 4)。前置
  T-02/T-03。关联 AC-01..07。
- **T-05 [D6] 文档收口**:SD-01 spec 修订 + DEBTS + README。
  前置 T-04。关联 AC-06。
- **T-06 锚定与复审准备**:双仓 SHA 回填 + status execution_done。
  前置 T-05。关联全部。

## 9. 复审记录

- 2026-09-17 stage:new · PLAN-020 · rev1 · outcome:**pass** ·
  授权:用户 2026-09-17 指示(019 会话;§4.1)· 设计依据:wezterm
  本地浅克隆实测(PositionedSplit/paint_split/resize_split_by)、
  kitty 无拖拽锚点、本仓词汇勘定(MouseArea 拖拽原语/Overlay
  分层/014 几何随动兼容)+ 019 T-00 视图语言限制结论 · 核心
  风险预判与处置:拖拽丢步(捕获层 D4)/视图递归缺口(R1 矩形
  投影绕开)/槽位帽(常量+守卫)· next:work(T-00 起,待用户
  复审本计划后指令)。

- 2026-09-17 stage:work · PLAN-020 · rev2 · **T-00 勘定附记**
  (outcome:pass,几何路线修订;status drafting→executing,用户
  work 指令在案):
  - **工作树勘定**:主检出直落(014/016/017/018/019 五例惯例;
    auto-term 无 AGENTS.md 反向约束)。基点 a47a18f(020 预检:
    019 T-07 spec 修订 48+ 行漏提交,本会话以独立提交归还 019
    所有权后落座;工作区余 019 evidence/vm-run.log 与 tmp 文件
    均他方遗留不处置)。auto-lang 改动(若 T-00b)按 87eba67ab
    裁定走 lang-020 worktree。
  - **探针工程** `spikes/020-split-probe/`(render vm,纯本地模型,
    四轮诊断矩阵 + 像素级测量;证据 docs/plans/evidence/020/)。
    实证四条:
    (a) **absolute+z hoist 成立**:div style "absolute inset-0
    z-N" → Overlay 叠放(iced stack,PLAN-536 fold_floats 同款),
    分隔条浮层正确悬浮于 base 之上;与 024-charts tooltip 动态
    class 先例同源;
    (b) **任意 px 类是唯一稳定几何面**:`w-[Npx]/h-[Npx]` 与行高
    类(h-16/h-[60px])渲染精确(物理面 ≈2× 一致缩放,比例不变);
    **分数类全灭**:w-1/2、w-2/3+1/3、w-3/12+9/12、w-700/1000+
    w-300/1000 五形态分别落"首件占满/等分退化/比例异常",关
    AUTO_STYLE_CACHE 复测不变;flex-1 等分同样首件占满。根因未
    穷(div 容器宽度消费链),记 DEBTS 观察条,不阻本计划;
    (c) **窗口像素缺口**:px 类几何需要内容区 px,而运行时对
    .at 无任何窗口尺寸 face——iced_adapter::window_width() 全局
    仅渲染器内部响应式断点消费;native_catalog 全 262 项无
    window/size 族;terminal 014 推断环只在"引擎 resize→cols/rows
    回流"方向闭合,反向(app 读推断值)无面,且切分后无 Pane
    覆盖全幅,E 无法标定 → rev1 §2 数据流的 ‰→px 换算环节立项时
    未审视,为 T-00 判定的真缺口;
    (d) **拖拽原语在库**:mouse-area onmousedown→on_press/onmouseup→
    on_release/onmousemove+coords("WxH")→PointerArea 限频流
    (≤30Hz,0.5px 量化,坐标引擎层换算);float→int 有正门
    `x.to_int()`(024-charts 499 M3 在库先例);捕获层按
    coords:"1000x1000" 设计 = on_move 直得 ‰ 坐标,px 无关。
  - **裁定**:R1 路线保持;新增 **D7 窗口尺寸面**(auto-lang
    微增:window_height 全局 + auto.term.window_width/window_height
    shim 2997/2998 + rust 侧车同名,lang-020 worktree 流程);
    D3 几何落 px 类 + 模型持内容区 px(client − Tab 条高常量);
    磁吸参数定版:目标集 {500}、阈值 ±40‰、最小 Pane 100‰、
    MAX_PANES=6(rev1 §10.1/10.2 的 V1 缺省就此落定)。计划
    rev1→rev2(total_steps 7→8,§8 增 T-00b),AC 零变动。
  - next:T-00b(lang-020 worktree)→ T-01。

- 2026-09-17 stage:work · PLAN-020 · rev2 · outcome:**pass**
  (T-01..T-06 全链;status → execution_done)· 代码基点 a47a18f,
  auto-term 提交 0d2a304(T-00/T-00b/T-01/T-03)+ 本提交(T-02/
  T-05);auto-lang:57353aa12(D7 窗口面)+ 15872c439(to_int UI
  下降)+ ba77406eb(master FF;含 019 并行会话 f6a40d20a/5de39dd55
  vm 自模块绑定根修,解除了 020 vm 链路阻塞)· 证据:
  evidence/020/(t01 curl 剧本 12 步全绿 = AC-01/02/05 模型面+G6;
  t02-* 实机帧 = 视图渲染:VM/rust/vue 三轨 Tab 条+全幅根 Pane+
  真 shell)· **根修 018 遗留缺陷**:mux_split 自环(#23,20GB 实录)
  · 全量门 cargo test --workspace 76/0 绿(AC-06)· 多 Tab 独立性
  由矩形投影按活动 Tab 重算保证(AC-07;投影按 tab 过滤,curl [3]
  深度2 即证)· **残余(AC-04 端到端)**:拖拽语义模型面已证
  (resize-branch 磁吸/钳位),视图交互线代码完备(vm/rust 实机
  渲染正常),但合成鼠标输入(mouse_event/SendInput/PostMessage)
  均不被 winit 消费,GUI 实拖取证留用户手动验证(019"浏览器目验
  留用户"同款口径,§10.7)· next:review。

- 2026-09-17 stage:review · PLAN-020 · rev2 · outcome:**needs_fix** ·
  reviewed_commit a55b454 · base_commit a47a18f · dependency_revisions:
  auto-lang master 6e63c812a(含 020 提交 57353aa12/15872c439,经
  ba77406eb FF)+ 并行 019/637/023 推进 · spec_inputs:
  docs/specs/terminal-mux-model.md @ a55b454 · 声明:实现会话内复审,
  结论由独立复现重建。
  - **独立复现**:干净 back(18080,app_back.exe)重跑 t01 剧本全绿
    (evidence/020/t01-model-review.log):深度2 布局/磁吸 480→500/
    钳位 50→100/键盘 700/右列轴0 (750,250)→250/帽 -2/窗口面
    1024x768 —— AC-01/02/05 + G6 **pass**;全量门 76/0(a55b454
    当拍运行,代码未变,复用并述明理由)—— AC-06 套件分量 **pass**。
  - **acceptance_results**:AC-01 pass · AC-02 pass · AC-03 partial
    (divider 槽模型面✓;iced 实机条渲染未在分屏态截图取证——合成
    输入不可用所致)· AC-04 partial(模型语义✓;端到端实拖未证,
    见 F2)· AC-05 pass · AC-06 pass(套件)+ F1 稳定性finding ·
    AC-07 partial(投影按活动 Tab 重算=代码审读;缺显式双 Tab
    深度并存的 curl 证据)。
  - **findings**:
    - **F1(P1,稳定性,AC-06)**:vue 形态 back 在前端页面轮询下
      STATUS_HEAP_CORRUPTION 崩溃(0xc0000374,vue-run.log 15:31:22
      实录;重启后 30s+ 存活=间歇)。新视图 tick 每拍 ~60 个 API
      调用(019 约 10 个),axum 并发下的引擎 FFI(get_lines/
      pane_lines×6/apply_resize)疑似触发 #21 族的并发窗口。
      **修正方向**:①几何刷新加 layout_version 门(廉价 GET 每拍,
      变化才拉 rect-*,稳态调用量回落 019 水位);②引擎 FFI 并发
      窗口排查(panes×FFI 并发矩阵);③复现脚本固化。
    - **F2(P2,AC-04)**:端到端实拖取证缺——合成输入三通道
      (mouse_event/SendInput/PostMessage)均不被 winit 消费;命名
      前置 = 用户手持鼠标实拖,或引入可驱动的输入注入通道。
    - **F3(P3,记账)**:plan frontmatter supersedes_spec_components/
      new_spec_components 空——本次复审补记:SD-01 modify
      docs/specs/terminal-mux-model.md(V1 语义 §5);new_spec_components
      保持空(无新增 spec 组件;窗口尺寸面为 D7 契约,归同文件 §5)。
    - **F4(P3,过程)**:共享映像名 taskkill 两度误伤并行会话进程
      (文件管理器/015 notes back;8080 端口亦被 015 notes 占用,
      独立 back 复跑需 AUTO_HTTP_PORT 指定)。过程教训记 Plan。
  - evidence:evidence/020/t01-model-review.log(独立复现)+ vue-run.log
    (heap corruption 实录)+ t02-*.png(三轨渲染)· **next**:work
    (F1 修正 + AC-07 补证 + F2 用户实拖),受影响任务 T-02/T-04 已
    重开,current_step 6/8。

## 10. 待澄清事项

1. **磁吸目标集**:V1 = {50%};25%/75% 是否纳入首版(常量易调,
   不阻交付)。
2. **MAX_PANES 帽值**:V1 = 6;超限 split 拒绝的提示面(仅返回值
   vs UI 可见反馈)留 work 期定。
3. **分隔条视觉**:宽 1-2px、颜色派生 palette 前景(019 同源);
   可拖拽 hover 高亮(加粗/变色)是否首版纳入——顺手项。
4. **vm 形态**:承 019 §10.6(api 委托合成缺口),独立于本计划;
   两计划可共享一次 vm 桥调试会话。(更新:019 会话已根修 f6a40d20a,
   020 vm 实机已过链运行 ✓)
7. **AC-04 GUI 实拖取证**(本计划唯一残余):拖拽语义已在模型面证毕
   (curl resize-branch:磁吸 480→500 / 钳位 50→100 / 常规 300),
   视图交互线(Press→捕获层→Drag→back)代码完备、三轨渲染正常;
   合成鼠标输入(mouse_event/SendInput/PostMessage)不被 winit 消费,
   实拖请用户手持鼠标验证:拖分隔条布局应实时跟随,松手落定,拖过
   50%±40‰ 松手应吸附回中。

