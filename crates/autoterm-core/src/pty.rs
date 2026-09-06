//! autoterm-core::pty — PTY 会话:spawn、reader 线程、答案回写、
//! resize、kill/wait 生命周期。
//!
// SPDX-License-Identifier: Apache-2.0
//! 约定(继承 PLAN-001 结论,docs/designs/001):
//! - reader 线程独占主端读句柄,经 channel 送字节块;空 Vec 是 EOF 哨兵;
//! - DSR/DA 应答必须回写主端(`Event::PtyWrite`),否则 shell 不画提示符;
//! - resize 先仿真核心后 ConPTY。
//!
//! **ConPTY 平台事实(PLAN-002 T2 实测)**:主端读流在会话持活期间
//! **不会得到 EOF**——conhost 在输入写端/master 句柄关闭前不终止,
//! 输出管道保持打开(Unix PTY 在子进程退出后即 EOF,两者语义不同)。
//! 因此子进程退出检测一律走 `Child::try_wait`(见 `exited()`),
//! `eof()` 仅在 teardown 后才有意义。

use std::ffi::OsStr;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, channel};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize};

use crate::term::TermSession;

/// 一条完整的终端会话:PTY 子进程 + 仿真核心,一对一绑定。
pub struct PtySession {
    /// 仿真核心(网格/回滚/颜色);公开供渲染层快照。
    pub term: TermSession,
    master: Box<dyn MasterPty + Send>,
    child: Option<Box<dyn Child + Send + Sync>>,
    rx: Receiver<Vec<u8>>,
    writer: Box<dyn Write + Send>,
    eof: bool,
    /// 累计喂入仿真核心的字节数(取证/诊断)。
    bytes_fed: u64,
    /// reader 唤醒通道接收端(UI 事件驱动订阅用;取走后由订阅持有)。
    notify_rx: Option<Receiver<()>>,
    /// spawn 的程序名(UI 侧 Ctrl+C auto 模式按 shell 判定用)。
    program: String,
    /// shell 进程 ID(CTRL_C_EVENT 注入目标;child 持有期内有效)。
    shell_pid: Option<u32>,
}

