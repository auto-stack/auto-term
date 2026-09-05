//! autoterm-ui::widget — 终端网格自定义 widget(PLAN-003 保留式画布)
//!
// SPDX-License-Identifier: Apache-2.0
//! 绘制协议(T2/T3):
//! - 每行缓存一个 iced `Paragraph`(`with_spans`,同色 run 合并为
//!   span、前景色烘焙进 buffer——cryoglyph 逐字形 color_opt 优先);
//! - 行内容 digest(字符+前后景色)变化才重建 shaping;
//!   `Damage::Lines` 只对脏行做 digest 检查,`Full` 全量;
//! - 未脏行直接 `fill_paragraph` 复用(形状/布局缓存归我们所有,
//!   iced 场景 emit 仍全量——即时模式的绘制级剪裁至此绕开);
//! - 背景(非默认 bg)与光标反色块仍走 quad(每帧 emit,无形状成本)。

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use iced::advanced::layout::{Limits, Node};
use iced::advanced::text::Renderer as _;
use iced::advanced::text::{LineHeight, Paragraph, Shaping, Span, Text, Wrapping};
use iced::advanced::{
    Renderer as _, Widget, renderer, widget::Tree,
};
use iced::{
    Element, Font, Length, Point, Rectangle, Size, Theme,
    alignment, mouse,
};

use autoterm_core::{Color as TermColor, Damage, NamedColor, StyledChar, SelectionType, Side};

use crate::metrics::GridMetrics;
use crate::palette::to_iced_color;
use crate::{DEFAULT_BG, DEFAULT_FG, Message, SelectMsg};

type Para = <iced::Renderer as iced::advanced::text::Renderer>::Paragraph;

/// 一行的保留缓存:shaping 产物 + 内容 digest。
struct RowEntry {
    para: Para,
    digest: u64,
}

static ROW_CACHE: OnceLock<Mutex<Vec<Option<RowEntry>>>> = OnceLock::new();

fn row_cache() -> &'static Mutex<Vec<Option<RowEntry>>> {
    ROW_CACHE.get_or_init(|| Mutex::new(Vec::new()))
}

/// 取证(T3):Paragraph 重建计数(draw 结束滚动;仅 dev-tools)。
#[cfg(feature = "dev-tools")]
static REBUILDS_LAST: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "dev-tools")]
static REBUILDS_PREV: AtomicU64 = AtomicU64::new(0);

/// 读 Paragraph 重建计数(prev, last)。
#[cfg(feature = "dev-tools")]
pub fn paragraph_rebuilds() -> (u64, u64) {
    (
        REBUILDS_PREV.load(Ordering::Relaxed),
        REBUILDS_LAST.load(Ordering::Relaxed),
    )
}

/// 取证(T8):光标绘制状态(row*8192+col;u64::MAX=未画;仅 dev-tools)。
#[cfg(feature = "dev-tools")]
static CURSOR_DRAWN: AtomicU64 = AtomicU64::new(u64::MAX);

/// 读光标绘制状态(None=未画)。
#[cfg(feature = "dev-tools")]
pub fn cursor_drawn() -> Option<(usize, usize)> {
    let v = CURSOR_DRAWN.load(Ordering::Relaxed);
    (v != u64::MAX).then(|| ((v / 8192) as usize, (v % 8192) as usize))
}

/// 终端网格 widget。度量用 App 传入的实测 [`GridMetrics`]
/// (resize 与 draw 同源,右缘不裁剪由构造保证)。
pub struct TermGrid {
    pub lines: Vec<Vec<StyledChar>>,
    pub metrics: GridMetrics,
    pub damage: Damage,
    /// 回滚偏移(0=贴底);>0 时顶行右侧画 `↑N` 指示。
    pub scroll_offset: usize,
    /// 光标(视口相对;Hidden=None)→ 反色块。
    pub cursor: Option<(usize, usize)>,
}

/// 鼠标交互持久状态(经 `Tree` 跨帧存活;PLAN-004 T2)。
/// 拖选进行中标志 + 多击计数(500ms 内同格 2=Semantic、3=Lines)。
#[derive(Debug, Default)]
pub struct GridInteraction {
    dragging: bool,
    last_click_at: Option<Instant>,
    last_count: u8,
    last_cell: Option<(usize, usize)>,
}

