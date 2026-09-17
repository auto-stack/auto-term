//! PLAN-009 P2: cdylib FFI adapter —— Auto 侧(AutoLang a2r 产物)经
//! libloading 运行期加载本引擎(无环铁律:FFI 边界切断 Cargo 环)。
//!
//! 面形态(T6 细化,对齐 P1 组件数据面裁定 = 形态甲 props-feed):
//! - opaque handle(`AutotermEngine` 包 `PtySession`);
//! - 标量/字符串 C ABI,每帧流程:
//!   `feed_ready`(收割 reader 积压)→ `take_dirty_rows`(取损伤 + 刷新
//!   快照)→ `row_text` / `row_style`(按行取文本/样式,喂给组件);
//! - interrupt 保留 PLAN-008 双投递语义(直接走 PtySession::interrupt);
//! - 颜色标量编码:`(kind << 24) | value`,kind 0=Default、1=Indexed(0-255,
//!   含 vte Named 基础 16 色;≥256 的语义色归并 Default)、2=RGB(0xRRGGBB)。
//!
//! 安全边界:所有 `*mut AutotermEngine` 参数必须来自 `spawn` 且未经
//! `free`;空句柄返回哨兵值(-1 / 空串 / 0 行)。跨 FFI 无 panic:
//! 内部 catch 路径一律返回错误哨兵。

use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::PathBuf;

use crate::palette::{self, SCHEME_COUNT};
use crate::term::{Color as TermColor, StyledChar};
use crate::PtySession;

/// Opaque engine session(PTY 子进程 + 仿真核心)。
pub struct AutotermEngine {
    inner: PtySession,
    /// 最近一次快照(行文本 + 行样式),`take_dirty_rows` 刷新。
    snapshot: Vec<Vec<StyledChar>>,
    /// PLAN-018 D9:本会话配色方案 id(所有权铁律:scheme 属会话模型,
    /// per-handle 存储与校验;渲染端经 `palette_color` 纯查询取表)。
    palette: i32,
}

#[inline]
fn kind_color(c: TermColor) -> u32 {
    match c {
        // vte NamedColor:0-15 = 基础 16 色(discriminant 与 xterm base16
        // 对齐);≥256 = Foreground/Background/Cursor/Dim*/BrightForeground
        // 等语义色,不属于 Indexed(0-255) 契约 → 归并 Default,由宿主
        // 主题取默认前/背景(此前按原值传出,消费端 as u8 截断把
        // Background=257 折成 Indexed(1) 暗红——整屏红底黑字的根因)。
        TermColor::Named(n) if (n as u32) < 256 => 1u32 << 24 | n as u32,
        TermColor::Named(_) => 0,
        TermColor::Indexed(i) => 1u32 << 24 | i as u32,
        TermColor::Spec(rgb) => 2u32 << 24 | (rgb.r as u32) << 16 | (rgb.g as u32) << 8 | rgb.b as u32,
    }
}

#[inline]
fn ptr_or_null<'a>(h: *mut AutotermEngine) -> Option<&'a mut AutotermEngine> {
    // SAFETY: 句柄只能由 spawn 产生且 free 后不得再用(FFI 契约)。
    unsafe { h.as_mut() }
}

/// spawn 终端会话:`program` 为可执行文件名(NULL → 平台缺省 shell);
/// cols/rows 为网格几何(≤0 → 80×24)。失败返回 NULL。
#[unsafe(no_mangle)]
pub extern "C" fn autoterm_engine_spawn(
    cols: i32,
    rows: i32,
    program: *const c_char,
) -> *mut AutotermEngine {
    let cols = if cols <= 0 { 80 } else { cols as usize };
    let rows = if rows <= 0 { 24 } else { rows as usize };
    let prog = if program.is_null() {
        default_shell()
    } else {
        // SAFETY: 调用方保证指向 NUL 结尾字符串。
        match unsafe { CStr::from_ptr(program) }.to_str() {
            Ok(s) if !s.is_empty() => s.to_owned(),
            _ => default_shell(),
        }
    };
    match PtySession::spawn(&prog, Vec::<String>::new(), cols, rows) {
        Ok(session) => Box::into_raw(Box::new(AutotermEngine { inner: session, snapshot: Vec::new(), palette: 0 })),
        Err(_) => std::ptr::null_mut(),
    }
}

fn default_shell() -> String {
    std::env::var("COMSPEC").unwrap_or_else(|_| "cmd".to_owned())
}

