# PLAN-025 T-01 判决工件:滚动闪屏机理(复现器 + 计数曲线)

- 日期:2026-09-20
- 载体:auto-lang worktree `D:/autostack/.wt/lang-025/auto-lang`(branch
  `plan-025-dev`,基线 master `3df7b21a2`)
- 复现器:`crates/auto-lang/src/ui/terminal/iced/p025_scroll_render_tests.rs`
  (`cargo test -p auto-lang --features ui-iced,iced-layout-tests --lib p025_scroll -- --nocapture`)
- 仪器:`widget.rs refresh_row_cache`(自 draw 内联提取,行为等价)+
  重排计数埋点(`ROW_REBUILDS` 原子累计;`AUTO_MA_DBG=1` 实机同步留痕
  `[P25-ROWS]`)。复现链三层同构:伪引擎(不可变 scrollback,line id =
  绝对行号)→ 泵步进(term.rs feed_snapshot_inner 同构:重采窗投喂 +
  offset/history 回写)→ draw 缓存步进(真码)。

## 实测曲线(80×30 视口,500 行历史,notch = 3 行)

### 判决面 1:每帧行 Paragraph 重建数(候选 1:整窗重排)

- `[P025-CURVE] baseline per-notch rebuilds: [30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30]`
  ——滚轮连续回滚 10 notch,**每 notch 整窗 30/30 行全量重排**。滚动
  → 快照窗整体位移 → 视口槽位键全失效 → 一帧内集中 shaping 全部可见行
  (文本 shaping 是帧预算大头,009 T3 基准在案)。
- `[P025-STREAM] 1 printed line → 30/30 rows rebuilt` ——输出流同构受害:
  贴底打印 1 行 = 窗整体上移 = 槽位全变 = 整窗重排(次要收益面)。
- 静止帧零重排(digest 门控有效,闪屏非静止帧抖动)。

### 判决面 2:泵滞后空白带(候选 2:视图先行/窗口等泵)

时间线模型(帧 17ms、泵周期 50ms、notch 3 行、burst 10 notch):

- `[P025-LAG] 常规滚轮(30ms/notch): baseline max_blank=3行 frames=9; N=8: 0行 0帧`
- `[P025-LAG] 快速滚轮(10ms/notch): baseline max_blank=9行 frames=4; N=8: 1行 2帧`

视图随 iced scrollable 即时位移,窗口内容等 50ms Tick 泵:常规速率即出现
1 notch(3 行)顶缘空白带且跨 9 帧反复;快速速率达 9 行。泵帧窗口回中
(位移跳变)与空白带交替 = 肉眼"闪屏"的第二成分。

## 主因配比与修复取舍

| 候选 | 证据 | 判定 |
|---|---|---|
| 1 整窗重排(槽位键) | 每 notch 30/30;打印 1 行 30/30 | **主因**:帧预算尖峰 + 全屏内容同帧换血 |
| 2 泵滞后(空白带+跳变) | 常规 3 行×9 帧;快速 9 行×4 帧 | **次因**:带状空白 + 回中跳变 |

修复三件套取舍:

1. **绝对行号键行缓存(T-03 主修)**:行缓存键从视口槽位改绝对行号
   (锚 = 行窗首行绝对号,`history − display_offset` 推导,泵回写)。
   滚动仅新暴露行重排(≤ notch 行数 + 预取增量);输出流同步受益
   (打印 1 行 = 1 行重排)。判决 1 根修。
2. **预取窗 N=8(T-03 附带)**:泵经引擎瞬态 scroll 采样可见区上方/
   下方各 N 行(引擎零改动,FFI 既有面),新暴露行大概率已排好。判决 2
   常规速率清零(模型 0 行 0 帧)。
3. **视图锚定位(T-03 附带)**:draw 窗位从"引擎 offset(泵滞后)"改
   "视图观察目标(即时)"推导——窗口自首帧胶着视图,消灭回中跳变。
4. **滚动即时泵(T-04)**:**豁免转 DEBT**。预取 N=8 后残余 = 极端
   连滚(≥5 notch/帧持续)下 1-2 行瞬态薄带(模型:快速档 1 行×2 帧),
   由 50ms 泵周期兜底;事件驱动补泵的接线面(wheel 事件→消息→同帧泵)
   收益不足以抵消 022 回声抑制状态机的扰动风险。记录于 §10 DEBT,
   实机验收若仍见薄带再启用。

## 回归面

- `p025_static_frame_zero_rebuild`:静止零重排(非滚动帧零扰动基线);
- 泵滞后模型断言 = N=8 取值依据(常规清零/快速收窄);
- 基线曲线断言(每 notch 整窗)为现行语义锁定位,T-03 落地后翻转
  为"≤ 新暴露行 + 预取增量"(AC-01 headless 门)。
