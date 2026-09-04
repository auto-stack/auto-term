# DEBTS.md — 已知债务

> PLAN-001(建仓 + spike)收尾盘点 · 2026-09-05。
> spike 代码非正式架构,允许整体重写;以下是重写时必须还清的账。

## spike 已知债务(Phase 1 清偿)

| # | 债务 | 现状 | 清偿方向 |
| --- | --- | --- | --- |
| 1 | 整帧重拼渲染 | 每帧全网格重画,同色 run 合并 | `Term::damage()` 损伤重绘 |
| 2 | 无字形图集 | 每 run 独立 `fill_text` | glyph cache / cosmic-text 图集 |
| 3 | 固定字形度量 | Consolas advance 硬编码 + 窗口拟合;CJK 双宽未处理 | 实测字形 advance,宽字符占两格 |
| 4 | 16ms 轮询驱动 | `time::every` tick 轮询 PTY channel | reader 线程唤醒 + `Event::Wakeup` 事件驱动 |
| 5 | 无滚动回滚 UI | 回滚缓冲在仿真核心里,无 UI | `scroll_display` + 滚动条/滚轮 |
| 6 | 无光标块/选中/IME | 键盘只回写,无输入法 | 正式输入管线 |
| 7 | Ctrl+C / 关闭语义未系统验证 | spike 仅跑通 Ctrl+字母回写 | 关闭时子进程清理树、SIGINT 语义矩阵 |
| 8 | 仅 Windows 基座 | ConPTY 验证;Unix 由 portable-pty 承诺未验 | Linux/macOS 基座计划 |
| 9 | 三 probe 无统一 workspace 文档 | spike 即代码 | Phase 1 正式 crate 结构另立计划 |
| 10 | 颜色表硬编码 | 16 色/xterm256 表/默认前后景内置 | 主题系统 + NamedColor 全枚举映射(含 Dim/Bright 前景) |

## 观察(not debt,Phase 1 设计输入)

- pwsh/cmd 启动即发 DSR/DA 查询并等应答——嵌入方必须回写
  `Event::PtyWrite`(否则无提示符);
- ConPTY 翻译层吞吐 ≈100KB/s(20000 行实测),是链路瓶颈;
- ash 命令行 `$_`/`;` 触发其解析怪癖("program not found"),
  脚本走 `-File` 正常——ash 侧问题,另行反馈 auto-shell。