/// 多击判定窗口。
const MULTI_CLICK_WINDOW: Duration = Duration::from_millis(500);

impl Widget<Message, Theme, iced::Renderer> for TermGrid {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<GridInteraction>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(GridInteraction::default())
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        limits: &Limits,
    ) -> Node {
        Node::new(limits.max())
    }

    /// 鼠标事件地基(PLAN-004 T2):像素→格子→publish Select 消息族。
    /// 左键按下→Begin(计数 1=Simple/2=Semantic/3=Lines);按住移动→
    /// Extend(越界 clamp 到边缘格);左键释放→Finish;右键释放→Paste。
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        layout: iced::advanced::layout::Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<GridInteraction>();
        match event {
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(pos) = cursor.position_over(bounds) else { return };
                let (cell, side) = self.pixel_to_cell(pos, bounds);
                let now = Instant::now();
                let multi = state.last_click_at.is_some_and(|t| now - t <= MULTI_CLICK_WINDOW)
                    && state.last_cell == Some(cell)
                    && state.last_count < 3;
                let count = if multi { state.last_count + 1 } else { 1 };
                let ty = match count {
                    2 => SelectionType::Semantic,
                    3 => SelectionType::Lines,
                    _ => SelectionType::Simple,
                };
                state.last_click_at = Some(now);
                state.last_count = count;
                state.last_cell = Some(cell);
                state.dragging = true;
                shell.publish(Message::Select(SelectMsg::Begin { ty, cell, side }));
                shell.capture_event();
            }
            iced::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if !state.dragging {
                    return;
                }
                // 拖选越界不动视野(自动滚动非目标),Extend clamp 到边缘格
                let Some(pos) = cursor.position() else { return };
                let (cell, side) = self.pixel_to_cell(pos, bounds);
                shell.publish(Message::Select(SelectMsg::Extend { cell, side }));
                shell.capture_event();
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if !state.dragging {
                    return;
                }
                state.dragging = false;
                shell.publish(Message::Select(SelectMsg::Finish));
                shell.capture_event();
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right)) => {
                if !cursor.is_over(bounds) {
                    return;
                }
                shell.publish(Message::Paste);
                shell.capture_event();
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: iced::advanced::layout::Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Text
        } else {
            mouse::Interaction::None
        }
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
        let cell_px = self.metrics.cell_w;
        let line_px = self.metrics.line_h;
        let font_px = self.metrics.font_px;
        let row_width =
            cell_px * self.lines.first().map_or(0.0, |l| l.len() as f32);

        renderer.fill_quad(
            renderer::Quad { bounds, ..Default::default() },
            DEFAULT_BG,
        );

        // 背景层:非默认 bg 的 run(每帧 emit;无形状成本)
        for (y, line) in self.lines.iter().enumerate() {
            let line_y = bounds.y + y as f32 * line_px;
            if line_y > bounds.y + bounds.height {
                break;
            }
            let mut idx = 0usize;
            while idx < line.len() {
                if matches!(line[idx].bg, TermColor::Named(NamedColor::Background)) {
                    idx += 1;
                    continue;
                }
                let bg = line[idx].bg;
                let start = idx;
                while idx < line.len()
                    && term_color_key(line[idx].bg) == term_color_key(bg)
                {
                    idx += 1;
                }
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle::new(
                            Point::new(
                                bounds.x + start as f32 * cell_px,
                                line_y,
                            ),
                            Size::new((idx - start) as f32 * cell_px, line_px),
                        ),
                        ..Default::default()
                    },
                    to_iced_color(bg, false),
                );
            }
        }

        // 保留式文本层:每行 Paragraph 缓存 + damage 门控重建(T2/T3)
        let dirty: Option<&[usize]> = match &self.damage {
            Damage::Full => None,
            Damage::Lines(lines) => Some(lines.as_slice()),
        };
        let mut cache = row_cache().lock().expect("row cache");
        if cache.len() != self.lines.len() {
            cache.clear();
            cache.resize_with(self.lines.len(), || None);
        }
        #[cfg(feature = "dev-tools")]
        let mut rebuilds: u64 = 0;

        for (y, line) in self.lines.iter().enumerate() {
            let line_y = bounds.y + y as f32 * line_px;
            if line_y > bounds.y + bounds.height {
                break;
            }
            let needs_check = dirty.map_or(true, |d| d.contains(&y));
            if needs_check {
                let digest = hash_row(line);
                let stale = cache[y].as_ref().is_none_or(|e| e.digest != digest);
                if stale {
                    let para =
                        build_row_paragraph(line, row_width, line_px, font_px);
                    cache[y] = Some(RowEntry { para, digest });
                    #[cfg(feature = "dev-tools")]
                    {
                        rebuilds += 1;
                    }
                }
            }
            if let Some(entry) = cache[y].as_ref() {
                renderer.fill_paragraph(
                    &entry.para,
                    Point::new(bounds.x, line_y),
                    DEFAULT_FG,
                    bounds,
                );
            }
        }

        #[cfg(feature = "dev-tools")]
        {
            let prev = REBUILDS_LAST.load(Ordering::Relaxed);
            REBUILDS_PREV.store(prev, Ordering::Relaxed);
            REBUILDS_LAST.store(rebuilds, Ordering::Relaxed);
        }

        self.draw_scroll_badge(
            renderer,
            bounds,
            cell_px,
            line_px,
            font_px,
            *viewport,
        );
        self.draw_cursor(renderer, bounds, cell_px, line_px, font_px, *viewport);
    }
}

