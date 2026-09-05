# DEBTS.md — 已知债务

> PLAN-001 盘点 · PLAN-002 勾账 · PLAN-003(Phase 2 性能与输入)勾账 · 2026-09-05。

## 债务账本(PLAN-002 后状态)

| # | 债务 | PLAN-001 现状 | PLAN-002 后 | 清偿方向 |
| --- | --- | --- | --- | --- |
| 1 | 整帧重拼渲染 | 每帧全网格重画 | **已清偿(003)**:保留式画布——每行 Paragraph 缓存+digest 判异+脏行门控,末帧重建 0 | — |
| 2 | 无字形图集 | 每 run 独立 fill_text | **已清偿(003)**:每行单 Paragraph,shaping 由行缓存持有;GPU 字形缓存 iced 内置 | 图集化(wgpu atlas)留待真有性能需求时 |
| 3 | 固定字形度量 | Consolas 硬编码 + 拟合 | **已清偿**:cosmic-text 实测 cell_w(9.375px@16px),fit_ok 数值断言;宽字符 unicode-width(实测证伪 advance 法) | — |
| 4 | 16ms 轮询驱动 | time::every tick | **已清偿**:唤醒通道事件驱动(6s 59 次更新 vs 372 轮询),常态零定时器 | — |
| 5 | 无滚动回滚 UI | — | **已清偿**:滚轮/PgUp/PgDn/键入回正/↑N(offset=171 回顶实证) | 滚动条可选 |
| 6 | 无光标块/选中/IME | 键盘只回写 | 光标块已清偿(002);IME 003 调查完结:管线公面完备、验证需真人交互,**路由 Phase 3**;选中留账 | Phase 3 |
| 7 | Ctrl+C / 关闭语义 | 未验证 | 关闭已清偿(002);003 矩阵完结:raw-mode 客户端可用,**经典控制台程序需 win32 GenerateConsoleCtrlEvent(用户裁定中)** | 裁定后 ~30 行 win32 |
| 8 | 仅 Windows 基座 | 同 | 未动 | Linux/macOS 计划 |
| 9 | spike 无统一文档 | 同 | **已清偿**:crates/* 正式结构 + 001 设计文档 | — |
| 10 | 颜色表硬编码 | 16 色/xterm256 内置 | **已清偿**:palette.rs 全 NamedColor 映射(Dim×8/Bright/Dim 前景,TDD) | 主题系统可选 |

## 新增观察(PLAN-002 实测,设计输入)

- **ConPTY 无自然 EOF**:会话持活期间主端读流不结束(conhost 等
  输入端关闭)——退出检测必须走 try_wait(001 设计文档专节);
- `Scroll::Delta` 正=上翻历史(alacritty 上游符号约定);
- 字体 advance 不承载终端双格语义(本机 '中' 与 'M' advance 相等)
  ——宽字符判定用 unicode-width,勿再走字体度量;
- cosmic-text 需与 iced 同版 pin(0.15),否则双份字体系统。

## Phase 2 候选清单(优先级未定)

1. 字形图集 + 保留式画布(连带 #1/#2 的绘制级剪裁);
2. Ctrl+C/Ctrl+Break/关闭语义矩阵;IME/选中;
3. 光标形状(Underline/Beam)与闪烁;
4. Unix 基座适配;
5. iced↔auto-ui 生态对齐(`.at` 组件模型承载原生 widget 调查)。
