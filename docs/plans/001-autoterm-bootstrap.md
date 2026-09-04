---
plan_id: PLAN-001
status: reviewed
feature_name: AutoTerm 建仓与全链路 spike(PTY→仿真核心→渲染)
author: [zhaopuming]
created_at: 2026-09-05T00:14:01+08:00
updated_at: 2026-09-05T03:40:00+08:00

# Leave these EMPTY here — /auto-plan:review fills them:
supersedes_spec_components: []
new_spec_components:
  - "docs/designs/000-render-route.md: 新增"
touched_goals:
  - "goal-1: 宿主 PTY(ConPTY)可申请可读写,VT 字节流实证(9.0 B/s+ESC 序列)"
  - "goal-2: alacritty_terminal 0.26 库嵌入可行(真 PTY 集成测试绿)"
  - "goal-3: iced 渲染最小闭环(echo 回显 0.154s,resize 不崩)"
  - "goal-4: ash 冒烟四项全过(补全/流式不糊/表格对齐/真彩)"
  - "goal-5: 大输出数字(≈100KB/s,ConPTY 瓶颈)+ 渲染路线 A 拍板"

current_step: 9
total_steps: 9
---

# [PLAN-001] AutoTerm 建仓与全链路 spike(PTY → 仿真核心 → 渲染)

## 变更摘要

新建独立工程 `D:\autostack\auto-term`(产品名 **AutoTerm**),作为 AutoOS
虚拟桌面的**通用终端**基础设施。本计划只做两件事:

1. **建仓骨架**——README(定位声明:与 auto-shell 零构建依赖)、Cargo
   workspace、DEBTS.md;
2. **三个 spike 打通全链路**——`pty-probe`(宿主 PTY 申请 + VT 字节流)、
   `term-probe`(alacritty_terminal 仿真核心嵌入,headless)、
   `render-probe`(最小 iced 窗口渲染网格 + 键盘回写),外加 ash 冒烟与
   大输出粗测,最终产出**渲染路线决策文档**(方案 A:iced 一等应用
   vs 方案 B:winit/wgpu 独立原生)。

不修改 auto-shell 仓任何代码。ash 只作为**运行时配置**(默认 shell 路径),
不进依赖树。

## 目标

1. 验证硬前提:宿主基座(Windows ConPTY / Unix PTY)经 `portable-pty`
   可申请、可读写字节流——在 Windows 基座上当天可证;
2. 验证 `alacritty_terminal` 作为库嵌入的可行性(VT 状态机 + 网格读取 +
   回滚),不写 GUI 即可测;
3. 验证渲染最小闭环:iced 窗口里网格→字形、键盘→PTY 回写,能交互跑
   `echo` 级命令;
4. 用 ash 这个"苛刻 VT 客户端"冒烟:tail 动态重绘不糊、find 表格不错位、
   真彩色正确;
5. 大输出压测取得第一手数字(ConPTY 吞吐 + 渲染帧行为),支撑渲染路线
   A/B 拍板,结论沉淀为 `docs/designs/000-render-route.md`。

非目标(明确排除):tabs / splits / tmux 式持久会话(Phase 2+ 另立计划);
`.at`/auto-lang VM 集成(AutoUI 一等应用形态是路线 A 拍板**之后**的事);
macOS/Linux 基座适配(spike 仅 Windows,Unix 路径由 portable-pty 承诺,
后续计划验证)。

## 架构方案

四层结构,层间单向依赖:

```
应用层   tabs / splits / 会话(Phase 2+,不在本计划)
渲染层   候选 A:iced 自定义 widget 承载终端网格 ← 本计划拍板
         候选 B:winit + wgpu 字形图集自绘(逃生口)
仿真层   alacritty_terminal(Apache-2.0):VT 状态机、回滚缓冲、
         resize/真彩色/备用屏幕语义 —— 直接复用,不自写
PTY 层   portable-pty:Windows=CreatePseudoConsole(ConPTY),
         Unix=openpty —— 基座 OS 服务,本仓只消费
```

与外部系统的关系:

- **ash**(`D:\autostack\auto-shell`):仅运行时配置耦合(spawn 的默认
  shell 路径,经 `ASH_BIN`/配置传入)。验收含"依赖树无 ash-core"断言;
- **auto-ui / auto-lang VM**:路线 A 成立时,后续以 iced/`.at` 生态一等
  应用形态集成,由后续计划处理;本计划 spike 阶段直接依赖上游 iced;
- **AutoOS 虚拟桌面**:本仓产出的可执行文件是宿主普通进程,窗口长在
  桌面里,无 OS 层依赖(PTY 由基座内核/console 服务提供)。