/// 读 NUL 结尾 C 字符串(NULL/非 UTF-8 → None;spawn_ex 参数解析共用)。
fn read_c_string(p: *const c_char) -> Option<String> {
    if p.is_null() {
        return None;
    }
    // SAFETY: 调用方保证指向 NUL 结尾字符串。
    unsafe { CStr::from_ptr(p) }.to_str().ok().map(str::to_owned)
}

/// PLAN-018 D1 SpawnSpec face(face 16→17):扩展 spawn——program/argv/
/// cwd/几何。`argv` 为 C 字符串指针数组(`argc` 计数;NULL 或 argc≤0 =
/// 无参数,遇 NULL 提前止);`cwd` NULL/空 = 继承宿主;cols/rows ≤0 →
/// 80×24。失败返回 NULL。旧 `autoterm_engine_spawn` 语义零改动。
#[unsafe(no_mangle)]
pub extern "C" fn autoterm_engine_spawn_ex(
    program: *const c_char,
    argv: *const *const c_char,
    argc: i32,
    cwd: *const c_char,
    cols: i32,
    rows: i32,
) -> *mut AutotermEngine {
    let cols = if cols <= 0 { 80 } else { cols as usize };
    let rows = if rows <= 0 { 24 } else { rows as usize };
    let prog = match read_c_string(program) {
        Some(s) if !s.is_empty() => s,
        _ => default_shell(),
    };
    let mut args: Vec<String> = Vec::new();
    if !argv.is_null() && argc > 0 {
        for i in 0..argc as isize {
            // SAFETY: 调用方保证 argv 为 argc 个指向 NUL 结尾字符串的指针。
            let p = unsafe { *argv.offset(i) };
            if p.is_null() {
                break;
            }
            match read_c_string(p) {
                Some(s) => args.push(s),
                None => break,
            }
        }
    }
    let cwd_path = read_c_string(cwd).filter(|s| !s.is_empty()).map(PathBuf::from);
    match PtySession::spawn_in(&prog, args, cwd_path.as_deref(), cols, rows) {
        Ok(session) => Box::into_raw(Box::new(AutotermEngine { inner: session, snapshot: Vec::new(), palette: 0 })),
        Err(_) => std::ptr::null_mut(),
    }
}

/// 宿主→子进程字节(键盘输入)。空句柄/空缓冲为 no-op。
/// 键入即贴底回实时(终端惯例;回滚浏览中打字自动回到底部)。
#[unsafe(no_mangle)]
pub extern "C" fn autoterm_engine_write_input(
    h: *mut AutotermEngine,
    bytes: *const u8,
    len: usize,
) {
    let Some(engine) = ptr_or_null(h) else { return };
    if bytes.is_null() || len == 0 {
        return;
    }
    // SAFETY: 调用方保证 bytes 至少 len 字节可读。
    let slice = unsafe { std::slice::from_raw_parts(bytes, len) };
    engine.inner.write_input(slice);
    engine.inner.term.scroll_to_bottom();
}

/// 回滚浏览(PLAN-019 冒烟期用户追加):正=上翻历史,负=下回实时
/// (alacritty Scroll::Delta 语义,按行计;UI 滚轮经宿主 glue 排水到此)。
#[unsafe(no_mangle)]
pub extern "C" fn autoterm_engine_scroll(h: *mut AutotermEngine, delta_lines: i32) {
    let Some(engine) = ptr_or_null(h) else { return };
    engine.inner.term.scroll(delta_lines);
}

/// 当前回滚偏移(0 = 贴底实时;历史区行数;空句柄 0)。
#[unsafe(no_mangle)]
pub extern "C" fn autoterm_engine_scroll_offset(h: *mut AutotermEngine) -> i32 {
    let Some(engine) = ptr_or_null(h) else { return 0 };
    engine.inner.term.display_offset() as i32
}

/// 收割 reader 线程积压并喂仿真核心。返回 1 = 喂到了字节(可能变脏),
/// 0 = 无字节,-1 = 空句柄。
#[unsafe(no_mangle)]
pub extern "C" fn autoterm_engine_feed_ready(h: *mut AutotermEngine) -> i32 {
    let Some(engine) = ptr_or_null(h) else { return -1 };
    if engine.inner.drain() {
        1
    } else {
        0
    }
}

