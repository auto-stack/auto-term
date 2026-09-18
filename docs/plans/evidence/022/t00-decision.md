# PLAN-022 T-00 决策工件——虚拟滚动语义定稿 + 分屏缺陷定界

> 日期 2026-09-18 · 执行会话(auto-plan:work)· 读码基线:auto-lang
> master 1f4e3e32c(worktree lang-022)、auto-term 9853198 · 载体证据:
> evidence/021/t07-field-notes.md、vm-delegation-break.log、t07-rust-vehicle.png

## 1. 官方 scrollable 现状(腿一)

- 面上只有一条官方滚动通道:`View::Scrollable`(view.rs:684)→ iced
  0.14 `scrollable` 包装(renderer.rs `build_scrollable`:2338)。自绘条
  在 terminal/iced/widget.rs 内(SCROLLBAR_W=3/HIT_W=14/MIN_THUMB=16、
  press 链 :513-590、wheel :615、metrics :1021、draw :880),与官方
  scrollable 无关联——021 缺陷簇即自绘路径。
- 官方 scrollable 已有双臂(Plan 043):offset 写入(`.offset` prop →
  pending 队列 scroll_to,要求稳定 widget id)+ on_scroll 读出
  (Viewport 六测量 → ScrollMetrics 回调)。**缺虚拟滚动**:内容高
  = child layout 高,无"虚拟总高 + 可见窗物化"面。
- 官方样式 scrollbar_style()(renderer.rs:2437):thumb 3px 半透明、
  透明轨道——自绘条同款视觉,officially 支撑 thumb 比例/全程拖拽/
  跟随/hover/theme(4579d59e3 过渡形态问题清单的收敛面)。

## 2. 虚拟滚动语义定稿(腿二;AC-01/SD-02 语义基)

**归属**:滚动状态单源 = 引擎 `display_offset`(019 契约 §V1.10 延续,
不改);scrollable 像素 offset = **视图投影缓存**,不得反向成为状态源。

**模型**(终端场景,等高网格 CELL_H=16 常量,无折行——量化无损):

- 虚拟总高 `content_h = (rows + history) × CELL_H + 2×PAD`;可见窗
  = 引擎快照 rows 行(display_offset..display_offset+rows),仅物化
  可见行——widget 照旧画快照窗,内容高为虚拟值。
- 坐标映射(终端回滚语义,文本生长反向):
  `content_y(display_offset d) = (history − d) × CELL_H`。
  d=0(贴底实时)→ 网格落内容底;d=history(最早)→ 内容顶。
- **读出**(用户 wheel/thumb 拖拽):iced scrollable 原生消费 →
  on_scroll 闭包(terminal 侧直连 core,不经 app 消息)行量化
  `d_target = history − round(y/CELL_H)`,差值 `Δ = d_target − d`
  经 `terminal_queue_scroll_delta` 回灌引擎;泵后快照即新窗。
- **写入**(引擎侧滚动:键入贴底/scheme 无关):feed 时侦测
  display_offset 变化 → 经既有 offset pending 队列写回 scrollable
  (`scroll_to` 到 snap 像素位),thumb 跟随、贴底跟手,单拍收敛。
- 量化取整以 CELL_H 吸收(像素平滑不做,V1 行对齐;thumb 行程
  连续像素,由 iced 官方背书)。

**实现落位**(T-01):auto-lang `ui/terminal/iced/` 新增虚拟滚动包装
(唯一实现),renderer `View::Terminal` 臂(:4183)与 `Terminal::new`
standalone 形态共用;016 双构造点几何契约不破坏(组件固定尺寸公式
不变,包装层承载虚拟高)。**退役**(T-02):widget.rs 自绘条全集
(SCROLLBAR_*、ScrollbarDrag、scrollbar_metrics、press 链、wheel 臂、
draw thumb)。

