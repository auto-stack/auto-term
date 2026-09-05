//! autoterm-ui — AutoTerm 单窗口终端(iced,PLAN-002 正式架构)
//!
// SPDX-License-Identifier: Apache-2.0
//! 事件驱动:reader 线程经唤醒通道 → iced 订阅 → `Message::PtyBytes`,
//! 无轮询定时器(PLAN-001 spike 的 16ms tick 已移除;dev 钩子激活时
//! 才有粗粒度 dev timer)。

pub mod metrics;
pub mod palette;
pub mod widget;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use iced::keyboard::{Key, Modifiers};
use iced::stream;
use iced::{Color, Element, Size, Subscription, Task, time};
use std::hash::{Hash, Hasher};

use autoterm_core::PtySession;
use autoterm_core::{Damage, StyledChar};
pub use widget::TermGrid;
use metrics::GridMetrics;

/// 运行配置(bin 解析后传入)。
#[derive(Clone, Debug)]
pub struct AppConfig {
    pub shell: String,
    /// dev 取证:自动键入(可多段,"ms:text" 语法同 spike)。
    pub dev_autotype: Vec<String>,
    /// dev 取证:到时转储并退出(秒;0=不退出)。
    pub dev_exit_after: u64,
    /// dev 取证:退出前回滚的行数(正=上翻;转储回滚后视图)。
    pub dev_scroll: Option<i32>,
    /// dev 取证:退出时转储目标文件。
    pub dev_dump: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// PTY 有字节可读(唤醒,不载荷——update 里 drain)。
    PtyBytes,
    Key(Key, Modifiers),
    Resized(Size),
    WindowId(Option<iced::window::Id>),
    /// 滚轮回滚(正=上翻历史,同 Scroll::Delta 约定;行为行)。
    Scrolled(i32),
    /// 窗口关闭:同步杀子进程再退出(T8)。
    Closed(iced::window::Id),
    /// dev 钩子的粗定时(仅 dev 参数激活时订阅;常态不存在)。
    DevTick,
}

/// 订阅数据源:唤醒接收端的"一次性槽"(run_with 需 Hash,恒等即可)。
#[derive(Clone)]
struct NotifySlot(Arc<Mutex<Option<Receiver<()>>>>);

impl Hash for NotifySlot {
    fn hash<H: Hasher>(&self, state: &mut H) {
        0u64.hash(state);
    }
}

pub struct App {
    pub session: PtySession,
    pub config: AppConfig,
    pub metrics: GridMetrics,
    notify_slot: Arc<Mutex<Option<Receiver<()>>>>,
    window_id: Option<iced::window::Id>,
    /// 最近一次窗口视口尺寸(dev 转储 fit_ok 用)。
    pub last_viewport: Option<Size>,

    pub frames: u64,
    pub dirty_updates: u64,
    pub started: Instant,
    pub last_byte_at: Option<Instant>,
    input_sent_at: Option<Instant>,
    /// 损伤门控快照(T6):仅在字节到达/resize/scroll 后重建。
    pub damage: Damage,
    pub snapshot: Vec<Vec<StyledChar>>,
    pub snapshot_rebuilds: u64,
    /// 光标(视口相对;Hidden=None)。随快照一并刷新。
    pub cursor: Option<(usize, usize)>,

    queued_input: Vec<(Instant, Vec<u8>)>,
    exit_at: Option<Instant>,
}

impl App {
    pub fn new(config: AppConfig, cols: usize, rows: usize) -> anyhow::Result<Self> {
        let mut session =
            PtySession::spawn(&config.shell, std::iter::empty::<&str>(), cols, rows)?;
        let notify_slot = Arc::new(Mutex::new(session.take_notify_receiver()));
        let metrics = metrics::measure();
        let now = Instant::now();
        let queued_input = config
            .dev_autotype
            .iter()
            .map(|s| parse_input(s, now))
            .collect();
        let exit_at = (config.dev_exit_after > 0)
            .then(|| now + Duration::from_secs(config.dev_exit_after));
        Ok(Self {
            session,
            config,
            metrics,
            notify_slot,
            window_id: None,
            last_viewport: None,
            frames: 0,
            dirty_updates: 0,
            started: now,
            last_byte_at: None,
            input_sent_at: None,
            damage: Damage::Full,
            snapshot: Vec::new(),
            snapshot_rebuilds: 0,
            cursor: None,
            queued_input,
            exit_at,
        }
        .initialized())
    }

    pub fn title(&self) -> &str {
        "AutoTerm"
    }