/// 取损伤并刷新行快照:把自上次调用以来变脏的行号写进 `out_rows`
/// (≤cap 个),返回写入个数;全量脏返回 -1(调用方应重取全部行);
/// 空句柄返回 -2。
#[unsafe(no_mangle)]
pub extern "C" fn autoterm_engine_take_dirty_rows(
    h: *mut AutotermEngine,
    out_rows: *mut i32,
    cap: i32,
) -> i32 {
    let Some(engine) = ptr_or_null(h) else { return -2 };
    let damage = engine.inner.term.take_damage();
    engine.snapshot = engine.inner.term.visible_styled_lines();
    let rows_count = engine.snapshot.len() as i32;
    match damage {
        crate::Damage::Full => -1,
        crate::Damage::Lines(lines) => {
            if lines.is_empty() {
                return 0;
            }
            if out_rows.is_null() || cap <= 0 {
                return lines.len() as i32;
            }
            let cap = cap.min(rows_count);
            // SAFETY: 调用方保证 out_rows 可写 cap 个 i32。
            let out = unsafe { std::slice::from_raw_parts_mut(out_rows, cap as usize) };
            let n = lines.len().min(cap as usize);
            for (i, row) in lines.iter().take(n).enumerate() {
                out[i] = *row as i32;
            }
            n as i32
        }
    }
}

/// 取一行纯文本(UTF-8,NUL 结尾)写入 out_buf(≤cap 字节)。
/// 返回含 NUL 的总字节数(截断时大于已写字节数;行越界/空句柄 -1)。
#[unsafe(no_mangle)]
pub extern "C" fn autoterm_engine_row_text(
    h: *mut AutotermEngine,
    row: i32,
    out_buf: *mut c_char,
    cap: i32,
) -> i32 {
    let Some(engine) = ptr_or_null(h) else { return -1 };
    let Some(line) = engine.snapshot.get(row as usize) else { return -1 };
    let text: String = line.iter().map(|sc| sc.c).collect();
    let bytes = text.as_bytes();
    let out = unsafe { std::slice::from_raw_parts_mut(out_buf as *mut u8, cap.max(0) as usize) };
    let n = bytes.len().min(out.len().saturating_sub(1));
    out[..n].copy_from_slice(&bytes[..n]);
    out[n] = 0;
    (bytes.len() + 1) as i32
}

/// 取一行逐格样式:fg/bg 交错写入 out(u32 标量色,编码见模块头)。
/// 需要 2×cols 容量;返回写入的 u32 个数(行越界/空句柄/容量不足 -1)。
#[unsafe(no_mangle)]
pub extern "C" fn autoterm_engine_row_style(
    h: *mut AutotermEngine,
    row: i32,
    out: *mut u32,
    cap: i32,
) -> i32 {
    let Some(engine) = ptr_or_null(h) else { return -1 };
    let Some(line) = engine.snapshot.get(row as usize) else { return -1 };
    if out.is_null() || cap < (line.len() * 2) as i32 {
        return -1;
    }
    // SAFETY: 调用方保证 out 可写 cap 个 u32。
    let out = unsafe { std::slice::from_raw_parts_mut(out, cap as usize) };
    for (i, sc) in line.iter().enumerate() {
        out[i * 2] = kind_color(sc.fg);
        out[i * 2 + 1] = kind_color(sc.bg);
    }
    (line.len() * 2) as i32
}

/// 光标:可见时写 (row, col) 并返回 1;不可见 0;空句柄 -1。
#[unsafe(no_mangle)]
pub extern "C" fn autoterm_engine_cursor(
    h: *mut AutotermEngine,
    out_row: *mut i32,
    out_col: *mut i32,
) -> i32 {
    let Some(engine) = ptr_or_null(h) else { return -1 };
    match engine.inner.term.cursor() {
        Some((row, col)) => {
            if let Some(p) = unsafe { out_row.as_mut() } { *p = row as i32; }
            if let Some(p) = unsafe { out_col.as_mut() } { *p = col as i32; }
            1
        }
        None => 0,
    }
}

/// 014 泄漏定位:reader→drain 积压字节(正常稳定在单 tick 输出量级;
/// 无界增长 = 产出侧失控或消费侧停摆)。空句柄 -1。
#[unsafe(no_mangle)]
pub extern "C" fn autoterm_engine_pending_bytes(h: *mut AutotermEngine) -> i32 {
    let Some(engine) = ptr_or_null(h) else { return -1 };
    engine.inner.pending_bytes().min(i32::MAX as u64) as i32
}

