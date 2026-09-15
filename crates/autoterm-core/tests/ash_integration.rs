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
//! ⑤ resize 存活 ⑥ 色彩深度自证 ⑦(PLAN-008)事件注入下的 ash
//! 行为三态矩阵(idle/builtin/external)。
//!
//! PLAN-008 注:007 的两个 F2 `#[ignore]` 复现器(cmd/pwsh 版)已
//! 退役——其"运行中命令可中断"语义由 `tests/ctrl_event.rs` 门禁
//! 转正(事件注入修复);本文件保留 ash×事件的行为探针。
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
/// reedline 自渲染;发 0x03 后出现**新的提示符行**(提示符计数 +1),shell
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
    // PLAN-018 F-3 修复:ash Plan 322 起**缺省提示符符号**从 `❯` 改为
    // 模式感知 `>`(ash/ash/src/frontend/repl.rs update_prompt 注记),
    // 测试曾因此双测齐红(计数恒 0)。行首 `>` 为现行缺省;`❯` 兜底
    // 兼容旧 ash/用户配置覆盖。
    let is_prompt_line =
        |l: &String| l.trim_start().starts_with('>') || l.contains('❯');
    let prompt_count = |s: &PtySession| {
        s.term.visible_lines().iter().filter(|l| is_prompt_line(l)).count()
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
        t.visible_lines().iter().filter(|l| is_prompt_line(l)).count() > before
    });
    // shell 存活且继续响应:紧跟的命令必须能跑
    let marker = format!("after_c_{}", std::process::id());
    s.write_input(format!("echo {marker}\r").as_bytes());
    wait_for(&mut s, Duration::from_secs(10), "废弃行后 marker 上屏", |t| {
        t.visible_lines().iter().any(|l| l.contains(&marker))
    });
    assert!(!s.exited(), "空闲提示符的 Ctrl+C 不应杀死 shell");
}

/// [PLAN-008 ⑦] 事件注入(Break→C 双发)下的 ash 行为三态矩阵:
/// idle(空闲提示符)/ builtin(`sleep 8` 内建)/ external(外部
/// timeout)。**实测(26200)idle 态事件整体终止 ash**(无 ctrl
/// handler,默认 handler 生效)——UI auto 模式给 ash 豁免(仅字节
/// 路径)的直接依据;builtin 态事件惰性(ash 存活,内建自然结束
/// 后才响应);external 态子进程被 Break 终止后 ash 回归。
/// 断言用**析取**(Responsive | Exited)钉死"事件确实到达",具体
/// 分支如实记录(执行记录 + designs/003 §4.1),不强行规定 ash
/// 该死该活——那是 #13 的裁定面。
#[test]
fn ctrl_c_event_effect_on_ash() {
    let Some(bin) = ensure_ash() else { return };

    #[derive(Debug)]
    enum Outcome {
        /// marker 上屏(ash 存活);late = 晚于 2s(内建/子进程自然
        /// 结束后才响应,即事件在内建期惰性)
        Responsive { late: bool },
        Exited,
    }
    /// interrupt 后投 marker,预算内判定:进程退出 or marker 可跑。
    /// 并行测试负载下 conhost 事件派发偶发丢失——每 3s 周期补发一次
    /// (interrupt 幂等,双投递语义不变;把"丢失"转化为"延迟")。
    fn probe_outcome_with(s: &mut PtySession, tag: &str, budget: Duration) -> Outcome {
        let sent = Instant::now();
        assert!(s.interrupt(), "helper 在场应广播成功({tag})");
        let marker = format!("ash_ev_{tag}_{}", std::process::id());
        s.write_input(format!("echo {marker}\r").as_bytes());
        let deadline = sent + budget;
        let mut next_resend = sent + Duration::from_secs(3);
        let mut resends = 0u32;
        loop {
            s.drain();
            if s.exited() {
                eprintln!("[008 探针矩阵] {tag}: Exited(事件终止 ash 进程,补发 {resends})");
                return Outcome::Exited;
            }
            if s.term.visible_lines().iter().any(|l| l.contains(&marker)) {
                let late = sent.elapsed() > Duration::from_secs(2);
                eprintln!(
                    "[008 探针矩阵] {tag}: Responsive(late={late},marker 上屏,ash 存活,补发 {resends})"
                );
                return Outcome::Responsive { late };
            }
            let now = Instant::now();
            if now >= next_resend && now < deadline {
                resends += 1;
                next_resend = now + Duration::from_secs(3);
                eprintln!("[008 探针矩阵] {tag}: 尚无果,周期补发 interrupt(第 {resends})");
                let _ = s.interrupt();
            }
            assert!(
                now < deadline,
                "{tag}: 预算 {budget:?} 内既不退出也不响应(补发 {resends});当前网格:\n{}",
                grid_text(s)
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    // idle:空闲提示符直接注入(实测:Exited——auto 豁免依据)
    {
        let mut s = spawn_ash_with_prompt(&bin);
        probe_outcome_with(&mut s, "idle", Duration::from_secs(10));
    }
    // builtin:sleep 8 内建阻塞中注入(F1:raw mode 不读 stdin)。
    // 实测事件惰性:ash 存活,sleep 8 自然结束后 marker 才上屏
    // (Responsive-late);预算 15s 覆盖两分支。
    {
        let mut s = spawn_ash_with_prompt(&bin);
        s.write_input(b"sleep 8\r");
        wait_for(&mut s, Duration::from_secs(10), "sleep 8 上屏", |t| {
            t.visible_lines().iter().any(|l| l.contains("sleep 8"))
        });
        let out = probe_outcome_with(&mut s, "builtin", Duration::from_secs(20));
        assert!(
            matches!(out, Outcome::Responsive { late: true } | Outcome::Exited),
            "builtin 态:应惰性存活(自然结束响应)或终止,得 {out:?}"
        );
    }
    // external:外部 timeout(Break 可终止)运行中注入——ash 外部
    // 子进程路径(frontend/subprocess.rs 临时退 raw mode)应正常
    // 中断回提示符。ping 对 Break 免疫属已知残留缺口,不作矩阵对象。
    {
        let mut s = spawn_ash_with_prompt(&bin);
        s.write_input(b"timeout /t 30\r");
        wait_for(&mut s, Duration::from_secs(10), "timeout 倒计时", |t| {
            t.visible_lines()
                .iter()
                .any(|l| {
                    l.contains("Waiting")
                        || l.contains("等待")
                        // ash 状态行倒计时(`⏳ timeout /t 30 · 10.0s · …`)
                        || l.contains('⏳')
                })
        });
        probe_outcome_with(&mut s, "external", Duration::from_secs(15));
    }
}
