//! autoterm-ctrlc — Ctrl+C 事件注入辅助进程(PLAN-008,DEBTS #12/F2)。
//!
//! 背景:向 ConPTY 主端写 0x03 不会触发 conhost 的 CTRL_C_EVENT
//! (007 F2 坐实,与 shell 无关);`GenerateConsoleCtrlEvent` 只能发给
//! 与调用者共享同一控制台的进程。本 helper 是 winpty 经典手法
//! (003 §4 候选通路 #1):挂进 shell 所在的 ConPTY 隐藏控制台后
//! 广播 CTRL_C_EVENT,自身免疫后即退。
//!
//! 用法:`autoterm-ctrlc <pid>`(pid = shell 进程 ID,由
//! `PtySession::interrupt` 传入)。
//! exit 码:0 = 事件已广播;2 = AttachConsole/事件失败;
//! 3 = 用法错误。非 Windows 目标为可编译占位。
//!
// SPDX-License-Identifier: Apache-2.0

#[cfg(windows)]
mod imp {
    // 手写 kernel32 FFI(零新依赖):签名全标量,BOOL = i32。
    // 不用 windows-sys/winapi,维持 Cargo.toml 零改动(计划验收#5)。
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn FreeConsole() -> i32;
        fn AttachConsole(dwProcessId: u32) -> i32;
        fn SetConsoleCtrlHandler(
            handler: Option<unsafe extern "system" fn(u32) -> i32>,
            add: i32,
        ) -> i32;
        fn GenerateConsoleCtrlEvent(dwCtrlEvent: u32, dwProcessGroupId: u32) -> i32;
    }

    const CTRL_C_EVENT: u32 = 0;

    /// 挂进目标控制台并广播 CTRL_C_EVENT。
    /// 返回进程 exit 码(0 成功 / 2 attach 或广播失败)。
    pub fn send_ctrl_c(pid: u32) -> i32 {
        unsafe {
            // 脱离宿主当前控制台(若有):一个进程只能挂一个控制台。
            let _ = FreeConsole();
            if AttachConsole(pid) == 0 {
                return 2;
            }
            // 自身免疫:广播会命中同控制台的我们,忽略之。
            let _ = SetConsoleCtrlHandler(None, 1);
            let ok = GenerateConsoleCtrlEvent(CTRL_C_EVENT, 0);
            let _ = FreeConsole();
            if ok == 0 {
                2
            } else {
                0
            }
        }
    }
}

#[cfg(not(windows))]
mod imp {
    /// 非 Windows 占位:interrupt() 在该平台走字节路径,本 helper 不会被调用。
    pub fn send_ctrl_c(_pid: u32) -> i32 {
        eprintln!("autoterm-ctrlc: 仅支持 Windows(非 Windows 由 interrupt() 字节路径承担)");
        1
    }
}

fn main() {
    let Some(pid) = std::env::args().nth(1).and_then(|a| a.parse::<u32>().ok()) else {
        eprintln!("用法: autoterm-ctrlc <pid>  (pid = shell 进程 ID)");
        std::process::exit(3);
    };
    std::process::exit(imp::send_ctrl_c(pid));
}
