//! Ctrl+C 事件注入门禁(PLAN-008,DEBTS #12/F2 修复验收)。
//!
//! cmd 恒在 Windows 场景可得 → 本文件是 #12 的**必跑**直接门禁,
//! 不依赖 ash;PLAN-007 留档的两个 `#[ignore]` 复现器语义在此转正。
//! pwsh 缺席显式 skip(007 风格)。
//!
//! **执行期实证改写(OS build 26200.9168,T3 诊断)**:ConPTY 客户端
//! 对 CTRL_C_EVENT 广播免疫、CTRL_BREAK_EVENT 可达 → interrupt() =
//! C→Break 双发。挂死命令主体选 **timeout**(默认 handler,Break 即
//! 终止);**ping 类对 Break 特殊处理(打印统计后继续)是本 build 的
//! 残留缺口**,以 `#[ignore]` 复现器留档(健康 build 上预期该用例
//! 失败,即 C 通道修复面到位)。详见 designs/003 §4.1。
//!
//! 注:辅助进程解析三级里有两级是 env(见 pty.rs `resolve_helper_bin`),
//! 三用例共享测试进程 → 以 ENV_LOCK 串行防互染(cargo test 默认并行)。
//!
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock, PoisonError};
use std::time::{Duration, Instant};

use autoterm_core::{PtySession, TermSession};

/// 串行锁:③ 改 env 的窗口内,①/② 的 interrupt() 不能并发解析。
static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

// edition 2024:env 写操作 unsafe。SAFETY:本文件所有用例经
// ENV_LOCK 串行,进程内无并发 env 访问(其他测试文件为独立进程)。
fn set_env(k: &str, v: Option<String>) {
    // SAFETY: 见上——ENV_LOCK 保证串行。
    unsafe {
        match v {
            Some(v) => std::env::set_var(k, v),
            None => std::env::remove_var(k),
        }
    }
}

/// 可见网格文本(断言与超时诊断共用;与 ash_integration.rs 同源,
/// 文件自包含,不建 tests/common)。
fn grid_text(s: &PtySession) -> String {
    s.term.visible_lines().join("\n")
}

