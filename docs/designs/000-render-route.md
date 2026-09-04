# 000 · 渲染路线决策:iced 一等应用(方案 A)vs winit/wgpu 独立原生(方案 B)

> PLAN-001 产出 · 2026-09-05 · Windows 基座(Win11 26200,ConPTY)
> 证据目录:`docs/designs/evidence/001-smoke/`

## 结论

**方案 A(iced 自定义 widget 一等应用)成立**,进入 Phase 1 正式实现。
方案 B(winit+wgpu 独立自绘)保留为逃生口,不启用。

理由:A 判据四项全部满足(见核对表),且未触发任何 B 条件;iced 0.14
自定义 widget 能拿到完整每帧绘制权,spike 以"最贵"的整帧重拼(无字形
图集、无损伤重绘、16ms 轮询驱动)仍维持 62fps,说明性能余量足够——
正式实现只需做减法(损伤重绘、字形图集、事件驱动),没有需要 fork
上游才能解决的结构性冲突。

## 判据核对(A 判据 → 证据)

| 判据 | 结果 | 证据 |
| --- | --- | --- |
| 网格整帧渲染延迟肉眼可接受 | ✅ | 130KB 突发(20000 行)下 25s 内 1562 帧 ≈62fps 持平垂直同步;滚动单调递增无糊屏(bench-pwsh/ash-20000.txt 快照序列) |
| resize 无可感知撕裂 | ✅ | 两次 resize(首帧校正 113x32、程序化 760x500)无崩溃无撕裂;PrintWindow 截图边框/右缘完整(render-find-colors.png) |
| 输入回显无粘滞 | ✅ | `echo hi` 键入→回显 0.154s;ash Tab 补全/表格/流式输出全部正常交互 |
| 自定义 widget 无结构性障碍(每帧绘制权) | ✅ | `Widget::draw` 内 `fill_quad`+`fill_text` 按格/按 run 绘制,网格、色块、文本全自主 |

B 触发条件核查:无一项成立——不需要 fork iced;帧调度(time::every 16ms)
与文本管线(Shaping::Basic 单色 run)均按需求工作。

## 三 probe 证据

### pty-probe(宿主 PTY 申请 + VT 字节流)

