//! PLAN-009 T7: 独立进程集成测试——libloading 加载 `autoterm_core.dll`
//! 跑 echo 往返 + resize + interrupt 全链(008 语义:中断杀命令不杀终端)。
//!
//! 这就是 Auto 侧未来消费形态的预演:面对的只有本文件的 FFI 签名 +
//! DLL 文件,不触及 autoterm-core rlib 面。
//!
//! 运行:`cargo test -p autoterm-engine-ffi-tests`(构建顺序由 crate 依赖
//! 保证:autoterm-core 先出 cdylib 产物)。

use std::ffi::{c_char, c_int, CString};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use libloading::{Library, Symbol};

// DLL 面(与 crates/autoterm-core/src/ffi.rs 的 C ABI 一一对应)。
type Spawn = unsafe extern "C" fn(c_int, c_int, *const c_char) -> *mut core::ffi::c_void;
type WriteInput = unsafe extern "C" fn(*mut core::ffi::c_void, *const u8, usize);
type FeedReady = unsafe extern "C" fn(*mut core::ffi::c_void) -> c_int;
type TakeDirtyRows = unsafe extern "C" fn(*mut core::ffi::c_void, *mut c_int, c_int) -> c_int;
type RowText = unsafe extern "C" fn(*mut core::ffi::c_void, c_int, *mut c_char, c_int) -> c_int;
type Resize = unsafe extern "C" fn(*mut core::ffi::c_void, c_int, c_int);
type Interrupt = unsafe extern "C" fn(*mut core::ffi::c_void) -> c_int;
type IsExited = unsafe extern "C" fn(*mut core::ffi::c_void) -> c_int;
type Free = unsafe extern "C" fn(*mut core::ffi::c_void);

struct Engine {
    lib: Library,
    handle: *mut core::ffi::c_void,
    spawn: Symbol<'static, Spawn>,
    write_input: Symbol<'static, WriteInput>,
    feed_ready: Symbol<'static, FeedReady>,
    take_dirty_rows: Symbol<'static, TakeDirtyRows>,
    row_text: Symbol<'static, RowText>,
    resize: Symbol<'static, Resize>,
    interrupt: Symbol<'static, Interrupt>,
    is_exited: Symbol<'static, IsExited>,
    free: Symbol<'static, Free>,
}

// SAFETY: DLL 面本身是单线程消费约定(与 Auto 侧一致);句柄仅本线程使用。
unsafe impl Send for Engine {}

impl Engine {
    fn load() -> Self {
        let dll = dll_path();
        unsafe {
            let lib = Library::new(&dll)
                .unwrap_or_else(|e| panic!("加载 cdylib 失败({}): {e}", dll.display()));
            // libloading 惯用姿势:Symbol 的借用期拉平为 'static——Library
            // 本体保存在 Self 里同生共死,实际安全(官方文档认可的转写)。
            let spawn: Symbol<'static, Spawn> = std::mem::transmute(
                lib.get::<unsafe extern "C" fn(c_int, c_int, *const c_char) -> *mut core::ffi::c_void>(
                    b"autoterm_engine_spawn\0",
                )
                .unwrap(),
            );
            let handle = spawn(80, 24, std::ptr::null());
            assert!(!handle.is_null(), "autoterm_engine_spawn 返回 NULL");
            Self {
                handle,
                spawn,
                write_input: std::mem::transmute(
                    lib.get::<unsafe extern "C" fn(*mut core::ffi::c_void, *const u8, usize)>(b"autoterm_engine_write_input\0").unwrap(),
                ),
                feed_ready: std::mem::transmute(
                    lib.get::<unsafe extern "C" fn(*mut core::ffi::c_void) -> c_int>(b"autoterm_engine_feed_ready\0").unwrap(),
                ),
                take_dirty_rows: std::mem::transmute(
                    lib.get::<unsafe extern "C" fn(*mut core::ffi::c_void, *mut c_int, c_int) -> c_int>(b"autoterm_engine_take_dirty_rows\0").unwrap(),
                ),
                row_text: std::mem::transmute(
                    lib.get::<unsafe extern "C" fn(*mut core::ffi::c_void, c_int, *mut c_char, c_int) -> c_int>(b"autoterm_engine_row_text\0").unwrap(),
                ),
                resize: std::mem::transmute(
                    lib.get::<unsafe extern "C" fn(*mut core::ffi::c_void, c_int, c_int)>(b"autoterm_engine_resize\0").unwrap(),
                ),
                interrupt: std::mem::transmute(
                    lib.get::<unsafe extern "C" fn(*mut core::ffi::c_void) -> c_int>(b"autoterm_engine_interrupt\0").unwrap(),
                ),
                is_exited: std::mem::transmute(
                    lib.get::<unsafe extern "C" fn(*mut core::ffi::c_void) -> c_int>(b"autoterm_engine_is_exited\0").unwrap(),
                ),
                free: std::mem::transmute(
                    lib.get::<unsafe extern "C" fn(*mut core::ffi::c_void)>(b"autoterm_engine_free\0").unwrap(),
                ),
                lib,
            }
        }
    }

    fn write(&mut self, s: &str) {
        let bytes = s.as_bytes();
        unsafe { (self.write_input)(self.handle, bytes.as_ptr(), bytes.len()) };
    }

    fn feed(&mut self) -> c_int {
        unsafe { (self.feed_ready)(self.handle) }
    }

    /// 当前全屏快照(仅取文本;样式面由 row_style 符号承担,见 T8/T9)。
    fn snapshot(&mut self) -> Vec<String> {
        let mut rows = [0 as c_int; 64];
        let n = unsafe { (self.take_dirty_rows)(self.handle, rows.as_mut_ptr(), 64) };
        let _ = n;
        let mut out = Vec::new();
        for r in 0..40 {
            let mut buf = [0 as c_char; 512];
            let need =
                unsafe { (self.row_text)(self.handle, r, buf.as_mut_ptr(), 512) };
            if need < 0 {
                break;
            }
            let bytes: Vec<u8> = buf
                .iter()
                .take(need as usize - 1)
                .filter(|&&c| c != 0)
                .map(|&c| c as u8)
                .collect();
            out.push(String::from_utf8_lossy(&bytes).into_owned());
        }
        out
    }

    fn exited(&mut self) -> c_int {
        unsafe { (self.is_exited)(self.handle) }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        unsafe { (self.free)(self.handle) };
    }
}

