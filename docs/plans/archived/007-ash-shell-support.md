---
plan_id: PLAN-007
status: archived
feature_name: ash (AutoShell) 独立运行验证与测试门禁
author: [衍星居士]
created_at: 2026-09-06
updated_at: 2026-09-06

# /auto-plan:review 时填写
supersedes_spec_components:
  - "DEBTS.md: 修改(#7 行交叉引用更新,#12/#13 新增,头部账期)"
  - "README.md: 修改(运行节新增'用 ash 跑'小节)"
new_spec_components:
  - "docs/designs/003-ash-compatibility.md: 新增(ash 兼容性结论:场景矩阵、R1-R6 裁定、F1/F2 证据链与候选修复通路、虚拟桌面接入契约、遗留)"
  - "crates/autoterm-core/tests/ash_integration.rs: 新增(ash 门禁 7 用例 + 2 个 F2 复现器 #[ignore],AUTOTERM_ASH_BIN/兄弟仓三级探测门控)"
touched_goals:
  - "ash(AutoShell)独立运行验证:从手动清单升格为必跑自动门禁(ash 在场即断言,缺席显式 skip)"
  - "DEBTS #7(Ctrl+C/关闭语义)推进:事件断裂由'待复测'坐实为 #12 确认缺陷,win32 GenerateConsoleCtrlEvent 直调方向确认"

current_step: 9
total_steps: 9
---

# [PLAN-007] ash (AutoShell) 独立运行验证与测试门禁

## 变更摘要

AutoTerm 的既定使命是**在 AutoOS 虚拟桌面里跑 ash**(README 定位声明;PLAN-001
"ash 只作为运行时配置"),但至今的自动化验证实际落在 **cmd 与 pwsh** 上:
`live_pty.rs` 只 spawn `cmd`,dev 冒烟钩子默认 `--shell pwsh`
(`crates/autoterm-ui/src/bin/autoterm.rs:15`),ash 仅出现在 002/003 的
**手动**试用清单与 003 的三 shell 中断冒烟转储里——没有一道**必跑的自动门禁**。

