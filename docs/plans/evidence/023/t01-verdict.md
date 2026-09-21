# PLAN-023 T-01 判决工件 · 交互投递死亡根因

- 日期:2026-09-19;执行:auto-plan:work(worktree lang-023,基线
  plan-022-dev e5ddf41ba)。
- 证据件:`t01-probe-output.txt`(探针矩阵全输出);探针源 =
  auto-lang worktree `crates/auto-lang/examples/p023_probe.rs`
  (划痕件,随 T-02/T-03 提交入 plan-023-dev)。
- 版本锚:iced_core-0.14.0 / iced_widget-0.14.2 / iced_runtime-0.14.0 /
  iced_test-0.14.0(crates.io registry,只读);auto-lang renderer.rs
  @ e5ddf41ba。

## 0. 判决(一句话)

**iced 上游无 bug、不需 fork。死亡 = 本仓 into_iced 映射误用
(`opaque` 捕获边界 × `build_floating_layer` spacer 几何的组合错位)
+ 测试伪影(`View::on_click` 在构建期调用闭包)双案叠加。**
模拟器现象与实车症状**同根**:都收敛到"absolute 浮层的 opaque
捕获边界 ≠ 内容矩形"这一投递语义缺陷;模拟器侧再叠加 on_click
伪影,把"底层浮层死"放大成"全灭冻结"假象。

## 1. 三轨证据表

| # | file:symbol | 观测 | 推断 |
|---|---|---|---|
| E1 | iced_core-0.14.0 `shell.rs:49-61` `capture_event/is_event_captured` | capture = `event_status=Captured` 标志;`Shell` 每事件新建 | 计划 §2.1"仅定义无调用点、复位路径未定位"**不成立**:调用点遍布 iced_widget(button/mouse_area/scrollable/text_input/opaque 等 60+ 处);复位由构造保证 |
| E2 | iced_runtime-0.14.0 `user_interface.rs:310-379` `UserInterface::update` | base 树路径 `let mut shell = Shell::new(messages)` 在 per-event 闭包内(:318);overlay 路径同(:219) | **capture 不跨事件泄漏**;计划头号疑点排除。附带:base 捕获时 `self.overlay = None`(:331-333)只影响绘制缓存,不断投递 |
| E3 | iced_widget-0.14.2 `stack.rs:249-264` `Stack::update` | 子节点**逆序**(顶层先)更新,`is_event_captured()` 即短路 | 顶层浮层认领后低层全收不到;但若无人 capture 则逐层下发 |
| E4 | iced_widget-0.14.2 `stack.rs:164-208` `Stack::layout` | 所有子层用相同 `Limits(ZERO,size)` 布局,**全部落原点 (0,0)** | Stack 无子层定位能力;偏移只能靠层内 spacer——与 E5 组合成陷阱 |
| E5 | iced_widget-0.14.2 `helpers.rs:664-686` `opaque` | content 更新后,凡 `ButtonPressed && cursor.is_over(自身 bounds)` **无条件 capture** | opaque 语义 = "我的矩形内按压归我层"(层边界);Released/滚轮不 capture |
| E6 | auto-lang `renderer.rs:2614-2670` `build_floating_layer` | 非零偏移浮层 = `column[Space(top), row[Space(left), content]]`,Shrink 根 | 层元素 bounds = **整条 spacer 行**:`(0..left+content_w)×h`——覆盖从原点起的全部区域,含下层浮层 |
| E7 | auto-lang `renderer.rs:4038/4113`(Row/Column 臂)`stk.push(opaque(abs_el))`;`:4549`(Overlay 臂)同构 | abs_el(含 spacer 几何)整体包 opaque 入栈 | **E4×E5×E6 复合 = 根因**:带偏移浮层的 opaque 边界覆盖下层区域 → 下层 ButtonPressed 被顶层吞(E5 无条件 capture,Stack 短路) |
| E8 | iced_core-0.14.0 `mouse/cursor.rs:18-24,38-44,57-68` `Cursor::Levitating` | `Levitating.position()→None→is_over()→false`;`stack.rs:266-273` 仅当顶层**交互件认领该点**才 levitate | levitate = CSS 式分层悬停语义,**不是**死亡机理(与 V2 全通一致);022 的"光标抬升级联"假设(8d56cf936)方向排除 |
| E9 | auto-lang `view.rs:1445-1450` `ViewBuilder::on_click` | `self.button_onclick = Some(_handler(()))`——**闭包在 builder 期立即调用** harvest 静态消息 | p022 测试的计数副作用只在 `into_iced()` 构建时执行一次(计数=1),此后永不执行。**"三轮冻结 A=1/B=1"是测量伪影**;022"单轮双击通过"是空洞通过(1/1 恒真) |
| E10 | 探针矩阵 `t01-probe-output.txt` V2 | **无 opaque:6 连击完美投递 msgs=[1,2,1,2,1,2]** | opaque 是死亡唯一必要成分;Stack/布局/按钮本体无罪 |
| E11 | 探针矩阵 V1/V3/V4/V5 + R1/R2 | V1(镜像 p022):A 击 [Captured,Ignored] 无消息(opaque 吞 Pressed)、B 全收;R2 双零偏移:顶层全收、底层全灭 | 与 E4-E7 推演逐跳吻合;[Captured,Ignored] 特征 = 上层 opaque 吞 Pressed 后无人认领 Released |
| E12 | 探针 R3(载具树形状:槽1 左零偏移+槽2 右 left-[300]+分隔条 8px MouseArea+0×0 捕获层) | L=[Captured,Ignored] 无消息(**左槽死**);R=[Captured,Captured] msg=2(**右槽活**);DIV msg=7/8(分隔条活) | 载具树形状在模拟器确定性复现同类死亡:底层槽位按压被上层 spacer 行 opaque 吞;当前载具空闲态捕获层 dragW=0 无害(c793bd2 常驻层不构成空闲态杀手) |
| E13 | auto-term `app/rust-workspace/app/src/main.rs`(生成载具) | 槽位 `absolute z-10 top/left w/h`(left 锚)+ 分隔条 z-20 + 拖拽捕获层 z-30(top-0 left-0 w-[dragW] h-[dragH],空闲 0×0,Press 时全窗);back `db.at rects_recompute`:50/50 横分 pane-1 x=0、pane-2 x=500 | 实车与模拟器走同一 `UserInterface::update`+Stack+opaque 路径;E7 缺陷在实车树成 | 

