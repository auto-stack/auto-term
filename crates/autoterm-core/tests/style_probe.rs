//! 014 探针:ash 会话里引擎逐格样式(row_style)是否有非默认色——
//! 证明「ANSI → 引擎解析 → FFI 样式面」链路,colors 经旁路上屏的前半段。
//! Run: cargo test -p autoterm-core --test style_probe -- --nocapture

use autoterm_core::{autoterm_engine_feed_ready, autoterm_engine_free, autoterm_engine_row_style,
                    autoterm_engine_spawn, autoterm_engine_take_dirty_rows,
                    autoterm_engine_write_input};
use std::ffi::{CString};

#[test]
fn probe_ash_styles() {
    let Some(ash) = find_ash() else {
        println!("ash.exe not found — skip");
        return;
    };
    let program = CString::new(ash).unwrap();
    let e = autoterm_engine_spawn(100, 30, program.as_ptr());
    assert!(!e.is_null(), "spawn ash");
    std::thread::sleep(std::time::Duration::from_millis(1200));

    let mut non_default_seen = false;
    for round in 0..6 {
        unsafe { autoterm_engine_feed_ready(e) };
        let mut dirty = [0 as i32; 64];
        unsafe { autoterm_engine_take_dirty_rows(e, dirty.as_mut_ptr(), 64) };
        std::thread::sleep(std::time::Duration::from_millis(200));
        let mut styles = [0u32; 1024];
        let n = unsafe { autoterm_engine_row_style(e, 0, styles.as_mut_ptr(), 1024) };
        let non_default = (0..(n.max(0) as usize) / 2)
            .filter(|i| styles[i * 2] >> 24 != 0)
            .count();
        println!("round{round}: row0 style-pairs={} non-default-fg={}", n / 2, non_default);
        if round == 5 {
            for (i, s) in styles.iter().take(8).enumerate() {
                println!("  cell{} fg={:#010x} bg={:#010x}", i / 2, styles[(i / 2) * 2], styles[(i / 2) * 2 + 1]);
            }
            let mut styles20 = [0u32; 1024];
            let n20 = unsafe { autoterm_engine_row_style(e, 20, styles20.as_mut_ptr(), 1024) };
            println!("row20 pairs={} first-bg={:#010x} second-bg={:#010x}", n20 / 2, styles20[1], styles20[3]);
        }
        if non_default > 0 {
            non_default_seen = true;
        }
        if round == 2 {
            let cmd = b"ls\r\n";
            unsafe { autoterm_engine_write_input(e, cmd.as_ptr(), cmd.len()) };
        }
    }
    unsafe { autoterm_engine_free(e) };
    assert!(non_default_seen, "ash banner/ls 应产生非默认前景色");
}

fn find_ash() -> Option<String> {
    for p in [
        r"D:\autostack\auto-shell\ash\target\debug\ash.exe",
        r"D:\autostack\auto-shell\ash\target\release\ash.exe",
    ] {
        if std::path::Path::new(p).is_file() {
            return Some(p.to_string());
        }
    }
    None
}
