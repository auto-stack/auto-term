---
plan_id: PLAN-002
status: execution_done
feature_name: AutoTerm Phase 1 单窗口 MVP(正式架构 + 仿真回归 + 损伤重绘/事件驱动)
author: [zhaopuming]
created_at: 2026-09-05T09:36:11+08:00
updated_at: 2026-09-05T12:30:00+08:00

# Leave these EMPTY here — /auto-plan:review fills them:
supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 9
total_steps: 10
---

# [PLAN-002] AutoTerm Phase 1 单窗口 MVP(正式架构 + 仿真回归 + 损伤重绘/事件驱动)

## 变更摘要

把 PLAN-001 的三个 spike 升格为**正式架构**:`crates/autoterm-core`
(PTY 会话 + alacritty_terminal 封装,含仿真回归测试)与
`crates/autoterm-ui`(iced 单窗口终端,事件驱动 + 损伤重绘 + 实测字形
度量 + 滚动回滚 + 光标块)。spike 三 crate 保留为归档不动。清偿
DEBTS #1/#3/#4(整帧重拼→损伤重绘、固定度量→实测、轮询→事件驱动)
与 #5/#7 部分(回滚 UI、关闭语义),并产出 Phase 1 设计文档
`docs/designs/001-phase1-architecture.md`(正式架构的耐久记录,
000 决策链的延续)。

## 目标

1. **正式 crate 结构**:`autoterm-core`(可独立测试的终端会话层)+
   `autoterm-ui`(窗口层),层间单向依赖,spikes/ 原样归档;
2. **仿真回归种子化**:纯 VT 字节级断言套件(光标/颜色/备用屏幕/
   宽字符/回滚/DSR 应答)+ 真 PTY 集成测试,双轨回归;
3. **渲染管线三项清偿**:损伤重绘(Term::damage 驱动)、实测字形
   advance(cosmic-text,含宽字符双格判定)、事件驱动 PTY→UI
   (reader 线程唤醒,替换 16ms 轮询);
4. **MVP 可日用**:滚动回滚(滚轮/PgUp/PgDn)、光标块、窗口关闭
   杀子进程、Ctrl+C;
5. **设计沉淀**:`docs/designs/001-phase1-architecture.md` 记录正式
   架构与 000→002 决策链。

非目标(排除):tabs/splits(Phase 2+);字形图集级 GPU 优化
(损伤重绘+事件驱动已获主要收益,图集留性能专项);IME/选中/主题
系统;Unix 基座适配(仍 Windows-only,另立计划);`.at`/auto-lang
VM 集成(路线 A 生态对齐,待 auto-ui 仓前置调查)。

## 架构方案

```
crates/autoterm-ui    iced 单窗口:widget(损伤感知)、键盘/滚轮、
                      字形度量、窗口生命周期        ──仅依赖──┐
crates/autoterm-core  PtySession(portable-pty:spawn/reader 线程/
                      答案回写/resize/kill 语义)+
                      TermSession(alacritty_terminal 封装:
                      feed/pump/damage/scroll/快照)  ←────┘
spikes/               PLAN-001 归档,不再演进
```

数据流(事件驱动,替换 spike 的 16ms 轮询):

```
子进程 ─ConPTY→ reader 线程 ─channel→ iced subscription(唤醒)
   ▲                                        │ feed/pump
   └─ 写主端(答案/键盘) ←─ update() ←────┘
draw():damage(行级) → 只重画脏行;光标块反色;回滚经 scroll_display
```

