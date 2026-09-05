//! autoterm-core::term — `alacritty_terminal` 仿真核心封装
//!
// SPDX-License-Identifier: Apache-2.0
//! 由 PLAN-001 spike 升格为正式架构(docs/designs/001)。
//!
//! 嵌入体验第一手结论(版本 0.26.0,详见 docs/designs/000 附录):
//! - 喂入路径:`vte::ansi::Processor::advance(&mut Term, bytes)`,
//!   `Term` 自身实现 `vte::ansi::Handler`,无需自写状态机;
//! - `EventListener::send_event` 是宿主与仿真核心的唯一事件出口,
//!   其中 `Event::PtyWrite` 是 DSR/DA/OSC 查询的**应答**,必须回写
//!   PTY 主端——pwsh 启动即发 `ESC[6n`/`ESC[c` 查询,不回写就不画提示符;
//! - `renderable_content().display_iter` 按行主序给出可见区全部 Cell,
//!   自带 point 坐标,快照无需自己切行;
//! - resize 语义:`Term::resize(Dimensions)`,自实现 Dimensions 即可。

pub use alacritty_terminal;
pub use alacritty_terminal::vte::ansi::{Color, NamedColor, Rgb};

use std::sync::mpsc::{Receiver, Sender, channel};

use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::term::{Config, Term, TermDamage};
use alacritty_terminal::vte::ansi::Processor;

/// 一帧的损伤描述(owned;由 `TermDamage` 转换)。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Damage {
    Full,
    Lines(Vec<usize>),
}

/// 网格尺寸(cols × rows);实现 Dimensions 供 `Term::new`/`Term::resize`。
#[derive(Clone, Copy, Debug)]
pub struct GridSize {
    pub cols: usize,
    pub rows: usize,
}

impl Dimensions for GridSize {
    fn total_lines(&self) -> usize {
        self.rows
    }
    fn screen_lines(&self) -> usize {
        self.rows
    }
    fn columns(&self) -> usize {
        self.cols
    }
}

/// 把仿真核心抛出的事件排进 channel;宿主经 [`TermSession`] 消费。
struct ChannelListener {
    tx: Sender<Event>,
}

impl EventListener for ChannelListener {
    fn send_event(&self, event: Event) {
        let _ = self.tx.send(event);
    }
}

/// 一格的带样式快照(render-probe 用)。
#[derive(Clone, Copy, Debug)]
pub struct StyledChar {
    pub c: char,
    pub fg: Color,
    pub bg: Color,
}

/// PTY 字节流 ↔ 网格 的会话封装:feed 进、应答出、快照读、resize。
pub struct TermSession {
    term: Term<ChannelListener>,
    parser: Processor,
    events: Receiver<Event>,
    size: GridSize,
    exited: bool,
    dirty: bool,
}

impl TermSession {
    pub fn new(cols: usize, rows: usize) -> Self {
        let (tx, events) = channel();
        Self {
            term: Term::new(Config::default(), &GridSize { cols, rows }, ChannelListener { tx }),
            parser: Processor::new(),
            events,
            size: GridSize { cols, rows },
            exited: false,
            dirty: true,
        }
    }

    /// 喂一段 PTY 主端读到的字节流。
    pub fn feed(&mut self, bytes: &[u8]) {
        self.parser.advance(&mut self.term, bytes);
    }

    /// 消费积压事件:返回应写回 PTY 主端的字节(DSR/DA/OSC 应答)。
    ///
    /// 不回写时,启动即发查询的 shell(pwsh/cmd 均是)不会画提示符。
    pub fn pump(&mut self) -> Vec<u8> {
        let mut writes = Vec::new();
        while let Ok(event) = self.events.try_recv() {
            match event {
                Event::PtyWrite(s) => writes.extend_from_slice(s.as_bytes()),
                Event::Wakeup => self.dirty = true,
                Event::Exit | Event::ChildExit(_) => self.exited = true,
                _ => {}
            }
        }
        writes
    }

    /// 网格是否又变了(Wakeup)或此前从未快照过。
    pub fn take_dirty(&mut self) -> bool {
        std::mem::replace(&mut self.dirty, false) || self.exited
    }

    pub fn exited(&self) -> bool {
        self.exited
    }

    /// 通知仿真核心窗口尺寸变化。
    pub fn resize(&mut self, cols: usize, rows: usize) {
        if self.size.cols == cols && self.size.rows == rows {
            return;
        }
        self.size = GridSize { cols, rows };
        self.term.resize(self.size);
        self.dirty = true;
    }

    /// 回滚浏览,方向同 grid::Scroll::Delta 约定:**正=上翻历史,
    /// 负=下回实时**(alacritty 上游语义,UI 滚轮按此映射)。
    pub fn scroll(&mut self, delta_lines: i32) {
        use alacritty_terminal::grid::Scroll;
        self.term.scroll_display(Scroll::Delta(delta_lines));
        self.dirty = true;
    }

    /// 当前回滚偏移(0 = 贴底实时)。
    pub fn display_offset(&self) -> usize {
        self.term.grid().display_offset()
    }

    /// 回滚历史行数(已滚出视口的行数)。
    pub fn history_size(&self) -> usize {
        self.term.grid().history_size()
    }

    /// 取走当前损伤并复位(损伤重绘,PLAN-002 T6)。
    /// `Full`=整帧重画;`Lines`=脏行号集合(视口相对,含光标行)。
    pub fn take_damage(&mut self) -> Damage {
        let damage = match self.term.damage() {
            TermDamage::Full => Damage::Full,
            TermDamage::Partial(iter) => {
                Damage::Lines(iter.map(|l| l.line).collect())
            }
        };
        self.term.reset_damage();
        damage
    }

    pub fn size(&self) -> (usize, usize) {
        (self.size.cols, self.size.rows)
    }

    /// 可见区纯文本快照(每行一个 String,行尾空白已去)。
    pub fn visible_lines(&self) -> Vec<String> {
        self.visible_styled_lines()
            .iter()
            .map(|line| {
                line.iter().map(|sc| sc.c).collect::<String>().trim_end().to_string()
            })
            .collect()
    }

    /// 可见区带样式快照(前台/后台色随格携带,真彩/256 色证据来源)。
    /// 回滚时 display_iter 给出绝对网格行(历史区为负),此处映射回
    /// 视口行号:`row = line + display_offset`。
    pub fn visible_styled_lines(&self) -> Vec<Vec<StyledChar>> {
        let (cols, rows) = (self.size.cols, self.size.rows);
        let display_offset = self.term.grid().display_offset() as i32;
        let mut lines: Vec<Vec<StyledChar>> = vec![Vec::with_capacity(cols); rows];
        for indexed in self.term.renderable_content().display_iter {
            let cell = indexed.cell;
            let row = (indexed.point.line.0 + display_offset) as usize;
            let line = lines
                .get_mut(row)
                .expect("display_iter 越界:绝对行+偏移超出视口");
            line.push(StyledChar { c: cell.c, fg: cell.fg, bg: cell.bg });
        }
        lines
    }
}