spike 三 probe 的数据流(单向,渲染帧驱动):

```
子进程(bash/pwsh/ash) ─ConPTY─▶ 字节流 ─feed─▶ alacritty_terminal 网格
     ▲                                              │
     └── 写主端 ◀── 键盘事件 ◀── iced 窗口(每帧读网格渲染)◀─┘
```

## 技术栈

- Rust(edition 以执行时最新稳定为准),Cargo workspace,spike 期间单
  workspace 多 bin(`spikes/` 下三个 crate);
- `portable-pty`(wezterm 系,Apache-2.0/MIT):PTY 跨平台抽象;
- `alacritty_terminal`(Apache-2.0):终端仿真核心;
- `iced`(路线 A 候选渲染栈,AutoUI 同源);
- (路线 B 候选,仅作对比不引入)`winit` + `wgpu`;
- 参考先例:**cosmic-term**——libcosmic(iced 家族)之上的终端,同样以
  alacritty_terminal 为核心,是路线 A 架构成立的公开先例。

版本号一律执行时取最新稳定,不在此预锁(避免计划文档过期)。

## 需求分析与背景调查

> 本仓为新建工程,尚无 spec ledger / backend——本节种子来自立项讨论
> (2026-09-04/05,auto-shell 会话)与生态现状,非 specs/overview。

1. **为什么是独立仓**:通用终端对 shell 零专属逻辑,是 AutoOS userland
   基础设施;放 auto-shell 会造成"不依赖 shell 的项目住在 shell 仓"的
   依赖方向倒挂,且 plan 流/回归/发布节奏互相干扰。D:\autostack 兄弟仓
   布局(auto-lang、auto-ui、auto-ai…)即为此准备。命名:仓 `auto-term`,
   产品 **AutoTerm**(ATerm 与 Linux 老牌终端 aterm 及 CWI 项重写工具
   撞名,弃用);
2. **ash 是最苛刻的首批客户**:auto-shell Plan 074-077 已落地运行中命令
   动态块、AI 回合动态渲染、长输出摘要冻结、表格渐进渲染(E1-E5),
   全部依赖 raw mode + 逐帧重绘 + VT 直通——AutoTerm 的仿真正确性将被
   ash 日常锻炼;ash 在 Windows Terminal 下走 ConPTY 的 VT 直通便宜路径,
   在 AutoTerm 下同理;
3. **PTY 结论**:虚拟桌面跑在基座上,PTY 是基座 OS 的进程/IO 基础设施
   (Unix=内核 pty 驱动;Windows=ConPTY,Win10 1809+,用户态翻译层),
   外壳层只消费不实现。Alacritty/WT/VS Code 在 Windows 上全部走
   CreatePseudoConsole,无第二条路,成本人人平等;
4. **ash-gui 定位已澄清**:ash-gui-auto(`.at`→auto-lang VM→iced,挂
   ash-server 真 ash-core)是 AutoUI 先锋应用、结构化 GUI shell 路线,
   与 AutoTerm(PTY+VT 基础设施)是两个物种、并行不悖。AutoTerm 不等
   AutoUI 成熟(渲染路线 B 为逃生口),但也优先争取路线 A(与桌面同栈,
   并以终端级文本渲染需求反哺 AutoUI);
5. **Windows ConPTY 已知特性**(spike 观察项):翻译层吞吐不如 Unix
   PTY、Win11 有直通改进、resize/关闭/Ctrl+C 有历史怪癖——大输出粗测
   的数字要按此解读。

## 详细设计

### spikes/pty-probe(bin)

- `--shell <exe>`(默认 `pwsh`,回退 `powershell`)、`--duration <秒>`
  (默认 3)、`--hexdump`;
- 流程:`portable-pty` 申请 PTY → spawn 子进程 → 主端读到超时为止,
  字节原样落 stderr(hexdump 可读形态)+ 统计字节率;
