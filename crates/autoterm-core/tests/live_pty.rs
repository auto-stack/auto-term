//! 真 PTY 集成测试:spawn `cmd /c echo hello_term_probe`,字节流喂进
//! alacritty_terminal 仿真核心,断言输出出现在网格里(spike 期唯一自动断言)。

use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use portable_pty::{CommandBuilder, PtySize};
use autoterm_core::TermSession;

/// 从阻塞 reader 收集全部输出直到 EOF,带总超时保护。
fn drain_until_eof(reader: Box<dyn Read + Send>, timeout: Duration) -> Vec<u8> {
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    thread::spawn(move || {
        let mut reader = reader;
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => {
                    let _ = tx.send(Vec::new());
                    break;
                }
                Ok(n) => {
                    if tx.send(buf[..n].to_vec()).is_err() {
                        break;
                    }
                }
            }
        }
    });
    let mut all = Vec::new();
    let deadline = Instant::now() + timeout;
    loop {
        let now = Instant::now();
        assert!(now < deadline, "等待子进程 EOF 超时");
        match rx.recv_timeout(deadline - now) {
            Ok(chunk) if chunk.is_empty() => break,
            Ok(chunk) => all.extend_from_slice(&chunk),
            Err(_) => break,
        }
    }
    all
}

#[test]
fn live_pty_output_reaches_grid() {
    let pty_system = portable_pty::native_pty_system();
    let pair = pty_system.openpty(PtySize::default()).expect("openpty");

    let mut cmd = CommandBuilder::new("cmd");
    cmd.arg("/c");
    cmd.arg("echo");
    cmd.arg("hello_term_probe");
    let mut child = pair.slave.spawn_command(cmd).expect("spawn cmd");
    drop(pair.slave);

    let reader = pair.master.try_clone_reader().expect("reader");
    let mut writer = pair.master.take_writer().expect("writer");

    let bytes = drain_until_eof(reader, Duration::from_secs(15));
    assert!(!bytes.is_empty(), "子进程应有输出");

    let mut session = TermSession::new(80, 24);
    session.feed(&bytes);
    let answers = session.pump();
    // ConPTY 会替子进程发查询;应答回写与否不影响本断言,但保持真实链路形状。
    if !answers.is_empty() {
        writer.write_all(&answers).expect("write answers");
    }

    let text = session.visible_lines().join("\n");
    assert!(
        text.contains("hello_term_probe"),
        "网格应包含子进程输出;实际网格:\n{text}"
    );

    let _ = child.kill();
    let _ = child.wait();
}
