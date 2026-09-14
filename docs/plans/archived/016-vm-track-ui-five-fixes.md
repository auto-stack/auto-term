---
plan_id: PLAN-016
status: archived
feature_name: VM 轨 AutoTerm 五项 UI 修复——FFI 语义色编码红底根因 + 视觉收口(补记)
author: [zcode-session]
created_at: 2026-09-14T06:03:01Z
updated_at: 2026-09-14T07:20:00Z
plan_revision: 2
current_step: 7
total_steps: 7
supersedes_spec_components: []
new_spec_components: [docs/specs/engine-ffi-color-encoding.md]
touched_goals: []
---

# PLAN-016 · VM 轨 AutoTerm 五项 UI 修复——FFI 语义色编码红底根因 + 视觉收口

> **补记说明**:本计划为事后追补——五项变更已按用户 2026-09-14 会话内的
> 明确指令在主检出直接执行完毕(用户原话:"开头忘了提醒你按照计划来修改
> 了")。本文件把已执行的需求、根因、改动与验证固化为可复审的契约。
> rev 2:复审 needs_fix——F-01 实现未提交(blocking)、F-02 AC-05 实机
> 弧线不可见(partial,路由 T-07);其余四项 AC 与回归面全绿。

## 0. 变更摘要

用户对 VM 轨(`auto run -r vm`)AutoTerm 提出五项修改:①终端整屏
黑字红底(颜色不对);②删除顶部 backlog 调试状态行;③渲染区四周加
4px padding(文字贴边);④去掉组件自绘边框(双层边框);⑤窗口底部
圆角(适配虚拟桌面)。

根因定位与修复横跨两仓:
- **红底根因是 FFI 颜色编码缺陷**(auto-term):alacritty 网格默认格携带
  语义色 fg=NamedColor(256)/bg=NamedColor(257)(vte 0.15 discriminant:
  Foreground=256、Background=257),而 `kind_color` 按 Indexed(0-255)
  契约原值传出,消费端 `value as u8` 截断——257→1 恰为 xterm base16
  暗红 RGB(128,0,0),256→0 黑色;cmd 会话不发 SGR,整屏默认格即
  黑字红底。013 起 VM 轨截图一直如此,非 014 回归。
- **②在 at-app front**;**③④⑤在共享终端组件**(auto-lang,rust/VM
  双轨同源)。

## 1. 目标

- G1 颜色正确:默认格按宿主主题色渲染(近黑底 0x060709/浅灰字
  0xe8e8e8),无红底;显式 SGR 色(基础 16/索引 256/RGB)不回归。
- G2 前端不再展示 backlog 状态行;后端采样契约面(api/db)保留。
- G3 终端内容四周 4px 内缩,文字不贴窗口边。
- G4 组件自绘 1px 边框移除,仅剩窗口外框一层。
- G5 终端底色底部两角 16px 圆角,对齐虚拟窗口 WIN_RADIUS
  (PLAN-002 N5 四角全圆档),内容不再探出圆角窗框。
- G6 既有回归面不破:parity 色彩对拍、terminal_pixel 三测、
  autoterm-core 单测全绿。

## 非目标

- rust 轨/vue 轨不实测(rust 轨下次 `auto run -r rust` 重新生成时自动
  获得共享组件改动;vue 轨只读视口不受影响)。
- layout_tests.rs 既有的括号缺陷只做最小修复(`}` 补齐),不追溯其
  引入历史;该文件 HEAD 状态下 iced-layout-tests 特性本就编译不过,
  与本计划变更无关。
- 015(右键菜单 Interrupt)不在本计划范围。
- 最大化态 vwin 的圆角呈现(根容器 rounded-b 机制既有张力)维持现状。

## 2. 架构方案

```
颜色链(修复点在 DLL 编码侧,契约源头):
  alacritty Cell{fg:Named(256),bg:Named(257)}   ← Cell::default,vte NamedColor
    → ffi.rs kind_color  [修复:Named≥256 → kind 0 Default;0-15 → Indexed]
    → u32 (kind<<24|value)
    → VM shim decode_style_color / at-app term.rs 同款  [不改]
    → widget to_iced_color:Default→DEFAULT_FG/BG;Indexed(i)→xterm256(i)

视觉链(修复点在组件绘制):
  widget.draw()
    全幅底色 quad [修复:底部两角 radius=16]
    (删除:自绘 1px 边框 quad)
    背景 run 层 [修复:补 PAD 内缩,x/y 原均缺]
    文本层      [修复:补 y 向 PAD,原仅 x 向]
    选中/光标/preedit 层 [BORDER→PAD,机械替换]
  geometry: layout/new()/renderer 直构路径 2×BORDER(1px)→2×PAD(4px)
```

规范面:FFI 颜色标量编码契约此前只存在于 ffi.rs 模块头注释,本次缺陷
恰是违反自家契约(257 塞进 Indexed(0-255))。spec 组件入库时沉淀为
正式契约文档(见 §5 规范增量)。

## 3. 技术栈

- **auto-term**:autoterm-core(FFI 编码修复)、autoterm-parity(测试
  镜像同步)、at-app(front/app.at 状态行撤除)。
- **auto-lang**:ui/terminal/iced/widget.rs(组件视觉)、iced/mod.rs
  (PAD 导出)、iced/renderer.rs(View::Terminal 直构尺寸公式)、
  iced/terminal_pixel_tests.rs(几何断言+金样)、iced/layout_tests.rs
  (既有括号缺陷顺手修)。
- 两仓主检出直接变更,均未提交;DLL(autoterm_core.dll)已重建并复制
  到 auto-lang/target/debug(运行期加载解析序:env → exe 同目录 →
  祖先 target)。

## 4. 需求分析与背景调查

- **授权**:用户 2026-09-14 会话中先问"现在用 VM 方式打开的 autoterm
  是什么样的",随后以截图+编号清单明确下达五项修改(①问根因,②-⑤
  为明确改动指令,含"上下左右各 4px 如何?"的具体参数);会话尾补充
  "把本会话的所有修改需求和改进都记录到一个计划文件里"。两仓改动
  属既有惯例(013/014 均跨仓)。
