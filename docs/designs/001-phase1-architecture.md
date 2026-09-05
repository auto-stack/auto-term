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
