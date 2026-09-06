---
plan_id: PLAN-005
status: archived
feature_name: AutoTerm Phase 4 交互批(Rust 版功能齐全:自动滚动 + 块选 + 右键菜单 + 选中色 + 光标形状/闪烁)
author: [zhaopuming]
created_at: 2026-09-05T20:30:00+08:00
updated_at: 2026-09-06T11:30:00+08:00

# Leave these EMPTY here — /auto-plan:review fills them:
supersedes_spec_components:
  - "docs/designs/001-phase1-architecture.md: 004→005 决策链追加——004 选中架构(P004-3/4)由本计划扩展出块选/自动滚动/上下文菜单/选中色/光标形状五件套"
new_spec_components:
  - "docs/designs/evidence/005-interaction/: Phase 4 交互批取证(块选几何 block_aligned、自动滚动绝对行锚定、菜单浮层开合、选中色混色、三形光标、闪烁四重门控+零唤醒回归、5 分钟手工清单)与 4 个可复用扫描脚本"
  - "crates/autoterm-ui/src/lib.rs: SelectMsg.at_edge/App drag_scroll/menu/focused/cursor_blink_on 状态面 + 条件订阅(50ms 自动滚动、500ms 闪烁)+ parse_hex_color(a2r 可表达)"
  - "crates/autoterm-ui/src/widget.rs: begin_selection_type/edge_band/menu_item_at 纯函数 + is_block 列带/菜单浮层/光标形状分形渲染"
touched_goals:
  - "goal-1: 拖选闭环做全——块选(Alt+拖选恒 Block 与多击正交)+ 视口边缘自动滚动(50ms×2 行)+ 块选逐行截断复制,像素与转储双证据"
  - "goal-2: 交互完备——右键上下文菜单(复制/粘贴/全选)取代直接粘贴,动作与 004 快捷键同路径,ESC/外点关闭"
  - "goal-3: 观感可配与终端保真——--selection-color RRGGBB[AA](默认 e8e8e8@25% 不变)、DECSCUSR 三形分形渲染、聚焦 500ms 闪烁(失焦/隐藏/菜单/preedit 不闪,空闲零唤醒 frames=5@11s)"
  - "goal-4: 台账——DEBTS Phase 4 批次 1-4 勾销,'Rust 功能齐全'达成、#8 Auto 化启动条件就位;README 交互表与 001 决策链更新"

current_step: 11
total_steps: 11
---

# [PLAN-005] AutoTerm Phase 4 交互批(Rust 版功能齐全:自动滚动 + 块选 + 右键菜单 + 选中色 + 光标形状/闪烁)

## 变更摘要

