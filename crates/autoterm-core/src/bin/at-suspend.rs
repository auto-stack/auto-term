//! 014 泄漏取证:外部"暂停键"——NtSuspendProcess 挂起目标进程全部
//! 线程(debugger 暂停语义)。
//!
//! 动机:进程内 mem_guard 的冻结只关消息平面,拦不住原生侧增长
//! (17:09 事故实证:10.4GB 冻结后仍 +7GB 才死)。外部全线程挂起无
//! 自锁风险,现场完整保留,挂起后可 procdump -ma 取泄漏栈。
//!
//! 用法:at-suspend <pid>。幂等(重复挂起无害);现场取证完成后用
//! 任务管理器结束进程即可(无随附恢复工具——取证场景以 kill 收尾)。

#[cfg(windows)]
fn main() {
    let Some(pid) = std::env::args()
        .nth(1)
        .and_then(|s| s.parse::<u32>().ok())
        .filter(|&p| p != 0)
    else {
        eprintln!("usage: at-suspend <pid>");
        std::process::exit(1);
    };
    #[link(name = "Kernel32")]
    unsafe extern "system" {
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut core::ffi::c_void;
        fn CloseHandle(handle: *mut core::ffi::c_void) -> i32;
    }
    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn NtSuspendProcess(process: *mut core::ffi::c_void) -> i32;
    }
    const PROCESS_SUSPEND_RESUME: u32 = 0x0800;
    unsafe {
        let proc = OpenProcess(PROCESS_SUSPEND_RESUME, 0, pid);
        if proc.is_null() {
            eprintln!("at-suspend: OpenProcess({pid}) 失败(进程不存在/权限不足)");
            std::process::exit(2);
        }
        let status = NtSuspendProcess(proc);
        CloseHandle(proc);
        if status < 0 {
            eprintln!("at-suspend: NtSuspendProcess 失败 0x{status:08X}");
            std::process::exit(3);
        }
        println!("at-suspend: pid {pid} 已挂起(全部线程;现场保留,可 procdump)");
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("at-suspend: 仅 Windows");
}
