//! render-probe — 路线 A 探针:iced 窗口渲染终端网格 + 键盘回写 PTY(spike)
//!
// SPDX-License-Identifier: Apache-2.0
//! spike 代码:非正式架构,允许整体重写(docs/plans/001)。
//!
//! 形状:iced application(16ms tick 驱动)→ 后台线程读 PTY → feed
//! alacritty_terminal(经 term-probe 封装)→ 每帧自定义 widget 按
//! 单元格色绘制网格;键盘可打印键/Enter/Backspace/方向键写 PTY 主端;
//! 窗口 resize 换算 cols×rows 后同时通知 ConPTY 与仿真核心。
//!
//! spike 取证参数(无人值守冒烟/粗测用,交互模式不传即可):
//!   --auto-input "echo hi\r"   延迟后自动键入(\r \n \t 转义)
//!   --auto-resize              中途程序化 resize 一次,验证不崩
//!   --exit-after <秒>          到时转储并退出
//!   --dump <file>              退出时转储网格文本+指标(供 T6/T7 证据)

use std::fmt::Write as _;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, channel};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use clap::Parser;
use iced::advanced::layout::{Limits, Node};
use iced::advanced::text::Renderer as _;
use iced::advanced::{Renderer as _, Widget, renderer, widget::Tree, text::LineHeight, text::Shaping, text::Wrapping};
use iced::keyboard::{Key, Modifiers};
use iced::{
    Color, Element, Font, Length, Point, Rectangle, Size, Subscription, Task,
    Theme, time,
};
use portable_pty::{CommandBuilder, MasterPty, PtySize};
use term_probe::{Color as TermColor, NamedColor, StyledChar, TermSession};

/// Consolas 等宽 advance 宽度(em)。spike 用固定字形度量,不做字形图集;
/// 走 Font::MONOSPACE(Windows 上解析到 Consolas)。
const CELL_ADVANCE_EM: f32 = 1126.0 / 2048.0;
const LINE_HEIGHT_EM: f32 = 1.25;
const FONT_PX: f32 = 16.0;

#[derive(Parser, Debug)]
#[command(name = "render-probe", about = "iced 终端网格渲染探针(路线 A,spike)")]
struct Flags {
    /// 要 spawn 的 shell 可执行文件(必填;ash 冒烟经此入口)
    #[arg(long)]
    shell: String,

    /// 自动键入的文本(\r \n \t 转义;多段用 --auto-input 重复传)
    #[arg(long = "auto-input")]
    auto_input: Vec<String>,

    /// 自动键入延迟(毫秒)
    #[arg(long, default_value = "1200")]
    auto_delay_ms: u64,

    /// 到时转储并退出的秒数(0 = 不自动退出)
    #[arg(long, default_value = "0")]
    exit_after: u64,

    /// 退出时转储网格与指标到该文件
    #[arg(long = "dump")]
    dump: Option<PathBuf>,

    /// 中途程序化 resize 一次,验证不崩
    #[arg(long)]
    auto_resize: bool,
}

enum Message {
    Tick,
    Key(Key, Modifiers),
    Resized(Size),
    WindowId(Option<iced::window::Id>),
}

struct Probe {
    session: TermSession,
    writer: Box<dyn Write + Send>,
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
    rx: Receiver<Vec<u8>>,

    frames: u64,
    dirty_frames: u64,
    bytes_in: u64,
    bytes_out: u64,
    started: Instant,
    first_byte_at: Option<Instant>,
    last_byte_at: Option<Instant>,
    input_sent_at: Option<Instant>,
    child_eof_at: Option<Instant>,
    resize_events: Vec<(Instant, u16, u16)>,

    queued_input: Vec<(Instant, Vec<u8>)>,
    auto_resize_at: Option<Instant>,
    exit_at: Option<Instant>,
    dump: Option<PathBuf>,
    window_id: Option<iced::window::Id>,
}

