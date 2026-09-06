//! ash (AutoShell) 集成门禁:真 PTY → 仿真核心 → 网格断言。(PLAN-007)
//!
//! 门控:`AUTOTERM_ASH_BIN` 优先;否则向上探测兄弟仓约定路径
//! `<repo>/../auto-shell/ash/target/{release,debug}/ash.exe`
//! (worktree 布局 `.wt/<grp>/<repo>` 下再多向上两级)。皆无则打印
//! skip 说明后返回——用例只可能因"ash 在场且行为不符"而失败,
//! 缺席 = 显式 skip,不假绿。
//!
//! 场景矩阵(详见 docs/designs/003-ash-compatibility.md):
//! ① 启动提示符 ② echo 往返 ③ Ctrl+C 中断 ④ exit/Ctrl+D 退出
//! ⑤ resize 存活 ⑥ 色彩深度自证。
//!
// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use autoterm_core::{PtySession, TermSession};

/// 解析待测 ash.exe:env 优先(给了但不存在 → None,统一走 skip 分支,
/// 验收#1 要求"缺席 skip 而非 fail");兄弟仓约定路径兜底。
fn ash_bin() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("AUTOTERM_ASH_BIN") {
        let p = PathBuf::from(p);
        return p.is_file().then_some(p);
    }
    // 仓根 = 本 crate manifest 上两级(crates/autoterm-core → 仓根);
    // 主仓命中 ../auto-shell,worktree(.wt/<grp>/<repo>)命中 ../../../。
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("manifest dir 不在预期深度");
    ["../auto-shell", "../../auto-shell", "../../../auto-shell"].iter().find_map(|rel| {
        ["release", "debug"].iter().find_map(|profile| {
            let p = repo_root
                .join(rel)
                .join("ash")
                .join("target")
                .join(profile)
                .join("ash.exe");
            p.is_file().then_some(p)
        })
    })
}

/// 用例通用开头:拿不到 ash 就打印 skip 说明并返回 None(调用方 return)。
fn ensure_ash() -> Option<PathBuf> {
    let bin = ash_bin().or_else(|| {
        eprintln!(
            "skip: AUTOTERM_ASH_BIN unset/invalid, sibling ash.exe not found \
             (../auto-shell/ash/target/{{release,debug}}/ash.exe)"
        );
        None
    });
    if let Some(p) = &bin {
        eprintln!("ash under test: {}", p.display());
    }
    bin
}

/// 可见网格文本(断言与超时诊断共用)。
fn grid_text(s: &PtySession) -> String {
    s.term.visible_lines().join("\n")
}