impl TermGrid {
    /// 像素坐标 → (视口格 (row, col), 格内左右侧)。
    /// 越界(拖出边缘)clamp 到边缘格;侧界取格中点(拖选锚点语义)。
    fn pixel_to_cell(&self, pos: Point, bounds: Rectangle) -> ((usize, usize), Side) {
        let cols = self.lines.first().map_or(1, |l| l.len()).max(1);
        let rows = self.lines.len().max(1);
        let fx = (pos.x - bounds.x) / self.metrics.cell_w;
        let fy = (pos.y - bounds.y) / self.metrics.line_h;
        let col = (fx.floor() as i32).clamp(0, cols as i32 - 1) as usize;
        let row = (fy.floor() as i32).clamp(0, rows as i32 - 1) as usize;
        let side = if fx - fx.floor() >= 0.5 { Side::Right } else { Side::Left };
        ((row, col), side)
    }

    fn draw_scroll_badge(
        &self,
        renderer: &mut iced::Renderer,
        bounds: Rectangle,
        cell_px: f32,
        line_px: f32,
        font_px: f32,
        viewport: Rectangle,
    ) {
        if self.scroll_offset == 0 {
            return;
        }
        let badge = format!("↑{}", self.scroll_offset);
        let badge_w = badge.chars().count() as f32 * cell_px + cell_px;
        let bg_bounds = Rectangle::new(
            Point::new(bounds.x + bounds.width - badge_w - cell_px, bounds.y),
            Size::new(badge_w, line_px),
        );
        renderer.fill_quad(
            renderer::Quad { bounds: bg_bounds, ..Default::default() },
            DEFAULT_BG,
        );
        renderer.fill_text(
            plain_text(badge, badge_w, line_px, font_px),
            bg_bounds.position(),
            DEFAULT_FG,
            viewport,
        );
    }