impl Probe {
    fn new(flags: &Flags) -> Result<Self> {
        let (cols, rows) = (110, 36); // 初始 1000x650 窗口的近似换算,首帧 resize 事件会校正
        let pty_system = portable_pty::native_pty_system();
        let pair = pty_system
            .openpty(PtySize { rows: rows as u16, cols: cols as u16, ..Default::default() })
            .context("openpty")?;

        let mut cmd = CommandBuilder::new(&flags.shell);
        cmd.env_remove("TERM"); // 让子进程用 ConPTY 翻译层的默认,而不是继承宿主终端
        cmd.env("TERM", "alacritty");
        let child = pair
            .slave
            .spawn_command(cmd)
            .with_context(|| format!("spawn {}", flags.shell))?;
        drop(pair.slave);

        let reader = pair.master.try_clone_reader().context("reader")?;
        let writer = pair.master.take_writer().context("writer")?;
        let master = pair.master;

        let (tx, rx) = channel::<Vec<u8>>();
        thread::spawn(move || {
            let mut reader = reader;
            let mut buf = [0u8; 16384];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => {
                        let _ = tx.send(Vec::new());
                        break;
                    }
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                }
            }
        });

        let now = Instant::now();
        let auto_delay = Duration::from_millis(flags.auto_delay_ms);
        let queued_input = flags
            .auto_input
            .iter()
            .map(|s| (now + auto_delay, unescape(s)))
            .collect();

        Ok(Self {
            session: TermSession::new(cols, rows),
            writer,
            master,
            child,
            rx,
            frames: 0,
            dirty_frames: 0,
            bytes_in: 0,
            bytes_out: 0,
            started: now,
            first_byte_at: None,
            last_byte_at: None,
            input_sent_at: None,
            child_eof_at: None,
            resize_events: Vec::new(),
            queued_input,
            auto_resize_at: flags.auto_resize.then(|| now + Duration::from_secs(2)),
            exit_at: (flags.exit_after > 0)
                .then(|| now + Duration::from_secs(flags.exit_after)),
            dump: flags.dump.clone(),
            window_id: None,
        })
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            time::every(Duration::from_millis(16)).map(|_| Message::Tick),
            iced::keyboard::listen().map(|event| match event {
                iced::keyboard::Event::KeyPressed { key, modifiers, .. } => {
                    Message::Key(key, modifiers)
                }
                _ => Message::Tick,
            }),
            iced::window::resize_events()
                .map(|(_id, size)| Message::Resized(size)),
        ])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => self.tick(),
            Message::Key(key, mods) => {
                if let Some(bytes) = key_to_bytes(&key, &mods) {
                    let _ = self.writer.write_all(&bytes);
                }
                Task::none()
            }
            Message::WindowId(id) => {
                self.window_id = id;
                Task::none()
            }
            Message::Resized(size) => {
                let cell_px = FONT_PX * CELL_ADVANCE_EM;
                let line_px = FONT_PX * LINE_HEIGHT_EM;
                let cols = ((size.width / cell_px).floor() as usize).max(10);
                let rows = ((size.height / line_px).floor() as usize).max(4);
                if let Ok(old) = self.master.get_size() {
                    if old.cols as usize == cols && old.rows as usize == rows {
                        return Task::none();
                    }
                }
                // 先通知仿真核心,再通知 ConPTY;顺序差异记入决策文档观察项。
                self.session.resize(cols, rows);
                if let Err(e) = self.master.resize(PtySize {
                    rows: rows as u16,
                    cols: cols as u16,
                    ..Default::default()
                }) {
                    eprintln!("resize(ConPTY) failed: {e}");
                }
                self.resize_events.push((Instant::now(), cols as u16, rows as u16));
                Task::none()
            }
        }
    }

    fn tick(&mut self) -> Task<Message> {
        let mut got_bytes = false;
        loop {
            match self.rx.try_recv() {
                Ok(chunk) if chunk.is_empty() => {
                    self.child_eof_at.get_or_insert_with(Instant::now);
                    break;
                }
                Ok(chunk) => {
                    self.first_byte_at.get_or_insert_with(Instant::now);
                    self.last_byte_at = Some(Instant::now());
                    self.bytes_in += chunk.len() as u64;
                    self.session.feed(&chunk);
                    got_bytes = true;
                }
                Err(_) => break,
            }
        }
        let answers = self.session.pump();
        if !answers.is_empty() {
            self.bytes_out += answers.len() as u64;
            let _ = self.writer.write_all(&answers);
        }

        let now = Instant::now();
        let mut task = Task::none();
        self.queued_input.retain(|(at, bytes)| {
            if now >= *at {
                let _ = self.writer.write_all(bytes);
                self.input_sent_at.get_or_insert_with(Instant::now);
                false
            } else {
                true
            }
        });
        if let Some(at) = self.auto_resize_at {
            if now >= at {
                self.auto_resize_at = None;
                // 1000x650 → 760x500:足以触发 ConPTY resize 路径
                if let Some(id) = self.window_id {
                    task = iced::window::resize(id, Size::new(760.0, 500.0));
                }
            }
        }
        if let Some(at) = self.exit_at {
            if now >= at {
                self.dump_state();
                let _ = self.child.kill();
                return iced::exit();
            }
        }

        self.frames += 1;
        if got_bytes || self.session.take_dirty() {
            self.dirty_frames += 1;
        }
        task
    }

    fn dump_state(&mut self) {
        let Some(path) = self.dump.clone() else { return };
        let mut out = String::new();
        let (cols, rows) = self.session.size();
        let _ = writeln!(out, "shell_grid: {cols}x{rows}");
        let _ = writeln!(
            out,
            "uptime_s: {:.3}",
            self.started.elapsed().as_secs_f64()
        );
        let _ = writeln!(out, "frames: {}", self.frames);
        let _ = writeln!(out, "dirty_frames: {}", self.dirty_frames);
        let _ = writeln!(out, "bytes_in: {}", self.bytes_in);
        let _ = writeln!(out, "bytes_out_answers: {}", self.bytes_out);
        if let (Some(f), Some(l)) = (self.first_byte_at, self.last_byte_at) {
            let _ = writeln!(
                out,
                "first_byte_after_start_s: {:.3}",
                (f - self.started).as_secs_f64()
            );
            let _ = writeln!(
                out,
                "last_byte_after_start_s: {:.3}",
                (l - self.started).as_secs_f64()
            );
            if let Some(sent) = self.input_sent_at {
                let _ = writeln!(
                    out,
                    "last_byte_after_input_s: {:.3}",
                    (l - sent).as_secs_f64()
                );
            }
        }
        if let Some(eof) = self.child_eof_at {
            let _ = writeln!(
                out,
                "child_eof_after_start_s: {:.3}",
                (eof - self.started).as_secs_f64()
            );
        }
        for (at, c, r) in &self.resize_events {
            let _ = writeln!(
                out,
                "resize_event: t={:.3}s -> {c}x{r}",
                (*at - self.started).as_secs_f64()
            );
        }
        let _ = writeln!(out, "=== grid_text_begin ===");
        for line in self.session.visible_lines() {
            let _ = writeln!(out, "{line}");
        }
        let _ = writeln!(out, "=== grid_text_end ===");

        // 颜色证据:非默认前/背景格计数 + 首个真彩样本
        let styled = self.session.visible_styled_lines();
        let mut nondefault_fg = 0usize;
        let mut nondefault_bg = 0usize;
        let mut truecolor_sample: Option<(usize, usize, TermColor, TermColor)> = None;
        for (y, line) in styled.iter().enumerate() {
            for (x, sc) in line.iter().enumerate() {
                if !matches!(sc.fg, TermColor::Named(NamedColor::Foreground)) {
                    nondefault_fg += 1;
                }
                if !matches!(sc.bg, TermColor::Named(NamedColor::Background)) {
                    nondefault_bg += 1;
                    if truecolor_sample.is_none() {
                        truecolor_sample = Some((y, x, sc.fg, sc.bg));
                    }
                }
            }
        }
        let _ = writeln!(out, "cells_nondefault_fg: {nondefault_fg}");
        let _ = writeln!(out, "cells_nondefault_bg: {nondefault_bg}");
        if let Some((y, x, fg, bg)) = truecolor_sample {
            let _ = writeln!(out, "truecolor_sample_at: row={y} col={x} fg={fg:?} bg={bg:?}");
        }

        let _ = std::fs::write(&path, out);
        eprintln!("dumped: {}", path.display());
    }

    fn view(&self) -> Element<'_, Message> {
        Element::new(TermGrid {
            lines: self.session.visible_styled_lines(),
        })
    }
}