Rust 参考实现的"功能齐全"批(总排序裁定 2026-09-05:先做齐做稳
Rust,Auto 复刻后置):①**拖选到边缘自动滚动**(004 裁定#4 归组项,
至此 004 的 Phase 4 欠账启动清偿);②**块选(Block)**——core 公面
现成(`SelectionType::Block`/`to_range`/`selection_to_string` 均有
block 分支),补 UI 触发与矩形渲染;③**右键上下文菜单**
(复制/粘贴/全选);④**选中色可配置**(CLI `--selection-color`,
去 25% α 硬编码);⑤**光标形状**(DECSCUSR:Block/Underline/Beam,
SgrCursorShape 随快照已暴露)+ **闪烁**(聚焦时条件定时,保住
"空闲零唤醒")。约束:全部新代码保持 **a2r 可表达形态**(DEBTS #8)。

## 目标

1. **拖选自动滚动**:按住拖选、光标越过视口上下边缘时按拍滚动
   (方向、速度受控);松开即停;滚动中选中随内容锚定(绝对行
   语义,004 已有);
2. **块选**:Alt+左键拖选(待澄清#1)触发矩形选中,高亮按列带
   渲染,复制走 `selection_to_string` 的 block 分支(每行截断拼
   接);
3. **右键上下文菜单**:右键弹浮层菜单(复制/粘贴/全选,待澄清
   #4),项动作复用 004 的 copy/paste 消息路径;菜单外点击/ESC
   关闭;**右键原"直接粘贴"行为被菜单取代**;
4. **选中色配置**:`--selection-color RRGGBB[AA]` CLI 参数(默认
   维持现值 e8e8e8@25%),经 App 传入 TermGrid;
5. **光标形状/闪烁**:shell 经 DECSCUSR(ESC[N q)设形——Block
   (现状反色块)/Underline/Beam;闪烁默认开(待澄清#3,500ms
   相位,仅窗口聚焦且光标可见时挂条件定时,空闲仍零唤醒);
6. **决策链与台账**:001 追加 004→005 节;DEBTS #6 光标形状子项
   清账、Phase 4 批次勾销重排;README 交互表更新。

非目标:Ctrl+C 稳定版复测(债务 #7,等环境,不在本批);IME 人工
清单(用户侧动作,非本计划交付);配置文件体系(仅 CLI 参数);
Unix 基座(独立轨道);Auto 复刻(#7/#8,后置轨道)。

## 架构方案

```
自动滚动:widget 拖选中检测"越界Extend"(clamped cell 停留边缘)
  ──publish──▶ Message::Select(Extend{cell 在边缘行/列}) 循环
  App 检测连续边缘 Extend → 置 drag_scroll: Option<Dir>
  条件订阅 time::every(50ms) → Scrolled(±N) → refresh
  (core 选中锚定绝对行,滚动自然带动;Finish 清 drag_scroll)

块选:widget ButtonPressed 时按 Alt 修饰 → Begin{Block}
  高亮:is_block=true 时逐行画同一列段(col_begin..col_last 对齐)
  复制:core selection_to_string 的 Block 分支(现成)

右键菜单:右键释放 → Message::ContextMenu(Option<(格,项集)>)
  App.menu: Option<MenuState{at, hover}>;view 传 TermGrid
  draw 末层画浮层(quad 底 + 文本 + hover 高亮);再点击/ESC → 关
  菜单项动作 = publish 004 既有消息(Copy=copy_selection 同路径)

选中色:Args --selection-color → AppConfig.selection_color: Color
  → App 传入 TermGrid.selection_color → 高亮 quad 用之

光标:core cursor() 扩展返回 shape(渲染快照 cursor.shape 现成)
  Underline=底部 2px quad;Beam=左缘 2px quad;Block=现状
  闪烁:App.cursor_visible: bool + 条件订阅 time::every(500ms)
  (仅 focused && shape!=Hidden && !menu 时挂),取反驱动重绘
  焦点事件:iced window focus 事件 → App.focused(新增状态)
```

## 技术栈

同 004(iced 0.14 advanced + alacritty_terminal 0.26),零新依赖
(块选/形状/定时订阅均为现有依赖公面)。**新增纪律:所有新代码
保持 a2r 可表达形态**(DEBTS #8 约束:idiomatic、避免 Rust 特有
奇技——不引入宏魔术/生命周期体操,数据流走 plain struct +
enum message,便于将来 Auto 复刻 diff 对齐)。

## 需求分析与背景调查

> 种子:spec ledger P004-3/P004-4(选中架构与详细设计——本计划
> 是其 Phase 4 欠账的直接延续)+ DEBTS Phase 4 清单 1-4 项 +
> 001 附录(004 决策链/取证方法学)。

1. **用户总排序裁定(2026-09-05)**:Rust 功能齐全+测试完备 →
   才做 #7 调研/#8 Auto 复刻——本计划即"第一步"的主体;
2. **API 事实(004 已核实,registry 源)**:
   - `SelectionType::Block` 公开;`to_range` 有 `range_block`
     分支(矩形、去半格语义);`selection_to_string` 有 Block
     分支(逐行截断 + `\n`);`SelectionRange.is_block` 字段随
     快照暴露——**core 侧零新增,UI 触发+渲染即可**;
   - `RenderableContent.cursor.shape: CursorShape` 现成
     (`{Block, Underline, Beam, Hidden}`),DECSCUSR 由 core
     解析(`ESC[2 q` 等落 cursor_shape)——ui 侧只需透传与
     分形渲染;
   - iced 条件订阅:`Subscription::batch` 动态增删
     `time::every`(004 dev-tools 已用同机制,生产面首次引入
     条件定时——**须守住"空闲零唤醒"**:仅聚焦+光标可见/
     拖选越界时挂);
3. **004 交接面**:widget `GridInteraction`(Tree 态)已有多击
   计数与拖选标志;`pixel_to_cell` 已 clamp 边缘(自动滚动的
   "停留边缘"信号就在这里);右键现走直接 Paste(本计划改菜单);
4. **取证基建复用**:scan-select.ps1(PrintWindow+DPI+带色扫描)
   /dev-select 注入(自愈式)/dev 转储——本计划扩 `--dev-select`
   block 前缀、转储增 cursor_shape/scroll_offset 已有;
5. **风险**:闪烁/自动滚动的条件定时若泄漏(忘摘)会破空闲零
   唤醒——验收含静态断言(转储 frames 增速)。

## 详细设计

### core:块选与光标形状透传(T1)

- `term.rs` `cursor()` 扩展:返回 `(row, col, CursorShape)` 或
  新增 `cursor_shape()`(保持旧签名兼容调用点最小改);
- sim_regression 增 1 用例:feed `ESC[4 q` 后 shape==Underline
  (DECSCUSR 通路);Block 选中 2 用例(范围矩形语义 + 文本逐行
  截断)。

### ui:块选触发与渲染(T2)

- widget `ButtonPressed(Left)` 分支:mods 含 Alt → ty=Block
  (多击计数与 Alt 正交:Alt 按下时恒 Block,不参与 1/2/3 击);
- 高亮渲染:`sel.is_block` 时逐行画 `col_begin..col_last` 窄带
  (矩形对齐,不跨全行);文本层不动;
- dev-select 解析器增 `block` 前缀(现 parser 三前缀外补一)。

### ui:拖选自动滚动(T3)

- App 增 `drag_scroll: Option<i32>`(方向行数);handle_select
  的 Extend 分支检测:cell.row==0 且指针在上边缘带(由 widget
  发布 `at_edge: Option<Vertical>` 扩展字段)→ 置 drag_scroll;
  离开边缘/Finish → 清;
- 条件订阅:`drag_scroll.is_some()` 时挂 `time::every(50ms)` →
  `Message::Scrolled(delta)`(复用现滚轮回滚路径,选中随绝对行
  锚定自动跟随);菜单打开时暂停。

### ui:右键上下文菜单(T4)

- `Message::ContextMenu(Option<(usize,usize)>)`(格坐标或像素,
  存 MenuState);widget 右键释放改发 ContextMenu(原 Paste 通道
  由菜单项"粘贴"承担);
- TermGrid 增 `menu: Option<MenuState>` 字段;draw 最顶层画:
  底 quad + 边框 + 三项文本(复制/粘贴/全选)+ hover 反色;
- 交互:菜单开着时点击命中项 → publish 对应动作(Copy=
  clipboard_shortcut 同函数 / Paste=现消息 / 全选=Begin{Lines,
  视口左上}+Extend{视口右下});未命中或 ESC → 关闭;
- 命中检测在 `pixel_to_cell` 同域(菜单矩形像素判定,纯函数
  可单测)。

### ui:选中色配置(T5)

- bin 增 `--selection-color RRGGBB[AA]`(clap,默认 `e8e8e8`+
  α0.25 硬编码值移入 AppConfig);解析 hex → iced::Color 纯函数
  + 单测(6/8 位、非法回退默认);
- App → TermGrid.selection_color: Color;高亮 quad 改用之。

### ui:光标形状与闪烁(T6-T7)

- 渲染分形:Underline=格底 2px 亮 quad;Beam=格左 2px 亮 quad
  (不反色字形);Block 维持 004 反色块;
- `App.focused: bool`(iced window focus 事件接入,新增消息);
- 闪烁:`cursor_blink_on: bool`(相位),条件订阅 500ms 取反,
  仅 `focused && shape!=Hidden && menu.is_none()`;光标不可见
  帧跳过光标绘制(preedit 挂起时同样不闪);
- 转储增 `cursor_shape:`/`cursor_visible:` 行(dev 取证)。

### 取证与文档(T8-T9)

- 冒烟矩阵:块选高亮像素扫描(列带几何)/DECSCUSR 三形转储+
  Beam 像素(左缘亮线)/自动滚动注入(dev-select 边缘 Extend 模
  拟 → 转储 scroll_offset>0 且 selection_text 跨滚动区)/菜单
  开合(dev-hook 注入 ContextMenu 坐标 → 像素扫描菜单底 quad);
- 001 追加 004→005 决策链;DEBTS 勾账(Phase 4 批次 1-4 项 +
  光标形状早年账);README 交互表增块选/菜单/形状/闪烁。

## 测试设计

- 自动:core TDD(DECSCUSR 1 例 + Block 2 例);ui 单测
  (hex 解析、菜单命中矩形、Alt 修饰判定纯函数化);
- 半自动:dev-select block 注入 + 转储断言 + 像素扫描;DECSCUSR
  注入(dev-autotype `printf` 转义)+ 转储形状断言;边缘 Extend
  注入断言 scroll_offset;菜单注入像素断言;
- 手动:真鼠标 Alt 拖选/菜单手感、闪烁不刺眼(5 分钟清单);
- 空闲零唤醒回归:静止 10s 转储 frames 增速 ≈0(闪烁条件定时
  不泄漏);
- 不做:性能基准、配置文件体系、Unix。

## 验收标准

1. `cargo test --workspace` 全绿(含 core 新 3 例 + ui 新单测);
2. Alt+拖选块选:矩形高亮像素证据(列带对齐),复制文本逐行
   无拖尾(dev 注入断言);
3. 拖选越上/下边缘自动滚动,选中跨滚动区锚定正确(转储
   scroll_offset 与 selection_text 断言),松开即停;
4. 右键菜单:三项可见(像素证据),复制/粘贴/全选动作与 004
   快捷键同路径;ESC/外点击关闭;右键不再直接粘贴(README 更新);
5. `--selection-color ff0000` 冒烟:高亮带色变(像素扫描命中色
   更新),默认不传维持现色;
6. DECSCUSR 三形:转储 shape 断言 + Beam/Underline 像素证据;
   聚焦时闪烁、失焦/隐藏/菜单时停;静止 10s frames 增速≈0;
7. 双构建绿(默认/dev-tools);默认构建零 dev 面不变;
8. 001 含 004→005 决策链;DEBTS Phase 4 批次勾销;README 更新。

## 执行步骤

- [✅ 已完成] **T1** core 透传(TDD):`cursor_shape()` + sim 3 失败用例
      (DECSCUSR Underline/Block 范围/Block 文本)再实现。
      验证:`cargo test -p autoterm-core --test sim_regression`
      (14 用例绿)—— 14/14 绿(3 新例全过;Block 分支 core 现成)
- [✅ 已完成] **T2** 块选 UI:widget Alt→Begin{Block};`is_block` 列带
      渲染;dev-select 增 `block` 前缀。
      验证:`cargo build -p autoterm-ui` 绿 + dev-select block
      注入转储(selection_text 逐行断言)—— build 绿;转储
      selection_text "AABBB\nCCDDD\nEEFFF"、cells=15(3×5 矩形);
      另修 selection_cells 指标按块选几何计
- [✅ 已完成] **T3** 块选像素取证:scan-select.ps1 列带扫描断言。
      验证:evidence/005-interaction/block-*.txt 含几何结果——
      block-scan.txt:4 带全 x52-142、spread=0、block_aligned=True、
      band_cells≈5(注入列带 2..=6);新增 scan-block.ps1(连续
      run 门 + 带内 x 一致性断言)
- [✅ 已完成] **T4** 自动滚动:Extend 增 at_edge 字段;App drag_scroll
      状态 + 条件订阅 50ms;Finish/离缘清除。
      验证:dev 注入边缘 Extend → 转储 scroll_offset>0 且
      selection_text 含滚动前内容——scroll_offset=11(顶满钳制)、
      selection_text L06..L15(锚定绝对行 5=L15,滚动揭露行尽收)、
      selection_range 绝对坐标证锚定;边缘保持注入语法 `:up/:down`;
      顺手修 DevTick 退出饿死 bug(边缘模式 task 恒非空)
- [✅ 已完成] **T5** 右键菜单:ContextMenu 消息族 + MenuState + draw
      浮层 + 三项动作接 004 路径 + 命中纯函数单测。
      验证:`cargo test -p autoterm-ui` 绿 + 菜单像素扫描——
      13 单测绿(命中纯函数 TDD 抓出负 y 饱和 cast bug);扫描
      menu_panel_visible=True(bbox≈注入位 88×78 逻辑)、close 注入
      后 34px 残留 closed=True;CJK 标签走 YaHei(MONOSPACE 豆腐块
      为 004 已知);右键直接粘贴已由菜单取代
- [✅ 已完成] **T6** 选中色:--selection_color 参数 + hex 解析单测 +
      TermGrid.selection_color 贯通。
      验证:单测绿 + `--dev-select`+新色冒烟像素命中色变更——
      16 单测绿(含 hex 3 例);ff0000 冒烟 avg_red_rgb=76.1,15.1,18.1
      (恰为 25% 混色预期)selcolor_changed=True;默认色路径由 T3
      灰扫描旁证不变
- [✅ 已完成] **T7** 光标形状:cursor_shape 分形渲染(Underline/Beam
      quad)+ 转储 cursor_shape 行。
      验证:dev-autotype DECSCUSR 注入 → 转储三形断言 + Beam
      像素证据——三形扫描全 True(underline 19×4 底条/beam 4×38
      左条/block 19×38 满格),转储 shape 三例断言;注意点:
      DECSCUSR 须经 shell 输出(键入字节被 PSReadLine 吃掉)
- [✅ 已完成] **T8** 光标闪烁:focused 状态接入 + 500ms 条件订阅 +
      可见性门控。
      验证:聚焦闪烁(转储 cursor_visible 翻转) + 静止 10s
      frames 增速≈0(零唤醒回归)——16 单测绿;blink_toggles:
      失焦 0 / 强制聚焦 ~2s 内 4 次(2Hz 翻转)、失焦回稳常亮;
      frames=5@11s 静止(基线 116@有内容,字节路径零唤醒);
      相位 off 帧跳过光标绘制(cursor_drawn=none 同源)。
      初值 focused=false(后台起窗不闪,真聚焦才挂定时)
- [✅ 已完成] **T9** 证据归档:evidence/005-interaction/(README 索引 +
      各 .png/.txt)。
      验证:归档文件存在且含像素扫描结果——31 个文件
      (4 扫描脚本 + 7 png + 12 txt/转储 + README);README 含
      结论、手工 5 分钟清单、方法学发现(PSReadLine 吞键入
      转义等 4 条)
- [✅ 已完成] **T10** 文档:001 追加 004→005 决策链;DEBTS Phase 4 批次
      勾销与重排;README 交互表(块选/菜单/形状/闪烁/选中色)。
      验证:`grep -c "004→005" docs/designs/001-phase1-architecture.md`
      ≥1——=1;DEBTS 1-4 勾销+启动条件达成注记+账本 #10 更新;
      README 交互表 Phase 3/4 合并(右键粘贴→菜单等全部更新)
- [✅ 已完成] **T11** 收尾:全量回归 + 双构建 + 无 ash + 默认构建零
      dev 面。
      验证:`cargo test --workspace` 绿 + `! cargo tree
      --workspace | grep -q ash-core` + 双构建绿 + 默认构建
      拒收 --dev-select——workspace 15 组全绿(31 用例:core
      sim 14 + ui lib 16 + live_pty/pty_lifecycle 2);ash-core
      0 命中;默认/dev-tools 双构建绿;默认构建 --dev-select →
      exit 2 "unexpected argument"(零 dev 面)

## 复审记录

**复审人**:ZCode(/auto-plan:review)· **时间**:2026-09-06 ·
**基准**:worktree `plan-005-dev`,diff 7cd32a7..HEAD(39 文件
+2436/-102,10 提交 T1-T10)。

### 逐条验收(重验,不信勾选框)

| # | 标准 | 判定 | 证据 |
| --- | --- | --- | --- |
| 1 | `cargo test --workspace` 全绿 | **pass** | 复审重跑:15 组 result-ok、37 用例 0 失败(含 sim 14、ui lib 16、live_pty 15s 实 PTY);core 新 3 例 + ui 新单测(16)在内 |
| 2 | 块选像素 + 逐行复制 | **pass** | block-scan.txt:4 带 x52-142、spread=0、block_aligned=True ×4;block-dump.txt:selection_text "AABBB\nCCDDD\nEEFFF"、cells=15(3×5) |
| 3 | 边缘自动滚动 + 锚定 + 松开即停 | **pass** | autoscroll-dump.txt:scroll_offset=11(顶满钳制)、selection_range (-4,0)-(5,4) 绝对坐标、text L06..L15(锚定+滚动揭露);松开即停=Finish/离缘清 drag_scroll(lib.rs:773/784,代码核) |
| 4 | 菜单三项 + 同路径动作 + 关闭 + 右键不粘贴 | **pass**(注 a) | menu-scan.txt:panel_visible=True、close 后 34px 残留;动作 Copy=copy_selection/Paste=paste_from_clipboard/全选=视口 Lines(lib.rs:707);右键 release 改发 ContextMenu(widget.rs:316);README 表更新 |
| 5 | --selection-color 冒烟 + 默认不变 | **pass** | selcolor-scan.txt:avg_red_rgb=76.1,15.1,18.1=理论混色、changed=True;默认 e8e8e8@25% 即 004/T3 已证灰(71,73,76)路径不变 |
| 6 | 三形转储+像素;闪烁门控;零唤醒 | **pass** | cursor-{underline,beam,block}-scan.txt 全 True(19×4 / 4×38 / 19×38)+ 三 dump shape 断言;blink-focused-dump.txt:toggles=4(聚焦 ~2s,2Hz)、frames=5@11s 静止(基线 116@有内容) |
| 7 | 双构建绿 + 默认零 dev 面 | **pass** | 复审重跑:default/dev-tools 两构建 0 error;默认构建 --dev-select → exit 2 "unexpected argument" |
| 8 | 001 决策链 / DEBTS 勾销 / README | **pass** | grep "004→005"=1;DEBTS "已清偿(005 T"=4;README 交互表 3 关键词命中 |

### 遗漏/延后/workaround 猜寻

- **无遗漏**:11 任务逐项对 diff,T1-T10 子件(at_edge/订阅/门控/
  纯函数/前缀/转储行)均在;diff 无新增 TODO/FIXME/HACK。
- **无未批准延后**:非目标(Ctrl+C 复测/IME 人工清单/配置文件/
  Unix/Auto)为计划声明、裁定既定;自动化不可覆盖的真机手感
  (Alt 拖选手感/闪烁观感/菜单点击)以 5 分钟手工清单形式落于
  evidence README——属计划声明的"手动"层,非缩水。
- **待澄清 1-5**:均按计划文本预授权默认执行(Alt 手势/仅 CLI/
  闪烁默认开/三项菜单/50ms×2 不分档),无遗留用户决策。

### 债务候选(不阻塞,记录在案)

- **a**:menu-scan.txt 的 `menu_closed_by_esc` 标名不准——实际由
  dev-menu `:close`(ContextMenu(None),与外点关闭同消息路径)驱动;
  ESC 键关闭为代码核(lib.rs Key 门),无像素证据。已入手工清单;
  证据文件名不改(历史工件,README 已注明)。
- **b**:拖选中失焦可能丢失 ButtonReleased → drag_scroll 残留
  Some(50ms 订阅存活)至下次 Begin/Finish。低概率、可自愈,
  建议后续计划加 Focused(false) 时清 drag_scroll。
- **c**:菜单中文标签依赖 Microsoft YaHei UI(缺字体系统退豆腐块;
  MONOSPACE 无 CJK 为 004 已知限制)。Unix 轨道需重新评估字体栈。
- **d**:CursorShape::HollowBlock 渲染落入 Block 分支(未画空心)。
  计划未要求,观测记录。

### 判定

全部 8 项验收 pass,无阻塞债务 → **status: reviewed**,可进入
`/auto-plan:merge`。(注意:复审期间 main 已被 plan-006 并行推进,
merge 时需处理 worktree 落后 main 的合并。)

## 待澄清事项

1. **块选触发手势**:建议 **Alt+左键拖选**(Windows Terminal/
   conhost 惯例;与现有多击计数正交)。备选:Ctrl+Shift+拖
   (与复制/粘贴快捷键同域易混)。默认按建议执行;
2. **选中色配置面**:建议本批只做 **CLI `--selection-color`**
   (配置文件体系另立计划,避免本批范围膨胀)。默认按建议执行;
3. **光标闪烁默认**:建议**默认开**(聚焦时 500ms 相位;失焦/
   隐藏/菜单时停;preedit 挂起时停)。备选:默认不闪、仅形状
   (闪烁另给开关)。默认按建议执行;
4. **右键菜单项集**:建议**复制/粘贴/全选**三项(全选=视口
   Lines 全选)。备选:加"清屏"(发 ESC 命令)——清屏有状态
   交互,建议后置。默认按建议执行;
5. **自动滚动速度**:建议 50ms/拍 × 2 行(拖至上/下缘即滚,
   距缘远近不分档——分档属调优,后置)。默认按建议执行。
