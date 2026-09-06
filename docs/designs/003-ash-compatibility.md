# 003 · ash (AutoShell) 兼容性结论

> PLAN-007 产出 · 2026-09-06。
> 主题:auto-term × ash(虚拟桌面目标 shell)的**独立运行验证**——
> 此前自动化只覆盖 cmd(live_pty)与 pwsh(默认 shell),ash 仅存于
> 手动清单。本文记录场景矩阵结果、R1–R6 风险裁定、两项执行中发现
> (F1/F2)及虚拟桌面接入契约。
>
> 决策链:000(渲染路线)→ 001(正式架构)→ 002(Auto 化可行性);
> 本文是 ash 轨道的首个系统性取证。

## 1. 结论速览

| 维度 | 结论 |
| --- | --- |
| ash 在 AutoTerm 里**能不能跑** | **能**。7 个 core 级门禁用例全绿(启动/echo/Ctrl+C 废弃行/exit/Ctrl+D/resize/色彩自证),UI 级三配方取证通过 |
| 色彩 | ash 认 `TERM=alacritty` + `COLORTERM=truecolor` 为 24-bit 真彩(`color info` 网格文本自证) |
| 退出语义 | `exit` 与 Ctrl+D(0x04)均正路径退出(`try_wait`,无孤儿) |
| resize | 80x24→120x40 核心尺寸同步,REPL 存活且继续响应 |
| **Ctrl+C 中断运行中命令** | **已修复(008,2026-09-06)**:辅助进程广播控制事件(Break→C 双投递),`cmd/pwsh` 运行中命令可中断(UI 铁证);ash 受 F1 限制 auto 豁免为仅字节;残留缺口 = ping 类(Break 免疫)与 ash 内建,见 §4.1 |
| 冷启动 | 热机(release 产物,`~/.ashrc` 已存在)下 spawn→提示符 < 100ms(全套 7 用例含双 spawn 共 0.36s) |

## 2. 场景矩阵

### 2.1 core 级(`crates/autoterm-core/tests/ash_integration.rs`,必跑门禁)

| # | 用例 | 断言 | 结果 |
| --- | --- | --- | --- |
| ① | `spawn_prompt_renders` | 裸 ash 启动后网格出现非空行(间接行使 DSR/DA 应答回写);`bytes_fed>0` | ✅ 0.11s |
| ② | `echo_roundtrip` | 键入行(reedline 自渲染)+ echo 输出双双上屏 | ✅ 0.16s |
| ③ | `ctrl_c_aborts_input_line` | 0x03 → 废弃当前输入行 → 新提示符(❯ 计数+1)→ 后续命令可跑 → shell 存活 | ✅ 0.36s |
| ④a | `exit_command_terminates` | `exit\r` → `exited()` ≤10s | ✅ |
| ④b | `ctrl_d_eof_terminates` | 空行 0x04 → `exited()` ≤10s(实测一次即退) | ✅ |
| ⑤ | `resize_survives` | resize 后 `term.size()==(120,40)`,marker 继续上屏,存活 | ✅ |
| ⑥ | `color_info_reports_truecolor` | 网格含 `24-bit truecolor` | ✅ `Color depth: 24-bit truecolor (COLORTERM=truecolor, TERM=alacritty)` |
| ⑦ | `ctrl_c_event_effect_on_ash` | 事件注入下 ash 三态矩阵(idle/builtin/external,析取断言) | ✅ 008,见 §4.1 |

门控:`AUTOTERM_ASH_BIN` env 优先 → 兄弟仓约定路径探测
(`../auto-shell`、`../../`、`../../../auto-shell`,主仓与 `.wt/<grp>/<repo>`
worktree 布局都覆盖)→ 皆无则打印 skip 说明(缺席不假绿)。

### 2.2 UI 级(半自动配方,`--dev-autotype/--dev-dump`)

| 配方 | 内容 | 证据(dump 关键行) | 结果 |
| --- | --- | --- | --- |
| A | echo 往返 | `❯ echo ash_ui_probe` + 输出行 `ash_ui_probe`(2 次命中) | ✅ |
| B | 真彩 | `Ash 24-bit Truecolor Rainbow!` + `Color depth: 24-bit truecolor (…)` | ✅ |
| C | sleep 30 + 0x03 + echo after_c | `after_c` **0 次命中**,`❯ sleep 30` 冻结至 16s dump | ⚠️ 007 时点 F2 之 UI 级证据(008 后:ash 内建仍惰性,同结果;修复证据改由 cmd 主体,见 §4.1) |

