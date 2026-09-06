---
plan_id: PLAN-008
status: archived
feature_name: Ctrl+C 中断通路修复(DEBTS #12,F2:win32 控制台事件注入)
author: [衍星居士]
created_at: 2026-09-06
updated_at: 2026-09-06

# /auto-plan:review 时填写
supersedes_spec_components:
  - "docs/designs/003-ash-compatibility.md: 修改(§1 速览改已修复/§2.1 矩阵复现器行换三态探针/新增 §4.1 修复实施/§5 契约中断可用+部署同目录/§6 遗留勾账)"
  - "DEBTS.md: 修改(#12 清账(双发+残留缺口+分发约束)、#7 关账(win32 直调落地)、#13 补三态矩阵与协调请求、头部账期)"
  - "README.md: 修改(新增 Ctrl+C 中断节+部署约束、交互表 Ctrl+C 行、下一步更新)"
new_spec_components:
  - "crates/autoterm-core/src/bin/autoterm-ctrlc.rs: 新增(事件注入辅助进程:手写 kernel32 FFI 零依赖,Break→C 双发,全事件免疫 handler,exit 0/2/3,诊断参数 c|break)"
  - "crates/autoterm-core/tests/ctrl_event.rs: 新增(#12 必跑门禁 3 用例(cmd≤5s 中断/pwsh 回提示符/helper 缺失降级)+ ping 类残留缺口 #[ignore] 复现器)"
  - "crates/autoterm-core/src/pty.rs::interrupt(): 新增(PtySession 控制台事件注入:helper 三级解析、CREATE_NO_WINDOW、3s 有界等待、0xC000013A 同判成功、降级字节路径)"
  - "crates/autoterm-ui/src/lib.rs::CtrlCMode/handle_ctrl_c: 新增(--ctrl-c-mode auto|byte|event|both,auto 下 ash 豁免;裸 Ctrl+C 双投递,DevTick 孤 0x03 同路)"
touched_goals:
  - "DEBTS #12(Ctrl+C 中断通路)修复落地:虚拟桌面'打断挂死命令'刚需兑现(003 §5 契约第 4 条转'可用'),#7 随 win32 直调关账"
  - "DEBTS #8(Auto 化)行为对拍前提:P3 oracle 的中断语义可用;新代码保持 a2r 可表达形态(纯函数/枚举/match,无奇技)"

current_step: 9
total_steps: 9
---

# [PLAN-008] Ctrl+C 中断通路修复(DEBTS #12 / F2)

## 变更摘要

PLAN-007 坐实 F2:向 portable-pty ConPTY 主端写 0x03 **不触发**
conhost 的 CTRL_C_EVENT——Ctrl+C 无法中断**运行中**的命令,且与
shell 无关(cmd/pwsh 对照探针同样断裂),AutoTerm 现有代码层无法
就地修复(DEBTS #12)。虚拟桌面场景"AI/用户打断挂死命令"是刚需
(003 §5 契约第 4 条明示此依赖),且 #8 Auto 复刻 P3 的行为对拍
也需要中断语义可用。

本计划按 003 §4 候选通路 #1(winpty 经典手法)实施修复:
**新增零依赖辅助进程 `autoterm-ctrlc.exe`**(手写 kernel32 FFI:
`AttachConsole(shell_pid)` 挂进 ConPTY 隐藏控制台后
`GenerateConsoleCtrlEvent(CTRL_C_EVENT, 0)` 广播),
`PtySession` 新增 `interrupt()` 注入事件,UI 层裸 Ctrl+C 改为
**事件 + 0x03 字节双投递**(事件管运行中命令,字节管 raw 行编辑器
如 reedline/PSReadLine)。验收门禁 = 新建 `tests/ctrl_event.rs`
(cmd 恒可得:5s 内真实中断)接棒 007 留档的两个 `#[ignore]`
复现器,外加 ash×事件三态探针矩阵(裁定 UI auto 模式对 ash 的
豁免门——F1/#13 协调点)。**这是 002 以来首个动产品代码的修复
计划**;#7(win32 直调方向)随本案关账。

## 目标

1. **核心能力**:`PtySession::interrupt()` 在 Windows 上经辅助进程
   向 shell 所在 ConPTY 控制台广播 CTRL_C_EVENT,使 `cmd /c
   ping -n 30` 这类运行中命令在 **≤5s** 内真正中断(cmd 恒在场的
   自动门禁,不依赖 ash);
2. **双投递语义**:UI 裸 Ctrl+C = 事件(中断运行中命令)+ 0x03
   字节(废弃输入行,raw 行编辑器消费)——007 门禁 ③
   (`ctrl_c_aborts_input_line`)语义保留不回归;
3. **模式可配**:`--ctrl-c-mode <auto|byte|event|both>`(默认
   auto);auto 下按 shell 豁免(ash 在 #13 落地前豁免为 byte,
   依据 T4 探针矩阵按预授权规则裁定);
4. **门禁接棒**:007 两个 `#[ignore]` 复现器退役(语义转正由
   ctrl_event.rs 门禁承担),`ash_integration.rs` 回归 0 ignored;
5. **台账关账**:DEBTS #12 清账、#7 关账(win32 直调落地)、
   #13 追注 ash 实测行为与协调请求;003 §4 增"修复实施"小节、
   §5 契约第 4 条更新("中断:可用");
6. **零新依赖**:手写 extern "system" FFI,Cargo.toml/Cargo.lock
   零改动(验收断言);新代码保持 a2r 可表达形态(DEBTS #8 约束,
   006 结论:引擎绑定侧留 Rust adapter,helper 属 adapter 域)。

### 非目标(明确排除)

- **F1 / #13 修复**(ash 内建不可中断、装 ctrl handler)——归
  auto-shell 仓,本仓只出探针矩阵与协调请求;
- Unix 的 interrupt 语义(SIGINT/进程组)——Unix 基座独立轨道,
  非 Windows 分支走现有字节路径占位;
- Ctrl+Break(CTRL_BREAK_EVENT)映射——helper 已留位但本计划
  不接线(003/003 号计划已有结论记录,无用户反馈驱动);
- 上游 portable-pty 提 issue/PR(候选通路 #3)——修复验证后
  另起轻量动作,不阻塞本案;
- conhost 稳定版差异调查(候选通路 #2)——事件注入后不再依赖
  conhost 翻译行为,调查失去必要性,#7 据此关账。

## 架构方案

四层中动两层(core 新增 helper bin + 方法;ui 一处拦截),分层
职责不变:

```
crates/autoterm-core
├── src/pty.rs                    # +shell_pid 字段、+interrupt()、
│                                 #  +resolve_helper_bin()(三级解析)
├── src/bin/autoterm-ctrlc.rs     # 新增辅助进程(手写 FFI,~70 行)
├── tests/ctrl_event.rs           # 新增:cmd 恒在场的中断门禁
└── tests/ash_integration.rs      # 复现器退役 + ash×事件探针矩阵
crates/autoterm-ui
├── src/lib.rs                    # 裸 Ctrl+C 拦截(handle_ctrl_c)、
│                                 #  DevTick 注入 0x03 同路、mode 纯函数
└── src/bin/autoterm.rs           # --ctrl-c-mode 参数
```

**辅助进程数据流**(winpty 经典手法,003 §4 通路 #1):

```
UI 裸 Ctrl+C ──▶ App::handle_ctrl_c(按 mode)
                   │ ① write_input([0x03])      ──▶ 主端字节(reedline 消费)
                   │ ② session.interrupt()
                   │     ├─ resolve_helper_bin()  ──▶ autoterm-ctrlc.exe 路径
                   │     └─ spawn(CREATE_NO_WINDOW, wait ≤3s)
                   ▼
autoterm-ctrlc.exe <shell_pid>
  FreeConsole → AttachConsole(pid)          # 挂进 ConPTY 隐藏控制台
  → SetConsoleCtrlHandler(None, TRUE)       # helper 自身免疫
  → GenerateConsoleCtrlEvent(CTRL_C_EVENT, 0)  # 广播同控制台全部进程
  → FreeConsole → exit 0
                   ▼
conhost 派发 CTRL_C_EVENT ──▶ 运行中子进程(ping 等)终止,
                              cmd/pwsh 各自 handler 存活回提示符
```

要点:
- 广播安全性:ConPTY 隐藏控制台是本会话私有的,挂着的只有 shell
  及其子进程——广播(进程组 0)无外溢面;
- helper 缺失/失败(AutoTerm 部署不完整、attach 失败):
  `interrupt()` 降级 `write_input([0x03])` + `log::warn`,行为
  回到修复前,绝不 panic;
- 解析三级(与 007 `ash_bin()` 同风格):运行时 env
  `CARGO_BIN_EXE_AUTOTERM_CTRLC`(cargo 为本 crate 集成测试注入)
  → env `AUTOTERM_CTRLC_BIN`(人工/部署覆盖)→ `current_exe()`
  向上 ≤3 级父目录找同名 exe(测试布局 target/debug/deps →
  target/debug;安装布局同目录)。

## 技术栈

全部现有依赖(portable-pty、anyhow、clap、log);win32 调用用
`extern "system"` 手写声明(kernel32 四函数,签名均为标量),
**零新 crate**。

## 需求分析与背景调查

### spec 依据(总览)

- **P006-1**:虚拟桌面轨道抬高"ash/AutoTerm 里真实可跑+可打断"
  的优先级;**P007-2**:场景③ 语义重构待裁定 + #12 修复另立——
  本计划即该后续(F2 通路层修复);
- **P007-4**(designs/003 §4):候选通路 #1 注入辅助进程(winpty
  手法)被列为代价最低方向;§5 契约第 4 条"中断:当前不可用,
  需先修 DEBTS #12"——本计划直接兑现该契约项;
- **DEBTS #7**:清偿方向明写"复现则 win32 直调立项;007 注:方向
  即 #12 修复通路"——本计划落地 win32 直调,#7 关账;
- **P005-2 / DEBTS #8 约束**:新代码保持 a2r 可表达形态;006
  结论(002 设计文档 §5 档四)引擎绑定(PTY/事件注入)留
  Rust adapter 域——helper 与 interrupt() 即 adapter 域,UI 层
  的 mode 判定保持纯函数可表达。

### 现状证据(file:line)

| 事实 | 证据 |
|---|---|
| child 句柄已持有(可取 PID) | `crates/autoterm-core/src/pty.rs:31`;portable-pty `Child::process_id() -> Option<u32>`(trait,registry lib.rs:141) |
| 0x03 现路径 = 键盘→字节直写 | `crates/autoterm-ui/src/lib.rs:658`(`key_to_bytes`→`write_input`),拦截先例在 :651-656 |
| 复现器留档(恒 false 谓词,转正须改写) | `crates/autoterm-core/tests/ash_integration.rs:261`(cmd)、:277(pwsh),`wait_for` 语义 :67-79(超时 panic) |
| dev 注入写点(配方 C 通路) | `crates/autoterm-ui/src/lib.rs:491-499`(DevTick→`write_input`) |
| 门禁③ 现语义(不可回归) | `ash_integration.rs:227` `ctrl_c_aborts_input_line`("空闲 Ctrl+C 不杀 shell"断言) |
| 候选通路与证据链 | `docs/designs/003-ash-compatibility.md` §4(三条通路排序、:117-122)、DEBTS #12 行 |

### 风险清单

| # | 风险 | 对应 |
|---|---|---|
| R1 | AttachConsole 到 ConPTY 隐藏控制台失败(权限/句柄边缘) | helper exit 码区分(2=attach 失败);interrupt() 降级字节+warn;ctrl_event 门禁①即实测 |
| R2 | 事件到达 ash:空闲态默认 handler 可能**终止整个 ash**(F1 放大) | T4 三态探针矩阵实测 + auto 模式 ash 豁免门(文件名 stem=="ash",#13 落地后撤),预授权裁定规则见 T4 |
| R3 | 双投递伪影(cmd 打印 ^C、pwsh 双提示符) | 化妆性、真实终端同表现;实测伪影过扰则 T5 内改 mode 默认并记录 |
| R4 | 部署布局 helper 缺失 | 三级解析 + 降级路径 + README 说明(单 exe 拷贝部署需带两文件) |
| R5 | 广播误伤 | ConPTY 控制台会话私有,构造上无外溢(见架构方案) |

## 详细设计

### `crates/autoterm-core/src/bin/autoterm-ctrlc.rs`(T1)

```rust
//! Ctrl+C 注入辅助:挂进目标进程的控制台后广播 CTRL_C_EVENT。
//! 用法: autoterm-ctrlc <pid>;exit 0=成功,2=attach 失败,3=用法。
// SPDX-License-Identifier: Apache-2.0
// 非 windows 目标:占位 main,打印不支持后 exit 1(保持可编译)。

#[cfg(windows)]
mod imp {
    // 手写 kernel32 FFI(零新依赖):BOOL=i32,全部标量签名
    #[link(name = "kernel32")]
    extern "system" {
        fn FreeConsole() -> i32;
        fn AttachConsole(dwProcessId: u32) -> i32;
        fn SetConsoleCtrlHandler(handler: Option<unsafe extern "system" fn(u32) -> i32>, add: i32) -> i32;
        fn GenerateConsoleCtrlEvent(dwCtrlEvent: u32, dwProcessGroupId: u32) -> i32;
    }
    const CTRL_C_EVENT: u32 = 0;

    pub fn run(pid: u32) -> i32 {
        unsafe {
            let _ = FreeConsole();                       // 脱离宿主(若挂)控制台
            if AttachConsole(pid) == 0 { return 2; }     // 挂进 ConPTY 隐藏控制台
            let _ = SetConsoleCtrlHandler(None, 1);      // 自身免疫本事件
            let ok = GenerateConsoleCtrlEvent(CTRL_C_EVENT, 0); // 广播
            let _ = FreeConsole();
            if ok == 0 { 2 } else { 0 }
        }
    }
}
```

### `pty.rs` 增量(T2)

- 字段:`shell_pid: Option<u32>`(spawn 后 `child.process_id()`
  存副本)、`program: String`(:57 既有变量,供 UI 侧 ash 判定,
  `pub fn program(&self) -> &str` 暴露);
- `pub fn interrupt(&mut self) -> bool`:windows → pid 缺席(已
  reap)返 false;`resolve_helper_bin()` 取路径 →
  `std::process::Command::new(path).arg(pid)` +
  `creation_flags(0x08000000)`(CREATE_NO_WINDOW,防控制台窗口
  闪烁)spawn,`wait_timeout ≤3s`(自旋 try_wait 50ms 步进,不引
  wait-timeout crate);失败/超时/缺席 → `log::warn!` +
  `write_input(&[0x03])` 返 false;成功返 true;
- `fn resolve_helper_bin_from(exe: &Path) -> Option<PathBuf>`:
  exe 起 1..=3 级父目录寻 `autoterm-ctrlc.exe`(纯函数可单测);
  外壳 `resolve_helper_bin()` 依次查 env
  `CARGO_BIN_EXE_AUTOTERM_CTRLC` → env `AUTOTERM_CTRLC_BIN` →
  `resolve_helper_bin_from(&current_exe())`;
- `#[cfg(not(windows))]` 的 interrupt() = `write_input(&[0x03])`
  返 true(占位,Unix 轨道另立)。

### `tests/ctrl_event.rs`(T3)

- 自带 `grid_text`/`wait_for` 助手(与 ash_integration.rs:62-79
  同源,注释注明;不建 tests/common,保持各门禁文件自包含);
- ① `interrupt_terminates_cmd_child`:spawn `cmd /c ping -n 30
  127.0.0.1` → 等 "Reply from" → `interrupt()` 返 true →
  轮询 `exited()` ≤5s(**#12 的直接门禁,cmd 恒在场**);
- ② `interrupt_returns_pwsh_prompt`:`where pwsh` 探测缺席即
  eprintln skip + return;在场:提示符 → `ping -n 30
  127.0.0.1\r` → 等 "Reply from" → `interrupt()` → `echo
  marker\r` ≤10s 上屏 + `!exited()`;
- ③ `interrupt_helper_missing_falls_back`:`AUTOTERM_CTRLC_BIN=
  Z:/nonexistent.exe`(std::env::set_var,测试串行;两级 env 先于
  walk-up,故指向不存在即缺席)→ cmd /c ping 会话 → `interrupt()`
  返 false、不 panic、2s 内 `!exited()`(降级=修复前行为)。

### ash 探针矩阵与复现器退役(T4)

- 删除 `f2_repro_cmd_interrupt`/`f2_repro_pwsh_interrupt`
  (ash_integration.rs:261-296)——cmd/pwsh 语义已由 ctrl_event.rs
  转正,复现器使命完成;文件头注释同步(引用 PLAN-008);
- 新增 `ctrl_c_event_effect_on_ash`(ash 缺席 skip),三态:
  - **idle**:提示符下 `interrupt()` → 析取断言"存活且 marker
    可跑"或 `exited()`,记录实际分支;
  - **builtin**:`sleep 30\r` 上屏后 `interrupt()` → 同析取;
  - **external**:`ping -n 30 127.0.0.1\r`(ash 跑外部 ping)→
    `interrupt()` → ping 输出停止且提示符回归/`exited()` 析取。
- **预授权裁定规则**(执行记录落矩阵后即时裁定,不回询):
  idle 态 ash 被 exit(默认 handler)→ auto 模式保留 ash 豁免;
  idle 态存活(handler 已在)→ 不引入豁免门,ash 同享 both;
  矩阵结论同步 DEBTS #13 行(auto-shell 协调请求)。

### UI 接线(T5)

- `pub fn is_bare_ctrl_c(key: &Key, mods: &Modifiers) -> bool`
  纯函数(Character 'c' + CONTROL 且非 SHIFT);
- `Message::Keyboard` 分支在剪贴板拦截(lib.rs:651-656)之后、
  `key_to_bytes`(:658)之前:`if is_bare_ctrl_c(&key,&mods) {
  return self.handle_ctrl_c(); }`;
- `fn handle_ctrl_c(&mut self) -> Task<Message>`:按
  `self.config.ctrl_c_mode`——byte=仅 `write_input([0x03])`;
  event=仅 `session.interrupt()`;both=两者;auto=
  `resolve_effective_mode(&self.config.shell)`(stem eq_ignore_case
  "ash" → byte,#13 落地后撤,否则 both);函数纯化便于单测;
- DevTick 注入(lib.rs:491-493):`bytes == [0x03]` 时改调
  `handle_ctrl_c()` 同路(保配方 C 的 dev 证据 = 真实按键语义);
- `AppConfig.ctrl_c_mode` + bin `--ctrl-c-mode <auto|byte|event|
  both>`(clap ValueEnum,默认 auto,autoterm.rs:23 旁);
- 单测:is_bare_ctrl_c 三例(裸=真/Ctrl+Shift=假/无 Ctrl=假,
  既有 :1388 断言迁移对齐)、resolve_effective_mode 两例
  (ash→byte、cmd/pwsh→both)。

### UI 取证(T6)

007 配方 C 原样复跑(`--dev-autotype` 重复传参语义,003 §2.2):

```bash
cargo run -p autoterm-ui --features dev-tools -- --shell $ASH \
  --dev-autotype "4000:sleep 30\r" --dev-autotype "6000:\x03" \
  --dev-autotype "8000:echo after_c\r" --dev-exit-after 16 \
  --dev-dump target/ash-smoke-c2.txt
```

- T4 裁定 ash 豁免生效时:上命令加 `--ctrl-c-mode both` 取证
  (dump 含 `after_c`,对照 007 的 0 命中即修复生效),另补默认
  auto 一条对照 dump(after_c 0 命中 = 豁免证据);
- 无豁免时:默认 auto 即含 after_c。

### 文档与台账(T7)

- designs/003 追加 §4.1 修复实施(机制/双投递/ash 三态矩阵/
  ctrl-c-mode 表);§5 第 4 条改"中断:可用(默认 auto,见 §4.1)";
- DEBTS:#12 清账(门禁①证据)、#7 关账(win32 直调落地,稳定版
  conhost 调查随事件注入失去必要性)、#13 追注矩阵 + 协调请求、
  头部账期;
- README:交互表 Ctrl+C 行为一句(双投递 + helper 部署说明:
  拷贝部署需 autoterm.exe 与 autoterm-ctrlc.exe 同目录)。

## 测试设计

- **自动**(必跑):ctrl_event.rs ①③(cmd 恒在场=门禁本体)+
  ②(pwsh 缺席显式 skip);ash_integration 原有 7 门禁不回归 +
  `ctrl_c_event_effect_on_ash`(缺席 skip,在场析取断言);
  ui 单测(is_bare_ctrl_c/resolve_effective_mode);
- **半自动**:配方 C 复跑 dump(事件生效)+ auto 豁免对照 dump;
  证据引述进 003 §4.1;
- **手动**:真机 Ctrl+C 手感(pwsh 跑长命令/ash 外部命令各一次,
  观察伪影 R3)——5 分钟清单,不构成本计划验收;
- **回归底线**:`cargo test --workspace` 双构建绿;零新依赖
  (Cargo.toml/lock 零 diff);空闲零唤醒不受影响(interrupt 仅
  按键时触发,无新增常驻订阅)。

## 验收标准

1. `cargo test -p autoterm-core --test ctrl_event` 全绿:①cmd
   会话 interrupt 后 ≤5s `exited()`;②pwsh 在场绿/缺席显式
   skip;③helper 缺席降级(返 false、不 panic、会话存活);
2. ash_integration:0 个 ignored(复现器退役),原 7 门禁全绿
   (含"空闲 Ctrl+C 不杀 shell"),`ctrl_c_event_effect_on_ash`
   按析取绿且矩阵落执行记录;
3. UI:单测绿(拦截排除 Ctrl+Shift+C 剪贴板路径);配方 C dump
   含 `after_c` 且 uptime < 15s;auto 豁免(若引入)有对照 dump;
4. DEBTS #12 清账、#7 关账、#13 追注;003 含 §4.1;README 更新;
5. 零新依赖:`git diff` 不含 Cargo.toml/Cargo.lock;
   `cargo tree --workspace` 无 ash-core/auto-shell;
6. 双构建绿(默认/dev-tools);ash 缺席环境复跑 ctrl_event 仍
   全绿(①③仅依赖 cmd)。

## 执行步骤

> 约定:全程在 worktree `.wt/auto-term-008/auto-term`(由
> /auto-plan:work 创建);ash.exe 用
> `D:/autostack/auto-shell/ash/target/release/ash.exe`(缺席则
> ash 门禁显式 skip,ctrl_event 门禁不受影响)。

- **T1** helper bin:新建 `crates/autoterm-core/src/bin/
  autoterm-ctrlc.rs`(§详细设计代码骨架:手写 FFI 四函数、
  FreeConsole→AttachConsole→免疫→广播→FreeConsole、exit 码
  0/2/3、非 windows 占位 main)。
  验证:`cargo build -p autoterm-core --bins` 0 error;
  `./target/debug/autoterm-ctrlc.exe 4` 打印 attach 失败并
  exit 2(PID 4=System 无控制台,验证参数链无害)。
  [✅ 已完成] build 0 error(edition 2024 需 `unsafe extern`,已按);
  PID 4 探针 exit=2、无参数 exit=3——参数链与错误码路径均验。

- **T2** core `interrupt()`:pty.rs 增 `shell_pid`/`program`
  字段与访问器;`interrupt()`(spawn CREATE_NO_WINDOW + ≤3s
  自旋 wait + 降级字节+warn + 返 bool,非 windows 字节占位);
  `resolve_helper_bin_from` 纯函数 + 三级解析外壳;
  lib 单测:resolve_helper_bin_from 的目录走查(target/debug/
  deps 布局两跳命中、根布局不越 3 级)。
  验证:`cargo test -p autoterm-core --lib` 绿;`cargo build
  -p autoterm-core` 0 error。
  [✅ 已完成] 单测 resolve_helper_bin_walks_up_max_three_levels 绿
  (TDD:先红 E0425 后绿);build 0 error。偏差一处:诊断用
  eprintln 而非 log::warn——core 无 log 依赖,加依赖违反验收#5
  (Cargo 零 diff),零依赖约束优先。

- **T3** ctrl_event 门禁:新建 `tests/ctrl_event.rs`(①cmd
  5s 中断 / ②pwsh skip-gate / ③helper 缺席降级,§详细设计)。
  验证:`cargo test -p autoterm-core --test ctrl_event --
  --nocapture` 全绿(②绿或显式 skip 行)。
  [✅ 已完成,按 T3 诊断证据大幅改写,见待澄清#5] 实测矩阵(build
  26200.9168):CTRL_C 广播被 ConPTY 客户端吞掉(零效果)/CTRL_BREAK
  可达/CONIN$ 按键注入无效 → helper 改 **Break→C 双发**(Break 先落),
  interrupt() 接受 exit=0xC000013A(helper 被自己广播的事件杀死 =
  conhost 对 AttachConsole 进程的 handler 派发竞态,事件必已广播);
  ①挂死命令换 `timeout`(Break 可终止;ping 类对 Break 特殊处理属
  残留缺口,#[ignore] 复现器留档);③伪 helper 用 where.exe(拿 pid
  当文件名必败)更确定性;三用例 ENV_LOCK 串行(env 互染防护)。
  5/5 连跑全绿(3 passed + 1 ignored,~3.0s);②pwsh 实测不杀
  Break 免疫的 ping 子进程,同样换 timeout。

- **T4** 复现器退役 + ash 探针:删 ash_integration.rs 两个
  `#[ignore]` 复现器(:261-296),文件头注释同步;新增
  `ctrl_c_event_effect_on_ash` 三态析取探针;按预授权规则裁定
  auto 豁免门(矩阵落执行记录)。
  验证:`cargo test -p autoterm-core --test ash_integration --
  --nocapture`(ash 在场)全绿 0 ignored;缺席 8 行 skip。
  [✅ 已完成] 复现器退役(cmd/pwsh 语义由 ctrl_event.rs 接棒);
  探针矩阵实测:**idle=Exited**(双发整体终止 ash,auto 豁免坐实)、
  **builtin=Responsive-late**(事件在内建期惰性——F1 数据扩展:真
  事件也不打断内建,ash 存活至自然结束)、**external=Responsive-fast**
  (Break 可终止子进程死,ash 即回)。builtin 用 sleep 8(30s 版预算
  装不下"惰性存活"分支);external 用 timeout(ping 免疫 Break 不作
  矩阵对象)。3/3 稳定 8 过 0 ignore;缺席 8 行 skip。

- **T5** UI 接线:is_bare_ctrl_c 纯函数 + Keyboard 分支拦截 +
  handle_ctrl_c(mode 分派)+ DevTick 孤 0x03 同路 +
  `--ctrl-c-mode` 参数 + 单测三组。
  验证:`cargo test -p autoterm-ui` 全绿(新增 ≥5 断言)。
  [✅ 已完成] 7 个新单测全绿(is_bare_ctrl_c 三态/ash 豁免/auto 展开
  /模式透传/解析),ui 套件 23+3 全绿;`--ctrl-c-mode` 入 --help;
  默认构建仍拒 --dev-select(零 dev 面不变)。handle_ctrl_c 的
  Both|Auto 分支合并(auto 经 resolve_effective_mode 后不会出现
  Auto,枚举穷尽性保险)。DevTick 孤 0x03 改 collect-then-process
  (retain 闭包借不了整个 self)。

- **T6** UI 取证:配方 C 复跑(必要时 `--ctrl-c-mode both` 覆盖
  + auto 对照),dump 落 worktree target/,关键行核对
  (`after_c` 命中/豁免对照 0 命中)。
  验证:grep dump 关键行,证据路径记执行记录。
  [✅ 已完成,场景主体按实测换靶] 配方 C 的 `sleep 30` 是 ash **内建**
  ——T4 已证事件在内建期惰性,场景不成立。换 **cmd 作 shell**
  (auto=both)+`for /l 10000000` 死循环(输入不敏感、cmd 处理 Break
  中止循环):**both=after_c 2 命中、last_tick=45580(7s 注入点停)**;
  **byte 对照=after_c 0、last_tick=137585(跑到 dump,F2 原样)**。
  ash×UI 取证四连否如实记录:builtin sleep 惰性 / ping 免疫 Break /
  外部 pwsh -c Start-Sleep 对 Break 存活(ash-smoke-c2-both.txt,0
  命中)/ timeout 对任意按键提前退出(污染源,不能作证据)——ash 下
  无干净"输入不敏感+Break 可终止"样本,ash-auto 豁免的 UI 级对照以
  单测+T4 矩阵+cmd-byte 对照支撑。dump 落 worktree target/
  (cmd-smoke-{both,byte}.txt、ash-smoke-c2-*.txt)。

- **T7** 文档与台账:003 §4.1 + §5 契约更新;DEBTS #12/#7/#13
  + 头部;README 行为与部署说明。
  验证:`grep -n "PLAN-008" docs/designs/003-ash-compatibility.md
  DEBTS.md README.md` 各命中;diff 审阅。
  [✅ 已完成] 003 增 §4.1(机制/通道矩阵/ash 三态/模式语义/UI 证据/
  残留缺口政策)与 §5 契约第 4 条改"可用"+新增第 5 条部署约束
  (回归门禁顺移第 6);§6 遗留勾 F2、F1 附三态数据;DEBTS #12
  清账(残留缺口与分发约束入清偿方向)、#7 关账、#13 补证、头部
  账期;README 增"Ctrl+C 中断"节 + 交互表行改 + 下一步更新
  (#7 复测项移除)。三文件 UTF-8 校验过,grep 各命中。

- **T8** 全量回归:`cargo test --workspace` 与默认构建
  `cargo build --workspace` 双绿(默认构建 `--ctrl-c-mode` 仍在
  ——产品参数非 dev 面,与 `--dev-select` 被拒对照记录);
  `git diff --stat` 无 Cargo.toml/Cargo.lock;缺席模拟
  (`AUTOTERM_ASH_BIN=Z:/nonexistent.exe`)ash 门禁 skip 不回归;
  `cargo tree -p autoterm-core` 无外部仓路径。
  验证:各命令输出记执行记录。
  [✅ 已完成] `cargo test --workspace` **6/6 全绿**(探针按实测加固:
  并行套件负载下 conhost 事件派发偶发丢失——加 3s 周期补发
  interrupt(幂等),预算 idle 10s/builtin 20s/external 15s,把
  "丢失"转"延迟";加固前 3/5 失败,加固后 6/6);默认构建绿 +
  `--dev-select` 仍拒 + `--ctrl-c-mode bogus` 警告回退 auto;diff
  vs base 9 文件 **无 Cargo.toml/Cargo.lock**(零新依赖);缺席模拟
  8 行 skip + result ok;`cargo tree -p autoterm-core` 无外部仓路径。
  注意:`--ctrl-c-mode bogus` 验证后 GUI 正常启动(有效运行),取证
  时需挂 --dev-exit-after 或手动关窗。

- **T9** 收尾:执行记录补齐(updated_at、探针矩阵、豁免裁定、
  证据路径),阻塞(若有)记 open-questions,移交
  /auto-plan:review。
  验证:计划文件各任务勾选框与验证输出一一对应。
  [✅ 已完成] T1–T9 勾选框与验证输出一一对应(各任务行内);
  探针矩阵/豁免裁定/证据路径均已落账(T4/T6 行 + 003 §4.1);
  阻塞与裁定项集中记于待澄清#5(用户裁定:双发形态与 #12 关账);
  worktree 7 commits(T1-T8),分支 plan-008-dev 待 /auto-plan:review。

## 复审记录

**Reviewer**:ZCode(/auto-plan:review)· 2026-09-06 · 复验全部在 worktree
`.wt/auto-008/auto-term`(branch `plan-008-dev`,8 commits 含复审 salvage)
内重跑,不信勾选框。

### 验收逐条(全部 pass)

| # | 验收 | 裁定 | 证据 |
| --- | --- | --- | --- |
| 1 | ctrl_event 全绿(cmd≤5s/pwsh skip-gate/降级) | **pass**(挂死命令 timeout 替代 ping = 计划内实证改写,见分歧#1) | 复跑 3 passed + 1 ignored(3.08s);cmd 会话 interrupt 后 ≤5s `exited()`;pwsh 提示符回归且存活;where.exe 伪 helper → 返 false、不 panic、会话存活 |
| 2 | ash_integration 0 ignored,7 门禁+探针绿 | **pass** | 复跑 8 passed 0 failed 0 ignored(14.73s);缺席模拟 8 行 skip + result ok;探针三态析取绿(本次 external 实测 Exited(补发 2)——首派发被负载吞、补发命中已回 idle 的 ash,与三态状态机自洽,析取覆盖) |
| 3 | UI 单测+dump 证据+零唤醒 | **pass** | ui lib 23 passed(含 7 新);cmd-smoke-both:after_c×2、last_tick=45580(恰 7s 注入点);cmd-smoke-byte:after_c×0、last_tick=137585(跑满);uptime 13.0s<15;零唤醒 idle 10s 无注入 frames=8(005 基线 5@11s 同量级,无新增常驻订阅) |
| 4 | DEBTS/#12/#7/#13 + 003 §4.1 + README | **pass**(复审补 3 处陈旧行,见分歧#4) | DEBTS #12/#7 标已清偿(008)、#13 补证;003 §4.1+契约第 4/5 条;README Ctrl+C 节+部署约束;三文件 UTF-8 校验过 |
| 5 | 零新依赖 | **pass** | diff vs 0d82334 共 9 文件(+887/−53),无 Cargo.toml/Cargo.lock;`cargo tree -p autoterm-core` 无 ash-core/auto-shell;diff 无新增 TODO/FIXME |
| 6 | 双构建绿 + ash 缺席 ctrl_event 绿 | **pass** | 复审全量套件(门禁唯一一次):18 套件 result-ok、0 FAILED;默认构建绿且 `--dev-select` 拒(exit 2)、`--ctrl-c-mode bogus` 警告回退 auto;`AUTOTERM_ASH_BIN=Z:/nonexistent.exe` 下 ctrl_event 仍 3 passed |

### 代码级抽查(trust code over plan)

- 拦截顺序:clipboard_shortcut(lib.rs:665)→ is_bare_ctrl_c(:673)→
  key_to_bytes(:677)——Ctrl+Shift+C 复制路径优先,裸 Ctrl+C 截住;
- helper 双发顺序:Break(autoterm-ctrlc.rs:92)先于 C(:94)——
  被派发竞态杀死时有效通道已落地;
- pty.rs:216 `code == 0 || code == STATUS_CONTROL_C_EXIT` 同判成功;
- DevTick(lib.rs:506)与键盘(:674)两路均走 handle_ctrl_c。

### 计划↔实现分歧(全部已明示,无静默)

1. **修复形态重构**(最重要,待澄清#5 用户裁定):草稿"广播
   CTRL_C_EVENT"在本 build(26200.9168)对 ConPTY 客户端零效果,
   T3 诊断矩阵(C 吞/Break 达/CONIN$ 无效)→ 落地为 Break→C 双发;
   门禁挂死命令由 ping 换 timeout(ping 对 Break 免疫 = 残留缺口,
   #[ignore] 复现器留档,健康 build 转红即 C 通道到位)。
2. 双投递默认 both(计划即如此);ash auto 豁免按 T4 预授权规则引入
   (idle 态事件整体终止 ash 的实测),#13 落地后撤。
3. UI 取证主体由 ash 换 cmd:ash 下无"输入不敏感+Break 可终止"干净
   样本(四连否,T6 记录);ash-auto 豁免以单测+T4 矩阵+cmd-byte
   对照支撑。
4. 复审 salvage(3f98330):003 §1 速览行/§2.1 复现器行/§2.2 配方 C
   行三处陈旧(仍述"不可用/已退役复现器"),已同步为修复后语义。

### 遗漏/延后/workaround 猎查

- 遗漏:无——9 任务对 diff 逐项核;既有 :1388 剪贴板断言保留且语义
  仍成立;手动 5 分钟手感清单按计划测试设计"不构成验收"(用户侧)。
- 延后(均有记录、计划规则允许):F1 修复归 auto-shell(#13,非目标);
  上游 portable-pty issue(待澄清#3,可选);残留缺口闭环待健康
  build(#[ignore] 复现器在位)。
- Workaround(均文档化):eprintln 代 log(零依赖验收#5 约束优先,
  T2 证据行注明);interrupt 接受 0xC000013A(派发竞态的语义化,
  helper 头注);探针 3s 周期补发(测试稳健性,产品面无此逻辑)。

### 债务候选(复审新增)

- **a**:探针 external 态实测有 Responsive(0 补发)与 Exited(补发 2)
  两种落点——补发机制命中"已回 idle 的 ash"会终止之;属探针自身
  观测行为非产品缺陷,003 §4.1 矩阵可补一行"external 落点受补发
  时序影响"(不阻塞,记录在案)。
- **b**:UI 层 idle 提示符下 cmd/pwsh 收到双投递会打 ^C/
  Control-Break 标记(R3 化妆性,真实终端同表现)——手动清单
  (用户侧)覆盖观感确认。

**路由:pass → status: reviewed**,可 `/auto-plan:merge`。
待澄清#5(双发形态 + #12 以"双发+残留缺口"关账)为用户裁定项,
merge 前用户可否决改走"维持开账待稳定版 OS"分支。

## 待澄清事项

1. **双投递默认**(both):事件管运行中命令、字节管 raw 行编辑器,
   两者消费方不重叠故默认双发;实测若现双重提示符等伪影(R3),
   在 T5/T6 内按证据调整默认并记录——不需预先裁定。
2. **ash 豁免门是临时协调物**:文件名 stem=="ash" 的判定在
   auto-shell #13(装 ctrl handler)落地后撤销;撤销动作记
   DEBTS #13 行,不另立计划。若 T4 探针实测 ash idle 态本就
   存活,豁免门直接不引入(预授权,执行记录注明)。
3. **007 待澄清#5(T3 门禁语义重构)随本案实质关账**:字节路径
   门禁(007 ③)保留原样,原语义("中断运行中命令")由
   ctrl_event 门禁①②恢复——两层并存,不再需要用户单独裁定;
   若用户仍不认可,本案 review 时提出即可。
4. **helper 分发形态**:同 crate 兄弟 bin(cargo 产物天然同
   目录),不做单 exe 嵌嵌/自解压(丑且易错);README 说明拷贝
   部署需两文件同目录。虚拟桌面(auto-os)打包注意此约束。
5. **[执行中发现,待用户裁定] 26200 build 上 CTRL_C 广播全通道
   被吞,修复形态与计划草稿不同**。T3 诊断矩阵:①CTRL_C_EVENT
   广播(AttachConsole helper)对 ConPTY 客户端零效果;②CTRL_
   BREAK_EVENT 广播可达(cmd 打 "Control-Break"、ping 打统计);
   ③CONIN$ 按键记录注入无效。据此 interrupt() 落地为 **Break→C
   双发**(Break 先落:对默认 handler 进程普遍有效且是本 build
   唯一通道;健康 build 上 C 承担规范语义)+ exit=0xC000013A 同判
   成功(conhost 对 AttachConsole 进程 handler 派发竞态,helper
   会被自己广播的事件杀死=事件已广播)。**残留缺口**:ping 类对
   Break 特殊处理(打统计后继续)的命令在本 build 仍不可中断
   (#[ignore] 复现器留档,健康 build 预期转红即可移除);门禁①②
   的挂死命令因此从 ping 换成 timeout(Break 可终止)。#12 是否
   以"双发+残留缺口"形态关账,请用户裁定;若不认可,替代项是
   维持 #12 开账待稳定版 OS 复测(同 #7 前例)。
