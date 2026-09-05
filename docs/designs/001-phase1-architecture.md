# 001 · AutoTerm Phase 1 正式架构(单窗口 MVP)

> PLAN-002 产出 · 2026-09-05 · Windows 基座(ConPTY 实证)
> 决策链:`docs/designs/000-render-route.md`(路线 A 拍板)→ 本文
> (正式架构)。spike 三 crate 保留在 `spikes/` 作证据归档。

## 结论

Phase 1 正式架构成立并落地:`crates/autoterm-core`(PTY 会话 +
仿真核心,10 个回归用例)+ `crates/autoterm-ui`(事件驱动 iced 单
窗口,实测度量、回滚 UI、反色光标、关闭语义)。三项 DEBTS 清偿
(#1 部分/#3/#4)均有数字证据;两项计划假设被实测**证伪并修正**
(宽字符判定、ConPTY EOF 语义),一项能力受 iced 即时模式约束
**路由 Phase 2**(绘制级损伤剪裁)。

## 正式 crate 结构

```
crates/autoterm-core (lib)
├─ term.rs   TermSession:alacritty_terminal 0.26 封装
│            feed / pump(应答) / resize / scroll / take_damage /
│            cursor / visible_(styled_)lines / display_offset / history
├─ pty.rs    PtySession:portable-pty 0.9
│            spawn / reader 线程(字节通道+唤醒通道) / write_input /
│            drain(feed+回写应答) / resize(先 sim 后 ConPTY) /
│            kill / wait / exited(try_wait) / Drop 安全网
└─ tests/    live_pty(真 PTY→网格)/ sim_regression(8 用例纯 VT)/
             pty_lifecycle(生命周期)

crates/autoterm-ui (lib + bin `autoterm`)
├─ lib.rs      App 状态机 + 事件驱动订阅 + dev 取证钩子(--dev-*)
├─ metrics.rs  实测字形度量(cell_w)+ unicode-width 宽字符判定
├─ palette.rs  NamedColor 全映射(Dim×8/Bright/Dim 前景,xterm256)
└─ widget.rs   TermGrid 自定义 widget(同色 run、↑N、反色光标、取证)
```

依赖单向:ui → core;core 零 GUI 依赖,可独立 CI 回归。

## 事件驱动数据流(替换 spike 的 16ms 轮询)

```
子进程 ─ConPTY→ reader 线程 ─┬─ 字节块通道 → PtySession::drain()
                             └─ 唤醒通道(()) → 转发线程 →
                                iced stream::channel → Message::PtyBytes
update():drain()(喂仿真核心 + 回写 DSR/DA 应答)→ take_damage →
         按需重建快照 → view()
常态零定时器:空闲无唤醒(实测 6s 仅 59 次更新,spike 轮询同期 372);
dev 钩子激活时才有 50ms DevTick(不触发重绘)。
```

订阅承载方式:`Subscription::run_with(Hash 槽, …)` + 唤醒接收端
"一次性槽"(Arc<Mutex<Option<Receiver>>>),首次运行取走接收端、
转发线程阻塞 recv → `sender.try_send`。

## 损伤重绘协议(及 iced 即时模式的边界)

- **语义**:`TermSession::take_damage()` → `Damage::{Full, Lines}`
  (视口相对行号,取后即 `reset_damage`);上游恒把光标行计入损伤
  ("Always damage current cursor")——静默拍 = 仅光标行。
- **已落地**:①damage API(8 用例回归:单行写入=行级、清屏=Full、
  静默=光标行);②快照重建门控(仅字节到达/resize/scroll 重建,
  实测 16 次重建对应 13 次脏更新+初始+resize);③draw 取证计数。
- **边界(实测发现)**:iced 即时模式每帧全量重建渲染场景——
  **跳过未脏行的绘制会把它们清空**,绘制级脏行剪裁在公面上不可
  实现;shaping 级缓存由 iced/cosmic 内部承担(同内容 fill_text
  命中形状缓存)。
- **Phase 2 路由**:绘制级剪裁需保留式画布(每行缓存 Paragraph,
  以 `Paragraph::compare` 判差异、仅脏行重建形状)或 wgpu 自管
  层——与字形图集专项(000 缺口#2)合并设计。

## 字形度量与宽字符策略

- **cell_w 实测**:cosmic-text Buffer shaping `'M'`(monospace
  family),与 iced 渲染同字体系统 → advance 一致;本机实测
  9.375px(0.586em,匹配到的并非 Consolas;resize 与 draw 同源
  度量,`fit_ok: cols×cell_w ≤ viewport_w` 数值断言防右缘裁剪)。
- **行高**:1.25em 相对值(cosmic-text 0.15 公面不暴露字体
  ascent/descent;终端惯例)。
- **宽字符**:**unicode-width**(East Asian Width)判定。计划原文
  "实测 advance ≥1.9×cell" 被实测证伪——本机 monospace 字体对
  '中' 的 advance 与 'M' 完全相等(9.375px),字体 advance 不承载
  终端双格语义;与 alacritty 上游一致改用属性表判定。

## 回滚模型

- 视口偏移 `display_offset`(0=贴底);`scroll(delta)` 透传
  `Scroll::Delta`——**正=上翻历史,负=下回实时**(alacritty 上游
  符号约定,T3 实测钉死)。
- UI 映射:滚轮 3 行/格(wheel 正 y=上翻)、PgUp/PgDn 整页
  (消费,不进 PTY)、任意键入先回正(终端惯例);offset>0 时
  顶右 `↑N` 指示。
- 快照注意:`display_iter` 给出**绝对网格行**(历史区为负),
  视口行 = `绝对行 + display_offset`(T3 越界教训)。

## 关闭语义与子进程生命周期

- 窗口 `close_events` → kill + wait(同步回收)→ exit;
  `PtySession::Drop` 作异常路径安全网(同样 kill+wait)。
- 实证:关闭后 pwsh 计数 4→3(被杀即 autoterm 的子进程,无孤儿)。

## ConPTY 平台事实(本计划实测补充,000 附录延伸)

1. **主端读流无自然 EOF**:会话持活期间(输入写端/master 句柄
   未关)conhost 不终止,输出管道保持打开——子进程退出检测必须
   走 `Child::try_wait`;spike 期 `live_pty` 的 15s "等待"实为被
   静默超时掩盖(修正后全套 core 测试 15s×2 → <0.1s)。
2. `Scroll::Delta` 符号约定如上;`damage()` 恒含光标行。

## 000→002 决策链与残留缺口

- 000 拍板路线 A(iced)→ 002 验证可行并落地正式架构;
- DEBTS(PLAN-001)清偿:#3 固定度量→实测 cell_w ✅;#4 轮询→
  事件驱动 ✅(59 vs 372);#1 整帧重拼→damage 管线**部分清偿**
  (快照门控+API,绘制级剪裁路由 Phase 2);#5 回滚 UI ✅;
  #7 关闭语义 ✅(Ctrl+C 矩阵仍欠);#10 全色表 ✅;
- 残留(Phase 2+):字形图集/保留式画布(含绘制级损伤剪裁)、
  IME/选中、Ctrl+C/Break 语义矩阵、Unix 基座、光标闪烁与形状
  (Underline/Beam 已取 shape 未分形状渲染)。

### 附录补注(PLAN-003 T1 PoC,2026-09-05)

保留式画布前提验证通过:`iced::advanced::text::paragraph::Plain<P>`
公面可用——构造走 iced_graphics 全局字体系统(OnceLock 惰性,
headless 测试无需 application);`update()` 内建 content 比对 +
`compare`(注意:compare 只看版式参数不看文本,content 由 Plain
先比)+ Bounds 差异走 resize;同内容反复 update 零重建。
T2-T4 按 `Plain<Paragraph>` 落地行缓存。

### 附录补注:Ctrl+C / Ctrl+Break 语义矩阵(PLAN-003 T6/T7,2026-09-05)

**矩阵实测**(证据 evidence/003-matrix/):

| 客户端 | Ctrl+C(裸 0x03 字节) | 普通键中断 |
| --- | --- | --- |
| ash REPL(raw mode 直读字节) | ✓ 响应(行编辑重置/回提示符) | ✓ |
| pwsh `Start-Sleep`(控制台 API) | ✗ 无 ^C 回显、不中断 | ✓ |
| cmd `timeout`(控制台 API) | ✗ 不中断 | ✓('x' 即退) |

**平台结论**:ConPTY 输入管道的裸 0x03 字节**不会被翻译为
CTRL_C_EVENT**——经典控制台 API 程序(ReadConsole 系)收不到
中断;VT/raw-mode 客户端(自读字节,如 ash)不受影响。

**Ctrl+Break 双重阻断**(T7):①iced 0.14 `key::Named` 无
Break/Cancel 变体,事件层不可分辨;②即使可分辨,CTRL_BREAK_EVENT
与真实 Ctrl+C 事件同走 win32 `GenerateConsoleCtrlEvent(pid, event)`
——portable-pty 不暴露子进程组句柄。

**路由**:真 Ctrl+C/Ctrl+Break 需本仓引入 win32 直调模块
(经 child pid 发 GenerateConsoleCtrlEvent;BREAK 常数 0,
CTRL_C 常数 1 取决于句柄语义)——待用户裁定(计划 003
待澄清#3),批准后约 30 行 win32 代码可闭环。

### 附录补注:IME 可行性(PLAN-003 T8,2026-09-05)

**结论:管线公开可用,落地路由 Phase 3。**

- iced 0.14 公面完备:`Event::InputMethod(Opened/Preedit/Commit/
  Closed)`、`Shell::request_input_method(cursor_rect, purpose)`、
  `InputMethod::Enabled { preedit }` 支持 **over-the-spot** 模式
  (运行时代为叠加显示预编辑串,widget 无需自绘);
- 落地成本:TermGrid 需补 `Widget::update` 事件处理(当前仅
  size/layout/draw)以在聚焦时请求 IME、经 Shell 发布 Preedit/
  Commit 到 App 消息;预编辑期间需挂起键盘直写(避免双写);
- **验证约束(路由主因)**:IME 组合输入无法经 `--dev-autotype`
  无人值守触发——取证强依赖真人中文输入法交互;按本仓"程序化
  证据为主"的政策,列入 Phase 3(或用户点名时以手动清单验收)。

## 002→003 决策链(PLAN-003,2026-09-05)

1. **保留式画布落地**(兑现 002 验收#4 的路由):每行
   `Para::with_spans` 缓存 + 行 digest(字符+前后景色)判异 +
   `Damage::Lines` 脏行门控;`compare` 只看版式参数不含文本,故
   digest 由我们自持;span 前景色烘焙进 buffer,cryoglyph 逐字形
   color_opt 优先渲染(源码级+338 彩色像素证据);背景与光标仍走
   quad。**绘制级剪裁至此闭环**:末帧脏行内容未变时重建数为 0。
2. **健壮性修复(20000 行首现)**:唤醒转发线程遇 iced 通道 Full
   即退出的 bug——尾部字节无人 drain;改为 is_full 重试后 3 连跑
   bytes_fed 恒定、输出完整。
3. **Ctrl+C/Ctrl+Break 矩阵结论**:ConPTY 不把裸 0x03 翻译为
   CTRL_C_EVENT(经典控制台 API 程序收不到中断;raw-mode 客户端
   如 ash 正常);Ctrl+Break 被 iced 键层与 win32 双重阻断——
   真事件需 `GenerateConsoleCtrlEvent` win32 直调(待用户裁定)。
4. **IME**:管线公面完备(over-the-spot 叠加),落地需 Widget::
   update;验证强依赖真人输入法交互,路由 Phase 3。
5. **dev-tools feature**:`--dev-*`/DevTick/转储/取证静态量全部
   cfg 门控,默认构建零 dev 面;log+env_logger 替换 eprintln;
   Message::NoOp 清偿空唤醒复用。

残留缺口(Phase 3+):真 Ctrl+C/Break(win32 裁定后 ~30 行)、
IME 落地、Unix 基座、光标形状/闪烁、选中。

### 附录补注:WT/Alacritty 的 Ctrl+C 机制与本机实测(复审期补充调查)

**上游源码事实**(microsoft/terminal,src/terminal/parser/
InputStateMachineEngine.cpp + src/host/input.cpp):
- **无人调用 GenerateConsoleCtrlEvent**。两条公道:
  ①裸 0x03:输入状态机 `_DoControlCharacter` 把 ETX 特判,合成为
  VK'C'+LEFT_CTRL_PRESSED 按键事件(down+up)→ `WriteCtrlKey` →
  `HandleGenericKeyEvent` → 行输入模式下 `HandleCtrlEvent
  (CTRL_C_EVENT)`;
  ②win32-input 编码(`ESC[vk;sc;uc;kd;cs;rc_`,DECSET 9001 握手后
  WT 用之):同样经 `WriteCtrlKey`,源码注释明言"即使非控制键也走
  此路,以确保 Ctrl+C/Ctrl+Break 被正确处理";
- ConPTY 信号管道(PtySignalInputThread)只承载 Resize/ShowHide/
  ClearBuffer/SetParent,**无 Ctrl 信号**;
- Alacritty 即裸 0x03 路径(其 input 映射 Ctrl+C→字节 3)。

**本机实测矩阵**(Win11 26200):
| 通道 | 结果 |
| --- | --- |
| 最小 ConPTY 管道(portable-pty 直连)+ 裸 0x03 → cmd/ping | 不中断 |
| 最小 ConPTY 管道 + win32 编码按键事件 → cmd/ping | 不中断 |
| **真 Windows Terminal + 真实 ^C 键**(SendKeys 实测) | **同样不中断** |

**结论**:上游机制在源码层完备,但本机(26200 内部版)conhost 的
0x03/编码→CTRL_C_EVENT 翻译未生效,WT 与 AutoTerm 同病——是
机器/版本级行为,非本仓实现缺口。Phase 3 首任务修正为:先在稳定
OS 版本上复测(若正常则本机为内部版回归,无需任何代码;若复现,
再评估 AttachConsole+GenerateConsoleCtrlEvent 的 win32 直调)。

### 附录补注:稳定版复测裁定(PLAN-004 T7,2026-09-05)

**用户裁定(2026-09-05)**:当前无稳定版 Windows 环境可执行复测。
T7 按"待环境"执行:

- **矩阵就绪**:三通道复测脚本与证据骨架沿用 evidence/003-matrix/
  (裸 0x03 / win32 编码 / 真 WT+真实 ^C),在稳定版机器上重跑即得数;
- **结果栏:待环境**——26200 内部版的矩阵结论维持上文(三通道全不
  中断),稳定版数据到位之日补测即关账;
- **挂账**:DEBTS #7 持有该项(不阻塞 Phase 3 其余交付);
- **决策树不变**:稳定版正常 → 内部版回归关账(零代码);复现 →
  win32 直调(AttachConsole+GenerateConsoleCtrlEvent,未文档化路径)
  立项。

### 附录补注:IME 落地(PLAN-004 T8,2026-09-05)

**003 的路由在 004 兑现**,事件地基即 T2 的 `Widget::update`:

- **管线**:任意事件 → `request_input_method(Enabled{cursor:
  终端光标格矩形, purpose: Terminal})`(winit `set_ime_allowed/
  cursor_area`,组合窗随光标);`Event::InputMethod::{Preedit,Commit,
  Closed}` → `Message::SetPreedit/CommitIme`——preedit 挂起显示
  **不写 PTY**,Commit 清挂起 + `write_input` 直写;
- **over-the-spot 首选已试**:preedit 交 runtime 覆盖层
  (`Enabled{preedit: Some}` → iced_winit `draw_preedit`)。**本机
  不落屏**:main-events 相相位对 `State::Updated{input_method}` 的
  消费直接忽略(仅 redraw 相相位应用),dev 埋点 381 次请求、App
  状态与 view 链路均证请求已发出——运行时消费边界,按计划裁定
  次序降级**自绘**(preedit 在光标格内联 + 2px 下划线,CJK 双格宽
  估算);
- **像素证据**(evidence/004-select/):下划线 4 物理行(2px×200%
  缩放)× 318px = **恰 17 格**(7 CJK×2 + IME×1 格)× 9.375px × 2
  ——自绘渲染几何精确;视觉复核提示符后内联带下划线;
- **人工清单待执行**(真实 IME 合成无法自动键入,003 已裁定):
  pwsh/ash 各 5 分钟——拼音组句/上屏/中英切换/Esc 取消;通过后
  截图归档。清单跑法:启动 autoterm → Win+Space 切微软拼音 →
  提示符组句 → Enter 上屏 → Shift 切中英 → Esc 取消预编辑。