- 预期证据:输出含 `1b 5b`(ESC [)序列、字节率非零。

### spikes/term-probe(lib + bin + 集成测试)

- 封装 `Term::new` + 自定义 `EventListener`,`feed()` 喂字节流,定时
  快照网格可见文本(`renderable_content` 或等价 API)打印;
- 集成测试(真 PTY,非录制流):spawn `cmd /c echo hello_term_probe`
  (或 pwsh 等价),读到 EOF→feed→断言网格含 `hello_term_probe`;
- 这是仿真核心嵌入体验的第一手记录:API 是否顺手、resize 语义如何
  触发,写入决策文档附录。

### spikes/render-probe(bin,路线 A 探针)

- iced `Application`/新 API 等价物,自定义 widget:每帧从 term-probe
  封装读网格 → 按行列拼 monospace 文本渲染(spike 允许整帧重拼,不做
  字形图集——链路正确性优先,性能留给决策后的正式实现);
- 键盘:可打印字符/Enter/Backspace → 写 PTY 主端;resize → 通知仿真
  核心(观察 ConPTY resize 怪癖);
- `--shell <exe>` 必填,ash 冒烟经此入口:`--shell %ASH_BIN%`。

### 渲染路线判据(决策标准,写入 000 决策文档)

- **A 判据**:iced 下能以可接受代价实现——网格整帧渲染的延迟肉眼可
  接受、resize 无可感知撕裂、输入回显无粘滞;且自定义 widget API 无
  结构性障碍(能拿到每帧绘制权);
- **B 触发**:上述任一不成立,或 iced 帧调度/文本管线与终端需求冲突
  到需要 fork 上游才能解决;
- 决策文档记录:三 probe 证据、大输出数字(字节率、帧表现)、A/B 结论
  与理由、对 auto-ui 的反哺清单(等宽字形、损伤重绘、低延迟输入)。

### ash 冒烟清单(手动,render-probe 里完成)

- [x] `ash` 交互启动,提示符/补全正常;
- [x] 跑产生 tail 动态渲染的命令(如长输出外部命令),重绘不糊屏;
- [x] `find` 渐进表格不错位、冻结表格完整;
- [x] 真彩色/256 色样本正确(如 ash 内置着色输出)。

> 证据:`docs/designs/evidence/001-smoke/`(终态转储 + 流式快照序列 +
> PrintWindow 截图)。四项全部通过:提示符 `❯`+状态行正常;Tab 补全触发
> (补出历史命令);2000 行 0.37s 流式输出干净无糊屏、长输出摘要冻结生效
> (61–63+80 折叠);find 表格三行对齐、边框连续;截图验证 256 色/真彩
> run 渲染正确(find 绿/-name 蓝/*.md 琥珀/边框 240 灰/提示符绿),
> 修复右缘裁剪后边框与时钟完整。

## 测试设计

- 自动:term-probe 集成测试(真 PTY→仿真核心→网格断言)是本计划唯一
  自动断言,也是后续正式工程的回归种子;
- 半自动:pty-probe 字节率统计(人工核对非零 + ESC 序列存在);
- 手动:render-probe 交互冒烟 + ash 冒烟清单 + 大输出粗测记录数字;
- 明确不做:性能自动化基准(Phase 1+ 另立)、跨平台矩阵(仅 Windows)。

## 验收标准

1. 仓骨架存在:README(含定位与"零构建依赖"声明)、.gitignore、
   Cargo workspace、DEBTS.md、docs/plans、docs/designs;
2. `cargo tree --workspace` 无 `ash-core`/`auto-shell` 任何路径依赖;
3. pty-probe 在 Windows 基座读到非零字节率且含 ESC 序列;
4. term-probe 集成测试通过(网格中出现子进程输出);
5. render-probe 可交互:键入 `echo hi` 得到回显;resize 不崩;
6. ash 冒烟清单全勾;
7. `docs/designs/000-render-route.md` 存在且含 A/B 结论与证据数字;
8. spike 代码明确标注"非正式架构,允许整体重写"(README 或代码头注)。

## 执行步骤

- [x] **T1** 建仓骨架:写 `README.md`(定位声明:AutoOS 通用终端
      基础设施;与 auto-shell 仅运行时配置耦合;spike 代码非正式架构)、
      `.gitignore`(`target/`、`*.log`)、根 `Cargo.toml`(虚拟
      workspace,`members = []`,T2 起追加)。
      验证:`cd D:/autostack/auto-term && cargo metadata --no-deps >/dev/null && echo OK`
      [✅ 已完成] worktree `cargo metadata --no-deps` OK;README 含定位+零构建依赖声明+spike 重写声明;.gitignore 含 target/ 与 *.log;commit d20a8cc
- [x] **T2** 建 `spikes/pty-probe` crate(bin),依赖 `portable-pty`
      (最新稳定)、`anyhow`、`clap`;根 workspace `members` 追加
      `"spikes/pty-probe"`。
      验证:`cargo tree -p pty-probe --depth 1 | grep portable-pty`
      [✅ 已完成] portable-pty v0.9.0 + anyhow v1.0.104 + clap v4.6.6(derive);cargo tree 确认;commit de0c353
- [x] **T3** 实现 pty-probe:`--shell/--duration/--hexdump` 三参数,
      spawn 子进程、主端读到超时、hexdump 到 stderr + 字节率统计到
      stdout。
      验证:`cargo run -q -p pty-probe -- --duration 3` 输出字节率非零,
      stderr hexdump 中可见 `1b 5b`
      [✅ 已完成] pwsh 3s:bytes_total 27,9.0 B/s,hexdump 首行即 `1b 5b 31 74`;发现:pwsh 启动发 VT 查询(ESC[6n/ESC[c)等应答,嵌入方须回写;commit 同 T3
- [x] **T4** 建 `spikes/term-probe` crate(lib+bin):封装
      `alacritty_terminal`(`Term`+`EventListener`+`feed`+网格快照),
      bin 侧串起 pty-probe 同款 PTY 源,定时打印网格可见文本;
      集成测试 `tests/live_pty.rs`:spawn `cmd /c echo hello_term_probe`,
      断言网格含该串。
      验证:`cargo test -p term-probe`
      [✅ 已完成] 测试绿(live_pty_output_reaches_grid ok);bin 快照 1.51s 出现 `PS C:\Users\zhaop>` 提示符(应答回写 11B 后);嵌入体验结论已写入 lib.rs 头注(commit 同 T4)
- [x] **T5** 建 `spikes/render-probe` crate(bin):依赖 iced +
      term-probe(lib),自定义 widget 每帧读网格渲染 monospace 文本,
      键盘可打印键/Enter/Backspace 写 PTY 主端,窗口 resize 通知仿真
      核心。
      验证:`cargo build -p render-probe` 通过;手动 `cargo run -q -p
      render-probe -- --shell pwsh` 键入 `echo hi` 见回显
      [✅ 已完成] build 绿;auto-input "echo hi\r" 转储网格同时含 `echo hi` 与 `hi`(输入回显+命令输出);resize 2 次(113x32/86x25)不崩,6s 372 帧 ≈62fps;iced=0.14.0(tokio+advanced),需 `window::oldest()` 取窗 Id(Id::MAIN 已移除);commit 同 T5
- [x] **T6** ash 冒烟:构建/定位 ash 可执行(以 auto-shell 实际构建
      输出为准,如 `D:\autostack\auto-shell\ash\target\debug\ash.exe`,
      执行时核对),`cargo run -q -p render-probe -- --shell <ash.exe>`
      完成"ash 冒烟清单"四项并截图/记录存
      `docs/designs/000-render-route.md` 附录。
      验证:清单四项全勾,证据已记录
      [✅ 已完成] ash.exe=auto-shell/ash/target/release/ash.exe(release 版);四项全过,证据存 docs/designs/evidence/001-smoke/(截图+快照+转储);commit f7046f1
- [x] **T7** 大输出粗测:render-probe 里跑 `pwsh -c "1..20000 | %
      { $_ }"`(20000 行)与 ash 等价长输出,记录总耗时/末行到达延迟/
      滚动观感,数字记入决策文档(对照 ConPTY 翻译层预期解读)。
      验证:数字已在 000 文档,含至少一组对照(pwsh vs ash)
      [✅ 已完成] pwsh 直跑:130,571B,末行 1.334s(输入后),62fps;ash→pwsh:129,603B,1.271s——ash 透明无附加开销,瓶颈在 ConPTY 翻译层(≈100KB/s);滚动单调无糊屏;证据 evidence/001-smoke/bench-*.txt
- [x] **T8** 写 `docs/designs/000-render-route.md`:三 probe 证据、
      大输出数字、A/B 判据核对、结论与理由、对 auto-ui 反哺清单
      (等宽字形/损伤重绘/低延迟输入)。
      验证:`test -f docs/designs/000-render-route.md && grep -c
      "结论" docs/designs/000-render-route.md` ≥ 1
      [✅ 已完成] 文档落位,`grep -c 结论`=1;结论:方案 A(iced)成立,B 保留逃生口;commit 76426c7
- [x] **T9** 收尾:建 `DEBTS.md`(spike 已知债务:整帧重拼渲染、无
      字形图集、无滚动回滚 UI、仅 Windows、Ctrl+C/关闭语义未验);
      README 补"下一步"段(Phase 1 另立计划:单窗口 MVP+仿真正确性
      回归)。
      验证:`test -f DEBTS.md && ! cargo tree --workspace 2>/dev/null |
      grep -q ash-core && echo OK`
      [✅ 已完成] OK;注:cargo tree 里的 `ash` 是 wgpu 的 Vulkan 绑定同名 crate,`ash-core`/`auto-shell` 路径 0 条;DEBTS 含 10 项+观察项;README 下一步段已补;commit 395f1dd

## 复审记录

**复审人**:auto-plan:review(ZCode 会话)· **时间**:2026-09-05 03:40 +08:00
**复审场所**:worktree `D:/autostack/.wt/auto-term-001/auto-term`(分支
`plan-001-dev`,9 提交,工作区干净;diff 26 文件 +6136 行)

### 验收标准逐条复验(全部重跑,未采信打勾)

| # | 标准 | 结论 | 证据 |
| --- | --- | --- | --- |
| 1 | 仓骨架(README/.gitignore/workspace/DEBTS/docs) | **PASS** | 六项俱在;README 含定位+"零构建依赖"+"非正式架构"声明 |
| 2 | 依赖树无 ash-core/auto-shell | **PASS** | `cargo tree --workspace` grep 双零;注:wgpu 的 Vulkan `ash` crate 同名无关,已在 T9 证据注明 |
| 3 | pty-probe 非零字节率 + ESC 序列 | **PASS** | 复跑:9.0 B/s,`esc_seq_1b5b_found: true`,hexdump 首行 `1b 5b 31 74` |
| 4 | term-probe 集成测试通过 | **PASS** | full-suite `cargo test --workspace` 全绿,`live_pty_output_reaches_grid ok` |
| 5 | render-probe 可交互(echo 回显+resize 不崩) | **PASS** | 复跑:网格含 `PS> echo hi` 与 `hi`,两次 resize(113x32/86x25),373 帧无崩 |
| 6 | ash 冒烟清单全勾 | **PASS** | 复跑:提示符/状态行/`find` 表格(type/path 对齐)重现;四项执行期证据在 evidence/001-smoke(截图+快照+转储)完整 |
| 7 | 000 决策文档含 A/B 结论与数字 | **PASS** | `结论`×1,方案 A/B 表述×3,对照数字(130,571B/1.334s vs 129,603B/1.271s)在表 |
| 8 | spike 代码标注可整体重写 | **PASS** | "非正式架构,允许整体重写"×5(README+四个源文件头注) |

**Full-suite 门**(唯一一次,在本复审):`cargo test --workspace` 全绿。

### 遗漏 / 延后 / workaround 猎查

- **遗漏**:未发现。详细设计逐项对上 diff:pty-probe 三参数(含
  pwsh→powershell 回退代码)、term-probe lib 封装+bin 定时打印
  (--interval-ms)、render-probe `--shell` 必填、键盘/resize 链路、
  证据归档,全部在。
- **延后**:无未经批准的延后。Phase 1 清单(损伤重绘/字形图集/事件
  驱动/回滚 UI/Unix 基座)属计划内"非目标"+T9 DEBTS 明示。
- **Workaround(均已备案,不阻塞)**:
  1. 固定 Consolas advance + 窗口拟合替代实测字形度量(000 缺口#1、
     DEBTS#3);
  2. 16ms 轮询 tick 替代事件驱动(DEBTS#4);
  3. NamedColor 部分映射(Dim/Cursor/selection 落默认色,DEBTS#10);
  4. 手动 GUI 验收以 auto-input/--exit-after/--dump + PrintWindow 截图
    的等价取证替代人工键入(计划 T5/T6 证据条已注明)。
- **观察项(Phase 1 回归候选,非债务)**:备用屏幕(alt-screen)语义
  未被任何 probe 显式行使(如 `less` 全屏应用);pwsh→powershell
  回退分支为守护代码未实测;键盘非按键事件映射为 Tick 空操作(小瑕疵)。

### 结论

**通过 → `status: reviewed`**。8/8 标准全过,无阻塞债务;所有
workaround 均有备案且有 Phase 1 清偿路径。渲染路线 A(iced)结论
可信:判据四项均有重跑或像素级证据。可进入 `/auto-plan:merge`。

## 待澄清事项

1. **渲染路线 A/B 拍板**——本计划的核心产出即决策证据,T8 出结论;
2. **auto-plan 工具链移植**:本仓是否复制部署 plan backend/skills
   (auto-shell 那套),还是暂以纯文件流程跑计划——建议首两个计划
   纯文件,惯例固化后再定;
3. **iced 与 auto-ui/auto-lang VM 的集成路径**:路线 A 成立后,`.at`
   组件模型能否承载自定义原生 widget(需与 auto-ui 仓对齐,Plan 002+
   前置调查);
4. **Windows 最低版本口径**:ConPTY 要求 Win10 1809+,README 支持声
   明待定;
5. **ash.exe 构建产物路径**:以 auto-shell 实际输出为准,T6 执行时
   核对后回填本计划。
   → 已回填:T6 使用 `D:\autostack\auto-shell\ash\target\release\ash.exe`
   (release 构建已存在,直接消费)。
