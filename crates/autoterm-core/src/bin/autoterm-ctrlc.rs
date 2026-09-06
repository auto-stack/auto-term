//! autoterm-ctrlc — Ctrl+C 事件注入辅助进程(PLAN-008,DEBTS #12/F2)。
//!
//! 背景:向 ConPTY 主端写 0x03 不会触发 conhost 的 CTRL_C_EVENT
//! (007 F2 坐实);`GenerateConsoleCtrlEvent` 只能发给与调用者共享
//! 同一控制台的进程。本 helper 是 winpty 经典手法(003 §4 候选
//! 通路 #1):挂进 shell 所在的 ConPTY 隐藏控制台后广播控制事件,
//! 自身免疫后即退。
//!
//! **执行期实证(PLAN-008 T3,OS build 26200.9168)**:ConPTY 客户端
//! 对 CTRL_C_EVENT 广播免疫(零可见效果),CTRL_BREAK_EVENT 可达
//! (cmd 打 "Control-Break"、ping 打统计)。故默认**双发** C→Break:
//! 健康 OS/build 上 C 承担规范中断语义,本类 build 上 Break 兜底
//! (对默认 handler 的进程即终止;ping 类对 Break 特殊处理的除外,
//! 见 designs/003 §4.1)。诊断参数可单发(供 conhost 版本差异调查,
//! DEBTS #7 附注)。
//!
//! 用法:`autoterm-ctrlc <pid> [c|break]`(pid = shell 进程 ID;
//! 无第二参数 = 双发)。
//! exit 码:0 = 事件已广播;2 = attach/广播失败;3 = 用法错误。
//! 非 Windows 目标为可编译占位(interrupt() 在该平台走字节路径)。
//!
// SPDX-License-Identifier: Apache-2.0

#[cfg(windows)]
mod imp {
    use std::thread::sleep;
    use std::time::Duration;

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
    const CTRL_BREAK_EVENT: u32 = 1;

    /// 自身免疫 handler:对一切控制事件返回 TRUE(已处理)。
    /// 不能用 SetConsoleCtrlHandler(None, TRUE)——那只忽略 Ctrl+C,
    /// CTRL_BREAK 仍走默认 handler 杀死本进程(实测 exit=
    /// STATUS_CONTROL_C_EXIT,与 conhost 派发竞速即抖动)。
    unsafe extern "system" fn self_ignore(_ctrl_type: u32) -> i32 {
        1
    }

    /// 事件选择:`Dual`(默认,C→Break 双发)/ 仅 C / 仅 Break。
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum Event {
        Dual,
        CtrlC,
        CtrlBreak,
    }

    /// 挂进目标控制台并广播控制事件。
    /// 返回进程 exit 码(0 成功 / 2 attach 或广播失败)。
    ///
    /// 自身免疫的竞态说明:conhost 向 AttachConsole 挂入进程派发
    /// handler 的注册有时序竞态——即便先 SetConsoleCtrlHandler,
    /// 派发线程偶尔仍走默认 handler 把本进程杀死(exit=
    /// STATUS_CONTROL_C_EXIT)。这**无碍语义**:被事件杀死 ⇒
    /// 事件必然已广播(GenerateConsoleCtrlEvent 已返回成功)。
    /// 调用方(pty.rs interrupt)把 0xC000013A 与 0 同判成功。
    pub fn send(pid: u32, event: Event) -> i32 {
        unsafe {
            // 脱离宿主当前控制台(若有):一个进程只能挂一个控制台。
            let _ = FreeConsole();
            if AttachConsole(pid) == 0 {
                return 2;
            }
            // 自身免疫:广播会命中同控制台的我们,忽略之(C+Break 全免疫)。
            let _ = SetConsoleCtrlHandler(Some(self_ignore), 1);
            let send_evt = |evt: u32| {
                // SAFETY: 同一 unsafe 块内,函数指针来自上方 extern 声明。
                if GenerateConsoleCtrlEvent(evt, 0) == 0 {
                    2
                } else {
                    0
                }
            };
            match event {
                Event::Dual => {
                    // 先 Break 后 C:若竞态中途被杀,Break(对默认
                    // handler 进程普遍有效,且是 C 被吞 build 上的
                    // 唯一通道)已优先落地;C 随后锦上添花。
                    let mut ret = send_evt(CTRL_BREAK_EVENT);
                    sleep(Duration::from_millis(50));
                    let c = send_evt(CTRL_C_EVENT);
                    if ret == 0 && c != 0 {
                        ret = c;
                    }
                    ret
                }
                Event::CtrlC => send_evt(CTRL_C_EVENT),
                Event::CtrlBreak => send_evt(CTRL_BREAK_EVENT),
            }
            // 注:此处不 FreeConsole——FreeConsole 后的退出与事件
            // 派发竞态无害(退出码已定),省一次控制台抖动。
        }
    }
}

#[cfg(not(windows))]
mod imp {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum Event {
        Dual,
        CtrlC,
        CtrlBreak,
    }

    /// 非 Windows 占位:interrupt() 在该平台走字节路径,不会被调用。
    pub fn send(_pid: u32, _event: Event) -> i32 {
        eprintln!("autoterm-ctrlc: 仅支持 Windows(非 Windows 由 interrupt() 字节路径承担)");
        1
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(pid) = args.next().and_then(|a| a.parse::<u32>().ok()) else {
        eprintln!("用法: autoterm-ctrlc <pid> [c|break]  (pid = shell 进程 ID;默认双发 C+Break)");
        std::process::exit(3);
    };
    let event = match args.next().as_deref() {
        None | Some("") => imp::Event::Dual,
        Some("c") => imp::Event::CtrlC,
        Some("break") => imp::Event::CtrlBreak,
        _ => {
            eprintln!("未知事件参数(可用: c | break;缺省 = 双发)");
            std::process::exit(3);
        }
    };
    std::process::exit(imp::send(pid, event));
}
