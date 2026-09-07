//! PLAN-009 T8 手写胶水(**不转译**):libloading 包装 autoterm_core.dll
//! (P2 FFI 面的安全 Rust 化)。快照缓存在 glue 侧,`engine_rows` 回读。
//!
//! DLL 解析顺序:AUTOTERM_ENGINE_DLL 环境变量 → exe 同目录(dist 布局)
//! → ../target/debug → ../../target/debug(组内两仓布局均覆盖)。

use libloading::Library;
use std::ffi::{c_char, c_int, CStr, CString};
use std::sync::{Mutex, OnceLock};

static LIB: OnceLock<Library> = OnceLock::new();
static NEXT_HANDLE: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(1);
type SnapMap = std::collections::HashMap<i64, Vec<String>>;
static SNAPSHOTS: std::sync::LazyLock<Mutex<SnapMap>> = std::sync::LazyLock::new(|| Mutex::new(SnapMap::new()));

fn lib() -> &'static Library {
    LIB.get_or_init(|| {
        // 解析顺序:env → exe 目录同目录(003 §5 dist 布局:三件套同
        // 目录分发,PLAN-011 T2)→ 当前 exe 目录向上 4 级的 target/debug
        // (组内布局稳健,不受 CWD 影响;仿 autoterm-ctrlc 的 helper 解析)。
        let mut candidates: Vec<std::path::PathBuf> = Vec::new();
        if let Ok(p) = std::env::var("AUTOTERM_ENGINE_DLL") {
            candidates.push(std::path::PathBuf::from(p));
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                candidates.push(dir.join("autoterm_core.dll"));
                for anc in dir.ancestors().take(4) {
                    candidates.push(anc.join("target/debug/autoterm_core.dll"));
                }
            }
        }
        let path = candidates
            .into_iter()
            .find(|p| p.is_file())
            .expect("autoterm_core.dll 未找到(先 cargo build -p autoterm-core,或设 AUTOTERM_ENGINE_DLL)");
        unsafe { Library::new(&path) }.expect("autoterm_core.dll 加载失败")
    })
}

fn snapshots() -> std::sync::MutexGuard<'static, SnapMap> {
    SNAPSHOTS.lock().unwrap()
}


/// spawn(缺省 shell = COMSPEC/cmd);返回句柄(0 = 失败)。
pub fn engine_spawn(cols: i64, rows: i64) -> i64 {
    unsafe {
        let spawn: libloading::Symbol<
            unsafe extern "C" fn(c_int, c_int, *const c_char) -> *mut core::ffi::c_void,
        > = lib().get(b"autoterm_engine_spawn\0").unwrap();
        let h = spawn(cols as c_int, rows as c_int, std::ptr::null());
        if h.is_null() {
            return 0;
        }
        let handle = NEXT_HANDLE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        // SAFETY: 句柄由 spawn 签名约定持有,free 前有效;句柄↔指针映射由
        // 各按句柄导出函数用不到——这里把指针直接编码进 handle 无法 round
        // trip,故 glue 侧持有指针表。
        handles().insert(handle, h as i64);
        handle
    }
}

static HANDLES: OnceLock<Mutex<std::collections::HashMap<i64, i64>>> = OnceLock::new();

fn handles() -> std::sync::MutexGuard<'static, std::collections::HashMap<i64, i64>> {
    HANDLES.get_or_init(|| Mutex::new(std::collections::HashMap::new())).lock().unwrap()
}

fn ptr_of(handle: i64) -> *mut core::ffi::c_void {
    handles().get(&handle).copied().unwrap_or(0) as *mut core::ffi::c_void
}

/// 宿主→子进程一行输入(补 \r\n)。
pub fn engine_write_line(handle: i64, line: &str) {
    let h = ptr_of(handle);
    if h.is_null() {
        return;
    }
    let mut bytes = line.as_bytes().to_vec();
    bytes.extend_from_slice(b"\r\n");
    unsafe {
        let write: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void, *const u8, usize),
        > = lib().get(b"autoterm_engine_write_input\0").unwrap();
        write(h, bytes.as_ptr(), bytes.len());
    }
}