本计划补上这道门禁:在 `autoterm-core` 新增 **ash 专属集成测试**
(`tests/ash_integration.rs`,env 门控 + 兄弟仓路径自动探测,ash 缺失时明确
skip 而非静默通过),覆盖启动提示符、echo 回显、Ctrl+C 中断、exit/Ctrl+D
退出、resize 存活、色彩深度自证六个场景;外加三条 UI 级半自动冒烟配方
(含 `color rainbow` 真彩取证);结论沉淀为
`docs/designs/003-ash-compatibility.md`(R1–R6 风险逐项结论)。**不改
autoterm-core/ui 产品代码**——测试暴露的缺陷按证据记 DEBTS,修复另立
(≤5 行 trivial 例外,见待澄清#2)。

## 目标

1. **ash 集成测试门禁**:`cargo test -p autoterm-core --test ash_integration`
   在 ash.exe 可得时是必绿回归的一部分(6 用例),不可得时输出明确 skip
   说明——不再依赖"记得手动试一下 ash";
2. **六个核心场景硬断言**(真 PTY → 仿真核心 → 网格):
   ① 裸 `ash` 启动后提示符渲染(网格出现非空行;间接行使 DSR/DA 应答
   路径 `term.pump` 回写);② `echo <marker>` 往返出现在网格;
   ③ Ctrl+C(0x03)中断 `sleep 30` 后提示符回归、子进程存活;
   ④ `exit` 命令与 Ctrl+D(0x04)均令 `exited()` 为真(无孤儿);
   ⑤ resize 80x24→120x40 后会话存活且继续响应;
   ⑥ `color info` 在 TERM=alacritty + COLORTERM=truecolor 下自证
   "24-bit truecolor"(ash 的色彩判定落在网格文本里,可直接断言);
3. **UI 级半自动取证**:三条 `--shell <ash>` dev 冒烟配方(echo 往返 /
   真彩彩虹 + 深度报告 / Ctrl+C 中断),`--dev-dump` 证据引述进设计文档;
4. **结论沉淀**:`docs/designs/003-ash-compatibility.md` 记录场景矩阵
   结果与 R1–R6 风险逐项结论,发现的缺陷全部进 DEBTS(带 file:line);
5. **全量回归**:`cargo test --workspace` 绿(ash 缺席环境无回归)。

### 非目标(明确排除)

- 不改 auto-shell 仓任何代码(ash 侧问题记 DEBTS,由该仓处理);
- 不做虚拟桌面侧集成(ash 作为默认 shell 的拉起配置是 auto-os 侧计划);
- 不做性能基准、Unix 基座、tabs/splits;
- 不重写 live_pty.rs 为多 shell 矩阵(cmd 基线保留,ash 门禁独立成文件,
  避免把"永远可得"与"需环境"两类测试耦死)。

## 架构方案

**零产品代码改动的纯测试增项**,落在既有分层内:

```
crates/autoterm-core/tests/
├── live_pty.rs          # 既有 cmd 基线,不动
├── sim_regression.rs    # 既有纯 VT 回归,不动
└── ash_integration.rs   # 新增:ash 专属门禁(env 门控)
```

- **二进制定位**(`ash_bin()`):`AUTOTERM_ASH_BIN` 环境变量优先 →
  兄弟仓约定路径探测 `../auto-shell/ash/target/{release,debug}/ash.exe`
  (相对 auto-term 仓根)→ 皆无则用例体首行 `eprintln!` skip 说明 +
  `return`(不是 `#[ignore]`,保留"跑过即验证过"的语义);
- **断言通路**:复用 `PtySession::spawn/drain/write_input/exited/resize`
  与 `TermSession::visible_lines()`(term.rs:213,PLAN-002 已公开),
  不新增公共 API;
- **等待原语**:`wait_for(session, timeout, pred)` 轮询 `drain()` +
  50ms 小睡,超时 panic 并附当前网格文本(诊断友好);统一超时:首提示符
  ≤15s(ash 首启会建 `~/.ashrc`,冷启动偏慢),命令级 ≤10s;
- **UI 半自动**:既有 `--dev-autotype/--dev-exit-after/--dev-dump`
  钩子原样使用,证据文件落 `target/`(gitignore 内),关键行引述进
  设计文档。

## 技术栈

全部复用现有依赖(portable-pty、anyhow),**零新依赖**;测试即
`cargo test` 标准集成测试。

## 需求分析与背景调查

### spec 依据(总览)

- **P001-2 目标#4**:ash 定位为"苛刻 VT 客户端"冒烟对象;**P001-1**:
  "ash 只作为运行时配置(默认 shell 路径),不进依赖树"——本计划严格
  遵守(测试只引用二进制路径,不建构建依赖);
- **P002-5 / P003-5**:ash 仅进**手动**试用清单(pwsh/ash 各 10/5 分钟)
  与 IME 人工清单;
- **P003-2 目标#2**:pwsh/ash/cmd × {Ctrl+C, 窗口关闭} 中断矩阵——
  半自动转储,是 ash 至今唯一的自动化痕迹,且非必跑门禁;
- **P006-1**:Auto 化调查结论(虚拟桌面 iced 轨道)进一步抬高了
  "ash 必须在 AutoTerm 里真实可跑"的优先级——组件化(TermGrid)之前,
  先把行为基线钉死。

### 现状证据(file:line)

| 事实 | 证据 |
|---|---|
| 默认 shell = pwsh | `crates/autoterm-ui/src/bin/autoterm.rs:15` |
| 自动测试只 spawn cmd | `crates/autoterm-core/tests/live_pty.rs:13` |
| PTY 会话层 shell 无关(TERM=alacritty/COLORTERM=truecolor 自报家门) | `crates/autoterm-core/src/pty.rs:62-64` |
| 网格断言 API 已公开 | `crates/autoterm-core/src/term.rs:213`(`visible_lines`) |
| DEBTS 无任何 ash 条目 | `DEBTS.md`(grep 无命中) |

### ash 形态(D:\autostack\auto-shell\ash,本次调查实测)

- Rust 实现,`--version` 正常(exit 0);裸 `ash` 进交互 REPL,
  `ash -c <cmd>` 可单命令执行(实测 `echo`/`sleep` 均可用,`sleep 1`
  实测 1.026s);
- 行编辑器 **reedline 0.44 + crossterm 0.29**,叠加
  **ratatui-crossterm(scrolling-regions feature)**
  (`ash/Cargo.toml:32-44`)——DECSTBM/SU/SD 滚动区序列是真实风险面;
- 色彩深度由 `ASH_TERM24BIT → COLORTERM → TERM` 链判定
  (`ash/src/frontend/term/color.rs:26-55`);内置 `color info`
  (打印 "Color depth: … (COLORTERM=…, TERM=…)")与
  `color rainbow`(逐字符真彩)——**ash 自带的取证命令**
  (`ash/src/frontend/commands.rs:85-110`);
- 首启自动创建 `~/.ashrc`(帮助文本明示)——首提示符延迟需容忍。

### 风险清单(为什么 pwsh 测过 ≠ ash 能跑)

| # | 风险 | 本计划对应 |
|---|---|---|
| R1 | reedline 行编辑的按键编码依赖(Home/End/Delete/Ctrl+箭头回显) | 场景①②间接覆盖;深度按键矩阵留后续 |
| R2 | ratatui scrolling-regions(DECSTBM/SU/SD)在仿真核心+损伤渲染下的正确性 | 场景①②⑤过即基线成立;专项视觉取证留 UI 冒烟 |
| R3 | 启动期终端探测(DA/DSR 等)应答集合是否满足 ash | 场景①:提示符不渲染即暴露 |
| R4 | TERM=alacritty 是否被 ash 识别为全彩 | 场景⑥:`color info` 文本自证 |
| R5 | exit/Ctrl+D 后 ConPTY 不发 EOF(pty.rs:10-14 平台事实),退出检测必须走 `exited()` | 场景④ |
| R6 | ash.exe 不在 PATH,虚拟桌面将显式传绝对路径 | 门控机制天然覆盖(定位即注入路径) |

## 详细设计

### `crates/autoterm-core/tests/ash_integration.rs` 结构

```rust
//! ash (AutoShell) 集成门禁:真 PTY → 仿真核心 → 网格断言。
//! 门控:AUTOTERM_ASH_BIN 优先,兄弟仓约定路径兜底,皆无则 skip。

fn ash_bin() -> Option<PathBuf>;           // env → ../auto-shell/.../ash.exe 探测
fn grid_text(s: &PtySession) -> String;    // visible_lines().join("\n")
fn wait_for<F: Fn(&TermSession) -> bool>(  // 轮询 drain + 判定
    s: &mut PtySession, timeout: Duration, what: &str, pred: F,
);  // 超时 panic 附 grid_text 快照

#[test] fn spawn_prompt_renders()          // ① 非空行出现 + bytes_fed > 0
#[test] fn echo_roundtrip()                // ② marker 出现于网格
#[test] fn ctrl_c_interrupt_returns_prompt() // ③ sleep 30 + 0x03 + after-marker ≤10s
#[test] fn exit_command_terminates()       // ④a "exit\r" → exited() ≤10s
#[test] fn ctrl_d_eof_terminates()         // ④b 0x04(至多两次)→ exited()
#[test] fn resize_survives()               // ⑤ 120x40 + marker2 响应
#[test] fn color_info_reports_truecolor()  // ⑥ 网格含 "24-bit truecolor"
```

要点:

- 每个用例独立 `PtySession::spawn(bin, [], 80, 24)`(ash 裸跑即 REPL,
  无需参数;`--shell` 只收程序名的设计不受影响);`Drop` 兜底杀进程,
  用例内不显式清理也安全;
- marker 用随机后缀(如 `ash_probe_{pid}`)防串扰;
- 场景③判定逻辑:`sleep 30` 阻塞下,后续 `echo after_c` 的 marker 若能
  在 ≤10s 内出现,即证明中断生效(sleep 30 不可能自然结束);
- 场景⑥若实测**不是** 24-bit:不静默改断言迎合——原样断言失败,
  执行记录里按发现处理(进 open-questions + DEBTS + 设计文档 R4 结论),
  断言修正为实测值时必须附注释引用 DEBTS 条目号。

### UI 半自动冒烟配方(T7 执行时落成三条可复制命令)

```bash
ASH=D:/autostack/auto-shell/ash/target/release/ash.exe
# A echo 往返
cargo run -p autoterm-ui --features dev-tools -- --shell $ASH \
  --dev-autotype "4000:echo ash_ui_probe\r" --dev-exit-after 12 \
  --dev-dump target/ash-smoke-a.txt
# B 真彩取证(color rainbow + color info)
cargo run -p autoterm-ui --features dev-tools -- --shell $ASH \
  --dev-autotype "4000:color rainbow\r" "6500:color info\r" \
  --dev-exit-after 14 --dev-dump target/ash-smoke-b.txt
# C Ctrl+C 中断(sleep 30 + \x03 + 复验 marker)
cargo run -p autoterm-ui --features dev-tools -- --shell $ASH \
  --dev-autotype "4000:sleep 30\r" "6000:\x03" "8000:echo after_c\r" \
  --dev-exit-after 16 --dev-dump target/ash-smoke-c.txt
```

证据核对面(dump 含网格文本 + 损伤/帧计数,lib.rs:597-654):
A 含 `ash_ui_probe`;B 含 `Ash 24-bit Truecolor Rainbow!` 与
`Color depth: 24-bit truecolor`;C 含 `after_c` 且 uptime < 15s。
首提示符延迟超 4s 时按 dump 实况回调各段延迟(执行记录里注明)。

### `docs/designs/003-ash-compatibility.md` 骨架

场景矩阵表(6+3 场景 × 结果/证据)、R1–R6 逐项结论、ash 冷启动数字、
遗留问题(→ DEBTS 编号)、虚拟桌面接入契约提醒(显式绝对路径)。

## 测试设计

- **自动**(必跑门禁):`ash_integration.rs` 七个用例;ash 缺席时输出
  `skip: AUTOTERM_ASH_BIN unset, sibling ash.exe not found`(用例体
  return,测试仍算过——skip 语义显式打印,不假绿:断言只可能因
  "ash 在场且行为不符"而失败);
- **半自动**:三条 UI 配方,证据引述进 003 文档(002/003 惯例:
  程序化证据为主,截图不作验收依赖);
- **手动**:保留"ash 日用 5 分钟"清单(补全/表格/Ctrl+C/关闭无残留),
  仅作发布前人工复核,不构成本计划验收;
- **不做**:性能基准、多 shell 参数化矩阵重构、Unix。

## 验收标准

1. `AUTOTERM_ASH_BIN` 指向 ash.exe 时
   `cargo test -p autoterm-core --test ash_integration` 全绿(7 用例);
   指向不存在路径或未设且兄弟仓无产物时,7 用例均打印 skip 说明后通过;
2. 场景覆盖齐:①②③④a④b⑤⑥ 各自独立断言,不互相掩盖;
3. `cargo test --workspace` 绿(ash 缺席环境复跑确认无回归);
4. `docs/designs/003-ash-compatibility.md` 存在:场景矩阵 + R1–R6 逐项
   结论 + UI 配方证据引述;
5. 执行中发现的所有 ash×AutoTerm 缺陷均已记入 DEBTS.md(file:line
   证据),没有"口头发现";
6. 零产品代码改动(trivial 修复除外,且须在复审记录列明 diff 行数)。

## 执行步骤

> 约定:全程在 worktree `.wt/auto-term-007/auto-term`(由
> /auto-plan:work 创建);ash.exe 用
> `D:/autostack/auto-shell/ash/target/release/ash.exe`(执行时若只有
> debug 产物,路径换 debug 并在记录注明)。

- **T1** 建门禁骨架:新建
  `crates/autoterm-core/tests/ash_integration.rs`,实现 `ash_bin()`
  (AUTOTERM_ASH_BIN → `../auto-shell/ash/target/release/ash.exe` →
  `…/debug/ash.exe` → None)、`grid_text`、`wait_for`(超时 panic 附
  网格快照),及用例① `spawn_prompt_renders`(≤15s 等非空行,
  断言 `bytes_fed() > 0`)。
  验证:`AUTOTERM_ASH_BIN=<ash.exe> cargo test -p autoterm-core --test ash_integration -- --nocapture`
  用例①绿;去掉 env 后单跑①输出 skip 行。
  [✅ 已完成] ①绿(0.11s,提示符即渲染,bytes_fed>0);无 env 时三级向上探测在
  worktree 自动命中 `../../../auto-shell/.../release/ash.exe`;env 指向不存在路径
  时打印 skip 行通过(缺席不假绿)。探测层级为主仓/worktree 双布局的落地,
  比草稿的"仅 ../ 一级"更完整。

- **T2** 用例② `echo_roundtrip`:等提示符 → `write_input(b"echo ash_probe_xxx\r")`
  → `wait_for` 网格含 marker(≤10s)。
  验证:同上命令过滤 `echo_roundtrip` 绿。
  [✅ 已完成] echo_roundtrip 绿(0.16s,marker=ash_probe_<pid> 上屏)。

- **T3** 用例③ `ctrl_c_interrupt_returns_prompt`:提示符 →
  `sleep 30\r` → 等 "sleep 30" 上屏 → `\x03` → `echo after_c\r` →
  marker ≤10s 出现;断言 `!exited()`。
  验证:过滤 `ctrl_c` 绿,全程墙钟 < 20s。
  [✅ 已完成,但语义按发现重构,见待澄清#5] 原语义("中断运行中命令")
  在通路层以下断裂:发现 F2(0x03 经 portable-pty ConPTY 主端不触发
  CTRL_C_EVENT,cmd/pwsh 对照探针同样失败,与 shell 无关)+ 叠加
  发现 F1(ash 内建命令全程 raw mode 不读 stdin,`sleep` 不可中断,
  auto-shell sleep.rs:35)。门禁改守终端层可负责的语义:
  `ctrl_c_aborts_input_line`(0x03 转发正确 → reedline 废弃输入行 →
  新提示符 → 后续命令可跑 → shell 存活),0.36s 绿;F2 复现器
  (cmd/pwsh)保留为 `#[ignore]` 用例,修 DEBTS 后去 ignore 验证。

- **T4** 用例④a `exit_command_terminates` + ④b `ctrl_d_eof_terminates`:
  "exit\r" → 轮询 `exited()` ≤10s;新会话 0x04(空行,至多两次,间隔
  500ms)→ `exited()` ≤10s。
  验证:过滤 `terminates` 两用例绿。
  [✅ 已完成] exit_command_terminates + ctrl_d_eof_terminates 均绿
  (exit 走 try_wait 正路径;Ctrl+D 一次即退,未动用二次确认)。

- **T5** 用例⑤ `resize_survives`:提示符 → marker1 →
  `resize(120, 40)` → 断言 `term.size() == (120, 40)` →
  `echo marker2\r` → marker2 ≤10s 上屏。
  验证:过滤 `resize` 绿。
  [✅ 已完成] resize_survives 绿:80x24→120x40 仿真核心尺寸同步,
  resize 后 marker 继续上屏,shell 存活。

- **T6** 用例⑥ `color_info_reports_truecolor`:`color info\r` →
  网格含 "24-bit truecolor" ≤10s(不符预期时按详细设计§场景⑥的
  发现流程处理,不静默迎合)。
  验证:过滤 `color_info` 绿(或产出 DEBTS + 修正注释)。
  [✅ 已完成] color_info_reports_truecolor 绿,R4 风险排除。取证行:
  `Color depth: 24-bit truecolor (COLORTERM=truecolor, TERM=alacritty)`。

- **T7** UI 半自动冒烟:跑详细设计§三条配方,dump 落
  `target/ash-smoke-{a,b,c}.txt`;逐条核对证据面(A/B/C 各自的关键行);
  延迟若需调整,实况记入执行记录。
  验证:三个 dump 文件存在且 `grep` 命中各自关键行
  (`ash_ui_probe` / `Truecolor Rainbow` + `24-bit truecolor` / `after_c`)。
  [✅ 已完成,配方语法与 C 预期按实况修正] A:dump 含 `❯ echo ash_ui_probe`
  + 输出行(2 次命中);B:含 `Ash 24-bit Truecolor Rainbow!` 与
  `Color depth: 24-bit truecolor (COLORTERM=truecolor, TERM=alacritty)`;
  C:按 F2 结论预期翻转——`after_c` **0 次命中**,`❯ sleep 30` 冻结至
  16s dump(UI 层 F2 证据,与 core 探针一致)。语法修正:多段注入需
  重复 `--dev-autotype` 标志(clap Vec 按出现次数收集,单标志空格
  多值不被接受)。dump 落 worktree target/(不进版本库,证据引述进
  003 文档)。

- **T8** 撰写 `docs/designs/003-ash-compatibility.md`(矩阵表 +
  R1–R6 结论 + 冷启动数字 + T7 证据引述 + 遗留→DEBTS);同步
  DEBTS.md(新发现条目)与 README("用 ash 跑 AutoTerm"一行快速上手,
  含绝对路径示例)。
  验证:文档/README/DEBTS diff 审阅;DEBTS 新条目均带 file:line。
  [✅ 已完成] designs/003(速览/矩阵/R1-R6 裁定/F1+-F2 全证据链/虚拟桌面
  接入契约/遗留)落稿;DEBTS 新增 #12(F2,通路层,含候选修复通路)
  与 #13(F1,ash 侧跨仓协调),#7 行加 007 交叉引用,头部账期更新;
  README 增"用 ash 跑"小节(运行命令 + 门禁回归 + 003 指引)。

- **T9** 全量回归:`cargo test --workspace`(ash 在场)全绿;临时
  `AUTOTERM_ASH_BIN=Z:/nonexistent.exe` 复跑 `--test ash_integration`
  确认 7 用例 skip 而非 fail。
  验证:两次命令输出记录进执行记录。
  [✅ 已完成] `cargo test --workspace`(worktree,ash 自动探测在场):
  16 套件全 ok、0 FAILED(ash_integration 7 过 + 2 ignore);
  `AUTOTERM_ASH_BIN=Z:/nonexistent.exe` 复跑 ash_integration:
  7 用例各打印 1 行 skip 说明(共 7 行)后通过,非 fail。

## 复审记录

**Reviewer**:ZCode(auto-plan:review)· 2026-09-06 · 复验全部在 worktree
`.wt/auto-term-007/auto-term`(branch `plan-007-dev`,5 commits)内重跑,
不信勾选框。

### 验收逐条(全部 pass)

| # | 验收 | 裁定 | 证据 |
| --- | --- | --- | --- |
| 1 | env 指向 ash 全绿 / 缺席显式 skip | **pass** | `AUTOTERM_ASH_BIN=<release/ash.exe>` → 7 passed, 0 failed, 2 ignored(0.37s);`AUTOTERM_ASH_BIN=Z:/nonexistent.exe` → 7 用例各打印 1 行 skip 后通过(0 fail) |
| 2 | ①②③④a④b⑤⑥ 独立断言 | **pass**(③语义重构,见下) | 9 个 `#[test]` = 7 门禁 + 2 复现器;每用例独立 spawn + pid 后缀 marker,互不掩盖 |
| 3 | workspace 绿 + 缺席无回归 | **pass** | `cargo test --workspace`(worktree,ash 自动探测在场):全 suite 0 FAILED(合计 31 passed);缺席模拟即验收#1b |
| 4 | 003 文档三要件 | **pass** | `docs/designs/003-ash-compatibility.md` 六节俱全(速览/矩阵 2.1+2.2/R1-R6/F1+F2/接入契约/遗留),UI 证据行原文引述 |
| 5 | 缺陷全记 DEBTS(file:line) | **pass** | #12(F2,通路层)+ #13(F1,ash 侧,`cmd/commands/sleep.rs:35` 等);#7 行加交叉引用;无口头发现 |
| 6 | 零产品代码改动 | **pass** | `git diff 937d094..HEAD --stat`:仅 4 文件(DEBTS/README/tests/003),无 src/ 改动;无 trivial 修复需要列明 |

文件编码复核:003 与 DEBTS 经 Python 字节级校验为合法 UTF-8(管道乱码
为显示层伪象,未改动的原始行同样乱码可证)。

### 计划↔实现分歧(全部已在执行记录/003 中明示,无静默)

1. **T3 语义重构**(最重要):原"Ctrl+C 中断运行中命令"在通路层以下断裂
   (F2),AutoTerm 代码层不可修;门禁改守终端层语义(0x03 转发/废弃
   输入行),原语义复现器留档 `#[ignore]`。**待用户裁定(待澄清#5),
   修复另立计划(DEBTS #12)** —— 若用户不认可此重构,回 /auto-plan:work
   改回原语义(则该用例将常红直至 #12 修复)。
2. `ash_bin()` 探测三级(`../`、`../../`、`../../../`)vs 草稿一级:
   覆盖主仓+worktree 双布局,T1 证据已注明。
3. UI 配方多段注入需重复 `--dev-autotype` 标志(clap Vec 语义),003
   §2.2 已按实况修正命令文本。
4. 场景⑥"非 24-bit 则按发现流程"分支未触发(实测即 truecolor)。

### 遗漏/延后/workaround 猎查

- 遗漏:无——9 任务均有对应 diff 与验证输出。
- 延后(均有记录、经计划规则允许):F1 修复归 auto-shell 仓(计划非目标
  "不改 auto-shell");R1 深按键矩阵、R2 滚动区视觉专项在计划目标之外,
  003 §6 列为候选;F2 修复另立(待澄清#5 明示待裁定,非静默)。
- Workaround:产品代码零改动、零 TODO/魔改;唯一"降级"即分歧#1,已
  显式记录并留复现器。

### 债务候选(复审新增,无)

#12/#13 已在 T8 记档;R1/R2 深度覆盖为候选改进非债务。

**路由:pass → status: reviewed**,可 `/auto-plan:merge`。

## 待澄清事项

1. **ash 二进制定位策略**:默认 env 优先 + 兄弟仓路径兜底 + 缺席 skip。
   若虚拟桌面集成对路径有既定约定(如注册表/配置文件),T1 的探测顺序
   可在执行时对齐——默认不动。
2. **缺陷修复边界**:默认本计划只记证(DEBTS + 003 文档),修复另立
   计划;例外——≤5 行且不改变语义的 trivial 修复(如测试自身缺陷)
   可顺手修,复审记录列明。
3. **UI 冒烟的验收地位**:默认"半自动 + 证据引述入 003 文档",不阻塞
   计划关账(与 002/003 惯例一致)。
4. **虚拟桌面侧集成**(ash 为默认 shell 的拉起契约)不在本计划,建议
   后续在 auto-os 侧另立;本计划 003 文档的"接入契约提醒"节为其留锚点。
5. **[执行中发现,待用户裁定] F2:Ctrl+C 无法中断运行中的命令,且与
   shell 无关**。证据:①`cmd /c ping -n 30` + 0x03 → 5s 不退、无 ^C;
   ②交互 pwsh + ping + 0x03 → 10s marker 不上屏;③portable-pty 0.9.0
   spawn 无 CREATE_NEW_PROCESS_GROUP(排除进程组嫌疑),主端写管道
   无事件注入 API。根因落在 OS conhost 对 VT 输入 0x03 的控制事件翻译
   (Microsoft Q&A/wintty#155/winpty#116 记录的灰色地带),AutoTerm
   代码层无法就地修复。处置:T3 门禁改守"0x03 转发/废弃输入行"语义
   (已绿),原语义复现器转 `#[ignore]` 留档,F2 进 DEBTS(候选修复
   通路:OS conhost 版本差异调查 / GenerateConsoleCtrlEvent 注入辅助
   进程 / 上游 portable-pty)。**虚拟桌面影响:AI/用户无法打断挂死
   的命令——建议尽快另立修复计划。** 关联发现 F1(ash 内建命令不可
   中断,auto-shell 侧,已按"不改依赖仓"规则只记证)。