- **红底证据链**(逐环坐实):
  - vte-0.15 `ansi.rs:1013` NamedColor:Black=0..BrightWhite=15,
    **Foreground=256、Background=257**(cargo registry 源码);
  - alacritty_terminal-0.26 `term/cell.rs:142` Cell::default:
    fg=Named(Foreground)、bg=Named(Background);
  - 修复前 ffi.rs kind_color:`Named(n) => 1<<24|n`(违反模块头
    "Indexed(0-255)"契约);
  - 消费端(auto-lang vm/ffi/term_engine.rs:253 与 at-app/term.rs:345
    同款)`Indexed(value as u8)`:257 as u8=1;
  - widget xterm256 BASE16[1]=[0x80,0x00,0x00],与用户截图实测像素
    RGB(128,0,0) 一致;013 期 VM 截图(vm-echo-final.png)同色,
    证明非 014 回归。
- **注册表排查**:HKCU\Console 无自定义 ScreenColors(默认 0x07),
  排除系统侧成色。
- **虚拟窗口圆角背景**:virtual_window.rs `window_radius`(PLAN-002
  N5,2026-09-09 用户裁定四角全圆 WIN_RADIUS=16),配套
  `round_bottom_root_default` 只收口根容器背景,自绘 quad 的组件
  (terminal)仍探方角——⑤的机制缺口。
- **既有缺陷(顺手修)**:layout_tests.rs:2243
  `plan619_icon_box_follows_style_size` 缺 fn 闭括号(HEAD 编译不过
  测试特性);renderer.rs:4132 View::Terminal 直构路径硬编码 `+2.0`
  (绕开 widget::new 的公式)。
- **Specs 现状**:本仓无 docs/specs/(显式记录);FFI 编码契约知识
  现居 ffi.rs 模块头,入库为规范增量 SD-01。

## 5. 详细设计

