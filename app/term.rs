//! PLAN-013 T5 app/ 侧车模块( crates/auto-man sidecar 机制供给;017
//! at-app→app 置换):
//! libloading 包装 autoterm_core.dll,模块名 crate::term。源自
//! at-gen/src/engine.rs(逐行同语义);a2r 对 `use auto.term` 的下降
//! (`use crate::term::{…}`)由此闭合。快照缓存在 glue 侧,`engine_rows` 回读。
//!
//! DLL 解析顺序:AUTOTERM_ENGINE_DLL 环境变量 → exe 同目录(dist 布局)
//! → ../target/debug → ../../target/debug(组内两仓布局均覆盖)。

use libloading::Library;
use std::ffi::{c_char, c_int, CStr, CString};
use std::sync::atomic::Ordering;
use std::sync::{Mutex, OnceLock};

static LIB: OnceLock<Library> = OnceLock::new();
static NEXT_HANDLE: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(1);
type SnapMap = std::collections::HashMap<i64, Vec<String>>;
static SNAPSHOTS: std::sync::LazyLock<Mutex<SnapMap>> = std::sync::LazyLock::new(|| Mutex::new(SnapMap::new()));
// 014:光标格与视口几何(feed 时随拍采样,spawn 时落初值;db 经
// engine_cursor_*/engine_viewport_* 回读)。
static CURSOR: Mutex<(i64, i64)> = Mutex::new((0, 0));
static VIEWPORT: Mutex<(i64, i64)> = Mutex::new((0, 0));

// 014 追踪(AUTO_TERM_TRACE=1 开启):热路径计数,每 N 次限频输出。
static TRACE: OnceLock<bool> = OnceLock::new();
fn trace_on() -> bool {
    *TRACE.get_or_init(|| std::env::var("AUTO_TERM_TRACE").map(|v| v == "1").unwrap_or(false))
}
static FEED_CALLS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static FEED_BYTES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static RESIZE_CALLS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static PUMP_KEYS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

// ── 014 积压报警(用户可见通知面)────────────────────────────────────
// reader→drain 积压是内存暴涨主嫌疑:越过告警线(2MB,反压上限 8MB 之前
// 预警)即上升沿计次 + stderr 留痕(不依赖 AUTO_TERM_TRACE);回落到迟滞
// 下沿(512KB)解除。db/api 每拍采样,前端状态行横幅告知用户。
const BACKLOG_WARN_BYTES: i32 = 2 * 1024 * 1024;
const BACKLOG_CLEAR_BYTES: i32 = 512 * 1024;
/// (告警中, 待取走的上升沿次数)。take 语义:engine_backlog_take_alerts 取走。
static BACKLOG_STATE: Mutex<(bool, i64)> = Mutex::new((false, 0));

/// 积压上升沿检测:越线计次并报警;回落到迟滞下沿解除告警态。
fn check_backlog(pending_bytes: i32) {
    let mut st = BACKLOG_STATE.lock().unwrap();
    let (active, count) = *st;
    if !active && pending_bytes >= BACKLOG_WARN_BYTES {
        *st = (true, count + 1);
        drop(st);
        eprintln!(
            "[term-backlog] ⚠ reader→drain 积压 {pending_bytes} 字节越过告警线 \
             {BACKLOG_WARN_BYTES}(第 {} 次)——产出侧洪峰或消费侧停摆",
            count + 1
        );
    } else if active && pending_bytes <= BACKLOG_CLEAR_BYTES {
        *st = (false, count);
    }
}

/// 单调毫秒钟(丢帧预警限频用)。
fn mono_ms() -> u64 {
    static START: OnceLock<std::time::Instant> = OnceLock::new();
    START.get_or_init(std::time::Instant::now).elapsed().as_millis() as u64
}

/// 环逐出(丢帧)预警:溢出计数增长即 stderr 留痕(限频 1s,洪峰不刷屏)
/// ——丢帧不再是无痕事件。计数回退(新引擎会话)时重置基准。
static OVERFLOW_LOG: Mutex<(i64, u64)> = Mutex::new((0, 0)); // (上次打印计数, 上次打印 ms)