- portable-pty 0.9 `native_pty_system().openpty()` 在 Windows 基座
  直接拿到 ConPTY;spawn pwsh → 主端 3s 读到 27B,hexdump 首行即
  `1b 5b 31 74`(ESC[1t),字节率非零;
- **关键发现:pwsh 启动即发 VT 查询**(`ESC[6n` 光标报告、`ESC[c` DA1、
  `ESC[?9001h` win32-input 等)并**阻塞等待应答**——嵌入方必须回写
  应答,否则 shell 永不画提示符。这不是 bug,是 DSR/DA 协议义务。

### term-probe(alacritty_terminal 0.26 嵌入,headless)

- 喂入路径:`vte::ansi::Processor::advance(&mut Term, bytes)`——
  `Term` 自身实现 `Handler`,无需自写状态机;
- `EventListener::send_event` 是唯一事件出口,其中 `Event::PtyWrite`
  即 DSR/DA 应答:回写 11B 后 pwsh 提示符 1.51s 出现在网格;
- `renderable_content().display_iter` 按行主序带坐标遍历可见区,
  快照免切行;`Term::resize(Dimensions)` resize 语义清晰;
- 集成测试(真 PTY→feed→网格断言 `hello_term_probe`)绿,
  是正式工程的回归种子。

### render-probe(iced 0.14 窗口渲染 + 键盘回写)

- 自定义 widget 整帧重拼网格(同色 run 合并 fill_text),键盘
  可打印/Enter/Backspace/方向键/Ctrl+字母 → PTY 主端;窗口 resize
  → cols/rows 换算后同通知仿真核心与 ConPTY;
- ash 四项冒烟全过(见附录):彩色表格/动态流式/长输出折叠均正常;
- 发现并修复:固定字形度量误差导致最右列被裁——draw 侧改为按窗口
  实际宽高反解 cell/font 尺寸(网格拟合)后消除;Phase 1 应实测
  字形 advance(cosmic-text)替代拟合。

## 大输出粗测数字(20000 行,~130KB)

| 指标 | pwsh 直跑 | ash → pwsh | 解读 |
| --- | --- | --- | --- |
| 字节量(bytes_in) | 130,571 | 129,603 | 一致 |
| 末行到达(输入后) | 1.334s | 1.271s | ash 透明,零附加开销 |
| 折算吞吐 | ≈98KB/s | ≈102KB/s | **瓶颈在 ConPTY 翻译层**,与需求分析预期一致(Unix PTY 无此翻译,上限应更高) |
| 帧(25s) | 1549(~62fps) | 1562(~62fps) | 整帧重拼也不掉帧 |
| 脏帧 | 26 | 25 | 改损伤重绘后可再省 |
| 滚动观感 | 单调滚动,无糊屏 | 同左 | 快照序列证据 |

## 已知缺口(spike 债务 → Phase 1,非结构性)

1. **字形度量**:固定 Consolas advance(1126/2048 em)+ 拟合,未实测;
   宽字符(CJK 双宽)未处理;
2. **无损伤重绘**:每帧整帧重拼;`Term::damage()` 已可用,Phase 1 接上;
3. **16ms 轮询驱动**:`time::every` tick 轮询 PTY,应改为 reader 线程
   唤醒 + `Event::Wakeup` 事件驱动重绘;
4. **无字形图集**:每 run 独立 fill_text;正式实现做 glyph cache;
5. 无滚动回滚 UI、无光标块、无选中;Ctrl+C/关闭语义未系统验证;
6. 仅 Windows 基座验证;Unix 路径由 portable-pty 承诺未验。

## 对 auto-ui 的反哺清单(iced 生态共建)

1. **等宽字形管线**:实测 advance/字偶的 monospace 布局 API
   (网格坐标↔像素坐标换算),终端是极端案例,反哺文本布局;
2. **损伤重绘协议**:damage 区间合并 → 部分重绘的通用化;
3. **低延迟输入路径**:键盘事件→子进程回写的最短路径(IME 之外);
4. **整帧重拼的预算工具**:帧耗时/脏区统计(render-probe 的
   dirty_frames 思路产品化);
5. cosmic-term 先例复核:同样 iced 家族 + alacritty_terminal,证明
   该组合可承载正式终端,可互相借鉴 widget 结构。

## 附录

### ash 冒烟证据(evidence/001-smoke/)

- `render-find-colors.png`:PrintWindow 截图,find 表格边框连续、
  列对齐、彩色 run 正确(find 绿/-name 蓝/"*.md" 琥珀/边框 240 灰/
  提示符绿),状态行时钟完整;
- `ash-smoke-dump.txt` + `snaps/`:Tab 补全(补出历史命令)、
  ls/find 表格对齐、2000 行 0.37s 流式干净、长输出摘要冻结
  (61–63+80 折叠)生效;
- `bench-pwsh-20000.txt` / `bench-ash-20000.txt`:粗测原始数字。

### ConPTY 观察项(对照需求分析第 5 条)

- 翻译层吞吐 ≈100KB/s(20000 行数字),确实低于 Unix PTY 预期;
- resize 未见历史怪癖(两次 resize 干净);
- pwsh/cmd 启动即发查询序列,嵌入方回写应答后一切正常。

### iced 0.14 API 体验记录(Phase 1 参考)

- `iced::application(boot, update, view)`——boot 闭包产
  (State, Task);`.title/.window/.default_font/.subscription`;
- 窗口操作需先 `window::oldest().map(...)` 拿 `window::Id`
  (`Id::MAIN` 常量已移除);`iced::exit()` 可直接终止应用;
- 自定义 widget 需 `advanced` 特性;`Widget::{size, layout(&mut self,
  _, _, &Limits), draw}` 必需;文本经 `advanced::text::Renderer::
  fill_text(Text{..})`,色块经 `Renderer::fill_quad(Quad{..})`;
- 键盘:`keyboard::listen()` → `Event::KeyPressed{key, modifiers}`;
  shift 符号映射需自理(Character 不带 shift 结果);
- `time::every` 需 `tokio` 特性。

### alacritty_terminal 0.26 API 体验记录

- `Term::new(Config, &impl Dimensions, listener)`;`Config` 按值;
- Dimensions 自实现(total_lines/screen_lines/columns);
- 事件:`EventListener::send_event(&self, Event)` 单方法 +
  `VoidListener` 空实现;`Event::PtyWrite` 必须回写;
- `renderable_content().display_iter` 产 `Indexed<&Cell>`
  (point.line/point.column + c/fg/bg/flags);
- `damage()/reset_damage()` 可用于损伤重绘;`scroll_display` 可做
  回滚 UI。

### ash 行为观察(spike 中撞见,非 AutoTerm 问题)

- 命令行内 `$_`/`;` 会触发 ash 解析怪癖("program not found"),
  脚本走 `-File` 正常;
- Tab 补全从历史匹配整条命令;
- 未知命令自动委托 pwsh 执行(PowerShell 错误渲染完整)。