## 2. 单次按压全跳事件流(以 R1 左浮层点击 (150,150) 为例)

```
iced_test click() = [ButtonPressed, ButtonReleased](一次 update 批)
└─ UserInterface::update(events, cursor=(150,150))
   ├─ root.overlay() → None(无 overlay 族)→ base 树路径
   ├─ 事件1 ButtonPressed:Shell#1 新建
   │  └─ root = Stack[base, opaque(槽A), opaque(槽B spacer行)]
   │     └─ 逆序:opaque(槽B) 先
   │        ├─ content(column[Space(0),row[Space(300),B按钮]]) 更新:
   │        │  B 按钮 Pressed:cursor(150,150) not over B bounds(300..600)→ 无动作
   │        └─ opaque:is_mouse_press && (150,150) over 自身 bounds(0..600)×300
   │           → **capture_event()** ←←← 事件死于此
   │        └─ Stack::update:is_event_captured() → return(槽A/按钮A 从未见到事件)
   │  → status=[Captured]
   └─ 事件2 ButtonReleased:Shell#2 新建(capture 已复位)
      └─ 逆序:opaque(槽B):非 Press → 不 capture;B 按钮:is_pressed=false → 不 capture
         → opaque(槽A):非 Press → 不 capture;A 按钮:is_pressed=false(Pressed 被吞)→ 不发布
         → base:空 → 无人认领
   → status=[Ignored];计数闭包从未运行(且 E9:构建时已跑过一次,恒=1)
```

修复后(opaque 只包 content,spacer 行在外):同点位 opaque(槽B content) 边界
(300..600) 不含 (150,150) → 不 capture → 逐层下发 → opaque(槽A)→A 按钮
Pressed(is_pressed=true,capture)→ Released:is_pressed → publish → **A 收到**。

## 3. 归属判定(计划 §2 三岔)

- **上游 iced bug:否。** opaque/Stack/capture 语义自洽且成文
  (opaque=层边界捕获;Stack 子层共原点+逆序投递;capture 每事件
  复位)。误用在消费侧把 spacer 几何包进了 opaque 边界。
- **本仓 into_iced 映射误用:是(主刑)。** E6×E7:四消费点
  (build_floating_layer 定义 + Row 臂 + Column 臂 + Overlay 臂)
  把整层(含 spacer)交给 opaque。
- **模拟器伪影:部分是(次刑)。** 冻结轨迹 = E9 on_click 构建期
  调用 × 测试用闭包计数;投递缺陷本身模拟器与实车同根(E12/E13),
  非伪影。
- **同根性结论:同根。** 实车按压死(终端收不到点击→聚焦永切
  不过)= E7 opaque 过宽吞 Pressed;模拟器冻结 = E9 伪影叠加
  同一 E7。022 时代"移除 opaque 轨迹不变"实验因 E9 伪影致盲
  (计数与投递解耦,任何投递修复都不改变轨迹)——**022 已排除
  项第 1 条(Opaque 满幅捕获)应反转:opaque 恰是元凶,只是当时
  测不出来。**

