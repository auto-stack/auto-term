//! 014 泄漏取证:Job Object 提交上限启动器——进程 commit 超限即
//! **分配失败**(OS 级强制,不依赖进程内任何代码配合)。
//!
//! 动机:原生侧泄漏不吃进程内 guard(消息冻结/GuardAlloc 都看不见),
//! 但逃不过 OS 的账本:超限时 VirtualAlloc/HeapAlloc 直接失败,Rust 侧
//! 打出 "memory allocation of N bytes failed" 后 abort——失败点即
//! 泄漏分配点,WER/日志可取栈。这是"最后兜底 + 点名元凶"的双重角色。
//!
//! 用法:at-run-job <limit_mb> <exe> [args...]。子进程先 CREATE_SUSPENDED
//! 再入 job 后恢复,杜绝入 job 前逃逸;KILL_ON_JOB_CLOSE 保证 wrapper
//! 死则子进程死,不留逃出上限的孤儿。wrapper 阻塞等待子进程退出并
//! 透传其退出码。

#[cfg(windows)]
fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(limit_mb), Some(exe), rest) = (args.next(), args.next(), args.collect::<Vec<_>>())
    else {
        eprintln!("usage: at-run-job <limit_mb> <exe> [args...]");
        std::process::exit(1);
    };
    let Ok(limit_mb) = limit_mb.parse::<usize>() else {
        eprintln!("at-run-job: limit_mb 非数字");
        std::process::exit(1);
    };

    use std::os::windows::ffi::OsStrExt;
    let wide = |s: &str| -> Vec<u16> {
        std::ffi::OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    };
    let exe_path = std::fs::canonicalize(&exe)
        .map(|p| {
            // 剥 \\?\ 前缀:.NET 系运行时从模块路径推导基目录,扩展长度
            // 前缀会让其初始化炸掉(ServicePointManager 实测),普通 Win32
            // 路径对 CreateProcessW 同样有效。
            let s = p.to_string_lossy().into_owned();
            s.strip_prefix("\\\\?\\").map(str::to_owned).unwrap_or(s)
        })
        .unwrap_or_else(|_| exe.clone());
    let cmdline = if rest.is_empty() {
        format!("\"{exe_path}\"")
    } else {
        format!("\"{exe_path}\" {}", rest.join(" "))
    };
    let cwd = std::path::Path::new(&exe_path)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();

    #[link(name = "Kernel32")]
    unsafe extern "system" {
        fn CreateJobObjectW(attrs: *mut core::ffi::c_void, name: *const u16) -> *mut core::ffi::c_void;
        fn SetInformationJobObject(
            job: *mut core::ffi::c_void,
            class: i32,
            info: *const JobExtendedLimit,
            len: u32,
        ) -> i32;
        fn CreateProcessW(
            app: *const u16,
            cmdline: *mut u16,
            pa: *mut core::ffi::c_void,
            ta: *mut core::ffi::c_void,
            inherit: i32,
            flags: u32,
            env: *mut core::ffi::c_void,
            cwd: *const u16,
            si: *mut StartupInfoW,
            pi: *mut ProcessInformation,
        ) -> i32;
        fn AssignProcessToJobObject(
            job: *mut core::ffi::c_void,
            process: *mut core::ffi::c_void,
        ) -> i32;
        fn TerminateProcess(process: *mut core::ffi::c_void, exit_code: u32) -> i32;
        fn ResumeThread(thread: *mut core::ffi::c_void) -> u32;
        fn WaitForSingleObject(handle: *mut core::ffi::c_void, ms: u32) -> u32;
        fn GetExitCodeProcess(
            process: *mut core::ffi::c_void,
            code: *mut u32,
        ) -> i32;
        fn CloseHandle(handle: *mut core::ffi::c_void) -> i32;
    }

    // JOBOBJECT_EXTENDED_LIMIT_INFORMATION(x64 布局,repr(C) 自动对齐)。
    #[repr(C)]
    struct JobExtendedLimit {
        // JOBOBJECT_BASIC_LIMIT_INFORMATION
        per_process_user_time_limit: i64,
        per_job_user_time_limit: i64,
        limit_flags: u32,
        minimum_working_set_size: usize,
        maximum_working_set_size: usize,
        active_process_limit: u32,
        affinity: usize,
        priority_class: u32,
        scheduling_class: u32,
        // IO_COUNTERS
        read_op: u64,
        write_op: u64,
        other_op: u64,
        read_xfer: u64,
        write_xfer: u64,
        other_xfer: u64,
        process_memory_limit: usize,
        job_memory_limit: usize,
        peak_process_memory_used: usize,
        peak_job_memory_used: usize,
    }
    #[repr(C)]
    struct StartupInfoW {
        cb: u32,
        _reserved: *mut u16,
        _desktop: *mut u16,
        _title: *mut u16,
        _x: u32,
        _y: u32,
        _x_size: u32,
        _y_size: u32,
        _x_chars: u32,
        _y_chars: u32,
        _fill: u32,
        _flags: u32,
        _show: u16,
        _reserved2: u16,
        _reserved3: *mut u16,
        _std_in: *mut core::ffi::c_void,
        _std_out: *mut core::ffi::c_void,
        _std_err: *mut core::ffi::c_void,
    }
    #[repr(C)]
    struct ProcessInformation {
        process: *mut core::ffi::c_void,
        thread: *mut core::ffi::c_void,
        pid: u32,
        tid: u32,
    }

    const JOB_OBJECT_LIMIT_PROCESS_MEMORY: u32 = 0x0000_0100;
    const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: u32 = 0x0000_2000;
    const JOB_OBJECT_EXTENDED_LIMIT_INFORMATION: i32 = 9;
    const CREATE_SUSPENDED: u32 = 0x0000_0004;
    const INFINITE: u32 = 0xFFFF_FFFF;

    unsafe {
        let job = CreateJobObjectW(std::ptr::null_mut(), std::ptr::null());
        if job.is_null() {
            eprintln!("at-run-job: CreateJobObject 失败");
            std::process::exit(2);
        }
        let mut info: JobExtendedLimit = std::mem::zeroed();
        info.limit_flags = JOB_OBJECT_LIMIT_PROCESS_MEMORY | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        info.process_memory_limit = limit_mb * 1024 * 1024;
        if SetInformationJobObject(
            job,
            JOB_OBJECT_EXTENDED_LIMIT_INFORMATION,
            &info,
            std::mem::size_of::<JobExtendedLimit>() as u32,
        ) == 0
        {
            eprintln!("at-run-job: SetInformationJobObject 失败");
            std::process::exit(2);
        }

        let exe_w = wide(&exe_path);
        let mut cmdline_w = wide(&cmdline);
        let cwd_w = wide(&cwd.to_string_lossy());
        let mut si: StartupInfoW = std::mem::zeroed();
        si.cb = std::mem::size_of::<StartupInfoW>() as u32;
        let mut pi: ProcessInformation = std::mem::zeroed();
        if CreateProcessW(
            std::ptr::null(),
            cmdline_w.as_mut_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
            CREATE_SUSPENDED,
            std::ptr::null_mut(),
            cwd_w.as_ptr(),
            &mut si,
            &mut pi,
        ) == 0
        {
            eprintln!("at-run-job: CreateProcess({exe_path}) 失败");
            std::process::exit(2);
        }
        if AssignProcessToJobObject(job, pi.process) == 0 {
            eprintln!("at-run-job: AssignProcessToJobObject 失败(子进程终止)");
            TerminateProcess(pi.process, 1);
            std::process::exit(2);
        }
        ResumeThread(pi.thread);
        CloseHandle(pi.thread);
        println!(
            "at-run-job: pid={} 「{}」commit 上限 {limit_mb}MB(超限即分配失败;wrapper 存活期间有效)",
            pi.pid, exe_path
        );
        WaitForSingleObject(pi.process, INFINITE);
        let mut code: u32 = 1;
        GetExitCodeProcess(pi.process, &mut code);
        CloseHandle(pi.process);
        CloseHandle(job);
        std::process::exit(code as i32);
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("at-run-job: 仅 Windows");
}