复跑命令(多段注入须重复 `--dev-autotype` 标志;clap Vec 按**出现次数**
收集,单标志空格多值不被接受):

```bash
ASH=D:/autostack/auto-shell/ash/target/release/ash.exe
./target/debug/autoterm.exe --shell $ASH \
  --dev-autotype "4000:echo ash_ui_probe\r" \
  --dev-exit-after 12 --dev-dump target/ash-smoke-a.txt
./target/debug/autoterm.exe --shell $ASH \
  --dev-autotype "4000:color rainbow\r" \
  --dev-autotype "6500:color info\r" \
  --dev-exit-after 14 --dev-dump target/ash-smoke-b.txt
./target/debug/autoterm.exe --shell $ASH \
  --dev-autotype "4000:sleep 30\r" \
  --dev-autotype "6000:\x03" \
  --dev-autotype "8000:echo after_c\r" \
  --dev-exit-after 16 --dev-dump target/ash-smoke-c.txt
```

## 3. R1–R6 风险裁定

| # | 风险(计划期预判) | 裁定 | 依据 |
| --- | --- | --- | --- |
| R1 | reedline 按键编码依赖 | **基线成立**;深按键矩阵(Home/End/Delete/Ctrl+箭头/词跳转)未覆盖 | ①②③:普通键入、回车、0x03/0x04 控制键全部正确往返 |
| R2 | ratatui scrolling-regions(DECSTBM/SU/SD) | **基础路径已验证**;长回滚/菜单 overlay 的专项视觉取证遗留 | ⑤ + ②:每命令后 REPL 行内视口重绘未错位,网格断言全过 |
| R3 | 启动期终端探测应答不足 | **排除** | ①:提示符即刻渲染(ash 若有查询,应答路径已满足) |
| R4 | TERM=alacritty 不被识别为全彩 | **排除** | ⑥:`24-bit truecolor (COLORTERM=truecolor, TERM=alacritty)` |
| R5 | ConPTY 无 EOF 下的退出检测 | **排除** | ④a/④b:`try_wait` 路径两条退出通道都干净 |
| R6 | ash.exe 不在 PATH | **排除** | 全部场景显式绝对路径注入(门控机制天然覆盖) |

## 4. 执行中发现

### F1 · ash 内建命令不可被 Ctrl+C 中断(ash 侧)

- **现象**:REPL 跑 `sleep 30`(内建)期间发 0x03,10s 内无任何效果,
  后续键入的命令也不上屏(排队到内建结束才被消费)。
- **机理**:内建命令在 ash 进程内执行,全程保持 raw mode,ash 阻塞在
  `std::thread::sleep`(`auto-shell/src/cmd/commands/sleep.rs:35`)不读
  stdin;0x03 只是输入缓冲里的一个键事件。外部子进程路径才会临时退
  raw mode(`ash/src/frontend/subprocess.rs:43`)交给 conhost 事件机制。
- **归属**:auto-shell 仓(与终端无关,Windows Terminal 下同样如此)。
  auto-term 侧不做修复;建议 auto-shell 侧评估(内建执行期读 stdin 或
  定期轮询 0x03)。
- **DEBTS**:见 #13。

### F2 · 0x03 经 ConPTY 主端写入不触发 CTRL_C_EVENT(通路层,影响所有 shell)

- **现象**:
  1. `cmd /c ping -n 30` + 0x03 → 5s 不退出、无 `^C`(纯 cooked 对照);
  2. 交互 pwsh + `ping -n 30` + 0x03 → 10s 内后续 marker 不上屏;
  3. UI 级配方 C 同样复现(2.2)。
  即:**cmd、pwsh、ash 一视同仁**,此前 pwsh 基测试从未覆盖此语义
  (空闲提示符的 Ctrl+C 走 PSReadLine/reedline 自读 0x03,不依赖事件)。
- **排除的嫌疑**:portable-pty 0.9.0 spawn 无 `CREATE_NEW_PROCESS_GROUP`
  (读过 `portable-pty-0.9.0/src/win/psuedocon.rs:145`,标准
  `EXTENDED_STARTUPINFO_PRESENT` + PSEUDOCONSOLE 属性姿势)。