/// 014 报警面:reader 是否因积压超限暂停读取(反压中)。
/// 1 = 暂停中,0 = 正常,空句柄 -1。
#[unsafe(no_mangle)]
pub extern "C" fn autoterm_engine_backlog_paused(h: *mut AutotermEngine) -> i32 {
    let Some(engine) = ptr_or_null(h) else { return -1 };
    if engine.inner.backpressure_engaged() { 1 } else { 0 }
}

/// 014 内存哨兵冻结挂钩:返回 shell 子进程 PID(宿主挂起/恢复用);
/// 无子进程/空句柄 -1。
#[unsafe(no_mangle)]
pub extern "C" fn autoterm_engine_shell_pid(h: *mut AutotermEngine) -> i32 {
    let Some(engine) = ptr_or_null(h) else { return -1 };
    match engine.inner.shell_pid() {
        Some(pid) if pid <= i32::MAX as u32 => pid as i32,
        _ => -1,
    }
}

/// 014 环形缓冲溢出计数:被挤掉的输出块数(预警/取证;0 = 未溢出)。
#[unsafe(no_mangle)]
pub extern "C" fn autoterm_engine_overflow_count(h: *mut AutotermEngine) -> i32 {
    let Some(engine) = ptr_or_null(h) else { return -1 };
    engine.inner.overflow_dropped().min(i32::MAX as u64) as i32
}

/// resize(先仿真核心后 ConPTY 的既有顺序)。
#[unsafe(no_mangle)]
pub extern "C" fn autoterm_engine_resize(h: *mut AutotermEngine, cols: i32, rows: i32) {
    let Some(engine) = ptr_or_null(h) else { return };
    let cols = if cols <= 0 { 80 } else { cols as usize };
    let rows = if rows <= 0 { 24 } else { rows as usize };
    engine.inner.resize(cols, rows);
}

/// Ctrl+C 中断注入(008 双投递语义)。1 = 事件广播成功,0 = 降级字节路径。
#[unsafe(no_mangle)]
pub extern "C" fn autoterm_engine_interrupt(h: *mut AutotermEngine) -> i32 {
    let Some(engine) = ptr_or_null(h) else { return -1 };
    if engine.inner.interrupt() {
        1
    } else {
        0
    }
}

/// 子进程是否已退出。1 = 是,0 = 否,-1 = 空句柄。
#[unsafe(no_mangle)]
pub extern "C" fn autoterm_engine_is_exited(h: *mut AutotermEngine) -> i32 {
    let Some(engine) = ptr_or_null(h) else { return -1 };
    if engine.inner.exited() {
        1
    } else {
        0
    }
}

/// kill 子进程(强杀;会话资源仍需 free 释放)。
#[unsafe(no_mangle)]
pub extern "C" fn autoterm_engine_kill(h: *mut AutotermEngine) {
    let Some(engine) = ptr_or_null(h) else { return };
    engine.inner.kill();
}

/// 释放会话(free 后句柄失效,不得再用)。
/// PLAN-018 D9(rev2)配色方案面(face 17→19):设置本会话配色方案
/// (per-handle,所有权铁律)。0 = 成功;-1 = 空句柄;-2 = 未知方案 id。
#[unsafe(no_mangle)]
pub extern "C" fn autoterm_engine_set_palette(h: *mut AutotermEngine, scheme_id: i32) -> i32 {
    let Some(engine) = ptr_or_null(h) else { return -1 };
    if palette::palette(scheme_id).is_none() {
        return -2;
    }
    engine.palette = scheme_id;
    0
}

/// 方案槽位取色(纯函数,无柄;scheme 表引擎单源见 palette.rs)。
/// slot:0=def-fg、1=def-bg、2..=17=base16;is_fg 预留轴(0/1)。
/// 返回 0xRRGGBB;非法方案/槽位/轴 → 0xFFFF_FFFF 哨兵(RGB 最高位必 0,
/// 与合法值无歧义;宿主装载缓存时以此判定方案枚举终点)。
#[unsafe(no_mangle)]
pub extern "C" fn autoterm_engine_palette_color(scheme_id: i32, slot: i32, is_fg: i32) -> u32 {
    match palette::palette_color(scheme_id, slot, is_fg) {
        Some(rgb) => rgb,
        None => 0xFFFF_FFFF,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn autoterm_engine_free(h: *mut AutotermEngine) {
    if h.is_null() {
        return;
    }
    // SAFETY: 句柄由 spawn 的 Box::into_raw 产生。
    unsafe { drop(Box::from_raw(h)) };
}