/// 终端网格自定义 widget:spike 整帧重画(按格背景 + 按同色 run 画文本)。
struct TermGrid {
    lines: Vec<Vec<StyledChar>>,
}

impl<Message> Widget<Message, Theme, iced::Renderer> for TermGrid {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        limits: &Limits,
    ) -> Node {
        Node::new(limits.max())
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut iced::Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: iced::advanced::layout::Layout<'_>,
        _cursor: iced::mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let cell_px = FONT_PX * CELL_ADVANCE_EM;
        let line_px = FONT_PX * LINE_HEIGHT_EM;

        // 底色
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                ..Default::default()
            },
            Color::from_rgb8(0x10, 0x14, 0x18),
        );

        for (y, line) in self.lines.iter().enumerate() {
            let line_y = bounds.y + y as f32 * line_px;
            if line_y > bounds.y + bounds.height {
                break;
            }
            // 同 (fg,bg) 的连续格合成一个 run,一个 fill_text 调用
            let mut idx = 0usize;
            while idx < line.len() {
                let start = idx;
                let fg = line[idx].fg;
                let bg = line[idx].bg;
                while idx < line.len()
                    && term_color_key(line[idx].fg) == term_color_key(fg)
                    && term_color_key(line[idx].bg) == term_color_key(bg)
                {
                    idx += 1;
                }
                let content: String = line[start..idx].iter().map(|sc| sc.c).collect();
                let x0 = bounds.x + start as f32 * cell_px;
                // 非默认背景:画底色块
                if !matches!(bg, TermColor::Named(NamedColor::Background)) {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle::new(
                                Point::new(x0, line_y),
                                Size::new((idx - start) as f32 * cell_px, line_px),
                            ),
                            ..Default::default()
                        },
                        to_iced_color(bg, false),
                    );
                }
                renderer.fill_text(
                    iced::advanced::text::Text {
                        content,
                        bounds: Size::new((idx - start) as f32 * cell_px + cell_px, line_px),
                        size: FONT_PX.into(),
                        line_height: LineHeight::Absolute(line_px.into()),
                        font: Font::MONOSPACE,
                        align_x: iced::Alignment::Start.into(),
                        align_y: iced::alignment::Vertical::Top,
                        shaping: Shaping::Basic,
                        wrapping: Wrapping::None,
                    },
                    Point::new(x0, line_y),
                    to_iced_color(fg, true),
                    *viewport,
                );
            }
        }
    }
}