历史方向备注(不阻塞根修):022 实测"slot1 可交互/右面板死"与
R3 模拟器方向(左死右活)相反。候选解释:①022 观察期的右面板
处于"蓝屏未渲染"时代(G5 渲染修复前),交互死与渲染死混报;
②slot 内容空实/空层过滤时序。方向差异不影响修复有效性:捕获
边界=内容矩形后,任何栈序/任何槽位布局下都不存在跨槽吞按压。
T-04 实车用 AUTO_MA_DBG 复核方向。

## 4. 修复路径(→ T-02,计划 §5.2 三岔取"本仓"支)

根修原则:**opaque 捕获边界必须=浮层内容矩形;几何 spacer 不得
进入捕获边界。**

1. `build_floating_layer`(renderer.rs:2614):把 `opaque()` 下沉到
   content 级——row.push 处改 `opaque(content)`,函数语义变为
   "返回已含捕获边界的几何包装层";
2. Row 臂(:4035-4038)/Column 臂(:4110-4113):`pos=Some` 分支
   去掉外层 `opaque(...)`(内层已含),`pos=None` 分支保持
   `opaque(abs_el)`(零偏移层无 spacer,边界=内容矩形,语义已对);
3. Overlay 臂(:4548-4549):`stack![base_el, build_floating_layer(...)]`
   去外层 opaque(同 1);
4. 捕获层(z-30 drag catcher)不受影响:其自身 bounds 即内容矩形,
   拖拽期全窗吞按是**期望语义**(Drop 后归零)。
5. 上游 fork/[patch.crates-io] 决策项:**不需要**,SD-02 撤销
   (§7 规范增量表同步)。

验证预期:R1→6 击全达(消息流 [1,2,1,2,1,2]);R3→左槽复活
(msgs 含 1×2)、右槽/分隔条不变;`p022_stack` 全绿(转正面)。

## 5. 复跑指引(第三方可复现)

```bash
# 0) 环境:auto-lang worktree plan-023-dev(基线 e5ddf41ba 可复现死亡;
#    根修提交后同命令应全绿)
cd D:/autostack/.wt/lang-023/auto-lang

# 1) 死亡基线(p022 原测试,预期 FAILED 且轨迹恒 1 = E9 伪影):
cargo test -p auto-lang --features ui-iced,iced-layout-tests \
  --lib p022_stack -- --include-ignored --nocapture
# 期望轨迹: ["r1A->1","r1B->1","r2A->1","r2B->1","r3A->1","r3B->1"]

# 2) 探针矩阵(纯 iced V 系列 + 真实路径 R 系列 + 载具形状 R3):
cargo run -p auto-lang --example p023_probe --features ui-iced,iced-layout-tests
# 输出对照 docs/plans/evidence/023/t01-probe-output.txt:
#   V2 无 opaque 全通 = 上游无罪;V1/R1 复现吞按;R3 复现载具形状左槽死。

# 3) 根因断点(可选,符号级):在
#    iced_widget-0.14.2/src/helpers.rs:682(opaque capture_event)与
#    iced_widget-0.14.2/src/stack.rs:262(Stack 短路)下断/加打印,
#    R1 左击时 capture 的调用栈含 opaque{column[Space,Row[Space,...]]}
#    即 E6×E7 实锤。
```

注意:探针 example 为划痕件(未跟踪),T-02/T-03 时随根修/回归面
提交入 plan-023-dev;提交前以本指引第 2 步命令为准。

## 6. 对计划文面的勘误登记

| 计划处 | 原文 | 判决后 |
|---|---|---|
| §0 已排除 1 | "Opaque 满幅捕获——移除 opaque 包装轨迹不变" | **反转**:实验被 E9 伪影致盲;opaque 系元凶 |
| §2.1 grep 结论 | "capture 仅定义无调用点,头号疑点" | 排除:调用点在 iced_widget 60+ 处;Shell 每事件新建即复位 |
| §0 症状"每按钮最多收到首击" | A=1/B=3 型 | 实测 A=1/B=1 双冻结(E9);B 侧投递其实每轮都在发生 |
| §7 AC-02 断言"三轮交替 A=3/B=3" | 闭包计数语义下不可达 | 转正断言改为消息流序列(详见 T-03),AC-02 语义保持"三轮无冻结" |
| 规范增量 SD-02 | 视判决增设上游补丁策略 | **撤销**(无上游归属) |