/// 轮询 drain 直到谓词成立或超时;超时 panic 附网格快照。
fn wait_for<F: Fn(&TermSession) -> bool>(
    s: &mut PtySession,
    timeout: Duration,
    what: &str,
    pred: F,
) {
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

/// 轮询会话退出直到超时;超时 panic 附网格快照。
fn wait_exit(s: &mut PtySession, timeout: Duration, what: &str) {
    let deadline = Instant::now() + timeout;
    loop {
        if s.exited() {
            return;
        }
        s.drain(); // 顺带刷新网格,供超时诊断
        if Instant::now() >= deadline {
            panic!("{what} 超时({timeout:?});当前网格:\n{}", grid_text(s));
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// timeout 命令的输出标记(中英系统各取其一)。
fn timeout_marker(t: &TermSession) -> bool {
    t.visible_lines()
        .iter()
        .any(|l| l.contains("Waiting") || l.contains("等待"))
}

/// ① #12 直接门禁:interrupt()(C→Break 双发)后,`cmd /c timeout
/// /t 30` 会话 ≤5s 真实退出。0x03 字节路径做不到这一点(007 F2
/// 复现器实测:写字节后 5s 不退、无 ^C)。
#[test]
fn interrupt_terminates_cmd_child() {
    let _guard = lock_env();
    let mut s = PtySession::spawn(
        "cmd",
        ["/c", "timeout", "/t", "30"],
        80,
        24,
    )
    .expect("spawn cmd 失败");
    wait_for(&mut s, Duration::from_secs(10), "timeout 倒计时输出", |t| {
        timeout_marker(t)
    });
    assert!(s.interrupt(), "helper 在场(同 crate 构建产物)应广播成功");
    wait_exit(&mut s, Duration::from_secs(5), "interrupt 后 cmd 会话退出");
}

/// ② pwsh 版(007 `f2_repro_pwsh_interrupt` 语义转正):中断运行中
/// 命令后提示符回归、shell 存活。挂死命令用 timeout(Break 可终止;
/// 实测 pwsh 不杀对 Break 免疫的 ping 子进程——那属残留缺口,见
/// `ping_class_survives_dual_send_on_quirky_build`)。pwsh 非所有
/// 环境必装 → 缺席显式 skip。
#[test]
fn interrupt_returns_pwsh_prompt() {
    let _guard = lock_env();
    if std::process::Command::new("pwsh")
        .arg("--version")
        .output()
        .is_err()
    {
        eprintln!("skip: pwsh 不可用(未安装或不在 PATH)");
        return;
    }
    let mut s =
        PtySession::spawn("pwsh", std::iter::empty::<&str>(), 80, 24).expect("spawn pwsh 失败");
    wait_for(&mut s, Duration::from_secs(15), "pwsh 提示符", |t| {
        t.visible_lines()
            .iter()
            .any(|l| l.contains("PS") && l.trim().ends_with('>'))
    });
    s.write_input(b"timeout /t 30\r");
    wait_for(&mut s, Duration::from_secs(10), "timeout 倒计时输出", |t| {
        timeout_marker(t)
    });
    assert!(s.interrupt());
    let marker = format!("probe_marker_{}", std::process::id());
    s.write_input(format!("echo {marker}\r").as_bytes());
    wait_for(&mut s, Duration::from_secs(10), "中断后 marker 上屏", |t| {
        t.visible_lines().iter().any(|l| l.contains(&marker))
    });
    assert!(!s.exited(), "pwsh 应存活回提示符,而非被事件杀死");
}

/// ③ 降级门禁:helper 失败时(此处把解析指到 `where.exe`——拿
/// shell pid 当文件名搜索必非零退出)interrupt() 返 false、不 panic、
/// 降级写 0x03 字节,会话存活(= 修复前行为,部署不完整的兜底)。
#[test]
fn interrupt_falls_back_on_helper_failure() {
    let _guard = lock_env();
    let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
    let fake = PathBuf::from(system_root).join("System32").join("where.exe");
    assert!(fake.is_file(), "测试前提:where.exe 存在于 {fake:?}");
    let saved_override = std::env::var("AUTOTERM_CTRLC_BIN").ok();
    let saved_cargo = std::env::var("CARGO_BIN_EXE_AUTOTERM_CTRLC").ok();
    set_env("AUTOTERM_CTRLC_BIN", Some(fake.to_string_lossy().into_owned()));
    set_env("CARGO_BIN_EXE_AUTOTERM_CTRLC", None);
    let mut s = PtySession::spawn(
        "cmd",
        ["/c", "timeout", "/t", "30"],
        80,
        24,
    )
    .expect("spawn cmd 失败");
    wait_for(&mut s, Duration::from_secs(10), "timeout 倒计时输出", |t| {
        timeout_marker(t)
    });
    let ok = s.interrupt();
    // 还原 env(①/② 与后续任何测试依赖正常三级解析)
    set_env("AUTOTERM_CTRLC_BIN", saved_override);
    set_env("CARGO_BIN_EXE_AUTOTERM_CTRLC", saved_cargo);
    assert!(!ok, "伪 helper(where.exe)必然非零退出,应降级返 false");
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        assert!(
            !s.exited(),
            "降级字节路径不应中断运行中的 timeout(F2 平台事实)"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// [残留缺口复现器,健康 build 上预期失败(即去 ignore 验证 C 通道)]
/// ping 对 CTRL_BREAK 特殊处理(打印统计后**继续**);本 build
/// (26200.9168)CTRL_C 广播被吞,故双发后 ping 仍存活——reply 数
/// 持续增长。健康 OS/build 上 C 通道会终止 ping,本用例转绿即可
/// 移除。证据链见 designs/003 §4.1(008 T3 探针矩阵)。
#[test]
#[ignore = "26200 残留缺口:CTRL_C 广播被吞 + ping 类对 Break 免疫;健康 build 预期失败"]
fn ping_class_survives_dual_send_on_quirky_build() {
    let _guard = lock_env();
    let mut s = PtySession::spawn(
        "cmd",
        ["/c", "ping", "-n", "30", "127.0.0.1"],
        80,
        24,
    )
    .expect("spawn cmd 失败");
    wait_for(&mut s, Duration::from_secs(10), "ping 输出", |t| {
        t.visible_lines().iter().any(|l| l.contains("Reply from"))
    });
    let replies = |s: &PtySession| {
        s.term
            .visible_lines()
            .iter()
            .filter(|l| l.contains("Reply from"))
            .count()
    };
    assert!(s.interrupt());
    std::thread::sleep(Duration::from_secs(3));
    let n1 = replies(&s);
    std::thread::sleep(Duration::from_secs(3));
    s.drain();
    let n2 = replies(&s);
    assert!(
        n2 > n1,
        "本 build 预期 ping 存活(reply {n1} -> {n2});若稳定则 C 通道已修复,可去 ignore"
    );
}
