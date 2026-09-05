//! autoterm-ui::palette — vte ansi Color → iced Color 全映射
//! (PLAN-002 T8:NamedColor 全枚举,含 Dim/Bright 前景)。

use iced::Color;

use autoterm_core::{Color as TermColor, NamedColor};

use crate::{DEFAULT_BG, DEFAULT_FG};

const BASE16: [[u8; 3]; 16] = [
    [0x00, 0x00, 0x00], [0x80, 0x00, 0x00], [0x00, 0x80, 0x00], [0x80, 0x80, 0x00],
    [0x00, 0x00, 0x80], [0x80, 0x00, 0x80], [0x00, 0x80, 0x80], [0xc0, 0xc0, 0xc0],
    [0x80, 0x80, 0x80], [0xff, 0x00, 0x00], [0x00, 0xff, 0x00], [0xff, 0xff, 0x00],
    [0x00, 0x00, 0xff], [0xff, 0x00, 0xff], [0x00, 0xff, 0xff], [0xff, 0xff, 0xff],
];

fn rgb([r, g, b]: [u8; 3]) -> Color {
    Color::from_rgb8(r, g, b)
}

/// 暗色(Dim 变体):亮度×0.6。
fn dim(c: [u8; 3]) -> Color {
    let f = |v: u8| (v as f32 * 0.6) as u8;
    Color::from_rgb8(f(c[0]), f(c[1]), f(c[2]))
}

/// vte ansi Color → iced Color。
///
/// `is_fg` 决定 Foreground/Background 命中默认前景/背景;Cursor/
/// 选区等特殊名落到默认前景(不参与 MVP 渲染)。
pub fn to_iced_color(c: TermColor, is_fg: bool) -> Color {
    match c {
        TermColor::Spec(rgbv) => Color::from_rgb8(rgbv.r, rgbv.g, rgbv.b),
        TermColor::Indexed(i) => rgb(xterm256(i)),
        TermColor::Named(n) => match n {
            NamedColor::Black => rgb(BASE16[0]),
            NamedColor::Red => rgb(BASE16[1]),
            NamedColor::Green => rgb(BASE16[2]),
            NamedColor::Yellow => rgb(BASE16[3]),
            NamedColor::Blue => rgb(BASE16[4]),
            NamedColor::Magenta => rgb(BASE16[5]),
            NamedColor::Cyan => rgb(BASE16[6]),
            NamedColor::White => rgb(BASE16[7]),
            NamedColor::BrightBlack => rgb(BASE16[8]),
            NamedColor::BrightRed => rgb(BASE16[9]),
            NamedColor::BrightGreen => rgb(BASE16[10]),
            NamedColor::BrightYellow => rgb(BASE16[11]),
            NamedColor::BrightBlue => rgb(BASE16[12]),
            NamedColor::BrightMagenta => rgb(BASE16[13]),
            NamedColor::BrightCyan => rgb(BASE16[14]),
            NamedColor::BrightWhite => rgb(BASE16[15]),
            NamedColor::DimBlack => dim(BASE16[0]),
            NamedColor::DimRed => dim(BASE16[1]),
            NamedColor::DimGreen => dim(BASE16[2]),
            NamedColor::DimYellow => dim(BASE16[3]),
            NamedColor::DimBlue => dim(BASE16[4]),
            NamedColor::DimMagenta => dim(BASE16[5]),
            NamedColor::DimCyan => dim(BASE16[6]),
            NamedColor::DimWhite => dim(BASE16[7]),
            NamedColor::BrightForeground => Color::from_rgb8(0xff, 0xff, 0xff),
            NamedColor::DimForeground => Color::from_rgb8(0xa8, 0xa8, 0xa8),
            NamedColor::Foreground | NamedColor::Background | NamedColor::Cursor => {
                if is_fg { DEFAULT_FG } else { DEFAULT_BG }
            }
            _ => {
                if is_fg { DEFAULT_FG } else { DEFAULT_BG }
            }
        },
    }
}

/// xterm 256 色表(0-15 标准色,16-231 6×6×6 立方,232-255 灰阶)。
pub fn xterm256(i: u8) -> [u8; 3] {
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

#[cfg(test)]
mod tests {
    use super::*;
    use autoterm_core::Rgb;

    #[test]
    fn named_full_mapping() {
        // 16 基色 + Dim + Bright/Dim 前景各取一验,默认前景/背景方向性
        assert_eq!(to_iced_color(TermColor::Named(NamedColor::Red), true), rgb(BASE16[1]));
        assert_eq!(to_iced_color(TermColor::Named(NamedColor::BrightWhite), false), rgb(BASE16[15]));
        assert_eq!(to_iced_color(TermColor::Named(NamedColor::DimRed), true), dim(BASE16[1]));
        assert_eq!(to_iced_color(TermColor::Named(NamedColor::Foreground), true), DEFAULT_FG);
        assert_eq!(to_iced_color(TermColor::Named(NamedColor::Background), false), DEFAULT_BG);
        // Dim/普通可区分
        assert_ne!(
            to_iced_color(TermColor::Named(NamedColor::DimRed), true),
            to_iced_color(TermColor::Named(NamedColor::Red), true)
        );
    }

    #[test]
    fn truecolor_and_indexed() {
        assert_eq!(
            to_iced_color(TermColor::Spec(Rgb { r: 1, g: 2, b: 3 }), true),
            Color::from_rgb8(1, 2, 3)
        );
        assert_eq!(to_iced_color(TermColor::Indexed(208), true), rgb(xterm256(208)));
        assert_eq!(xterm256(16), [0, 0, 0]);
        assert_eq!(xterm256(231), [255, 255, 255]);
        assert_eq!(xterm256(255), [238, 238, 238]);
    }
}