impl PtySession {
    /// 申请 PTY 并 spawn 子进程;TERM 按真彩终端声明(alacritty/truecolor)。
    pub fn spawn<I, S>(program: &str, args: I, cols: usize, rows: usize) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let pty_system = portable_pty::native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: rows as u16,
                cols: cols as u16,
                ..Default::default()
            })
            .context("openpty: 基座 PTY 申请失败")?;

        let mut cmd = CommandBuilder::new(program);
        for arg in args {
            cmd.arg(arg);
        }
        // 不继承宿主终端的能力声明;AutoTerm 即终端,自报家门。
        cmd.env_remove("TERM");
        cmd.env("TERM", "alacritty");
        cmd.env("COLORTERM", "truecolor");

        let child = pair
            .slave
            .spawn_command(cmd)
            .with_context(|| format!("spawn {program} 失败"))?;
        let shell_pid = child.process_id();
        // 从端句柄用后即弃:保留会阻止 EOF 传播。
        drop(pair.slave);

        let reader = pair.master.try_clone_reader().context("take reader")?;
        let writer = pair.master.take_writer().context("take writer")?;
        let master = pair.master;

        let (tx, rx) = channel::<Vec<u8>>();
        // 每块字节后发一次唤醒:UI 订阅据此即时 drain(事件驱动,
        // 替换 spike 的 16ms 轮询)。发送端只活在 reader 线程——
        // 线程结束时通道断开,UI 侧订阅随之安静。
        let (tx_notify, notify_rx) = channel::<()>();
        thread::spawn(move || {
            let mut reader = reader;
            let mut buf = [0u8; 16384];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => {
                        let _ = tx.send(Vec::new()); // EOF 哨兵
                        let _ = tx_notify.send(());
                        break;
                    }
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                        let _ = tx_notify.send(());
                    }
                }
            }
        });

        Ok(Self {
            term: TermSession::new(cols, rows),
            master,
            child: Some(child),
            rx,
            writer,
            eof: false,
            bytes_fed: 0,
            notify_rx: Some(notify_rx),
            program: program.to_string(),
            shell_pid,
        })
    }

    /// 取走 reader 唤醒接收端(供 UI 事件驱动订阅;仅首次有效)。
    pub fn take_notify_receiver(&mut self) -> Option<Receiver<()>> {
        self.notify_rx.take()
    }

    /// 收割 reader 线程积压字节:喂仿真核心,DSR/DA 应答回写主端。
    /// 返回是否喂到了字节(调用方据此决定重绘)。
    pub fn drain(&mut self) -> bool {
        let mut fed = false;
        loop {
            match self.rx.try_recv() {
                Ok(chunk) if chunk.is_empty() => {
                    self.eof = true;
                    break;
                }
                Ok(chunk) => {
                    self.bytes_fed += chunk.len() as u64;
                    self.term.feed(&chunk);
                    fed = true;
                }
                Err(_) => break,
            }
        }
        let answers = self.term.pump();
        if !answers.is_empty() {
            let _ = self.writer.write_all(&answers);
        }
        fed
    }

    /// 累计喂入仿真核心的字节数。
    pub fn bytes_fed(&self) -> u64 {
        self.bytes_fed
    }

    /// 键盘输入等宿主→子进程字节。
    pub fn write_input(&mut self, bytes: &[u8]) {
        let _ = self.writer.write_all(bytes);
    }

    /// spawn 的程序名(Ctrl+C auto 模式的 shell 判定等)。
    pub fn program(&self) -> &str {
        &self.program
    }

    /// Ctrl+C 中断注入(PLAN-008,DEBTS #12/F2):Windows 经辅助进程
    /// `autoterm-ctrlc.exe` 挂进 shell 所在 ConPTY 控制台广播
    /// CTRL_C_EVENT,使**运行中**的命令真正可中断(裸 0x03 字节做不到,
    /// 007 F2 坐实)。helper 缺失/失败一律降级为纯字节路径(修复前
    /// 行为),绝不 panic。返回事件是否广播成功。
    ///
    /// 诊断走 stderr 而非 log:core 保持零日志依赖(计划验收#5,
    /// Cargo.toml 零 diff)。
    #[cfg(windows)]
    pub fn interrupt(&mut self) -> bool {
        use std::os::windows::process::CommandExt;
        use std::process::Command;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;

        let Some(pid) = self.shell_pid else {
            return false;
        };
        let fallback = |s: &mut Self, why: &str| {
            eprintln!("autoterm: interrupt 降级 0x03 字节路径({why})");
            s.write_input(&[0x03]);
            false
        };
        let Some(helper) = resolve_helper_bin() else {
            return fallback(self, "autoterm-ctrlc.exe 未找到(拷贝部署需与主程序同目录)");
        };
        let mut child = match Command::new(&helper)
            .arg(pid.to_string())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
        {
            Ok(c) => c,
            Err(e) => return fallback(self, &format!("helper spawn 失败: {e}")),
        };
        // 有界等待 ≤3s(50ms 步进自旋,不引 wait-timeout 依赖)。
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            match child.try_wait() {
                Ok(Some(status)) if status.code() == Some(0) => return true,
                Ok(Some(status)) => {
                    return fallback(
                        self,
                        &format!("autoterm-ctrlc exit={}", status.code().unwrap_or(-1)),
                    );
                }
                Ok(None) if Instant::now() < deadline => {
                    thread::sleep(Duration::from_millis(50));
                }
                Ok(None) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return fallback(self, "helper 3s 未返回");
                }
                Err(e) => return fallback(self, &format!("helper wait 失败: {e}")),
            }
        }
    }

    /// 非 Windows 占位:与既有字节路径同语义(SIGINT/进程组归 Unix
    /// 基座独立轨道)。
    #[cfg(not(windows))]
    pub fn interrupt(&mut self) -> bool {
        self.write_input(&[0x03]);
        true
    }

    /// resize:先仿真核心后 ConPTY(spike 验证过的顺序)。
    pub fn resize(&mut self, cols: usize, rows: usize) {
        self.term.resize(cols, rows);
        let _ = self.master.resize(PtySize {
            rows: rows as u16,
            cols: cols as u16,
            ..Default::default()
        });
    }

    /// 子进程输出流是否已结束(EOF 哨兵已到)。
    pub fn eof(&self) -> bool {
        self.eof
    }

    /// 子进程是否确已退出(try_wait 轮询,不依赖 EOF——见模块头
    /// ConPTY 平台事实)。
    pub fn exited(&mut self) -> bool {
        match self.child.as_mut() {
            Some(child) => matches!(child.try_wait(), Ok(Some(_))),
            None => true,
        }
    }

    /// 终止子进程(幂等:对已退出进程不报错)。
    pub fn kill(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill();
        }
    }

    /// 阻塞等待并回收退出状态。
    pub fn wait(&mut self) -> Result<portable_pty::ExitStatus> {
        let child = self.child.as_mut().context("child already reaped")?;
        child.wait().context("wait child")
    }
}