/// 轮询 drain 直到谓词成立或超时;超时 panic 附网格快照。
fn wait_for<F: Fn(&TermSession) -> bool>(s: &mut PtySession, timeout: Duration, what: &str, pred: F) {
    let deadline = Instant::now() + timeout;
    loop {
        s.drain();
        if pred(&s.term) {
            return;
        }
        if Instant::now() >= deadline {
            panic!("等待 {what} 超时({timeout:?});当前网格:\n{}", grid_text(s));
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// 等待 ash 首个提示符:任意可见行非空(裸 `ash` 即 REPL,无需参数;
/// 首启会建 ~/.ashrc,冷启动偏慢,预算 15s)。
fn spawn_ash_with_prompt(bin: &Path) -> PtySession {
    let mut s = PtySession::spawn(
        bin.to_str().expect("ash 路径非 UTF-8"),
        std::iter::empty::<&str>(),
        80,
        24,
    )
    .expect("spawn ash 失败");
    wait_for(&mut s, Duration::from_secs(15), "首个提示符(非空行)", |t| {
        t.visible_lines().iter().any(|l| !l.trim().is_empty())
    });
    s
}

/// ① 启动提示符:裸 ash 启动后网格出现非空行;提示符能画出即间接
/// 行使了 DSR/DA 应答路径(term.pump 回写,pty.rs 约定)。
#[test]
fn spawn_prompt_renders() {
    let Some(bin) = ensure_ash() else { return };
    let s = spawn_ash_with_prompt(&bin);
    assert!(
        s.bytes_fed() > 0,
        "提示符出现前必有字节喂入仿真核心(bytes_fed={})",
        s.bytes_fed()
    );
}

/// 轮询 exited() 直到成立或超时(ConPTY 平台事实:主端读流无 EOF,
/// 退出检测只能走 try_wait,见 pty.rs:10-14)。
fn wait_exit(s: &mut PtySession, timeout: Duration, what: &str) {
    let deadline = Instant::now() + timeout;
    while !s.exited() {
        assert!(
            Instant::now() < deadline,
            "等待 {what} 退出超时;网格:\n{}",
            grid_text(s)
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// ④a `exit` 命令退出:启动横幅明示 "Type 'exit' or press Ctrl+D
/// to exit";exit 后子进程确已退出、无孤儿(Drop 兜底之外的正路径)。
#[test]
fn exit_command_terminates() {
    let Some(bin) = ensure_ash() else { return };
    let mut s = spawn_ash_with_prompt(&bin);
    s.write_input(b"exit\r");
    wait_exit(&mut s, Duration::from_secs(10), "exit 命令");
}

/// ④b Ctrl+D(0x04)EOF 退出:空提示符行上发送;至多两次(部分行
/// 编辑器需要二次确认),间隔 3s,退出总预算 10s。
#[test]
fn ctrl_d_eof_terminates() {
    let Some(bin) = ensure_ash() else { return };
    let mut s = spawn_ash_with_prompt(&bin);
    s.write_input(b"\x04");
    let retry_deadline = Instant::now() + Duration::from_secs(3);
    while !s.exited() {
        if Instant::now() >= retry_deadline {
            s.write_input(b"\x04"); // 第二次 EOF 确认
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    wait_exit(&mut s, Duration::from_secs(10), "Ctrl+D EOF");
}

/// ⑤ resize 存活:80x24 → 120x40 后仿真核心尺寸同步、ash 继续响应
/// (ConPTY resize → ash 收到窗口变更;REPL 不崩、不糊)。
#[test]
fn resize_survives() {
    let Some(bin) = ensure_ash() else { return };
    let mut s = spawn_ash_with_prompt(&bin);
    let marker1 = format!("before_resize_{}", std::process::id());
    s.write_input(format!("echo {marker1}\r").as_bytes());
    wait_for(&mut s, Duration::from_secs(10), "resize 前 marker 上屏", |t| {
        t.visible_lines().iter().any(|l| l.contains(&marker1))
    });
    s.resize(120, 40);
    assert_eq!(s.term.size(), (120, 40), "仿真核心尺寸应同步 resize");
    let marker2 = format!("after_resize_{}", std::process::id());
    s.write_input(format!("echo {marker2}\r").as_bytes());
    wait_for(&mut s, Duration::from_secs(10), "resize 后 marker 上屏", |t| {
        t.visible_lines().iter().any(|l| l.contains(&marker2))
    });
    assert!(!s.exited(), "resize 不应杀死 shell");
}

/// ⑥ 色彩深度自证:`color info` 把 ash 的判定打印进网格
/// (commands.rs:95-110);TERM=alacritty + COLORTERM=truecolor
/// (pty.rs:63-64)下应为 "24-bit truecolor"。不符即 R4 风险坐实,
/// 按发现流程记档,不静默迎合。
#[test]
fn color_info_reports_truecolor() {
    let Some(bin) = ensure_ash() else { return };
    let mut s = spawn_ash_with_prompt(&bin);
    s.write_input(b"color info\r");
    wait_for(&mut s, Duration::from_secs(10), "color info 输出上屏", |t| {
        t.visible_lines()
            .iter()
            .any(|l| l.contains("Color depth:"))
    });
    let line = s
        .term
        .visible_lines()
        .iter()
        .find(|l| l.contains("Color depth:"))
        .expect("上面 wait_for 已保证存在")
        .trim()
        .to_string();
    eprintln!("ash color info: {line}");
    assert!(
        line.contains("24-bit truecolor"),
        "R4:TERM=alacritty+COLORTERM=truecolor 下 ash 应判 24-bit;实际: {line}"
    );
}

/// ② echo 往返:键入行由 reedline 自渲染、命令输出由 ash 写回,
/// marker 上屏即闭环(键盘 → PTY → shell → 仿真核心 → 网格)。
#[test]
fn echo_roundtrip() {
    let Some(bin) = ensure_ash() else { return };
    let mut s = spawn_ash_with_prompt(&bin);
    let marker = format!("ash_probe_{}", std::process::id());
    s.write_input(format!("echo {marker}\r").as_bytes());
    wait_for(&mut s, Duration::from_secs(10), "echo marker 上屏", |t| {
        t.visible_lines().iter().any(|l| l.contains(&marker))
    });
}

/// ③ Ctrl+C(0x03)废弃当前输入行:空闲提示符下键入未回车的文本,
/// reedline 自渲染;发 0x03 后出现**新的提示符行**(❯ 计数 +1),shell
/// 存活且继续执行后续命令。这守的是终端层可负责的语义:0x03 字节
/// 如实转发、raw-mode 应用(reedline)能消费。
///
/// [发现 F2 → 003/DEBTS] "Ctrl+C 中断**运行中**命令"在此通路不可用
/// 且**与 shell 无关**:向 portable-pty ConPTY 主端写 0x03 不会触发
/// conhost 的 CTRL_C_EVENT(cmd/pwsh 对照探针同样失败,见下方
/// `#[ignore]` 复现器)。与 [发现 F1](内建命令)叠加,当前 ash 下
/// 任何长命令都无法用 Ctrl+C 打断。修复通路在 portable-pty/OS
/// conhost 层,非本仓代码。
#[test]
fn ctrl_c_aborts_input_line() {
    let Some(bin) = ensure_ash() else { return };
    let mut s = spawn_ash_with_prompt(&bin);
    let prompt_count = |s: &PtySession| {
        s.term
            .visible_lines()
            .iter()
            .filter(|l| l.contains('❯'))
            .count()
    };
    s.write_input(b"echo should_not_run_123");
    wait_for(&mut s, Duration::from_secs(10), "已键入文本上屏", |t| {
        t.visible_lines()
            .iter()
            .any(|l| l.contains("should_not_run_123"))
    });
    let before = prompt_count(&s);
    s.write_input(b"\x03");
    wait_for(&mut s, Duration::from_secs(10), "Ctrl+C 后新提示符", |t| {
        t.visible_lines().iter().filter(|l| l.contains('❯')).count() > before
    });
    // shell 存活且继续响应:紧跟的命令必须能跑
    let marker = format!("after_c_{}", std::process::id());
    s.write_input(format!("echo {marker}\r").as_bytes());
    wait_for(&mut s, Duration::from_secs(10), "废弃行后 marker 上屏", |t| {
        t.visible_lines().iter().any(|l| l.contains(&marker))
    });
    assert!(!s.exited(), "空闲提示符的 Ctrl+C 不应杀死 shell");
}

/// [F2 复现器,修 DEBTS 该条后去掉 ignore 验证] `cmd /c ping -n 30`
/// 是纯 cooked-mode 对照:0x03 若被 conhost 翻译成 CTRL_C_EVENT,cmd
/// 会打印 ^C 并随 ping 终止而退出。当前观测:5s 内 cmd 不退出、无 ^C
/// ——主端写字节不触发事件。pwsh 版见 `f2_repro_pwsh_interrupt`。
#[test]
#[ignore = "F2 已知缺陷复现器(见 docs/designs/003-ash-compatibility.md 与 DEBTS);修复后去 ignore 验证"]
fn f2_repro_cmd_interrupt() {
    let mut s =
        PtySession::spawn("cmd", ["/c", "ping", "-n", "30", "127.0.0.1"], 80, 24).expect("spawn");
    wait_for(&mut s, Duration::from_secs(10), "ping 输出", |t| {
        t.visible_lines().iter().any(|l| l.contains("Reply from"))
    });
    s.write_input(b"\x03");
    wait_for(&mut s, Duration::from_secs(5), "cmd 随 CTRL_C_EVENT 退出", |_| false);
}

/// [F2 复现器(pwsh 版)] 交互 pwsh 跑 ping,0x03 后 marker 应在 10s
/// 内上屏(ping 30s 不可能自然结束)。当前观测:超时——pwsh 基于
/// PSReadLine,空闲提示符的 Ctrl+C 可用(自读 0x03),运行中命令的
/// 中断与 cmd 一样依赖 conhost 事件,同样断裂。
#[test]
#[ignore = "F2 已知缺陷复现器(见 docs/designs/003-ash-compatibility.md 与 DEBTS);修复后去 ignore 验证"]
fn f2_repro_pwsh_interrupt() {
    let mut s = PtySession::spawn("pwsh", std::iter::empty::<&str>(), 80, 24).expect("spawn");
    wait_for(&mut s, Duration::from_secs(15), "pwsh 提示符", |t| {
        t.visible_lines()
            .iter()
            .any(|l| l.contains("PS") && l.trim().ends_with('>'))
    });
    s.write_input(b"ping -n 30 127.0.0.1\r");
    wait_for(&mut s, Duration::from_secs(10), "ping 输出", |t| {
        t.visible_lines().iter().any(|l| l.contains("Reply from"))
    });
    s.write_input(b"\x03");
    let marker = format!("probe_marker_{}", std::process::id());
    s.write_input(format!("echo {marker}\r").as_bytes());
    wait_for(&mut s, Duration::from_secs(10), "中断后 marker 上屏", |t| {
        t.visible_lines().iter().any(|l| l.contains(&marker))
    });
}