fn check_overflow(dropped: i64) {
    let mut st = OVERFLOW_LOG.lock().unwrap();
    let (last_count, last_ms) = *st;
    if dropped < last_count {
        *st = (dropped, last_ms);
        return;
    }
    if dropped == last_count {
        return;
    }
    let now = mono_ms();
    if last_ms == 0 || now.saturating_sub(last_ms) >= 1000 {
        eprintln!(
            "[term-backlog] ⚠ 环形缓冲逐出(丢帧)累计 {dropped} 块(较上次 +{})——消费侧跟不上,旧输出被丢",
            dropped - last_count
        );
        *st = (dropped, now);
    }
}

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
        // 014:无条件留痕加载路径(一次性)——现场实测踩坑:exe 同目录
        // 常驻 11:35 旧 DLL(无 pending/反压导出),新 guard 从未真正上机,
        // 爆炸复发即源于此。日志里这行 = 当次运行真正生效的引擎。
        eprintln!("[term-dll] 加载引擎: {}", path.display());
        unsafe { Library::new(&path) }.expect("autoterm_core.dll 加载失败")
    })
}

fn snapshots() -> std::sync::MutexGuard<'static, SnapMap> {
    SNAPSHOTS.lock().unwrap()
}


/// spawn(缺省 shell = COMSPEC/cmd);返回句柄(0 = 失败)。
pub fn engine_spawn(cols: i64, rows: i64) -> i64 {
    *VIEWPORT.lock().unwrap() = (cols, rows);
    *CURSOR.lock().unwrap() = (0, 0);
    // 014 内存哨兵:注册冻结挂钩——超限时挂起 shell 子进程,掐断产出侧
    // (只丢消息拦不住非消息线程的增长)。幂等;未启哨兵阈值时无副作用。
    auto_lang::ui::mem_guard::set_freeze_hook(Box::new(move || {
        for h in handles().keys().copied().collect::<Vec<i64>>() {
            engine_suspend_child(h);
        }
    }));
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
    write_raw(h, &bytes);
}

/// 宿主→子进程裸字节写(无 \r\n 补缀;直键入 VT 串原样)。
pub fn engine_write_raw(handle: i64, data: &str) {
    let h = ptr_of(handle);
    if h.is_null() {
        return;
    }
    write_raw(h, data.as_bytes());
}

fn write_raw(h: *mut core::ffi::c_void, bytes: &[u8]) {
    unsafe {
        let write: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void, *const u8, usize),
        > = lib().get(b"autoterm_engine_write_input\0").unwrap();
        write(h, bytes.as_ptr(), bytes.len());
    }
}

/// 014 直键入泵:排空 terminal 组件键入队列(auto_lang::ui::terminal
/// 注册表,与 iced 渲染同进程),逐键裸写引擎。返回泵送键数。队列空 /
/// 句柄无效 = 0(no-op;vue back 进程无 widget,恒走此路径)。
pub fn engine_pump_input(handle: i64) -> i64 {
    let keys = auto_lang::ui::terminal::terminal_drain_all_inputs();
    let n = keys.len() as i64;
    if n == 0 {
        return 0;
    }
    let h = ptr_of(handle);
    if h.is_null() {
        return n;
    }
    for key in &keys {
        write_raw(h, key.as_bytes());
    }
    if trace_on() {
        let total = PUMP_KEYS.fetch_add(n as u64, Ordering::Relaxed) + n as u64;
        let calls = FEED_CALLS.load(Ordering::Relaxed);
        eprintln!("[term-trace] pump keys={n} total={total} (feed_calls={calls})");
    }
    n
}

// ── 014:几何随动 + 光标格(读注册表/glue 静态量,泵同款管线)────────

/// 应用 widget 请求的待定几何:注册表 `terminal_take_any_resize()` →
/// 引擎 resize → VIEWPORT 刷新。返回 1=已应用 0=无请求。
pub fn engine_apply_resize(handle: i64) -> i64 {
    let Some((cols, rows)) = auto_lang::ui::terminal::terminal_take_any_resize() else {
        return 0;
    };
    engine_resize(handle, cols as i64, rows as i64);
    *VIEWPORT.lock().unwrap() = (cols as i64, rows as i64);
    if trace_on() {
        let n = RESIZE_CALLS.fetch_add(1, Ordering::Relaxed) + 1;
        eprintln!("[term-trace] resize #{n}: {cols}x{rows}");
    }
    1
}

pub fn engine_viewport_cols(_handle: i64) -> i64 {
    VIEWPORT.lock().unwrap().0
}

pub fn engine_viewport_rows(_handle: i64) -> i64 {
    VIEWPORT.lock().unwrap().1
}

