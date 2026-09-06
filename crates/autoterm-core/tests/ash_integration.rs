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