    /// 构造尾拍:首帧快照就位(空网格 + Full 损伤)。
    fn initialized(mut self) -> Self {
        self.refresh_after_change();
        self
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![
            // PTY 字节唤醒:转发线程 block 在 core 的唤醒通道上,
            // try_send 进 iced 异步通道——字节到达即醒,无轮询。
            Subscription::run_with(NotifySlot(self.notify_slot.clone()), |slot| {
                let slot = slot.clone();
                stream::channel(
                    16,
                    move |mut sender: iced::futures::channel::mpsc::Sender<Message>| async move {
                        let NotifySlot(slot) = slot;
                        if let Some(rx) = slot.lock().unwrap().take() {
                            std::thread::spawn(move || {
                                loop {
                                    match rx.recv() {
                                        Ok(()) => {
                                            if sender.try_send(Message::PtyBytes).is_err() {
                                                break;
                                            }
                                        }
                                        Err(_) => break,
                                    }
                                }
                            });
                        }
                        iced::futures::future::pending::<()>().await
                    },
                )
            }),
            iced::keyboard::listen().map(|event| match event {
                iced::keyboard::Event::KeyPressed { key, modifiers, .. } => {
                    Message::Key(key, modifiers)
                }
                _ => Message::PtyBytes,
            }),
            iced::window::resize_events()
                .map(|(_id, size)| Message::Resized(size)),
            // 滚轮回滚:wheel 正 y = 上翻(进历史),每格 3 行
            iced::event::listen().map(|event| match event {
                iced::Event::Mouse(iced::mouse::Event::WheelScrolled {
                    delta: iced::mouse::ScrollDelta::Lines { y, .. },
                }) => Message::Scrolled((y * 3.0) as i32),
                _ => Message::PtyBytes,
            }),
            // 窗口关闭:同步清理子进程(T8)
            iced::window::close_events().map(Message::Closed),
        ];
        if self.config.dev_autotype.is_empty() && self.exit_at.is_none() {
            // 常态:无任何定时器,空闲零唤醒(验收标准 3)
        } else {
            subs.push(
                time::every(Duration::from_millis(50)).map(|_| Message::DevTick),
            );
        }
        Subscription::batch(subs)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PtyBytes => {
                // 键盘非按键事件也走到这里作空唤醒——drain 天然幂等
                self.pump();
                Task::none()
            }
            Message::DevTick => {
                let now = Instant::now();
                self.queued_input.retain(|(at, bytes)| {
                    if now >= *at {
                        self.session.write_input(bytes);
                        self.input_sent_at.get_or_insert_with(Instant::now);
                        false
                    } else {
                        true
                    }
                });
                if let Some(at) = self.exit_at {
                    if now >= at {
                        if let Some(delta) = self.config.dev_scroll {
                            self.session.term.scroll(delta);
                            self.refresh_after_change();
                        }
                        self.dump_state();
                        self.session.kill();
                        return iced::exit();
                    }
                }
                Task::none()
            }
            Message::Key(key, mods) => {
                // 回滚时键入先回正(终端惯例)
                if self.session.term.display_offset() > 0 {
                    self.session.term.scroll(i32::MIN);
                    self.refresh_after_change();
                }
                // PgUp/PgDn = UI 翻页回滚(消费,不进 PTY)
                match &key {
                    Key::Named(iced::keyboard::key::Named::PageUp) => {
                        let rows = self.session.term.size().1 as i32;
                        self.session.term.scroll(rows);
                        self.refresh_after_change();
                        return Task::none();
                    }
                    Key::Named(iced::keyboard::key::Named::PageDown) => {
                        let rows = self.session.term.size().1 as i32;
                        self.session.term.scroll(-rows);
                        self.refresh_after_change();
                        return Task::none();
                    }
                    _ => {}
                }
                if let Some(bytes) = key_to_bytes(&key, &mods) {
                    self.session.write_input(&bytes);
                }
                Task::none()
            }
            Message::Scrolled(delta) => {
                self.session.term.scroll(delta);
                self.refresh_after_change();
                Task::none()
            }
            Message::Closed(_id) => {
                // 关闭语义(T8):显式杀+等,窗口关闭不留孤儿进程;
                // Drop 安全网仍在(异常路径兜底)。
                self.session.kill();
                let _ = self.session.wait();
                iced::exit()
            }
            Message::WindowId(id) => {
                self.window_id = id;
                Task::none()
            }
            Message::Resized(size) => {
                self.last_viewport = Some(size);
                let cols =
                    ((size.width / self.metrics.cell_w).floor() as usize).max(10);
                let rows =
                    ((size.height / self.metrics.line_h).floor() as usize).max(4);
                self.session.resize(cols, rows);
                self.refresh_after_change();
                Task::none()
            }
        }
    }

    /// 收割 PTY 字节喂仿真核心;标记重绘。
    fn pump(&mut self) {
        if self.session.drain() {
            self.last_byte_at = Some(Instant::now());
            self.dirty_updates += 1;
            self.refresh_after_change();
        }
        self.frames += 1;
    }

    /// 内容变了(resize/scroll/字节到达)之后:取损伤、按需重建快照。
    /// 损伤门控(T6):快照仅在内容实际变化时重建,空闲帧免全网格遍历;
    /// 绘制级脏行剪裁在 iced 即时模式下会清空未脏行,不做(见
    /// docs/designs/001 的损伤协议节)。
    fn refresh_after_change(&mut self) {
        self.damage = self.session.term.take_damage();
        let content_changed = match &self.damage {
            Damage::Full => true,
            Damage::Lines(lines) => !lines.is_empty(),
        };
        if content_changed {
            self.snapshot = self.session.term.visible_styled_lines();
            self.cursor = self.session.term.cursor();
            self.snapshot_rebuilds += 1;
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        Element::new(TermGrid {
            lines: self.snapshot.clone(),
            metrics: self.metrics,
            damage: self.damage.clone(),
            scroll_offset: self.session.term.display_offset(),
            cursor: self.cursor,
        })
    }

    /// dev 转储:网格文本 + 计数(T5 起追加 metrics/fit、T6 runs、
    /// T7 偏移、T8 光标)。
    pub fn dump_state(&mut self) {
        let Some(path) = self.config.dev_dump.clone() else { return };
        let mut out = String::new();
        let (cols, rows) = self.session.term.size();
        let _ = std::fmt::Write::write_fmt(
            &mut out,
            format_args!("shell: {}\ngrid: {cols}x{rows}\n", self.config.shell),
        );
        let _ = std::fmt::Write::write_fmt(
            &mut out,
            format_args!(
                "metrics: font_px={:.3} cell_w={:.4} line_h={:.3} (cell_em={:.4})\n",
                self.metrics.font_px,
                self.metrics.cell_w,
                self.metrics.line_h,
                self.metrics.cell_w / self.metrics.font_px
            ),
        );
        match self.last_viewport {
            Some(v) => {
                let fit = (cols as f32 * self.metrics.cell_w) <= v.width + 0.5;
                let _ = std::fmt::Write::write_fmt(
                    &mut out,
                    format_args!(
                        "viewport: {:.1}x{:.1}\nfit_ok: {} (cols*cell_w={:.1} <= viewport_w={:.1})\n",
                        v.width, v.height, fit, cols as f32 * self.metrics.cell_w, v.width
                    ),
                );
            }
            None => out.push_str("viewport: unknown\nfit_ok: unknown\n"),
        }
        let _ = std::fmt::Write::write_fmt(
            &mut out,
            format_args!(
                "uptime_s: {:.3}\nframes: {}\ndirty_updates: {}\nsnapshot_rebuilds: {}\n",
                self.started.elapsed().as_secs_f64(),
                self.frames,
                self.dirty_updates,
                self.snapshot_rebuilds
            ),
        );
        match &self.damage {
            Damage::Full => out.push_str("damage_last: full\n"),
            Damage::Lines(l) => {
                let _ = std::fmt::Write::write_fmt(
                    &mut out,
                    format_args!("damage_last: lines={}\n", l.len()),
                );
            }
        }
        let _ = std::fmt::Write::write_fmt(
            &mut out,
            format_args!(
                "scroll_offset: {}\n",
                self.session.term.display_offset()
            ),
        );
        let (rb_prev, rb_last) = widget::paragraph_rebuilds();
        let _ = std::fmt::Write::write_fmt(
            &mut out,
            format_args!(
                "paragraph_rebuilds_prev: {rb_prev}\nparagraph_rebuilds_last: {rb_last}\n"
            ),
        );
        match widget::cursor_drawn() {
            Some((row, col)) => {
                let _ = std::fmt::Write::write_fmt(
                    &mut out,
                    format_args!("cursor_drawn_at: ({row},{col}) inverted=true\n"),
                );
            }
            None => out.push_str("cursor_drawn_at: none inverted=false\n"),
        }
        if let Some(t) = self.last_byte_at {
            let _ = std::fmt::Write::write_fmt(
                &mut out,
                format_args!("last_byte_after_start_s: {:.3}\n", (t - self.started).as_secs_f64()),
            );
        }
        if let Some(t) = self.input_sent_at {
            let _ = std::fmt::Write::write_fmt(
                &mut out,
                format_args!("input_sent_after_start_s: {:.3}\n", (t - self.started).as_secs_f64()),
            );
            if let Some(l) = self.last_byte_at {
                let _ = std::fmt::Write::write_fmt(
                    &mut out,
                    format_args!(
                        "last_byte_after_input_s: {:.3}\n",
                        (l - t).as_secs_f64()
                    ),
                );
            }
        }
        out.push_str("=== grid_text_begin ===\n");
        for line in self.session.term.visible_lines() {
            let _ = std::fmt::Write::write_fmt(&mut out, format_args!("{line}\n"));
        }
        out.push_str("=== grid_text_end ===\n");
        let _ = std::fs::write(&path, out);
        eprintln!("dumped: {}", path.display());
    }
}