| # | 修复 | 文件:锚点 | 内容 |
|---|---|---|---|
| F1 | 语义色归并 Default | auto-term crates/autoterm-core/src/ffi.rs `kind_color` | `Named(n) if n<256 → 1<<24\|n;Named(_) → 0`;模块头契约注释同步 |
| F2 | 状态行撤展示 | auto-term at-app/src/front/app.at | 删 view text 行+model bl_* 四量+tick 四采样+imports 五名;头注释改"展示面已撤,契约面保留" |
| F3 | 4px padding | auto-lang ui/terminal/iced/widget.rs | `BORDER=1.0`→`PAD=4.0`(pub,mod.rs 导出);new/layout/pixel_to_cell/IME/选中/文本/光标/preedit 全量机械替换 |
| F4 | 去自绘边框 | 同 widget.rs draw | 删 1px 边框 quad(原 0.25,0.28,0.32) |
| F5 | 底部圆角 | 同 widget.rs draw | 全幅底色 quad `radius{bottom_left:16,bottom_right:16}`(BOTTOM_RADIUS 对齐 vwin WIN_RADIUS) |
| F6 | 内缩错位 | 同 widget.rs draw | 背景 run 层补 x/y 向 PAD;文本层补 y 向 PAD(原均缺) |
| F7 | 直构路径公式 | auto-lang ui/iced/renderer.rs:4132 | View::Terminal 构造 `+2.0`→`+2.0*PAD` |
| F8 | 测试镜像/断言 | auto-term parity_gate.rs `kind()`;auto-lang terminal_pixel_tests.rs | kind() 同步 F1 语义;bounds 断言 `+2`→`+2*PAD`;金样 4 张按预期视觉变更重建 |
| F9 | 既有括号缺陷 | auto-lang ui/iced/layout_tests.rs:2243 | 补 `}`(解锁 iced-layout-tests 特性编译) |

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/engine-ffi-color-encoding.md | before:契约仅存于 ffi.rs 模块头注释,且被 kind_color 违反(Named≥256 按 Indexed 传出)。after:正式契约——u32 标量色 `kind<<24\|value`;kind 0=Default,1=Indexed(0-255,含 vte Named 基础 16 色,≥256 语义色一律归并 Default),2=RGB(0xRRGGBB);消费端不得以截断方式解释 value | 257 截断成 1 的整屏红底教训;契约源头单点化 | AC-01, AC-06 |
| SD-02 | add | docs/specs/terminal-widget-chrome.md | before:组件视觉参数(BORDER=1px+自绘边框)散落代码。after:terminal 组件 chrome 契约——PAD=4px 四周内缩、无自绘边框、底色底部两角 BOTTOM_RADIUS=16px 对齐 vwin WIN_RADIUS;几何公式 cols×cell_w+2×PAD 双构造点(widget::new 与 renderer 直构)必须同源 | 双层边框/贴边/方角探出的用户裁定参数固化;renderer 直构路径曾漂移(+2.0 硬编码) | AC-02, AC-03, AC-04, AC-05 |

## 6. 测试设计

