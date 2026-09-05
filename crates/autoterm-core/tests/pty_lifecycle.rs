//! PTY 会话生命周期回归:spawn → 输出 → 子进程退出(try_wait)→ kill+wait。
//! (PLAN-002 T2)
//!
//! ConPTY 平台事实(见 pty.rs 模块头):会话持活期间主端读流无 EOF,
//! 退出检测必须走 try_wait;本测试同时断言该行为(EOF 不来)以防回归。

use std::thread;
use std::time::{Duration, Instant};

use autoterm_core::PtySession;

#[test]
fn spawn_output_exit_kill_wait() {
    let mut session =
        PtySession::spawn("cmd", ["/c", "echo", "x"], 80, 24).expect("spawn cmd");

    // 输出到达网格(echo x 输出即结束,不等 EOF——ConPTY 不发)
    let deadline = Instant::now() + Duration::from_secs(15);
    let text = loop {
        session.drain();
        let text = session.term.visible_lines().join("\n");
        if text.contains('x') {
            break text;
        }
        assert!(Instant::now() < deadline, "等待输出超时;网格:\n{text}");
        thread::sleep(Duration::from_millis(20));
    };
    assert!(text.contains('x'), "网格应含输出 x;实际:\n{text}");

    // 子进程退出:try_wait 语义(不依赖 EOF)
    let exit_deadline = Instant::now() + Duration::from_secs(5);
    while !session.exited() {
        assert!(Instant::now() < exit_deadline, "子进程未退出");
        thread::sleep(Duration::from_millis(10));
    }
    // ConPTY 平台事实:会话持活期间 EOF 不来(若将来来了,此断言失败
    // 说明平台行为变化,需回看 docs/designs/001 的生命周期模型)
    assert!(!session.eof(), "ConPTY 持活期间不应有 EOF");

    // kill + wait 拿到退出状态(kill 幂等,对已退出进程不炸)
    session.kill();
    let _status = session.wait().expect("kill 后 wait 应返回退出状态");
}
