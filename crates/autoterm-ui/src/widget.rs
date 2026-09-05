//! autoterm-ui::widget — 终端网格自定义 widget(PLAN-002;承 spike
//! render-probe,度量策略:网格拟合窗口,T5 起换实测)。

use iced::advanced::layout::{Limits, Node};
use iced::advanced::text::Renderer as _;
use iced::advanced::{Renderer as _, Widget, renderer, text::LineHeight, text::Shaping, text::Wrapping, widget::Tree};
use iced::{
    Color, Element, Font, Length, Point, Rectangle, Size, Theme,
    alignment,
};

use std::sync::atomic::{AtomicU64, Ordering};

use autoterm_core::{Color as TermColor, Damage, NamedColor, StyledChar};

use crate::metrics::GridMetrics;
use crate::palette::to_iced_color;
use crate::{DEFAULT_BG, DEFAULT_FG};

/// 绘制取证(T6):fill_text 调用数,draw 结束时滚动更新。
static DRAW_RUNS_LAST: AtomicU64 = AtomicU64::new(0);
static DRAW_RUNS_PREV: AtomicU64 = AtomicU64::new(0);
/// 绘制取证(T8):光标绘制状态(row*8192+col;u64::MAX=未画)。
static CURSOR_DRAWN: AtomicU64 = AtomicU64::new(u64::MAX);

/// 读绘制 run 计数(prev, last)。
pub fn draw_runs() -> (u64, u64) {
    (
        DRAW_RUNS_PREV.load(Ordering::Relaxed),
        DRAW_RUNS_LAST.load(Ordering::Relaxed),
    )
}

/// 终端网格 widget。度量用 App 传入的实测 [`GridMetrics`]
/// (resize 与 draw 同源,右缘不裁剪由构造保证)。
///
/// `damage` 为本帧损伤语义标记(T8 光标行/未来保留式画布用);
/// iced 即时模式每帧全量重建场景,不能跳过未脏行的绘制——
/// 绘制级剪裁属 Phase 2 保留式画布(见 docs/designs/001)。
pub struct TermGrid {
    pub lines: Vec<Vec<StyledChar>>,
    pub metrics: GridMetrics,
    pub damage: Damage,
    /// 回滚偏移(0=贴底);>0 时顶行右侧画 `↑N` 指示(T7)。
    pub scroll_offset: usize,
    /// 光标(视口相对;Hidden=None)→ 反色块(T8)。
    pub cursor: Option<(usize, usize)>,
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
        // 实测度量(T5):cell_w 即 'M' 的 shaping advance,与 iced
        // 文本渲染同源;cols 由 resize 按同源度量算出,右缘天然不裁。
        let cell_px = self.metrics.cell_w;
        let line_px = self.metrics.line_h;
        let font_px = self.metrics.font_px;

        renderer.fill_quad(
            renderer::Quad { bounds, ..Default::default() },
            DEFAULT_BG,
        );

        let mut runs: u64 = 0;

        for (y, line) in self.lines.iter().enumerate() {
            let line_y = bounds.y + y as f32 * line_px;
            if line_y > bounds.y + bounds.height {
                break;
            }
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
                        size: font_px.into(),
                        line_height: LineHeight::Absolute(line_px.into()),
                        font: Font::MONOSPACE,
                        align_x: iced::Alignment::Start.into(),
                        align_y: alignment::Vertical::Top,
                        shaping: Shaping::Basic,
                        wrapping: Wrapping::None,
                    },
                    Point::new(x0, line_y),
                    to_iced_color(fg, true),
                    *viewport,
                );
                runs += 1;
            }
        }

        // 滚动 run 计数(取证:damage 门控快照 + iced 内部形状缓存
        // 承担 shaping 级缓存;emit 数恒为行内 run 总数)
        let prev = DRAW_RUNS_LAST.load(Ordering::Relaxed);
        DRAW_RUNS_PREV.store(prev, Ordering::Relaxed);
        DRAW_RUNS_LAST.store(runs, Ordering::Relaxed);
        let _ = &self.damage;

        // 回滚指示:顶行右侧 ↑N(T7)
        if self.scroll_offset > 0 {
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
                iced::advanced::text::Text {
                    content: badge,
                    bounds: Size::new(badge_w, line_px),
                    size: font_px.into(),
                    line_height: LineHeight::Absolute(line_px.into()),
                    font: Font::MONOSPACE,
                    align_x: iced::Alignment::Start.into(),
                    align_y: alignment::Vertical::Top,
                    shaping: Shaping::Basic,
                    wrapping: Wrapping::None,
                },
                Point::new(bg_bounds.x, bounds.y),
                crate::DEFAULT_FG,
                *viewport,
            );
        }

        // 光标块(T8):反色——块底=原前景色,字形=原背景色
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
                        iced::advanced::text::Text {
                            content,
                            bounds: Size::new(cell_px, line_px),
                            size: font_px.into(),
                            line_height: LineHeight::Absolute(line_px.into()),
                            font: Font::MONOSPACE,
                            align_x: iced::Alignment::Start.into(),
                            align_y: alignment::Vertical::Top,
                            shaping: Shaping::Basic,
                            wrapping: Wrapping::None,
                        },
                        rect.position(),
                        glyph_fg,
                        *viewport,
                    );
                    cursor_state = (row as u64) * 8192 + col as u64;
                }
            }
        }
        CURSOR_DRAWN.store(cursor_state, Ordering::Relaxed);
    }
}

/// 读光标绘制状态(None=未画)。
pub fn cursor_drawn() -> Option<(usize, usize)> {
    let v = CURSOR_DRAWN.load(Ordering::Relaxed);
    (v != u64::MAX).then(|| ((v / 8192) as usize, (v % 8192) as usize))
}

impl<'a, Message> From<TermGrid> for Element<'a, Message> {
    fn from(grid: TermGrid) -> Self {
        Element::new(grid)
    }
}

fn term_color_key(c: TermColor) -> u64 {
    match c {
        TermColor::Named(n) => 1 + n as u64,
        TermColor::Indexed(i) => 1_000 + i as u64,
        TermColor::Spec(_) => 2_000,
    }
}