**021 六缺陷反转映射**:thumb 太小/未对齐右缘 → 官方 rail 贴视口右
缘+比例正确;拖拽不到最早内容 → 官方全程 travel+引擎钳位 [0,history];
滚轮半屏钳位 → 官方 wheel 全程(widget 双写 `terminal_scroll`+
`queue_scroll_delta` 的互相打架路径随自绘臂退役);thumb 不随滚轮 →
引擎 feed 钩子写回;分屏缺陷 → §3。

## 3. rust 轨分屏缺陷定界(腿三;T-03 输入)

- **分隔条未渲染——定界闭合(代码级实证)**:app.at 分隔条 =
  `mouse-area(onmousedown)` 子件;rust codegen(`ui_gen/rust.rs`
  `tag_to_view_fn`:4732)`mouse-area` 落 `_ => "col"` 通配 → 空列,
  且 `add_event_to_builder`(:4878)只认 onclick/oncontextmenu/
  onchange——**onmousedown/onmousemove/onmouseup 全部丢弃**。空列
  再被 renderer `is_empty_stack_layer`(renderer.rs:21982,Column
  空 children 空判真)判空 → 整层不入 Stack → 分隔条不可见。VM 轨
  免疫(vue.rs Plan 499 M2 有 mouse-area 专臂)。修复 = codegen 补
  `mouse-area` → `View::MouseArea` 发射 + 事件映射(落 auto-lang,
  T-03 主修);renderer 判空对**带非透明底色的空容器**放行(加固)。
- **右面板蓝屏——静态定界未尽,留实机探针**:slot2 div 挂 Terminal
  子件,不触空层跳过;`top-[Npx]` 偏移解析在位(class.rs:1658 →
  TopOffset → dynamic_abs_layer_position 消费);registry 按 key 自
  建(renderer.rs:4184)、props 直喂。候选:①Init 只拉 slot1 全矩形,
  slot2..6 全矩形依赖 Tick 的 geomChanged 门(版本/窗口变量)——若
  rust 轨 in-process 副本的版本/投影返回异常,slot2=零尺寸原点;
  ②分屏后 pane-2 引擎 spawn 链 rust 轨断供(空快照=widget 早退,
  仅剩 margin 涂色,与"蓝底非黑底"的观察吻合度中等)。T-03 先修
  codegen,再以 vehicle 实测 + 门控日志(AUTO_MA_DBG 惯例)二择定因。

## 4. 命中区归属规则语义(腿三延伸;SD-01 语义基,定稿于 T-04)

- 官方滚动条命中区归 scrollable 组件(视口右缘 rail 宽度,iced 官方
  值),不再有自绘 14px 命中带;分隔条命中 = back 投影 divider 槽
  矩形(8px 视觉=命中,z-20)。z-order:divider(20)> pane 槽(10,
  含 scrollbar 轨道)。接缝重叠 → 结构操作(分隔条)优先于滚动。
- 落档:`terminal-widget-chrome.md`(SD-01);SD-02 虚拟滚动契约落
  `terminal-mux-model.md`(§V1.10 修订,滚动语义归模型契约)。

## 5. 澄清项裁定(§10 原有五项)

1. 行高量化:等高网格假设成立(016 契约 CELL_H=16、无折行),量化
   无损——§2 语义即定稿;
2. 换装清单:本计划只换终端 pane(非目标栏明示其余面板另立);
3. 滚动条视觉:iced 官方 scrollbar_style() 现款(3px 半透明 thumb、
   透明轨道),不另立主题规格;
4. 分屏缺陷归属:见 §3(codegen 主定界 + 实机探针补 slot2);
5. 并行会话冲突面:lang-026(p026 display)/642/647 在途;本计划改
   surface = terminal/iced/widget.rs、ui_gen/rust.rs、renderer.rs
   (Terminal 臂 + build_scrollable 邻域)、terminal-mux-model/chrome
   specs——开工前对 642/647 的 diff 面复查,冲突即回合协调。