fn dll_path() -> PathBuf {
    if let Ok(p) = std::env::var("AUTOTERM_ENGINE_DLL") {
        return PathBuf::from(p);
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target/debug/autoterm_core.dll")
}

/// 轮询直到快照任一行包含 needle 或超时(500ms 步进)。
fn wait_for_text(engine: &mut Engine, needle: &str, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        engine.feed();
        if engine.snapshot().iter().any(|l| l.contains(needle)) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    false
}

#[test]
fn ffi_echo_resize_interrupt_round_trip() {
    let mut engine = Engine::load();

    // —— echo 往返:写入命令 → 提示符/回显出现在网格文本 ——
    engine.write("echo autoterm_ffi_ok\r\n");
    assert!(
        wait_for_text(&mut engine, "autoterm_ffi_ok", Duration::from_secs(20)),
        "echo 往返:20s 内未在网格文本看到回显"
    );

    // —— resize:80×24 → 100×30,全量脏 + 第 29 行可读 ——
    unsafe { (engine.resize)(engine.handle, 100, 30) };
    engine.feed();
    let mut rows = [0 as c_int; 64];
    let dirty = unsafe { (engine.take_dirty_rows)(engine.handle, rows.as_mut_ptr(), 64) };
    assert_eq!(dirty, -1, "resize 后应报告 Full 损伤");
    let mut buf = [0 as c_char; 256];
    let need =
        unsafe { (engine.row_text)(engine.handle, 29, buf.as_mut_ptr(), 256) };
    assert!(need >= 0, "resize 后第 29 行(0 基)应可读");
    engine.write("echo resize_after_ok\r\n");
    assert!(
        wait_for_text(&mut engine, "resize_after_ok", Duration::from_secs(20)),
        "resize 后会话仍可用(echo 往返)"
    );

    // —— interrupt(008 双投递):长命令可中断,终端存活 ——
    // 主体用 timeout(008 对拍矩阵同款);ping 对 CTRL_BREAK 特殊处理
    // 不死(helper 头注明言),不能作中断主体。
    engine.write("timeout /t 60\r\n");
    assert!(
        wait_for_text(&mut engine, "60", Duration::from_secs(10)),
        "timeout 未进入等待态"
    );
    let ret = unsafe { (engine.interrupt)(engine.handle) };
    eprintln!("[diag] interrupt ret = {ret}");
    assert!(
        ret == 0 || ret == 1,
        "interrupt 返回值异常: {ret}(预期 1=事件广播/0=降级字节路径)"
    );
    // 终端未死:shell 仍在。
    assert_eq!(engine.exited(), 0, "中断只杀前台命令,不得杀终端会话");
    // shell 存活证据:中断后 echo 依旧往返(timeout 被杀、提示符回归)。
    engine.write("echo alive_after_interrupt\r\n");
    let ok = wait_for_text(&mut engine, "alive_after_interrupt", Duration::from_secs(20));
    if !ok {
        engine.feed();
        for (i, l) in engine.snapshot().iter().enumerate() {
            eprintln!("[diag] row {i:02}: {l:?}");
        }
        panic!("中断后 shell 无响应——008 语义破坏");
    }
}
