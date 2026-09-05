//! term-probe bin — PTY 源 → 仿真核心 → 定时打印网格可见文本(spike)
//!
// SPDX-License-Identifier: Apache-2.0
//! spike 代码:非正式架构,允许整体重写(docs/plans/001)。

use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use clap::Parser;
use portable_pty::{CommandBuilder, PtySize};
use term_probe::TermSession;

#[derive(Parser, Debug)]
#[command(name = "term-probe", about = "alacritty_terminal 嵌入探针(headless)")]
struct Args {
    /// 要 spawn 的 shell 可执行文件
    #[arg(long, default_value = "pwsh")]
    shell: String,

    /// 总运行时长(秒)
    #[arg(long, default_value = "6")]
    duration: u64,

    /// 网格快照打印间隔(毫秒)
    #[arg(long, default_value = "700")]
    interval_ms: u64,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let pty_system = portable_pty::native_pty_system();
    let pair = pty_system.openpty(PtySize::default()).context("openpty")?;
    let mut cmd = CommandBuilder::new(&args.shell);
    cmd.arg("-NoLogo");
    let mut child = pair
        .slave
        .spawn_command(cmd)
        .with_context(|| format!("spawn {}", args.shell))?;
    drop(pair.slave);

    let reader = pair.master.try_clone_reader().context("reader")?;
    let mut writer = pair.master.take_writer().context("writer")?;

    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    thread::spawn(move || {
        let mut reader = reader;
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => {
                    let _ = tx.send(Vec::new());
                    break;
                }
                Ok(n) => {
                    if tx.send(buf[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let mut session = TermSession::new(80, 24);
    let start = Instant::now();
    let deadline = start + Duration::from_secs(args.duration);
    let mut next_snapshot = start + Duration::from_millis(args.interval_ms);
    let mut total_bytes = 0usize;
    let mut total_answers = 0usize;

    loop {
        let now = Instant::now();
        if now >= deadline || session.exited() {
            break;
        }
        let wait = next_snapshot.saturating_duration_since(now);
        match rx.recv_timeout(wait.min(deadline - now)) {
            Ok(chunk) if chunk.is_empty() => {
                println!("--- EOF(子进程输出结束)@ {:.2}s", start.elapsed().as_secs_f64());
                break;
            }
            Ok(chunk) => {
                total_bytes += chunk.len();
                session.feed(&chunk);
                let answers = session.pump();
                if !answers.is_empty() {
                    total_answers += answers.len();
                    writer.write_all(&answers).ok();
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        if Instant::now() >= next_snapshot {
            next_snapshot += Duration::from_millis(args.interval_ms);
            println!("=== 网格快照 @ {:.2}s ===", start.elapsed().as_secs_f64());
            for (i, line) in session.visible_lines().iter().enumerate() {
                if !line.is_empty() {
                    println!("{i:02}| {line}");
                }
            }
        }
    }

    let _ = child.kill();
    println!(
        "stats: bytes_fed={total_bytes} answers_written={total_answers} elapsed={:.2}s",
        start.elapsed().as_secs_f64()
    );
    Ok(())
}