- **根因定位**:OS conhost 对 ConPTY 输入管道里 VT 字节 0x03 的控制事件
  翻译,文档设计(MSDN ConPTY 博客)与实践不符的已知灰色地带
  ([Microsoft Q&A](https://learn.microsoft.com/en-us/answers/questions/5832200/c-sent-to-the-stdin-of-a-program-running-under-a-p)、
  [wintty#155](https://github.com/deblasis/wintty/issues/155)、
  [winpty#116](https://github.com/rprichard/winpty/issues/116))。
  主端写管道**没有**事件注入 API;`GenerateConsoleCtrlEvent` 需要与子进程
  同一控制台的进程发起,AutoTerm 主端不满足。
- **影响**:虚拟桌面里 AI/用户**无法打断挂死的命令**;F1+F2 叠加下 ash
  的内建/外部长命令皆不可中断。DEBTS #7 的"经典控制台程序真事件仍需
  win32 GenerateConsoleCtrlEvent"备注即此方向,F2 把它从"待复测"坐实为
  "确认缺陷"。
- **候选修复通路**(按代价排序):
  1. **注入辅助进程**:AutoTerm 启动时附一个与子进程共享控制台的小
     helper(如 `conhost --headless` 伴随进程或自身 AttachConsole 到
     子进程控制台后 `GenerateConsoleCtrlEvent`)——winpty 的经典手法;
  2. 调查 OS conhost 版本差异(OpenConsole/WT 自带 conhost 是否翻译
     0x03;若是,评估 portable-pty 替换/升级通路);
  3. 上游 portable-pty 提 issue/PR(master 侧加 control-event 通道)。
- **DEBTS**:见 #12;复现器已留档(`#[ignore]` 用例,修复后去 ignore)。

### 4.1 修复实施(PLAN-008,2026-09-06)

> F2 的修复落地记录:机制、实测矩阵、模式语义与残留缺口。

- **机制**(003 §4 候选通路 #1,winpty 手法):新增辅助进程
  `crates/autoterm-core/src/bin/autoterm-ctrlc.rs`(`FreeConsole →
  AttachConsole(shell_pid) → 注册全事件免疫 handler →
  GenerateConsoleCtrlEvent 广播 → 退出`,手写 kernel32 FFI,零新依赖);
  `PtySession::interrupt()`(pty.rs)解析 helper(三级:测试 env →
  `AUTOTERM_CTRLC_BIN` → exe 目录向上 ≤3 级)后以 CREATE_NO_WINDOW
  拉起,缺失/失败降级纯字节路径(修复前行为,绝不 panic);
  UI 裸 Ctrl+C 在 `key_to_bytes` 前拦截(`is_bare_ctrl_c`,
  Ctrl+Shift+C 复制路径不受影响)→ `handle_ctrl_c` 按有效模式投递。
- **T3 诊断矩阵**(OS build 26200.9168,决定最终形态):

  | 通道 | 实测效果 |
  | --- | --- |
  | 主端写 0x03 字节 | 无事件(=F2 原判);仅 raw 行编辑器自读(reedline/PSReadLine 废弃行) |
  | CTRL_C_EVENT 广播(AttachConsole helper) | **被 ConPTY 客户端吞掉,零可见效果** |
  | CTRL_BREAK_EVENT 广播 | **可达**:cmd 打 `Control-Break`、ping 打统计(按其设计继续跑) |
  | CONIN$ KEY_EVENT 记录注入 | 零效果 |

  ⇒ `interrupt()` = **Break→C 双发**(Break 先落:对默认 handler 进程
  普遍有效,且是本类 build 的唯一通道;健康 build 上 C 承担规范语义,
  Break 多余但无害)。helper 被自己广播的事件杀死(exit=
  0xC000013A)是 conhost 对 AttachConsole 进程 handler 派发的时序
  竞态——**被事件杀死 ⇒ 事件必然已广播**,interrupt() 同判成功
  (Break 先发保证被杀前有效通道已落地)。
- **T4 ash×事件三态矩阵**(`ctrl_c_event_effect_on_ash`,析取断言):

  | 态 | 命令 | 实测 | 含义 |
  | --- | --- | --- | --- |
  | idle | (提示符) | **Exited**(事件整体终止 ash 进程) | ash 无 ctrl handler,默认 handler 生效 → **auto 模式 ash 豁免为仅字节的直接依据** |
  | builtin | `sleep 8` | **Responsive-late**(事件惰性,ash 存活至内建自然结束才响应 marker) | F1 数据扩展:**真事件也不打断内建**(raw mode 阻塞在 `std::thread::sleep`) |
  | external | `timeout /t 30` | **Responsive-fast**(Break 终止子进程,ash 即回提示符) | ash 外部子进程路径(frontend/subprocess.rs 临时退 raw mode)中断正常 |

- **`--ctrl-c-mode` 语义**(auto|byte|event|both,默认 auto):
  `byte`=仅 0x03(修复前行为);`event`=仅 interrupt();
  `both`=事件+字节(推荐:两消费方不重叠——事件中断运行中命令,
  字节供 raw 行编辑器废弃行);`auto`=按 shell 判定(**ash → byte**
  (文件名 stem 判定,#13 落地后撤豁免),其余 → both)。
- **UI 级证据**(cmd 主体,`for /l 1,1,10000000` 死循环 = 输入不
  敏感 + cmd 处理 Break 中止循环;`target/cmd-smoke-{both,byte}.txt`):
  - **both(=auto for cmd)**:`after_c` 2 命中,`last_tick=45580`
    (恰为 7s 注入 `\x03` 处停)——**中断生效,后续命令可跑**;
  - **byte 对照**:`after_c` 0 命中,`last_tick=137585`(跑到 13s
    dump,F2 行为原样)。
  - ash×UI 取证**四连否**(如实记录):builtin `sleep 30` 惰性 /
    `ping` 免疫 Break / 外部 `pwsh -c Start-Sleep 30` 对 Break 存活
    (ash-smoke-c2-both.txt,after_c 0)/ `timeout` 对任意按键提前
    退出(取证污染源,不能作证据)——ash 下无"输入不敏感+Break
    可终止"的干净挂死样本,ash-auto 豁免以单测+T4 矩阵支撑。
- **残留缺口**(26200 类 build):ping 类对 CTRL_BREAK 特殊处理
  (打统计后**继续**)的命令仍不可中断——`ctrl_event.rs` 的
  `#[ignore]` 复现器 `ping_class_survives_dual_send_on_quirky_build`
  留档,**健康 build 上该用例转红即 C 通道修复面到位,可去 ignore**。
- **门禁接棒**:007 的两个 F2 `#[ignore]` 复现器退役,语义由
  `tests/ctrl_event.rs` 转正(①cmd/timeout ≤5s 退出、②pwsh 回提示符
  (挂死命令同为 timeout——实测 pwsh 不杀 Break 免疫的 ping 子进程)、
  ③helper 缺失降级);ash_integration 回归 0 ignored 并新增三态探针。

## 5. 虚拟桌面接入契约(给 auto-os 侧的锚点)

1. **拉起**:显式绝对路径(如 `D:/autostack/auto-shell/ash/target/
   release/ash.exe`),无参数(裸跑即 REPL);ash.exe 不在 PATH。
2. **环境**:AutoTerm 自报 `TERM=alacritty` + `COLORTERM=truecolor`
   (`autoterm-core/src/pty.rs:62-64`),ash 判 24-bit 真彩,无需额外配置。
3. **退出**:`exit` / Ctrl+D 均可靠;关闭窗口杀子进程(002 已验)。
4. **中断**:**可用**(008 修复,机制/矩阵/残留缺口见 §4.1)——
   `--ctrl-c-mode`(默认 auto:ash=仅字节、其他 shell=事件+字节双投递)。
   已知边界:ping 类(Break 免疫)与 ash 内建(F1)仍不可中断;
   虚拟桌面对这两类挂死仍需进程级 kill 兜底。
5. **部署**:autoterm.exe 与 autoterm-ctrlc.exe **同目录**分发
   (008 Ctrl+C 事件注入的辅助进程;缺失时自动降级字节路径,
   修复前行为,不崩)。
6. **Auto 复刻应用形态**(PLAN-010 T5 追补,009 待澄清③授权):
   随包分发 `autoterm_engine` cdylib(`autoterm_core.dll`,debug/
   release 后缀随目标),与宿主同目录;以及与宿主同目录的
   `autoterm-ctrlc.exe`(interrupt 契约不变)。窗口标题为复刻侧
   自有("AutoTerm"),不复用 ash 标题。
6. **回归门禁**:任何 auto-term 变更后跑
   `cargo test -p autoterm-core --test ash_integration`
   (ash 在场即全量断言,缺席即显式 skip)。

## 6. 遗留

- ~~F2(本仓 DEBTS #12)~~ **已修复(PLAN-008,2026-09-06)**:见 §4.1
  (Break→C 双发 + auto 豁免;残留缺口 = ping 类 Break 免疫命令,
  `#[ignore]` 复现器留档待健康 build 验证);
- F1(auto-shell 侧,#13)→ 修复归 auto-shell(008 探针矩阵已供三态
  实测数据;ash 装 ctrl handler 后可撤 auto 豁免);
- R1 深按键矩阵、R2 滚动区专项视觉取证 → 可挂 Phase 4 批(005)或另立;
- ash 冷启动首启(`~/.ashrc` 创建)未单独计时(本机已有 rc;新装机
  预算 15s 已在门禁留足,实测远低于此)。
