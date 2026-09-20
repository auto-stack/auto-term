# 终端滚动渲染契约(绝对行键行缓存 + 预取窗 + 视图锚定)

> 来源:PLAN-025 A 部(SD-01,2026-09-20)。交付锚定:auto-lang
> `bb6dd5d22`(T-01 判决仪器)+ `fd380a64f`(T-02/T-03 根修);
> auto-term `231ddac`(泵链锚回写与预取采样)。判决证据:
> `docs/plans/evidence/025/t01-verdict.md`(基线:每 notch 整窗重排
> 30/30;泵滞后空白带 3-9 行×多帧)。

## 范围

rust 轨 iced terminal widget(`auto-lang ui/terminal/iced/widget.rs`)的
滚动渲染路径。022 滚动交互契约(thumb 比例/全程可滚/滚轮跟随/右缘对齐,
`terminal-widget-chrome.md`)不受本契约影响;vm/desktop 轨滚动渲染
不在本契约内(非目标,见 PLAN-025 §1)。

## 契约

### 1. 绝对行锚(display_offset 单源不变)

- 引擎 `display_offset` 仍是滚动状态单源(022 契约);泵
  (`app/term.rs feed_snapshot_inner`,Key 侧)每拍回写
  **行窗首行绝对行号** `window_anchor = history − display_offset`
  (`terminal_set_window_anchor`;几何替换承载)。
- 绝对行 id 语义 = 不可变 scrollback:同 id 恒同内容(引擎侧
  alacritty_terminal 0.26 数据模型);scrollback 封顶裁剪时 id 集体
  位移,由 digest 门控保正确(错位即重建,不出陈旧内容)。
- **未锚回退**:泵未回写锚(vm/desktop 臂、旧 sidecar)时,widget 按
  槽位语义渲染(行为 = 022 形态),零回归。

### 2. 行缓存绝对行号键

- 行 Paragraph 缓存键 = **行 id**(锚定模式)或槽位(回退),digest
  (内容+palette)门控重建:`ROW_CACHES: key → {行 id → RowEntry}`。
- **重排预算纪律**:滚动 K notch,每拍重建数 ≤ 新暴露行数(K·notch
  行)+ 预取增量;输出流打印 1 行 → 重建 1 行。headless 断言面
  `p025_scroll_render_tests`(auto-lang,feature `iced-layout-tests`);
  实机埋点 `AUTO_MA_DBG=1` → `[P25-ROWS] key=… rebuilt=N`。
- 缓存容量护栏:条目远离可见区即逐出(≤512 条),长滚动会话无界
  增长禁止。

### 3. 预取窗(N=8)

- 泵经引擎**瞬态 scroll** 采样可见区上/下各 N 行入 `window_store`
  (绝对 id → cells+digest;`terminal_feed_window_for`),终态 offset
  恒恢复泵目标(引擎 display_offset 不因预取漂移;引擎零改动)。
- 采样钳位 = 引擎同界(`[0, history]`),id 数学与引擎状态零漂移;
  预取行只进 store,**不进** `cells` 槽位面(props 文本/cursor/
  selection/vue 文本面零变)。
- N 取值依据:泵周期 50ms 内常规滚轮(30ms/notch)最大领先 3 行,
  N=8 清零空白带;极端连滚余 1-2 行瞬态薄带由泵周期兜底。

### 4. 视图锚定内容层

- 锚定模式 draw:可见行区间由**视口矩形**(iced scrollable 传入的
  viewport)推导(非引擎 offset),行位 = 行 id × CELL_H——窗口自
  首帧胶着视图,消灭泵滞后回中跳变;行源 = `window_store` ∪ 槽位
  回退(id ∈ [anchor, anchor+rows)),两者皆缺 = 历史留白(画布底色)。
- 槽位索引语义面(cursor/selection/preedit/`pixel_to_cell`)保持
  引擎锚定位移(`window_shift`)——与改造前逐值一致。

### 5. 禁止事项(防复发)

- 禁止行缓存回到"视口槽位键"形态(滚动/输出位移全槽失效 = 整窗
  重排 = 闪屏复发;判决工件在案)。
- 禁止 pump 之外的面写 `window_anchor`/`window_store`(单写者:
  `app/term.rs` 泵;widget 只读)。
- 禁止为预取改动 `crates/autoterm-core`(AC-07:引擎 FFI 面零 diff;
  瞬态 scroll 采样走既有 `scroll`/`row_text`/`row_style` 导出)。

### 6. 泵节拍门(即时泵,T-04)

- 悬置条款(2026-09-20 一轮)曾豁免;同日二轮实机(快拖半屏空白)
  触发启用。
- 计时器 16ms;`db.tick_gate()`:满拍间隔 ≥3 拍(~48ms,与旧 50ms
  Tick 等阶)或**任一可见 Pane 滚动增量待排**(`term_scroll_pending`,
  peek 探针 `engine_scroll_pending_for` → `terminal_scroll_delta_pending`,
  不排空)即开满拍(get_lines 全量泵)。
- 预取 N 随之 8 → 24(16ms 泵周期内极端滚轮 ~11 行 + 触控板像素滚 +
  余量);滚动期泵最密 60Hz,静止回落 ~20Hz。
- 语义:视图先行量在超出预取覆盖**之前**即被满拍追平——快滚/快拖
  空白带清零(模型断言:p025 门控场景两档速率 0 行 0 帧)。
