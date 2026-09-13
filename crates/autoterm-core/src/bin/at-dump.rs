//! 014 泄漏取证:MiniDumpWriteDump 落盘工具——观察哨自动 dump 用
//! (机器无 procdump 时的自力更生版;与 procdump 同款 DbgHelp 机制,
//! 同用户进程免提权)。配合 at-suspend:先挂起再 dump,栈停在泄漏现场。
//!
//! 用法:at-dump <pid> <dump.dmp> [full|mini](默认 mini = 线程栈+模块
//! 表,MB 级;full = 全内存——WS 20GB 时 dump 约 20GB,确认盘空间)。

#[cfg(windows)]
fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(pid_s), Some(path)) = (args.next(), args.next()) else {
        eprintln!("usage: at-dump <pid> <dump.dmp> [full|mini]");
        std::process::exit(1);
    };
    let Ok(pid) = pid_s.parse::<u32>() else {
        eprintln!("at-dump: pid 非数字");
        std::process::exit(1);
    };
    let full = args.next().map(|s| s.eq_ignore_ascii_case("full")).unwrap_or(false);

    use std::os::windows::io::AsRawHandle;
    #[link(name = "Kernel32")]
    unsafe extern "system" {
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut core::ffi::c_void;
        fn CloseHandle(handle: *mut core::ffi::c_void) -> i32;
        fn GetLastError() -> u32;
    }
    #[link(name = "dbghelp")]
    unsafe extern "system" {
        fn MiniDumpWriteDump(
            h_process: *mut core::ffi::c_void,
            pid: u32,
            h_file: *mut core::ffi::c_void,
            dump_type: u32,
            exception_param: *mut core::ffi::c_void,
            user_param: *mut core::ffi::c_void,
            callback_param: *mut core::ffi::c_void,
        ) -> i32;
    }
    const PROCESS_ALL_ACCESS: u32 = 0x001F_0FFF;
    // 0x2 = MiniDumpWithFullMemory(014 实测纠错:此前误用 0x20000
    // = IgnoreInaccessibleMemory,full 标志从未生效——dump 头部
    // `.dumpdebug` 可验);| 0x20000 对不可读页(guard/器件)容错,
    // 避免全量转储中途失败。
    const MINIDUMP_WITH_FULL_MEMORY: u32 = 0x0000_0002 | 0x0002_0000;

    let file = match std::fs::File::create(&path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("at-dump: 创建 {path} 失败: {e}");
            std::process::exit(2);
        }
    };
    unsafe {
        let proc = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);
        if proc.is_null() {
            eprintln!("at-dump: OpenProcess({pid}) 失败 GLE={}", GetLastError());
            std::process::exit(3);
        }
        let dump_type = if full { MINIDUMP_WITH_FULL_MEMORY } else { 0 };
        eprintln!("[at-dump] dump_type=0x{dump_type:08X} pid={pid}");
        let ok = MiniDumpWriteDump(
            proc,
            pid,
            file.as_raw_handle(),
            dump_type,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        CloseHandle(proc);
        if ok == 0 {
            eprintln!("at-dump: MiniDumpWriteDump 失败 GLE={}", GetLastError());
            std::process::exit(4);
        }
    }
    drop(file);
    let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    println!(
        "at-dump: pid {pid} → {path}({})完成,{size} 字节",
        if full { "full" } else { "mini" }
    );
}

#[cfg(not(windows))]
fn main() {
    eprintln!("at-dump: 仅 Windows");
}