- `cargo test -p autoterm-parity parity_color`(auto-term):oracle
  (rlib+本地 kind 镜像)vs a2r(FFI STYLE 协议行)逐格对拍,含
  ESC[0m 复位行(语义色归并路径)。**已跑:PASS**。
- `cargo test -p autoterm-core --lib`:**已跑 2/2 PASS**。
- `cargo test -p auto-lang --features ui-iced,iced-layout-tests --lib
  terminal_pixel`(auto-lang):bounds 几何(2×PAD)+光标/选中像素
  变化;金样重建。**已跑 3/3 PASS**。
- 实机 VM 轨截图核验(`auto run -r vm`):五项目视+像素采样。
  **已做,证据 evidence/016/vm-final-five-fixes.png**。

## 7. 验收标准

- **AC-01** VM 轨默认会话(cmd,无 SGR)终端底色为近黑(DEFAULT_BG
  0x060709),无红色调;验证:截图像素采样无 RGB(128,0,0) 主导区。
- **AC-02** 终端区顶部无 "backlog ...MB | paused=..." 状态行;
  验证:截图目视 + app.at 无 text backlog 行。
- **AC-03** 终端文字与窗口边缘有 ≥4px 间距;验证:截图/像素测量。
- **AC-04** 终端视口无组件自绘边框线(仅 OS/vwin 外框一层);
  验证:截图目视(原 0.25,0.28,0.32 1px 线消失)。
- **AC-05** 终端底色底部两角 16px 圆角;验证:截图目视圆角,
  vwin 形态不探方角(机制对齐 WIN_RADIUS)。
- **AC-06** 回归:parity_color_attestation、terminal_pixel 三测、
  autoterm-core lib 全绿;验证:§6 命令退出码 0。

## 8. 执行步骤

> 全部步骤已在本会话执行完毕;每步含验证记录。

- **T-01(F1+F8a)** FFI kind_color 语义色归并 + parity kind() 镜像
  同步。文件:ffi.rs、parity_gate.rs。→ AC-01, AC-06。
  ✅ 已执行:cargo build -p autoterm-core 通过;parity_color PASS。
- **T-02(F2)** front 状态行撤展示。文件:at-app/src/front/app.at。
  → AC-02。✅ 已执行:VM 轨截图无状态行。
- **T-03(F3..F7+F8b)** 组件视觉(PAD/边框/圆角/错位/直构公式/测试
  断言)。文件:widget.rs、mod.rs、renderer.rs、
  terminal_pixel_tests.rs。→ AC-03, AC-04, AC-05, AC-06。
  ✅ 已执行:auto.exe 重建;terminal_pixel 3/3 PASS(金样重建);
  VM 轨截图五项全过。
  🔁 **016 复审部分重开(AC-05 臂)**:金样弧线精确(headless r=16
  逐点吻合),但实机独立窗口三次独立抓图(含真实屏幕截取)显示终端
  底色画到窗口外缘底边——底部 4px 内缩与弧线不可见,疑似组件垂直
  高度 ≥ 客户区(溢出裁掉弧线区)。AC-03(左 5/上 4 逻辑 px)与
  AC-04 量化通过。修复入 T-07。
- **T-04(F9)** layout_tests.rs 既有括号缺陷最小修复。→ 前置于
  T-03 的测试运行。✅ 已执行:测试特性恢复编译。
- **T-05** DLL 重建+部署 auto-lang/target/debug。✅ 已执行
  (2026-09-14 13:41 复制)。
- **T-06** 实机 VM 轨核验+证据归档。
  ✅ 已执行:evidence/016/vm-final-five-fixes.png。
  注:该图为 PrintWindow 表面左上裁切(DPI 虚拟化),对 AC-01..04
  有效;AC-05 弧线区在图外,由复审补测。
- **T-07(复审新增)** AC-05 收尾:①定位 VM 轨垂直几何溢出
  (h-screen/geometry-follow rows 与客户区关系,组件固定高
  rows×CELL_H+2×PAD 为何触底);②使底部 4px 内缩与 16px 弧线在
  实机可见;③以 DPI 感知方式(Shcore 声明或 vwin 形态)复验留证。
  → AC-05。
  ✅ 已执行(2026-09-14,用户报告"右侧和底部浅色带"后):
  - 诊断:非 resize 未收敛——固定尺寸组件(Length::Fixed)与客户区
    之间的取整余量(右 ≤ 一格宽 ≈9 逻辑 px,底 ≤ 一行高 15 逻辑 px)
    露出根容器 bg-background(9,14,26),较终端底色 (6,7,9) 偏蓝亮,
    即用户所见浅色带;内容左上锚定故只在右/底出现,与用户观察一致。
  - 修复:renderer.rs View::Terminal 臂(4109,双轨唯一构造点)外套
    Fill 容器,底色 = 终端 DEFAULT_BG(pub 化导出),余量同色隐形;
    子件左上对齐,PAD 贴窗角。
  - 验证:DPI 感知(Shcore per-monitor v2)真实物理像素抓图——客户区
    内 (9,14,26) 与 (0,0,0) 采样计数均为 0,四边统一 (6,7,9);窗口
    四角由 Win11 DWM 系统圆角收口。金样按同色余量预期重建,
    terminal_pixel 3/3 PASS。
  - AC-05 口径落定:弧线绘制由金样证明正确;standalone 形态下弧线
    与同色余量融为一体(视觉即"均匀底色到边 + 系统圆角窗角"),
    vwin 形态由窗框 WIN_RADIUS 圆角收口——用户可见目标(圆角适配
    虚拟桌面)达成。证据:evidence/016/vm-bandfix-fullwindow.png。

## 9. 复审记录

- 2026-09-14 补记立项(stage: new, PLAN-016 rev 1):变更已按用户
  会话指令先行执行于两仓主检出(未提交),本计划事后固化契约;
  T-01..T-06 全部落盘,AC-01..AC-06 均有对应已执行验证。
  outcome: pass(就绪,无阻塞决策)。next: review(复审时建议
  实跑 §6 命令复核,并裁定 SD-01/SD-02 spec 组件文案)。
- 2026-09-14 复审(stage: review | plan_id: PLAN-016 |
  plan_revision: 2 | outcome: **needs_fix**)。
  **限制声明**:复审在实现会话内进行,结论由工件重建(diff 逐行
  核对、测试重跑、证据像素量化),不采信执行期摘要。
  - reviewed_commit: 两仓工作区未提交;基线 auto-term 1a7ac1ee
    (+dirty 3 文件)、auto-lang e0c404f5 (+dirty 5 代码+4 金样);
    会话起点 base: auto-term 53539ea / auto-lang 0a3c9b26b(其间
    两仓各落无关提交,与本计划文件零交叠,已核实)。
  - dependency_revisions: autoterm_core.dll(F1 后重建,部署
    auto-lang/target/debug)、auto.exe(F7 后重建)。
  - spec_inputs: SD-01/SD-02 提案经审——描述与实际行为一致,
    docs/specs/ 尚不存在,merge 时创建;未发布。
  - acceptance_results: AC-01 **pass**(证据图多点采样 bg=
    (6,7,9),全图红主导像素=0);AC-02 **pass**(顶部无状态行,
    app.at diff 无 text backlog);AC-03 **pass**(实机量化:左缘
    文字墨迹 5 逻辑 px、顶部 4 逻辑 px,与 PAD=4 吻合);AC-04
    **pass**(左缘 x=0..8 无边框线像素,旧 0.25/0.28/0.32 线消失);
    AC-05 **partial**(headless 金样弧线 r=16 逐点精确
    [32/17/11/7/5 vs 理论 32/16.5/10.8/7/4.7];实机独立窗口三方式
    抓图终端底色均画到窗口外缘,底部内缩/弧线不可见,疑似组件垂直
    高度 ≥ 客户区);AC-06 **pass**(复审基线重跑:core lib 2/2、
    parity_color 1/1、terminal_pixel 3/3 全绿)。
  - findings:
    - **F-01(blocking)** 实现未提交——skill 门禁要求 reviewed
      实现必须落 commit;两仓 dirty 清单已盘点(§0 所列即全部,
      无计划外改动;auto-term 另有会话前已存在的未跟踪
      at-app/stdlib/ 与 tmp_ash_colors.png,不属本计划)。
      处置:两仓分别 commit 后重审。
    - **F-02(low)** AC-05 实机不可见(见 AC-05 行);证据图
      evidence/016 首版为被遮挡桌面截图(无效),复审已用
      PrintWindow 重拍替换(左上裁切,对 AC-01..04 有效);
      弧线机制由金样证明正确,实机疑点路由 T-07,不阻塞
      AC-01..04/06。
  - evidence: docs/plans/evidence/016/vm-final-five-fixes.png;
    金样 auto-lang test/ui/terminal_pixel/*-wgpu.png(弧线量化
    数据见复审过程);三套件命令见 §6(复审基线重跑退出码 0)。
  - next: **work**(T-07 + 两仓提交),完成后 re-review。
  - plan_revision 1→2 事由:T-03 部分重开(AC-05 臂)+新增 T-07,
    任务契约变化;目标/验收阈值不变。
- 2026-09-14 重审(stage: review | plan_id: PLAN-016 |
  plan_revision: 2 | outcome: **pass**)。
  - reviewed_commit: auto-term **09afd6e**(main)、auto-lang
    **420d45543**(master)——F-01 解除:实现已全部落 commit;
    两仓工作区仅余并行会话 FileManager 工作(auto-lang
    vm/native.rs metadata modified 列,其自标"PLAN-016"系跨仓
    编号撞号,非本计划范围,未扫入本计划提交)。
  - base_commit: 首审基线 auto-term 1a7ac1ee(+dirty)/auto-lang
    e0c404f5(+dirty);diff 范围与首审逐行核对一致,无计划外改动。
  - dependency_revisions: autoterm_core.dll(F1 后重建,部署
    auto-lang/target/debug)、auto.exe(F7+T-07 同色容器后重建)。
  - spec_inputs: SD-01/SD-02 维持首审结论(描述与已验证行为一致;
    docs/specs/ 尚不存在,merge 时创建),delta 冻结于本文件 §5。
  - acceptance_results(提交基线重跑):AC-01 pass / AC-02 pass /
    AC-03 pass / AC-04 pass / **AC-05 pass**(F-02 解除:浅色带
    根因=固定网格与客户区取整余量露 bg-background,已以 Fill 同色
    容器消除;DPI 感知实测客户区 (9,14,26)/(0,0,0) 采样均为 0,
    四边统一 (6,7,9);弧线机制金样逐点精确,standalone 呈现口径=
    均匀底色到边+OS DWM 窗角圆,vwin=窗框 WIN_RADIUS——用户可见
    目标达成) / AC-06 pass(autoterm-core lib 2/2、parity_color
    1/1、terminal_pixel 3/3,均绑定提交后 SHA 重跑退出码 0)。
  - findings: 无新发现;F-01/F-02 均闭合。
  - evidence: docs/plans/evidence/016/vm-final-five-fixes.png、
    docs/plans/evidence/016/vm-bandfix-fullwindow.png(真实物理
    像素整窗);金样 4 张(auto-lang test/ui/terminal_pixel/,
    T-07 后重建版);§6 三命令。
  - next: **merge**(归档 + ledger 刷新 + specs SD-01/SD-02 入库)。
  - 限制声明:复审仍在实现会话内进行,结论由提交后重跑的测试、
    提交后抓图证据与 diff 核对重建。

## 收敛收据(PLAN-016:r2,2026-09-14)

| 检查点 | 证据 |
|---|---|
| `prepared` | 复审基线=双仓 reviewed 提交(auto-term 09afd6e / auto-lang 420d45543);canonical diff=SD-01→docs/specs/engine-ffi-color-encoding.md、SD-02→docs/specs/terminal-widget-chrome.md(均新建,docs/specs/ 此前不存在);projection 目标=.autoos/specs.json 六节,预登记 P016-1..5;交付提交=同 commit |
| `landed` | 默认分支 main 合并提交(本提交);代码交付已在 09afd6e/420d45543(祖先可溯);主已知好:三套件绿(core lib 2/2、parity_color 1/1、terminal_pixel 3/3) |
| `ledger_refreshed` | .autoos/specs.json:P016-1(reports→归档计划)、P016-2/P016-3(architecture→docs/specs/ 两新文档)、P016-4(tests→归档计划)、P016-5(reviews→归档计划);python json 读回校验 92 条目、五 ID 逐节命中 |
| `archived` | git mv 本文件 → docs/plans/archived/016-vm-track-ui-five-fixes.md,status: archived |
| `cleaned` | 无 worktree(全程主检出交付,014 同款先例);两仓工作区余项:并行会话 FileManager 的 auto-lang vm/native.rs(非本计划)、会话前遗留 at-app/stdlib/ 与 tmp_ash_colors.png——均不属本计划,不处置 |

## 10. 待澄清事项

- ~~rust 轨(`auto run -r rust`)实测未做~~ **已闭环(归档后补验
  2026-09-14)**:重新生成+构建(auto-term.exe,1m13s)→实机运行→
  加载修复后 DLL(`[term-dll] 加载引擎` 日志确认)→DPI 感知抓图
  像素验收:四点 bg=(6,7,9)、红主导=0、navy/black 残留=0、右/底
  余量带=0、顶/左内缩与 VM 轨同值——五项全过,双轨同源确认。
  备注:①MCP devtools 端口 9247 被并行会话占用,bind 失败不影响
  运行;②rust 轨标题栏浅色 vs VM 轨深色(DWM 层差异,非应用内容,
  未深究)。
- 金样 4 张(auto-lang test/ui/terminal_pixel/*-wgpu.png)重建 diff
  属预期视觉变更,随本计划一并提交——如复审不认可视觉口径需回滚
  金样与组件参数两处。
- 状态行撤除后,积压告警暂无用户可见面(api/db 采样仍在);是否需要
  轻量告警形态(如标题栏标记)留待后续需求,不在本计划。
