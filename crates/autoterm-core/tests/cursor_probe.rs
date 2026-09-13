//! 014 探针:观察 ConPTY 下 cmd 光标可见性/位置随输出的真实时序——
//! 定位「光标歪 + 不跟随键入」的采样语义。只打印,不断言。
//! Run: cargo test -p autoterm-core --test cursor_probe -- --nocapture

use autoterm_core::{autoterm_engine_cursor, autoterm_engine_feed_ready, autoterm_engine_free,
                    autoterm_engine_row_text, autoterm_engine_take_dirty_rows,
                    autoterm_engine_spawn, autoterm_engine_write_input};
use std::ffi::{c_char, c_int};

struct Eng(*mut autoterm_core::AutotermEngine);

impl Eng {
    fn feed(&self) {
        unsafe { autoterm_engine_feed_ready(self.0) };
        let mut rows = [0 as c_int; 64];
        unsafe { autoterm_engine_take_dirty_rows(self.0, rows.as_mut_ptr(), 64) };
    }
    fn cursor(&self) -> Option<(i32, i32)> {
        let (mut r, mut c) = (0 as c_int, 0 as c_int);
        let n = unsafe { autoterm_engine_cursor(self.0, &mut r, &mut c) };
        if n == 1 { Some((r, c)) } else { None }
    }
    fn row(&self, r: i32) -> String {
        let mut buf = [0 as c_char; 512];
        let need = unsafe { autoterm_engine_row_text(self.0, r, buf.as_mut_ptr(), 512) };
        if need < 0 {
            return String::new();
        }
        let n = (need as usize).saturating_sub(1).min(511);
        let bytes: Vec<u8> = buf[..n].iter().map(|&c| c as u8).collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }
    fn write(&self, s: &str) {
        let b = s.as_bytes();
        unsafe { autoterm_engine_write_input(self.0, b.as_ptr(), b.len()) };
    }
}

#[test]
fn probe_cursor_timeline() {
    let e = Eng(autoterm_engine_spawn(100, 30, std::ptr::null()));
    assert!(!e.0.is_null(), "spawn");
    std::thread::sleep(std::time::Duration::from_millis(800));
    e.feed();
    println!("== banner: cursor={:?}", e.cursor());

    // 逐键打字(50ms/键,模拟人速),每键后按 app 的 tick 语义采样:
    // cursor = FFI 光标;row(光标行文本) = 快照行——两者同一次 feed。
    let keys = ["d", "i", "r", "\r"];
    for k in keys {
        e.write(k);
        std::thread::sleep(std::time::Duration::from_millis(50));
        e.feed();
        let cur = e.cursor();
        let line = match cur {
            Some((r, _)) => e.row(r as i32),
            None => String::new(),
        };
        println!("key {k:?}: cursor={cur:?} curline={line:?}");
    }

    // dir 跑完(等输出排空)。
    for _ in 0..10 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        e.feed();
    }

    // 稳定提示符上逐键打 echo ABC,观察光标是否贴着回显走。
    let keys2 = [" ", "e", "c", "h", "o", " ", "A", "B", "C"];
    for k in keys2 {
        e.write(k);
        std::thread::sleep(std::time::Duration::from_millis(50));
        e.feed();
        let cur = e.cursor();
        let line = match cur {
            Some((r, _)) => e.row(r as i32),
            None => String::new(),
        };
        println!("key {k:?}: cursor={cur:?} curline={line:?}");
    }
    unsafe { autoterm_engine_free(e.0) };
}
