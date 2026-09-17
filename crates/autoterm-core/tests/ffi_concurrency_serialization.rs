//! PLAN-021 T-03 金样:同柄并发 FFI 串行化回归钉。
//!
//! 020 复审 P1 根因(仪器在案):vue sustained 页面轮询下 axum 多 worker
//! 对同柄并发 FFI,导出面裸 `&mut` 别名——feed_ready∥feed_ready(drain
//! →term.feed 双 &mut 推进 vte 网格)、take_dirty_rows∥row_text(snapshot
//! Vec 整体替换 vs 读侧悬垂)等腐蚀对 → STATUS_HEAP_CORRUPTION。
//! 本测试 6 线程同锤一柄(变异 + 读 + resize 混合流):修复后(FFI 边界
//! 每柄串行化)确定性通过;修复前本测试即堆破坏现场(进程级 0xc0000374)。
//!
//! 运行:`cargo test -p autoterm-core --test ffi_concurrency_serialization`

use std::ffi::CString;

#[test]
fn concurrent_ffi_on_one_handle_is_serialized_and_survives() {
    let prog = CString::new("cmd.exe").unwrap();
    let h = unsafe {
        autoterm_core::autoterm_engine_spawn_ex(
            prog.as_ptr(),
            std::ptr::null(),
            0,
            std::ptr::null(),
            80,
            24,
        )
    };
    assert!(!h.is_null(), "spawn_ex 应产出有效引擎句柄");

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(6);
    let addr = h as usize; // 裸指针 !Send,以 usize 过线程,柄内回转
    let mut workers = Vec::new();
    for t in 0..6u32 {
        workers.push(std::thread::spawn(move || {
            let h = addr as *mut autoterm_core::AutotermEngine;
            let mut rounds = 0u32;
            while std::time::Instant::now() < deadline {
                unsafe {
                    // 变异臂:drain(vte 网格推进)+ 快照整体替换。
                    autoterm_core::autoterm_engine_feed_ready(h);
                    let mut rows = [0i32; 64];
                    autoterm_core::autoterm_engine_take_dirty_rows(h, rows.as_mut_ptr(), 64);
                    // 读臂:读 snapshot 缓冲(修复前与替换臂悬垂竞态)。
                    let mut buf = [0i8; 512];
                    autoterm_core::autoterm_engine_row_text(h, 0, buf.as_mut_ptr(), 512);
                    let mut styles = [0u32; 1024];
                    autoterm_core::autoterm_engine_row_style(h, 0, styles.as_mut_ptr(), 1024);
                    autoterm_core::autoterm_engine_cursor(h, rows.as_mut_ptr(), rows.as_mut_ptr().offset(1));
                    // 几何臂:网格重分配(014 几何随动面)。
                    autoterm_core::autoterm_engine_resize(h, 80, 24);
                    if t == 0 {
                        autoterm_core::autoterm_engine_write_input(h, b"echo hi\r\n".as_ptr(), 9);
                    }
                }
                rounds += 1;
            }
            rounds
        }));
    }
    for w in workers {
        let rounds = w.join().expect("worker 线程不得 panic(堆破坏即 abort)");
        assert!(rounds > 0, "每 worker 都应完成若干轮");
    }
    unsafe {
        autoterm_core::autoterm_engine_kill(h);
        autoterm_core::autoterm_engine_free(h);
    }
}
