//! autoterm-ui::widget — 终端网格自定义 widget(PLAN-002;承 spike
//! render-probe,度量策略:网格拟合窗口,T5 起换实测)。

use iced::advanced::layout::{Limits, Node};
use iced::advanced::text::Renderer as _;
use iced::advanced::{Renderer as _, Widget, renderer, text::LineHeight, text::Shaping, text::Wrapping, widget::Tree};
use iced::{
    Color, Element, Font, Length, Point, Rectangle, Size, Theme,
    alignment,
};

use autoterm_core::{Color as TermColor, NamedColor, StyledChar};

use crate::metrics::GridMetrics;
use crate::{DEFAULT_BG, DEFAULT_FG};

/// 终端网格 widget。度量用 App 传入的实测 [`GridMetrics`]
/// (resize 与 draw 同源,右缘不裁剪由构造保证)。
pub struct TermGrid {
    pub lines: Vec<Vec<StyledChar>>,
    pub metrics: GridMetrics,
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
            }
        }
    }
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

/// vte ansi Color → iced Color(过渡版:Named 16 色 + 前景/背景默认,
/// T8 换 palette.rs 全映射)。
pub fn to_iced_color(c: TermColor, is_fg: bool) -> Color {
    const BASE16: [[u8; 3]; 16] = [
        [0x00, 0x00, 0x00], [0x80, 0x00, 0x00], [0x00, 0x80, 0x00], [0x80, 0x80, 0x00],
        [0x00, 0x00, 0x80], [0x80, 0x00, 0x80], [0x00, 0x80, 0x80], [0xc0, 0xc0, 0xc0],
        [0x80, 0x80, 0x80], [0xff, 0x00, 0x00], [0x00, 0xff, 0x00], [0xff, 0xff, 0x00],
        [0x00, 0x00, 0xff], [0xff, 0x00, 0xff], [0x00, 0xff, 0xff], [0xff, 0xff, 0xff],
    ];
    match c {
        TermColor::Spec(rgb) => Color::from_rgb8(rgb.r, rgb.g, rgb.b),
        TermColor::Indexed(i) => {
            let [r, g, b] = xterm256(i);
            Color::from_rgb8(r, g, b)
        }
        TermColor::Named(n) => {
            let base = match n {
                NamedColor::Black => Some(0),
                NamedColor::Red => Some(1),
                NamedColor::Green => Some(2),
                NamedColor::Yellow => Some(3),
                NamedColor::Blue => Some(4),
                NamedColor::Magenta => Some(5),
                NamedColor::Cyan => Some(6),
                NamedColor::White => Some(7),
                NamedColor::BrightBlack => Some(8),
                NamedColor::BrightRed => Some(9),
                NamedColor::BrightGreen => Some(10),
                NamedColor::BrightYellow => Some(11),
                NamedColor::BrightBlue => Some(12),
                NamedColor::BrightMagenta => Some(13),
                NamedColor::BrightCyan => Some(14),
                NamedColor::BrightWhite => Some(15),
                _ => None,
            };
            match base {
                Some(i) => {
                    let [r, g, b] = BASE16[i];
                    Color::from_rgb8(r, g, b)
                }
                None => {
                    if is_fg { DEFAULT_FG } else { DEFAULT_BG }
                }
            }
        }
    }
}

pub fn xterm256(i: u8) -> [u8; 3] {
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
            [steps[v / 36] as u8, steps[(v % 36) / 6] as u8, steps[v % 6] as u8]
        }
        _ => {
            let gray = 8 + (i - 232) as u8 * 10;
            [gray, gray, gray]
        }
    }
}
