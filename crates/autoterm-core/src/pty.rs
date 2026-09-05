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
use std::sync::mpsc::{Receiver, channel};
use std::thread;

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
        // 从端句柄用后即弃:保留会阻止 EOF 传播。
        drop(pair.slave);

        let reader = pair.master.try_clone_reader().context("take reader")?;
        let writer = pair.master.take_writer().context("take writer")?;
        let master = pair.master;

        let (tx, rx) = channel::<Vec<u8>>();
        thread::spawn(move || {
            let mut reader = reader;
            let mut buf = [0u8; 16384];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => {
                        let _ = tx.send(Vec::new()); // EOF 哨兵
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

        Ok(Self {
            term: TermSession::new(cols, rows),
            master,
            child: Some(child),
            rx,
            writer,
            eof: false,
        })
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

    /// 键盘输入等宿主→子进程字节。
    pub fn write_input(&mut self, bytes: &[u8]) {
        let _ = self.writer.write_all(bytes);
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
