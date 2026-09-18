# terminal 组件 chrome 契约(PAD/边框/圆角/底色)

> 来源:PLAN-016(SD-02,2026-09-14 复审通过;用户裁定 2026-09-14
> "上下左右各 4px")。交付锚定:auto-lang `420d45543`。实现:
> `crates/auto-lang/src/ui/terminal/iced/widget.rs` + renderer
> `View::Terminal` 臂(rust/VM 双轨唯一构造点)。

## chrome 参数

| 参数 | 值 | 约定 |
|---|---|---|
| 内容内缩 `PAD` | 4.0 逻辑 px,四周 | 文字不贴边;取代旧 1px 边框内缩 |
| 自绘边框 | 无 | 组件不画边框线,仅剩 OS/vwin 窗框一层 |
| 底角圆角 `BOTTOM_RADIUS` | 16.0 逻辑 px,仅底部两角 | 对齐虚拟窗口 `WIN_RADIUS`(PLAN-002 N5 四角全圆档);顶部归 chrome 不圆 |
| 默认前/背景 | `DEFAULT_FG` #e8e8e8 / `DEFAULT_BG` #060709 | 均为 pub 常量;renderer 侧余量容器与组件同源 |

## 几何公式(双构造点同源)

组件固定尺寸 = `cols×cell_w() + 2×PAD` × `rows×CELL_H + 2×PAD`。
两个构造点必须同公式,漂移即测试红:

1. `Terminal::new`(widget.rs);
2. renderer.rs `View::Terminal` 臂(直构)。

`cell_w()` 为首帧实测等宽 advance(回退近似 8.0);`CELL_H=16`。

## 余量底衬(浅色带防复发)

固定网格尺寸与客户区之间存在取整余量(右 ≤ 一格宽,底 ≤ 一行高)。
renderer 臂必须以 **Fill + `DEFAULT_BG` 同色容器**包裹组件(子件左上
对齐),否则余量露出根容器 `bg-background`(#090e1a,偏蓝亮),呈现为
用户可见的右/底"浅色带"。弧线机制由 pixel 金样背书;standalone 形态
弧线与同色余量融为一体,窗角圆角由 OS DWM(独立窗)与 vwin 窗框
`WIN_RADIUS`(虚拟桌面)分别收口。

## 几何随动联动

widget `layout()` 由可用空间反推 `cols/rows`(扣除 `2×PAD`),经
pending_resize 注册表 → 宿主 `apply_resize` → 引擎;退化可用空间
(<2 列)不发请求(014 护栏,见 architecture P014-2)。

## 滚动条与分隔条命中区归属规则(PLAN-022 SD-01)

> 来源:PLAN-022 T-04(2026-09-18;用户裁定承接 021 T-06/T-07——自绘
> 滚动条退役,官方滚动条换装)。实现锚定:auto-lang `plan-022-dev`
> (50a2d4ad4 / 9df7c5a5e)。实现面:iced 官方 `scrollable` 包装
> (renderer `into_iced` 的 `View::Terminal` 臂)+ back 投影 divider
> 槽(app.at z-20 薄条)。

### 归属规则

1. **滚动条命中区归 scrollable 组件**:终端滚动条的视觉/交互(轨道、
   拇指、wheel、拖拽)全部归外层 iced 官方 scrollable——命中区 =
   scrollable 视口右缘 rail 宽度(iced 官方值),不再有自绘 14px×8px
   命中带(自绘条退役:SCROLLBAR_*/ScrollbarDrag/press 链/wheel 臂)。
2. **分隔条命中 = back 投影 divider 槽矩形**:8px 视觉即命中(轴 1 竖
   条 w-[8px]/轴 0 横条 h-[8px],色 #8899aa),命中事件(mouse-area
   onmousedown→Press)归属 app 分隔条交互,磁吸/钳位在 back 落定
   (terminal-mux-model §V1.5 分隔条契约,不变)。
3. **z-order 与接缝优先级**:divider 层 z-20 > pane 槽 z-10(含滚动条
   轨道带);几何重叠时**结构操作(分隔条)优先于滚动**。滚动条不外溢
   pane 槽(scrollable 视口即槽矩形)。
4. **视觉规格**:官方 `scrollbar_style()` 现款——thumb 3px 半透明
   rgba(0.9,0.9,0.9,0.3)、透明轨道;不另立主题规格(§10.3 裁定)。

### 回归

- 组件级:`cargo test -p auto-lang --features ui-iced,iced-layout-tests
  --lib virtual_scroll`(坐标换算/读出/写臂状态机)+ `--lib terminal`
  (016 像素金样 + 014/016 契约)。
- 实机:三联合场景(拖分隔条/拖 thumb/接缝点击)用户实点(PLAN-022
  T-05)。

## 回归

`cargo test -p auto-lang --features ui-iced,iced-layout-tests --lib
terminal_pixel`(bounds=2×PAD 公式、光标/选中像素变化;金样
`test/ui/terminal_pixel/*-wgpu.png` 随视觉变更重建)。
