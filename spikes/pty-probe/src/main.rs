//! pty-probe — 宿主 PTY 申请 + VT 字节流探针
//!
// SPDX-License-Identifier: Apache-2.0
//! spike 代码:非正式架构,允许整体重写(docs/plans/001)。
//!
//! 流程:portable-pty 申请 PTY → spawn shell → 主端读到超时为止,
//! 字节原样落 stderr(`--hexdump` 换可读 hex 形态)+ 字节率统计到 stdout。

use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use clap::Parser;
use portable_pty::{CommandBuilder, PtySize};

/// 宿主 PTY 申请 + VT 字节流探针(spike)。
#[derive(Parser, Debug)]
#[command(name = "pty-probe", about = "宿主 PTY 申请 + VT 字节流探针(spike)")]
struct Args {
    /// 要 spawn 的 shell 可执行文件(默认 pwsh,缺失时回退 powershell)
    #[arg(long, default_value = "pwsh")]
    shell: String,

    /// 主端读取时长(秒)
    #[arg(long, default_value = "3")]
    duration: u64,

    /// stderr 以 hexdump 可读形态输出(默认字节原样)
    #[arg(long)]
    hexdump: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let shell = resolve_shell(&args.shell);

    let pty_system = portable_pty::native_pty_system();
    let pair = pty_system
        .openpty(PtySize::default())
        .context("openpty: 基座 PTY 申请失败")?;

    let cmd = CommandBuilder::new(&shell);
    let mut child = pair
        .slave
        .spawn_command(cmd)
        .with_context(|| format!("spawn {shell} 失败"))?;
    // 从端句柄用后即弃:保留它会阻止 EOF 传播。
    drop(pair.slave);

    let reader = pair.master.try_clone_reader().context("take reader")?;
    let _writer = pair.master.take_writer().context("take writer")?;

    let start = Instant::now();
    let (total, all_bytes) = read_until_timeout(reader, Duration::from_secs(args.duration), args.hexdump);
    let _ = child.kill();

    let elapsed = start.elapsed();
    eprintln!(); // 字节流后换行,避免统计与裸字节粘连
    println!("shell: {shell}");
    println!("duration_s: {:.3}", elapsed.as_secs_f64());
    println!("bytes_total: {total}");
    println!("bytes_per_sec: {:.1}", total as f64 / elapsed.as_secs_f64());
    println!(
        "esc_seq_1b5b_found: {}",
        find_subsequence(&all_bytes, &[0x1b, 0x5b])
    );
    Ok(())
}

/// 默认 shell 解析:显式传入则原样使用;`pwsh` 不在 PATH 时回退 `powershell`。
fn resolve_shell(shell: &str) -> String {
    if shell != "pwsh" || which("pwsh") {
        shell.to_string()
    } else {
        eprintln!("pwsh 不在 PATH,回退 powershell");
        "powershell".to_string()
    }
}

fn which(prog: &str) -> bool {
    let probe = if cfg!(windows) {
        std::process::Command::new("where").arg(prog).output()
    } else {
        std::process::Command::new("which").arg(prog).output()
    };
    matches!(probe, Ok(out) if out.status.success())
}

/// 阻塞 reader 交给后台线程,主线程按 deadline 收割,直到超时/EOF。
fn read_until_timeout(
    mut reader: Box<dyn Read + Send>,
    duration: Duration,
    hexdump: bool,
) -> (usize, Vec<u8>) {
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => {
                    let _ = tx.send(Vec::new()); // EOF 哨兵
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

    let deadline = Instant::now() + duration;
    let mut all = Vec::new();
    loop {
        let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
            break;
        };
        match rx.recv_timeout(remaining) {
            Ok(chunk) if chunk.is_empty() => break, // EOF
            Ok(chunk) => all.extend_from_slice(&chunk),
            Err(mpsc::RecvTimeoutError::Timeout) => break,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    let mut stderr = std::io::stderr().lock();
    if hexdump {
        write_hexdump(&mut stderr, &all);
    } else {
        let _ = stderr.write_all(&all);
    }
    (all.len(), all)
}

fn write_hexdump(out: &mut impl Write, bytes: &[u8]) {
    for (i, chunk) in bytes.chunks(16).enumerate() {
        let _ = write!(out, "{:08x}  ", i * 16);
        for b in chunk {
            let _ = write!(out, "{b:02x} ");
        }
        let _ = writeln!(out);
    }
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}
