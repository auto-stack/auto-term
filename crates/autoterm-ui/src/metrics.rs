//! autoterm-ui::metrics — 字形度量与宽字符判定(PLAN-002 T5)
//!
// SPDX-License-Identifier: Apache-2.0
//! - cell_w:`'M'` 在 monospace family 下的 shaping advance 实测
//!   (cosmic-text Buffer,与 iced 渲染同一字体系统,advance 一致);
//! - line_h:1.25em 相对行距——cosmic-text 0.15 公面不暴露字体
//!   ascent/descent,行高用终端惯例相对值(偏差记录于计划 T5 证据);
//! - is_wide:按 **unicode-width**(East Asian Width 属性)判定。
//!   计划原文的"实测 advance≥1.9×cell"被实测证伪:本机 monospace
//!   字体对 '中' 的 advance 与 'M' 完全相等(9.375px),字体 advance
//!   不承载终端双格语义——alacritty 上游同样用 unicode-width。

use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics as BufferMetrics, Shaping, Wrap};

/// 网格度量(实测产物,渲染与 resize 换算共用)。
#[derive(Clone, Copy, Debug)]
pub struct GridMetrics {
    pub font_px: f32,
    pub cell_w: f32,
    pub line_h: f32,
}

struct FontMeasurer {
    font_system: FontSystem,
    buffer: Buffer,
}

static MEASURER: OnceLock<Mutex<FontMeasurer>> = OnceLock::new();

use std::sync::{Mutex, OnceLock};

impl FontMeasurer {
    fn new() -> Self {
        let mut font_system = FontSystem::new();
        let buffer_metrics = BufferMetrics::relative(crate::FONT_PX, crate::LINE_HEIGHT_EM);
        let mut buffer = Buffer::new(&mut font_system, buffer_metrics);
        // 宽度给足,禁换行:单字符 advance 即布局宽度
        buffer.set_size(&mut font_system, Some(4096.0), Some(64.0));
        buffer.set_wrap(&mut font_system, Wrap::None);
        Self { font_system, buffer }
    }

    fn advance(&mut self, c: char) -> f32 {
        let attrs = Attrs::new().family(Family::Monospace);
        self.buffer.set_text(
            &mut self.font_system,
            &c.to_string(),
            &attrs,
            Shaping::Basic,
            None,
        );
        self.buffer.shape_until_scroll(&mut self.font_system, false);
        let mut w = 0.0;
        for run in self.buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                w += glyph.w;
            }
        }
        w
    }
}

/// 启动时实测一次。
pub fn measure() -> GridMetrics {
    let mut m = FontMeasurer::new();
    let cell_w = m.advance('M');
    let _ = MEASURER.set(Mutex::new(m));
    GridMetrics {
        font_px: crate::FONT_PX,
        cell_w,
        line_h: crate::FONT_PX * crate::LINE_HEIGHT_EM,
    }
}

/// 宽字符判定(East Asian Width = Wide/Fullwidth → 双格)。
pub fn is_wide(c: char) -> bool {
    c.width() == Some(2)
}

use unicode_width::UnicodeWidthChar;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_measured_cell_width_in_expected_range() {
        let m = measure();
        assert!(m.cell_w > 0.0, "cell_w 必须实测为正");
        let em = m.cell_w / m.font_px;
        assert!(
            (0.45..=0.75).contains(&em),
            "monospace advance 应在 0.5em 量级;实际 {em:.4}em"
        );
        assert!(m.line_h > m.font_px);
    }

    #[test]
    fn wide_char_detection() {
        assert!(is_wide('中'), "CJK 应判宽");
        assert!(is_wide('あ'), "假名应判宽");
        assert!(is_wide('Ａ'), "全角应判宽");
        assert!(!is_wide('M'), "ASCII 不应判宽");
        assert!(!is_wide(' '), "空格不应判宽");
    }
}