    fn draw_cursor(
        &self,
        renderer: &mut iced::Renderer,
        bounds: Rectangle,
        cell_px: f32,
        line_px: f32,
        font_px: f32,
        viewport: Rectangle,
    ) {
        #[cfg(feature = "dev-tools")]
        let mut cursor_state = u64::MAX;
        if let Some((row, col)) = self.cursor {
            if let Some(line) = self.lines.get(row) {
                if let Some(cell) = line.get(col) {
                    let block_bg = to_iced_color(cell.fg, true);
                    let glyph_fg = to_iced_color(cell.bg, false);
                    let rect = Rectangle::new(
                        Point::new(
                            bounds.x + col as f32 * cell_px,
                            bounds.y + row as f32 * line_px,
                        ),
                        Size::new(cell_px, line_px),
                    );
                    renderer.fill_quad(
                        renderer::Quad { bounds: rect, ..Default::default() },
                        block_bg,
                    );
                    let mut buf = [0u8; 4];
                    let content = cell.c.encode_utf8(&mut buf).to_string();
                    renderer.fill_text(
                        plain_text(content, cell_px, line_px, font_px),
                        rect.position(),
                        glyph_fg,
                        viewport,
                    );
                    #[cfg(feature = "dev-tools")]
                    {
                        cursor_state = (row as u64) * 8192 + col as u64;
                    }
                }
            }
        }
        #[cfg(feature = "dev-tools")]
        CURSOR_DRAWN.store(cursor_state, Ordering::Relaxed);
    }
}

/// 单样式文本(↑N/光标字形等小件,仍走 fill_text)。
fn plain_text(
    content: String,
    width: f32,
    line_px: f32,
    font_px: f32,
) -> Text<String, Font> {
    Text {
        content,
        bounds: Size::new(width, line_px),
        size: font_px.into(),
        line_height: LineHeight::Absolute(line_px.into()),
        font: Font::MONOSPACE,
        align_x: iced::Alignment::Start.into(),
        align_y: alignment::Vertical::Top,
        shaping: Shaping::Basic,
        wrapping: Wrapping::None,
    }
}

/// 一行 → 单 Paragraph:同前景色 run 合并为 span,颜色烘焙进
/// buffer(cryoglyph 逐字形 color_opt 优先渲染)。
fn build_row_paragraph(
    line: &[StyledChar],
    row_width: f32,
    line_px: f32,
    font_px: f32,
) -> Para {
    // 先聚合文本与 run 边界,再从稳定的切片建 span(避免自引用借用)
    let mut text = String::with_capacity(line.len());
    let mut runs: Vec<(usize, usize, TermColor)> = Vec::new();
    let mut idx = 0usize;
    while idx < line.len() {
        let fg = line[idx].fg;
        let start = idx;
        while idx < line.len() && term_color_key(line[idx].fg) == term_color_key(fg) {
            idx += 1;
        }
        let begin = text.len();
        for sc in &line[start..idx] {
            text.push(sc.c);
        }
        runs.push((begin, text.len(), fg));
    }
    let spans: Vec<Span<'_, ()>> = runs
        .iter()
        .map(|(begin, end, fg)| Span {
            text: std::borrow::Cow::Borrowed(&text[*begin..*end]),
            color: Some(to_iced_color(*fg, true)),
            ..Default::default()
        })
        .collect();

    Para::with_spans(Text {
        content: spans.as_slice(),
        // 宽度加一格余量,避免最末字符因舍入被折行
        bounds: Size::new(row_width + cell_px_slop(), line_px),
        size: font_px.into(),
        line_height: LineHeight::Absolute(line_px.into()),
        font: Font::MONOSPACE,
        align_x: iced::Alignment::Start.into(),
        align_y: alignment::Vertical::Top,
        shaping: Shaping::Basic,
        wrapping: Wrapping::None,
    })
}

/// 段落可用宽度余量(约一格)。
fn cell_px_slop() -> f32 {
    crate::FONT_PX * crate::CELL_ADVANCE_EM
}

/// 单行 digest:字符 + 前景 + 背景(风格变化也触发重建)。
fn hash_row(line: &[StyledChar]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for sc in line {
        sc.c.hash(&mut h);
        term_color_key(sc.fg).hash(&mut h);
        term_color_key(sc.bg).hash(&mut h);
    }
    h.finish()
}

fn term_color_key(c: TermColor) -> u64 {
    match c {
        TermColor::Named(n) => 1 + n as u64,
        TermColor::Indexed(i) => 1_000 + i as u64,
        TermColor::Spec(_) => 2_000,
    }
}

impl<'a> From<TermGrid> for Element<'a, Message> {
    fn from(grid: TermGrid) -> Self {
        Element::new(grid)
    }
}