/// dev-autotype 语法:"<延迟毫秒>:<文本>";无前缀即立即。转义
/// \r \n \t(同 spike)。
fn parse_input(s: &str, start: Instant) -> (Instant, Vec<u8>) {
    if let Some((n, rest)) = s.split_once(':') {
        if let Ok(ms) = n.parse::<u64>() {
            return (start + Duration::from_millis(ms), unescape(rest));
        }
    }
    (start, unescape(s))
}

fn unescape(s: &str) -> Vec<u8> {
    let mut out = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.peek() {
                Some('r') | Some('n') => {
                    chars.next();
                    out.push(b'\r');
                }
                Some('t') => {
                    chars.next();
                    out.push(b'\t');
                }
                Some(other) => {
                    out.push(*other as u8);
                    chars.next();
                }
                None => out.push(b'\\'),
            }
        } else {
            let mut buf = [0u8; 4];
            out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
        }
    }
    out
}

/// 键盘 → PTY 字节(承 spike:可打印/Enter/Backspace/Tab/方向键/
/// Home/End/Delete/PgUp/PgDn/Ctrl+字母;Shift 符号映射自理)。
pub fn key_to_bytes(key: &Key, mods: &Modifiers) -> Option<Vec<u8>> {
    const SHIFT_MAP: &[(char, char)] = &[
        ('1', '!'), ('2', '@'), ('3', '#'), ('4', '$'), ('5', '%'),
        ('6', '^'), ('7', '&'), ('8', '*'), ('9', '('), ('0', ')'),
        ('-', '_'), ('=', '+'), ('[', '{'), (']', '}'), ('\\', '|'),
        (';', ':'), ('\'', '"'), (',', '<'), ('.', '>'), ('/', '?'),
        ('`', '~'),
    ];
    match key {
        Key::Character(s) => {
            let mut c = s.chars().next()?;
            if mods.control() {
                c = c.to_ascii_lowercase();
                if c.is_ascii_lowercase() {
                    return Some(vec![(c as u8) - b'a' + 1]);
                }
                return None;
            }
            if mods.shift() && c.is_ascii_lowercase() {
                c = c.to_ascii_uppercase();
            } else if mods.shift() {
                if let Some((_, upper)) = SHIFT_MAP.iter().find(|(low, _)| *low == c) {
                    c = *upper;
                }
            }
            let mut buf = [0u8; 4];
            Some(c.encode_utf8(&mut buf).as_bytes().to_vec())
        }
        Key::Named(named) => match named {
            iced::keyboard::key::Named::Enter => Some(b"\r".to_vec()),
            iced::keyboard::key::Named::Backspace => Some(vec![0x7f]),
            iced::keyboard::key::Named::Tab => Some(b"\t".to_vec()),
            iced::keyboard::key::Named::Escape => Some(vec![0x1b]),
            iced::keyboard::key::Named::ArrowUp => Some(b"\x1b[A".to_vec()),
            iced::keyboard::key::Named::ArrowDown => Some(b"\x1b[B".to_vec()),
            iced::keyboard::key::Named::ArrowRight => Some(b"\x1b[C".to_vec()),
            iced::keyboard::key::Named::ArrowLeft => Some(b"\x1b[D".to_vec()),
            iced::keyboard::key::Named::Home => Some(b"\x1b[H".to_vec()),
            iced::keyboard::key::Named::End => Some(b"\x1b[F".to_vec()),
            iced::keyboard::key::Named::Delete => Some(b"\x1b[3~".to_vec()),
            _ => None,
        },
        _ => None,
    }
}

/// 默认前景/底色(与 widget 底色一致)。
pub const DEFAULT_FG: Color = Color::from_rgb8(0xe8, 0xe8, 0xe8);
pub const DEFAULT_BG: Color = Color::from_rgb8(0x10, 0x14, 0x18);

/// resize 换算用的度量基准(T5 实测化之前的过渡实现:
/// Consolas advance 0.5498em、行高 1.25em、字号 16px)。
pub const CELL_ADVANCE_EM: f32 = 1126.0 / 2048.0;
pub const LINE_HEIGHT_EM: f32 = 1.25;
pub const FONT_PX: f32 = 16.0;