pub fn engine_cursor_row(_handle: i64) -> i64 {
    CURSOR.lock().unwrap().0
}

pub fn engine_cursor_col(_handle: i64) -> i64 {
    CURSOR.lock().unwrap().1
}

/// 收割引擎输出并刷新 glue 侧快照(feed + 损伤行全量重采 + 光标采样 +
/// 逐格样式旁路:row_style 解码 → terminal_feed_cells_all,ash 彩色
/// 输出经此上屏)。
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
        // 光标格随拍采样(可见时刷新;隐藏保持上次值)。
        let cursor: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void, *mut c_int, *mut c_int) -> c_int,
        > = lib().get(b"autoterm_engine_cursor\0").unwrap();
        let (mut r, mut c) = (0 as c_int, 0 as c_int);
        if cursor(h, &mut r, &mut c) == 1 {
            *CURSOR.lock().unwrap() = (r as i64, c as i64);
        }
        // Full(-1)或脏行集都全量重采(行数由 rows 文本直至 -1 决定;
        // 上限 256 行,resize 钳位 MAX_RESIZE_ROWS=200 在册)。逐行取
        // 文本 + 逐格样式:文本进快照,样式走组件旁路上色。
        let row_text: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void, c_int, *mut c_char, c_int) -> c_int,
        > = lib().get(b"autoterm_engine_row_text\0").unwrap();
        let row_style: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void, c_int, *mut u32, c_int) -> c_int,
        > = lib().get(b"autoterm_engine_row_style\0").unwrap();
        // 014 泄漏定位:reader→drain 积压字节(暴涨时此项无界增长即坐实)。
        let pending: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void) -> c_int,
        > = lib().get(b"autoterm_engine_pending_bytes\0").unwrap();
        let pending_bytes = pending(h);
        // 014 报警面:越线检测(feed 拍频即检测频率,无需另起线程)。
        check_backlog(pending_bytes);
        // 014 丢帧可见性:环逐出计数增长即留痕(丢帧不再是"无痕"事件)。
        check_overflow(engine_backlog_dropped(handle));
        let mut lines = Vec::new();
        let mut fed_rows = 0usize;
        let mut fed_bytes = 0usize;
        for r in 0..256i32 {
            let mut buf = [0 as c_char; 512];
            let need = row_text(h, r, buf.as_mut_ptr(), 512);
            if need < 0 {
                break;
            }
            let n = (need as usize).saturating_sub(1).min(511);
            let bytes: Vec<u8> = buf[..n].iter().map(|&c| c as u8).collect();
            fed_rows += 1;
            fed_bytes += n;
            let text = String::from_utf8_lossy(&bytes).into_owned();
            // 逐格样式:fg/bg 交错 u32(kind<<24|value),2×cols 容量。
            let mut styles = [0u32; 1024];
            let styled = row_style(h, r, styles.as_mut_ptr(), 1024);
            let pairs = (styled.max(0) as usize) / 2;
            let mut cells: Vec<auto_lang::ui::terminal::TermCell> = Vec::with_capacity(pairs);
            for (ci, ch) in text.chars().enumerate() {
                if ci >= pairs {
                    break;
                }
                cells.push(auto_lang::ui::terminal::TermCell {
                    ch,
                    fg: decode_style_color(styles[ci * 2]),
                    bg: decode_style_color(styles[ci * 2 + 1]),
                });
            }
            if !cells.is_empty() {
                auto_lang::ui::terminal::terminal_feed_cells_all(r as usize, cells);
            }
            lines.push(text);
        }
        snapshots().insert(handle, lines);
        if trace_on() {
            let calls = FEED_CALLS.fetch_add(1, Ordering::Relaxed) + 1;
            let bytes = FEED_BYTES.fetch_add(fed_bytes as u64, Ordering::Relaxed) + fed_bytes as u64;
            let _ = fed_rows;
            if calls % 100 == 0 {
                eprintln!(
                    "[term-trace] feed calls={calls} total_bytes={bytes} rows_last={fed_rows} pending_bytes={pending_bytes}"
                );
            }
            // 积压超 8MB 立即告警(正常应 < 单 tick 输出量级)。
            if pending_bytes > 8 * 1024 * 1024 {
                FEED_CALLS.store(0, Ordering::Relaxed); // 重置限频,让下一条立刻打印
                eprintln!("[term-trace] ⚠ pending_bytes={pending_bytes} —— 产出侧失控/消费侧停摆");
            }
        }
    }
}