impl Drop for PtySession {
    fn drop(&mut self) {
        // 安全网:窗口关闭语义(T8)之外,任何析构路径都不留孤儿进程。
        self.kill();
        let _ = self.wait();
    }
}

/// 辅助进程定位(三级,与 007 ash_bin 同风格):
/// ① env `CARGO_BIN_EXE_AUTOTERM_CTRLC`(cargo 为本 crate 集成测试
///    注入,直指 build 产物);
/// ② env `AUTOTERM_CTRLC_BIN`(人工/部署覆盖);
/// ③ 当前 exe 向上 ≤3 级父目录(测试布局 target/*/deps → target/*,
///    安装布局同目录/上一级)。
#[cfg(windows)]
fn resolve_helper_bin() -> Option<PathBuf> {
    for var in ["CARGO_BIN_EXE_AUTOTERM_CTRLC", "AUTOTERM_CTRLC_BIN"] {
        if let Ok(p) = std::env::var(var) {
            let p = PathBuf::from(p);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    std::env::current_exe()
        .ok()
        .and_then(|exe| resolve_helper_bin_from(&exe))
}

/// 纯函数:exe 所在目录起,自身 + 向上至多 3 级父目录内找
/// `autoterm-ctrlc.exe`。
#[cfg(windows)]
fn resolve_helper_bin_from(exe: &Path) -> Option<PathBuf> {
    exe.parent().and_then(|dir| {
        dir.ancestors()
            .take(4)
            .find_map(|d| d.join("autoterm-ctrlc.exe").is_file().then(|| d.join("autoterm-ctrlc.exe")))
    })
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    /// T2 路径走查:target/{debug}/deps 布局向上命中;3 级之外不可达。
    /// (纯函数,临时目录自建布局,零新依赖;清理尽力而为。)
    #[test]
    fn resolve_helper_bin_walks_up_max_three_levels() {
        let base = std::env::temp_dir().join(format!("at_ctrlc_{}", std::process::id()));
        let deps = base.join("target").join("debug").join("deps");
        let bin = base.join("target").join("debug");
        std::fs::create_dir_all(&deps).unwrap();
        std::fs::write(deps.join("autoterm.exe"), b"").unwrap();
        std::fs::write(bin.join("autoterm-ctrlc.exe"), b"").unwrap();
        let found = resolve_helper_bin_from(&deps.join("autoterm.exe")).unwrap();
        assert_eq!(found, bin.join("autoterm-ctrlc.exe"));
        // helper 距 exe 4 级(3 级上限之外)→ None
        let far = base.join("a").join("b").join("c").join("d");
        std::fs::create_dir_all(&far).unwrap();
        std::fs::write(base.join("autoterm-ctrlc.exe"), b"").unwrap();
        assert_eq!(resolve_helper_bin_from(&far.join("x.exe")), None);
        let _ = std::fs::remove_dir_all(&base);
    }
}