fn term_color_key(c: TermColor) -> u64 {
    match c {
        TermColor::Named(n) => 1 + n as u64,
        TermColor::Indexed(i) => 1_000 + i as u64,
        TermColor::Spec(_) => 2_000,
    }
}

/// vte ansi Color → iced Color。Spec 走真彩;Indexed 走 xterm 256 色表;
/// Named 走 alacritty 默认前景/背景 + 16 色标准表。
fn to_iced_color(c: TermColor, is_fg: bool) -> Color {
    const BASE16: [[u8; 3]; 16] = [
        [0x00, 0x00, 0x00], [0x80, 0x00, 0x00], [0x00, 0x80, 0x00], [0x80, 0x80, 0x00],
        [0x00, 0x00, 0x80], [0x80, 0x00, 0x80], [0x00, 0x80, 0x80], [0xc0, 0xc0, 0xc0],
        [0x80, 0x80, 0x80], [0xff, 0x00, 0x00], [0x00, 0xff, 0x00], [0xff, 0xff, 0x00],
        [0x00, 0x00, 0xff], [0xff, 0x00, 0xff], [0x00, 0xff, 0xff], [0xff, 0xff, 0xff],
    ];
    use term_probe::NamedColor as NC;
    match c {
        TermColor::Spec(rgb) => Color::from_rgb8(rgb.r, rgb.g, rgb.b),
        TermColor::Indexed(i) => {
            let [r, g, b] = xterm256(i);
            Color::from_rgb8(r, g, b)
        }
        TermColor::Named(n) => {
            let base = match n {
                NC::Foreground | NC::Background => None,
                NC::Black => Some(0), NC::Red => Some(1), NC::Green => Some(2),
                NC::Yellow => Some(3), NC::Blue => Some(4), NC::Magenta => Some(5),
                NC::Cyan => Some(6), NC::White => Some(7),
                NC::BrightBlack => Some(8), NC::BrightRed => Some(9),
                NC::BrightGreen => Some(10), NC::BrightYellow => Some(11),
                NC::BrightBlue => Some(12), NC::BrightMagenta => Some(13),
                NC::BrightCyan => Some(14), NC::BrightWhite => Some(15),
                _ => None,
            };
            match base {
                Some(i) => {
                    let [r, g, b] = BASE16[i];
                    Color::from_rgb8(r, g, b)
                }
                None => {
                    if is_fg {
                        Color::from_rgb8(0xe8, 0xe8, 0xe8)
                    } else {
                        Color::from_rgb8(0x10, 0x14, 0x18)
                    }
                }
            }
        }
    }
}

