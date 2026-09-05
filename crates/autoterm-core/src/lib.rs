//! autoterm-core — AutoTerm 终端会话层(PTY + 仿真核心封装)
//!
// SPDX-License-Identifier: Apache-2.0
//! `term`:alacritty_terminal 封装(feed/pump/damage/scroll/快照);
//! `pty`:PTY 会话(spawn/reader 线程/答案回写/resize/kill)。
//! 由 PLAN-001 spikes/term-probe 升格而来(spikes/ 保留归档)。

pub mod pty;
pub mod term;

pub use pty::*;
pub use term::*;