与外部系统:ash 仍仅运行时配置耦合(`--shell`,默认 pwsh);
iced 升级沿 0.14(与 auto-ui 同栈,集成路径见待澄清 #2)。

## 技术栈

- Rust 2024,Cargo workspace(crates/*);
- portable-pty 0.9、alacritty_terminal 0.26(与 spike 同版起步,
  升级另立);
- iced 0.14(tokio + advanced),cosmic-text(字形度量,iced 内部
  同源依赖,显式引入);
- log + env_logger(替换 eprintln;dev 取证钩子沿用 spike 的
  auto-input/dump 思路,`--dev-*` 前缀)。

## 需求分析与背景调查

> 种子:本仓 spec ledger(P001 系列,纯文件流程,见待澄清 #1 之先例)
> + DEBTS.md + docs/designs/000-render-route.md。

1. **ledger 现状**:P001-2 五目标已达成;P001-3 四层架构与路线 A
   已拍板;P001-4 probe 设计与判据已沉淀;P001-5 term-probe 集成
   测试被点名为"后续正式工程的回归种子"——本计划兑现它;
   P001-6 验收全过。本计划为 ledger 增补 Phase 1 各节;
2. **DEBTS 清单映射**:#1 整帧重拼→T6 损伤重绘;#3 固定度量→T5
   实测;#4 轮询→T4 事件驱动;#5 回滚 UI→T7;#7 Ctrl+C/关闭→T8;
   #10 颜色表→T8 随光标块顺带补 NamedColor 全映射;#2 字形图集、
   #6 IME/选中、#8 Unix、#9 文档→仍留账(非目标);
3. **000 附录的 API 结论直接复用**:alacritty_terminal 0.26 喂入/
   事件/damage/display_iter/resize 语义;iced 0.14 application/
   window::oldest/advanced widget/fill_text/keyboard::listen——
   spike 撞墙点(如 Id::MAIN 移除)已记录,不再重踩;
4. **spike 取证钩子**(`--auto-input/--dump/--snapshot-dir`)验证了
   无人值守冒烟的价值,正式版保留为 `--dev-*` 调试参数(不进
   README 主文档);
5. **设计文档问题(用户 2026-09-05 提出)**:001↔002 桥接已由
   000 承担(结论+缺口+附录);Phase 1 正式架构自身落
   `docs/designs/001-phase1-architecture.md`(T9),编号顺延。

## 详细设计

### crates/autoterm-core(lib)

- `term.rs`:TermSession 迁移自 spikes/term-probe/src/lib.rs,新增
  `take_damage()`(透传 `Term::damage()`/`reset_damage()`)与
  `scroll(delta_lines)`(透传 `scroll_display`);
- `pty.rs`:PtySession —— openpty(PtySize 由 ui 侧传入)、spawn、
  reader 线程→`Receiver<Vec<u8>>`(EOF 哨兵空 vec)、
  `pump_and_answer(writer)`:feed 后取 PtyWrite 应答写回主端、
  resize(先 TermSession 后 master)、`kill()`;
- `tests/live_pty.rs`:迁移 spike 集成测试(`cmd /c echo
  hello_term_probe` → 网格断言);
- `tests/sim_regression.rs`:**纯 VT 字节**(不经 PTY)的回归矩阵:
  - 光标/ED/EL:清屏+定位+写入 → 指定行列文本;
  - SGR 38;2 真彩与 38;5 256 色 → StyledChar 色值断言;
  - DECSET 1049 备用屏幕:切入→写入→切出,主屏原内容保留;
  - CJK 宽字符:占两格(第二格 spacer);
  - 回滚:超屏输出后 scroll_display 回看历史行;
  - DSR:`ESC[6n` → pump 输出匹配 `\x1b[...;...R`;
- `tests/pty_lifecycle.rs`:spawn→EOF→`exited()` 真;drop(kill)
  后 `wait()` 拿到退出状态。

### crates/autoterm-ui(bin `autoterm`)

- `main.rs`:iced application(boot 闭包建 PtySession+TermSession,
  `window::oldest()` 记 Id);`--shell <exe>`(默认 pwsh)、
  `--dev-exit-after/--dev-autotype/--dev-dump`(冒烟取证钩子);
- 事件驱动订阅:reader 线程字节经 iced 订阅通道(0.14 实际 API,
  候选 `Subscription::run` + stream channel)发 `Message::PtyBytes`,
  替换 `time::every`;键盘 `keyboard::listen`、窗口
  `resize_events`;
- `metrics.rs`:启动时 cosmic-text 测 `advance('M')` 得 cell 宽,
  行高取字体 ascent+descent;`is_wide(c)` = advance≥1.9×cell;
- `widget.rs`:网格 widget 迁移自 render-probe,改动:
  - draw 按 `take_damage()` 只重画脏行(Full 退化为整帧);
  - 光标块:`renderable_content().cursor` 反色渲染;
  - 滚轮/Shift+滚轮 → `scroll(delta)`;PgUp/PgDn 翻页;
    回滚偏移>0 时顶行显示 `↑N` 指示;
  - NamedColor 全枚举映射(Dim/Bright 前景),16/256/真彩表
    收敛到 `palette.rs`;
- 关闭语义:窗口 `Event::Closed` → `PtySession::kill()` + wait,
  退出 app;
- 顶层 `src/lib.rs`(便于 ui 单测)+ `src/bin/autoterm.rs` 入口。

### docs/designs/001-phase1-architecture.md(T9 交付)

章节:正式 crate 结构图、事件驱动数据流时序、损伤重绘协议
(damage 语义→脏行集合→draw 剪裁)、字形度量与宽字符策略、
回滚模型(display_offset 与状态指示)、000→002 决策链与残留缺口
(引 DEBTS 未清项)。终态含"结论"节。

## 测试设计

- 自动:autoterm-core 三套测试(live_pty/sim_regression/
  pty_lifecycle)+ metrics 单测 + workspace 全量 `cargo test`;
- 半自动:`--dev-*` 钩子冒烟(echo 回显、2000 行流式、滚轮回滚
  快照、光标块绘制证据)——**以程序化证据为主**(网格转储、度量
  数值断言、draw 状态转储),截图仅作人工复核补充,不构成验收
  依赖(计划执行不依赖多模态会话);
- 手动:日常试用清单(pwsh/ash 各 10 分钟:补全/表格/真彩/
  Ctrl+C/关闭无残留进程 + 光标块/对齐目测);
- 不做:性能基准自动化(字形图集专项时另立)、Unix 矩阵。

## 验收标准

1. `crates/autoterm-core`、`crates/autoterm-ui` 存在且
   `cargo test --workspace` 全绿(spikes/ 原样未动);
2. sim_regression ≥ 6 个断言用例全过(光标/真彩/256 色/备用屏/
   宽字符/回滚/DSR 至少各一);
3. 事件驱动生效:常态空闲不触发重绘帧(日志/计数证据),
   字节到达即醒;
4. 损伤重绘:单行改动帧的重画 run 数 < 全网格 run 数(计数断言);
5. 实测度量:cell 宽来自 cosmic-text 实测(单测断言 >0 且与
   0.55em 量级一致);右缘无裁剪由**度量断言**保证
   (dev 转储含 cols×cell_w ≤ 视口宽 数值,程序化核验);
6. 滚轮回滚可用:大输出后回滚快照含历史行,顶行有偏移指示;
7. 光标块**绘制证据**:dev 转储含 `cursor_drawn_at`(行列)+
   反色标志(draw 状态),人工目测为可选补充;窗口关闭后无残留
   子进程(tasklist 验证);
8. `docs/designs/001-phase1-architecture.md` 存在且含"结论"节;
9. README 更新(Win10 1809+ 支持声明、autoterm 运行段);
   DEBTS 勾账(#1/#3/#4 清偿,#5/#7/#10 部分清偿并注明);
10. `cargo tree --workspace` 无 `ash-core`/`auto-shell` 路径依赖。

## 执行步骤

- [x] **T1** 建 `crates/autoterm-core`(lib):迁移
      `spikes/term-probe/src/lib.rs` → `crates/autoterm-core/src/term.rs`
      (包名/路径更名为 autoterm_core,公开 API 不变),
      `spikes/term-probe/tests/live_pty.rs` →
      `crates/autoterm-core/tests/live_pty.rs`;根 workspace `members`
      追加 `"crates/autoterm-core"`;spikes/ 不动。
      验证:`cargo test -p autoterm-core`(live_pty 过)
      [✅ 已完成] live_pty_output_reaches_grid ok;spikes/ 未动;API 面不变(TermSession/StyledChar/GridSize/Color/NamedColor/Rgb)
- [x] **T2** 写 `crates/autoterm-core/src/pty.rs`:PtySession
      (spawn/reader 线程 channel/EOF 哨兵/答案回写/resize 同步
      TermSession+master/kill+wait);新建
      `tests/pty_lifecycle.rs`:spawn `cmd /c echo x` → EOF →
      exited() 真 → kill 后 wait() 返回。
      验证:`cargo test -p autoterm-core --test pty_lifecycle`
      [✅ 已完成] 测试绿(0.05s)。**语义修正(平台事实)**:ConPTY 持活期间主端读流永不 EOF(conhost 等输入端关闭),exited() 改走 try_wait 而非 EOF 门控;测试同时断言"无 EOF"防平台行为变化;live_pty 顺带修掉被静默掩盖的 15s 超时(全测试 15s→0.1s)。EOF→exited 的计划原文与 Windows 现实不符,已记入 pty.rs 模块头
- [ ] **T3** 写 `crates/autoterm-core/tests/sim_regression.rs`:
      六类用例(光标定位/SGR 真彩+256/备用屏 1049/CJK 双格/
      scroll_display 回滚/DSR 应答 pump 匹配 `\x1b[...R`)。
      验证:`cargo test -p autoterm-core --test sim_regression`
      (≥6 用例绿)
- [x] **T4** 建 `crates/autoterm-ui`(lib+bin `autoterm`):迁移
      render-probe 的 app 骨架(boot/update/view/keyboard listen/
      window resize/oldest Id);**事件驱动**:reader 线程 → iced
      订阅通道 → `Message::PtyBytes`(移除 time::every 轮询);
      保留 `--shell`(默认 pwsh)与 `--dev-exit-after/--dev-autotype/
      --dev-dump` 取证钩子;members 追加。
      验证:`cargo build -p autoterm-ui` +
      `autoterm --shell pwsh --dev-exit-after 6 --dev-autotype "echo hi\r"
      --dev-dump <f>` 转储网格含 `hi`
      [✅ 已完成] build 绿;冒烟转储含 `echo hi` 与 `hi`,回显 54ms;6s 仅 59 次更新(=字节到达数,spike 轮询同期 372)——事件驱动实证;常态无定时器(dev 钩子激活时才有 50ms dev tick);订阅用 `run_with(Hash 槽)+stream::channel` 承载 core 唤醒通道
- [x] **T5** 写 `crates/autoterm-ui/src/metrics.rs`:cosmic-text
      实测 advance('M')=cell 宽、行高 ascent+descent、
      `is_wide(c)`(≥1.9×cell);widget 改用实测度量;dev 转储
      增加 `metrics` 行(cell_w/line_h/cols)与 `fit_ok:
      cols*cell_w<=viewport_w` 断言值。
      验证:`cargo test -p autoterm-ui metrics`(cell>0,数量级
      0.5–0.7em;CJK 判 wide)+ `--dev-dump` 冒烟转储含
      `fit_ok: true`
      [✅ 已完成] 测试 2 用例绿(cell_w=9.375px=0.586em 实测);冒烟 `fit_ok: true (993.8<=1000.0)`。**两处偏差(实测驱动)**:①行高用 1.25em 相对值——cosmic-text 0.15 公面无字体 ascent/descent;②is_wide 改用 unicode-width——本机 monospace 字体 '中' 的 advance 与 'M' 完全相等(9.375px),字体 advance 不承载终端双格语义(计划原文 ≥1.9×cell 不可达),与 alacritty 上游做法一致
- [x] **T6** 损伤重绘:autoterm-core `TermSession::take_damage()`
      (damage+reset_damage 透传);ui widget draw 按脏行集合剪裁,
      帧计数器(debug 日志)记重画 run 数。
      验证:`cargo test -p autoterm-core` + 冒烟日志:单行改动帧
      run 数 < 全网格 run 数
      [✅ 已完成·部分] core 测试 8 用例绿(含 damage 用例);冒烟 `damage_last: lines=1`(部分损伤贯通)、快照按需重建 16 次。**诚实边界(记入待澄清#6)**:iced 即时模式每帧全量重建场景,绘制级跳过未脏行会把它们清空——验收#4 的"run 数不等式"在绘制层不可达,损伤收益落在:①damage API(8 用例回归)②快照重建门控 ③draw 计数取证(draw_runs 恒 36);绘制级剪裁需保留式画布(Phase 2)
- [x] **T7** 回滚 UI:widget 滚轮/PgUp/PgDn →
      `TermSession::scroll(delta)`;display_offset>0 时顶行 `↑N`
      指示;任意键入回正。
      验证:冒烟(`--dev-autotype "pwsh -NoLogo -Command \"1..200\"\r"`
      + `--dev-scroll -10` 钩子)转储含首行 `1`
      [✅ 已完成] 滚轮(3 行/格)/PgUp/PgDn(整页)接线,键入回正,↑N 顶右指示;冒烟 200 行后回滚到顶(offset=171),转储首两行 `1`/`2`;注:计划原文 `-10` 的符号在 T3 已裁定为上游约定(正=上翻),取证用 `--dev-scroll 100000` 夹到顶
- [x] **T8** 光标块 + 关闭语义 + 全色表:`palette.rs` 收敛
      NamedColor 全映射(含 Dim/Bright 前景);渲染
      `renderable_content().cursor` 反色块,draw 状态经 dev 转储
      输出 `cursor_drawn_at: (row,col) inverted=true/false`;
      窗口 Closed → kill+wait 子进程。
      验证:冒烟 `--dev-dump` 含 `cursor_drawn_at` 且 inverted=true;
      关闭后 `tasklist | grep -c <autoterm 子进程名>` 为 0
      [✅ 已完成] palette 测试 2 例绿(全映射/xterm256/真彩);冒烟 `cursor_drawn_at: (3,19) inverted=true`;窗口关闭订阅 → kill+wait,pwsh 计数 4→3 无孤儿(Drop 安全网兜底异常路径)
- [x] **T9** 写 `docs/designs/001-phase1-architecture.md`(章节
      见详细设计;含 000→002 决策链与残留缺口)。
      验证:`test -f docs/designs/001-phase1-architecture.md &&
      grep -c "结论" docs/designs/001-phase1-architecture.md` ≥ 1
      [✅ 已完成] 文档落位,`grep -c 结论`=1;含全部实测发现(ConPTY 无自然 EOF、Delta 符号、字体 advance 证伪、iced 即时模式边界);commit adc0be4
- [x] **T10** 收尾:README(Win10 1809+ 支持声明、`cargo run -p
      autoterm-ui` 运行段、spikes 归档说明);DEBTS 勾账(#1/#3/#4
      清偿、#5/#7/#10 部分清偿注明);全量回归。
      验证:`cargo test --workspace` 绿 + `! cargo tree --workspace |
      grep -q ash-core` && echo OK
      [✅ 已完成] 全量 16 用例绿(sim 8/ui 4/live/lifecycle + spike 套件),无 ash-core/auto-shell 路径,cargo check 干净;README 三项声明齐;DEBTS 账本更新(#3/#4/#5/#9/#10 清偿,#1 部分+#6/#7 部分+#2/#8 留账,新增 4 条观察);commit 37daee5

## 复审记录

(待 /auto-plan:review 填写)

## 待澄清事项

1. **plan 工具链形态**:沿 PLAN-001 先例(待澄清#2 建议),本仓
   继续纯文件流程(手工 .autoos/specs.json),惯例固化到何时部署
   backend 待定;
2. **iced↔auto-ui 集成路径**(承 PLAN-001 待澄清#3):路线 A 生态
   对齐、`.at` 组件模型承载原生 widget 的可行性,需与 auto-ui 仓
   对齐——Phase 2 前置调查,不阻塞本计划;
3. **字形图集专项**(DEBTS#2):T6 损伤重绘 + T4 事件驱动是否已
   达日用性能,决定图集专项的优先级——T10 冒烟后回填结论;
4. **Ctrl+C 语义矩阵**(DEBTS#7 残留):pwsh/ash/cmd 下 Ctrl+C/
   Ctrl+Break/关闭的中断行为矩阵,Phase 2 系统验证;
5. **`--dev-*` 钩子去留**:正式版长期保留调试钩子还是抽到
   feature flag(`--features dev-tools`),T10 时定。
6. **验收#4 绘制级 run 不等式未达(T6 实测发现)**:iced 0.14 即时
   模式每帧全量重建渲染场景,跳过未脏行绘制=清空该行;损伤管线
   已交付 damage API/快照门控/计数取证三层,绘制级脏行剪裁需
   保留式画布(每行缓存 Paragraph 或 wgpu 层)——建议归入 Phase 2
   字形图集专项一并做,复审时请裁定按"部分达成+路由"接受或退回。