/// 收割引擎输出并刷新 glue 侧快照。
pub fn engine_feed_snapshot(handle: i64) {
    let h = ptr_of(handle);
    if h.is_null() {
        return;
    }
    unsafe {
        let feed: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void) -> c_int,
        > = lib().get(b"autoterm_engine_feed_ready\0").unwrap();
        feed(h);
        let take: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void, *mut c_int, c_int) -> c_int,
        > = lib().get(b"autoterm_engine_take_dirty_rows\0").unwrap();
        let mut rows = [0 as c_int; 64];
        take(h, rows.as_mut_ptr(), 64);
        // Full(-1)或脏行集都全量重采(行数由 rows 文本直至 -1 决定)。
        let row_text: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void, c_int, *mut c_char, c_int) -> c_int,
        > = lib().get(b"autoterm_engine_row_text\0").unwrap();
        let mut lines = Vec::new();
        for r in 0..64 {
            let mut buf = [0 as c_char; 512];
            let need = row_text(h, r as c_int, buf.as_mut_ptr(), 512);
            if need < 0 {
                break;
            }
            let n = (need as usize).saturating_sub(1).min(511);
            let bytes: Vec<u8> = buf[..n].iter().map(|&c| c as u8).collect();
            lines.push(String::from_utf8_lossy(&bytes).into_owned());
        }
        snapshots().insert(handle, lines);
    }
}

/// 视口行文本快照:先收割引擎输出(内联 feed + 损伤刷新)再回读。
pub fn engine_rows(handle: i64) -> Vec<String> {
    engine_feed_snapshot(handle);
    snapshots().get(&handle).cloned().unwrap_or_default()
}

/// 视口一行逐格样式:fg|bg 交错 u32 标量色(kind<<24|value,ffi.rs
/// 模块头编码)。签名对齐 `autoterm_engine_row_style`(handle,row,out,
/// cap),返回写入的 u32 个数(行越界/容量不足 -1)。需先 tick/feed
/// 刷新 dll 侧快照。PLAN-011 T1 色彩对拍消费。
pub fn engine_row_style(handle: i64, row: usize, out: &mut [u32]) -> i32 {
    let h = ptr_of(handle);
    if h.is_null() {
        return -1;
    }
    unsafe {
        let style: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void, c_int, *mut u32, c_int) -> c_int,
        > = lib().get(b"autoterm_engine_row_style\0").unwrap();
        style(h, row as c_int, out.as_mut_ptr(), out.len() as c_int)
    }
}

pub fn engine_resize(handle: i64, cols: i64, rows: i64) {
    let h = ptr_of(handle);
    if h.is_null() {
        return;
    }
    unsafe {
        let resize: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void, c_int, c_int),
        > = lib().get(b"autoterm_engine_resize\0").unwrap();
        resize(h, cols as c_int, rows as c_int);
    }
}

pub fn engine_interrupt(handle: i64) -> i64 {
    let h = ptr_of(handle);
    if h.is_null() {
        return -1;
    }
    unsafe {
        let interrupt: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void) -> c_int,
        > = lib().get(b"autoterm_engine_interrupt\0").unwrap();
        interrupt(h) as i64
    }
}

pub fn engine_is_exited(handle: i64) -> bool {
    let h = ptr_of(handle);
    if h.is_null() {
        return true;
    }
    unsafe {
        let exited: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void) -> c_int,
        > = lib().get(b"autoterm_engine_is_exited\0").unwrap();
        exited(h) == 1
    }
}

pub fn engine_free(handle: i64) {
    let h = ptr_of(handle);
    if h.is_null() {
        return;
    }
    unsafe {
        let free: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void),
        > = lib().get(b"autoterm_engine_free\0").unwrap();
        free(h);
    }
    handles().remove(&handle);
    snapshots().remove(&handle);
}