/// FFI 标量色 → 组件色((kind<<24)|value:0=Default 1=Indexed 2=RGB)。
fn decode_style_color(v: u32) -> auto_lang::ui::terminal::TermColor {
    let kind = v >> 24;
    let value = v & 0x00FF_FFFF;
    match kind {
        1 => auto_lang::ui::terminal::TermColor::Indexed(value as u8),
        2 => auto_lang::ui::terminal::TermColor::Rgb(
            (value >> 16) as u8,
            (value >> 8) as u8,
            value as u8,
        ),
        _ => auto_lang::ui::terminal::TermColor::Default,
    }
}

// ── 014 积压报警回读面(db/api → 前端状态行)──────────────────────

/// 当前积压 MB(live 采样,不必等 feed;句柄无效/符号缺失 = 0)。
pub fn engine_backlog_pending_mb(handle: i64) -> i64 {
    let h = ptr_of(handle);
    if h.is_null() {
        return 0;
    }
    unsafe {
        let pending: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void) -> c_int,
        > = lib().get(b"autoterm_engine_pending_bytes\0").unwrap();
        (pending(h).max(0) as i64) / (1024 * 1024)
    }
}

/// reader 是否反压暂停中(1/0)。符号缺席(旧 DLL)= 0,不 panic——
/// 报警面必须比被报警的路径更皮实。
pub fn engine_backlog_paused(handle: i64) -> i64 {
    let h = ptr_of(handle);
    if h.is_null() {
        return 0;
    }
    unsafe {
        let paused: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void) -> c_int,
        > = match lib().get(b"autoterm_engine_backlog_paused\0") {
            Ok(s) => s,
            Err(_) => return 0,
        };
        paused(h) as i64
    }
}

/// 取走自上次调用以来的积压报警次数(上升沿,take 语义)。
pub fn engine_backlog_take_alerts(_handle: i64) -> i64 {
    let mut st = BACKLOG_STATE.lock().unwrap();
    let taken = st.1;
    st.1 = 0;
    taken
}

/// 累计环逐出块数(丢帧计数;句柄无效/符号缺失 = 0)。
pub fn engine_backlog_dropped(handle: i64) -> i64 {
    let h = ptr_of(handle);
    if h.is_null() {
        return 0;
    }
    unsafe {
        let dropped: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void) -> c_int,
        > = match lib().get(b"autoterm_engine_overflow_count\0") {
            Ok(s) => s,
            Err(_) => return 0,
        };
        dropped(h).max(0) as i64
    }
}

// ── 014 内存哨兵冻结挂钩:挂起/恢复 shell 子进程(掐断产出侧)──────

#[link(name = "Kernel32")]
extern "system" {
    fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut core::ffi::c_void;
    fn CloseHandle(handle: *mut core::ffi::c_void) -> i32;
}
#[link(name = "ntdll")]
extern "system" {
    fn NtSuspendProcess(process: *mut core::ffi::c_void) -> i32;
    fn NtResumeProcess(process: *mut core::ffi::c_void) -> i32;
}

const PROCESS_SUSPEND_RESUME: u32 = 0x0800;

/// 取 shell 子进程 PID(空句柄/缺席 → None)。
fn engine_shell_pid(handle: i64) -> Option<u32> {
    let h = ptr_of(handle);
    if h.is_null() {
        return None;
    }
    unsafe {
        let pid: libloading::Symbol<unsafe extern "C" fn(*mut core::ffi::c_void) -> c_int> =
            lib().get(b"autoterm_engine_shell_pid\0").unwrap();
        let v = pid(h);
        if v > 0 { Some(v as u32) } else { None }
    }
}

/// 挂起/恢复 shell 子进程(NtSuspend/NtResume;哨兵冻结用)。
fn set_child_suspended(handle: i64, suspend: bool) -> bool {
    let Some(pid) = engine_shell_pid(handle) else { return false };
    unsafe {
        let proc = OpenProcess(PROCESS_SUSPEND_RESUME, 0, pid);
        if proc.is_null() {
            return false;
        }
        let ok = if suspend { NtSuspendProcess(proc) } else { NtResumeProcess(proc) };
        CloseHandle(proc);
        ok >= 0
    }
}

pub fn engine_suspend_child(handle: i64) -> bool {
    set_child_suspended(handle, true)
}

#[allow(dead_code)]
pub fn engine_resume_child(handle: i64) -> bool {
    set_child_suspended(handle, false)
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
