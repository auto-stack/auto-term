//! 真 PTY 集成测试:spawn `cmd /c echo hello_term_probe`,字节流喂进
//! alacritty_terminal 仿真核心,断言输出出现在网格里。
//! (PLAN-001 回归种子,PLAN-002 起经 PtySession 走完整会话路径)

use std::thread;
use std::time::{Duration, Instant};

use autoterm_core::PtySession;

#[test]
fn live_pty_output_reaches_grid() {
    let mut session =
        PtySession::spawn("cmd", ["/c", "echo", "hello_term_probe"], 80, 24)
            .expect("spawn cmd");

    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        session.drain();
        let text = session.term.visible_lines().join("\n");
        if text.contains("hello_term_probe") {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "等待子进程输出超时;网格:\n{text}"
        );
        thread::sleep(Duration::from_millis(20));
    }
}