fn xterm256(i: u8) -> [u8; 3] {
    const BASE16: [[u8; 3]; 16] = [
        [0x00, 0x00, 0x00], [0x80, 0x00, 0x00], [0x00, 0x80, 0x00], [0x80, 0x80, 0x00],
        [0x00, 0x00, 0x80], [0x80, 0x00, 0x80], [0x00, 0x80, 0x80], [0xc0, 0xc0, 0xc0],
        [0x80, 0x80, 0x80], [0xff, 0x00, 0x00], [0x00, 0xff, 0x00], [0xff, 0xff, 0x00],
        [0x00, 0x00, 0xff], [0xff, 0x00, 0xff], [0x00, 0xff, 0xff], [0xff, 0xff, 0xff],
    ];
    match i {
        0..=15 => BASE16[i as usize],
        16..=231 => {
            let v = (i - 16) as usize;
            let steps = [0, 95, 135, 175, 215, 255];
            let (r, g, b) = (
                steps[v / 36],
                steps[(v % 36) / 6],
                steps[v % 6],
            );
            [r as u8, g as u8, b as u8]
        }
        _ => {
            let gray = 8 + (i - 232) as u8 * 10;
            [gray, gray, gray]
        }
    }
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

fn key_to_bytes(key: &Key, mods: &Modifiers) -> Option<Vec<u8>> {
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
                // Ctrl+字母 → 控制字节(Ctrl+C=0x03 等)
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
            iced::keyboard::key::Named::PageUp => Some(b"\x1b[5~".to_vec()),
            iced::keyboard::key::Named::PageDown => Some(b"\x1b[6~".to_vec()),
            _ => None,
        },
        _ => None,
    }
}

fn main() -> Result<()> {
    let flags = Flags::parse();

    let window = iced::window::Settings {
        size: Size::new(1000.0, 650.0),
        ..Default::default()
    };

    iced::application(
        move || {
            let probe = Probe::new(&flags).expect("render-probe init");
            (probe, iced::window::oldest().map(Message::WindowId))
        },
        Probe::update,
        Probe::view,
    )
    .title("AutoTerm render-probe")
    .window(window)
    .default_font(Font::MONOSPACE)
    .subscription(Probe::subscription)
    .run()
    .map_err(|e| anyhow::anyhow!("iced run: {e:?}"))?;
    Ok(())
}
