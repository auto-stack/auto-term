//! PLAN-018 D9(rev2):配色方案面——scheme 表引擎单源。
//!
//! 用户 2026-09-14 实机验收裁定(PLAN-017 桌面门:浅色桌面终端恒黑底):
//! 以 Windows Terminal 式可选配色方案随引擎面扩建落地。本模块是 scheme
//! 表的唯一权威源(单源);渲染端(Default/base16 解析)经 FFI 查询面
//! (`autoterm_engine_palette_color`)取色,引擎 styled 输出的 kind_color
//! 语义编码零改(016 契约)——Default/Indexed/RGB 语义穿传,解析端换表。
//!
//! - scheme 0 = `CLASSIC_DARK`:与 016 定型行为逐字节一致(def fg/bg 常量
//!   与 auto-lang widget 冻结常量同值;base16 = xterm 标准盘)——零回归。
//! - scheme 1 = `LIGHT`:浅底深字 + base16 深底变体(取 Windows Terminal
//!   "Solarized Light" 官方盘,plan rev2 授权的 solarized-light 族)。
//!
//! 纯数据无状态;per-handle 的方案选择存 `AutotermEngine`(所有权铁律:
//! scheme 属会话模型,不属渲染组件),经 `autoterm_engine_set_palette`。

/// 一个配色方案:默认前/背景 + 基础 16 色(RGB 0xRRGGBB)。
pub struct Palette {
    pub def_fg: u32,
    pub def_bg: u32,
    pub base16: [u32; 16],
}

pub const SCHEME_CLASSIC_DARK: i32 = 0;
pub const SCHEME_LIGHT: i32 = 1;
/// 内置方案数(扩展位:用户自定义 scheme 上传为后续计划非目标)。
pub const SCHEME_COUNT: i32 = 2;

/// scheme 0:现行为逐字节定型(016 色彩契约基线)。
pub const CLASSIC_DARK: Palette = Palette {
    def_fg: 0xE8E8E8,
    def_bg: 0x060709,
    base16: [
        0x000000, 0x800000, 0x008000, 0x808000, 0x000080, 0x800080, 0x008080, 0xC0C0C0, //
        0x808080, 0xFF0000, 0x00FF00, 0xFFFF00, 0x0000FF, 0xFF00FF, 0x00FFFF, 0xFFFFFF,
    ],
};

/// scheme 1:浅底深字(Solarized Light 族底,2026-09-17 用户实机裁定
/// 可读性修订:原 Windows Terminal 官方盘亮白族=近底色致 cmd 亮白
/// 文本隐身、def_fg 偏浅、def_bg 饱和刺眼——三修:
/// ①白/亮白族反转深色(7=base01、8=base00、15=base02,亮白族原为
///   FDF6E3=与 bg 同色);②亮色族 vivid 化(10/12/14 原灰调);③
/// def_fg=base02 深青灰、def_bg=base2 柔和米白(原 base3 高亮奶油)。
pub const LIGHT: Palette = Palette {
    def_fg: 0x073642,
    def_bg: 0xEEE8D5,
    base16: [
        0x002B36, 0xDC322F, 0x859900, 0xB58900, 0x268BD2, 0xD33682, 0x2AA198, 0x586E75, //
        0x657B83, 0xCB4B16, 0x859900, 0xB58900, 0x268BD2, 0x6C71C4, 0x2AA198, 0x073642,
    ],
};

/// 按方案 id 取表(未知 id → None)。
pub fn palette(scheme_id: i32) -> Option<&'static Palette> {
    match scheme_id {
        SCHEME_CLASSIC_DARK => Some(&CLASSIC_DARK),
        SCHEME_LIGHT => Some(&LIGHT),
        _ => None,
    }
}

/// 槽位取色(FFI `palette_color` 的 rust 侧实现;slot 0=def-fg、1=def-bg、
/// 2..=17=base16;`is_fg` 预留轴(0/1 之外非法),当前槽位已含 fg/bg 语义)。
/// 非法方案/槽位 → None。
pub fn palette_color(scheme_id: i32, slot: i32, is_fg: i32) -> Option<u32> {
    if is_fg != 0 && is_fg != 1 {
        return None;
    }
    let pal = palette(scheme_id)?;
    match slot {
        0 => Some(pal.def_fg),
        1 => Some(pal.def_bg),
        2..=17 => Some(pal.base16[(slot - 2) as usize]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classic_dark_matches_frozen_render_constants() {
        let pal = palette(SCHEME_CLASSIC_DARK).unwrap();
        assert_eq!(pal.def_fg, 0xE8E8E8);
        assert_eq!(pal.def_bg, 0x060709);
        assert_eq!(pal.base16[0], 0x000000);
        assert_eq!(pal.base16[9], 0xFF0000);
        assert_eq!(pal.base16[15], 0xFFFFFF);
    }

    #[test]
    fn light_scheme_is_light_bg_dark_fg() {
        let pal = palette(SCHEME_LIGHT).unwrap();
        // 浅底:亮度明显高于深底基线 0x060709。
        let lum = |c: u32| {
            let (r, g, b) = (c >> 16, (c >> 8) & 0xFF, c & 0xFF);
            0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32
        };
        assert!(lum(pal.def_bg) > 200.0, "light def_bg 应为浅色: {:06X}", pal.def_bg);
        assert!(lum(pal.def_fg) < 150.0, "light def_fg 应为深字: {:06X}", pal.def_fg);
    }

    #[test]
    fn palette_color_slot_axes_and_invalid() {
        assert_eq!(palette_color(0, 0, 1), Some(0xE8E8E8));
        assert_eq!(palette_color(0, 1, 0), Some(0x060709));
        assert_eq!(palette_color(1, 0, 1), Some(0x073642));
        assert_eq!(palette_color(1, 1, 0), Some(0xEEE8D5));
        assert_eq!(palette_color(0, 2, 0), Some(0x000000));
        assert_eq!(palette_color(1, 17, 0), Some(0x073642));
        assert_eq!(palette_color(99, 0, 1), None, "未知方案");
        assert_eq!(palette_color(0, 18, 0), None, "槽位越界");
        assert_eq!(palette_color(0, 0, 2), None, "is_fg 非法轴");
    }

    #[test]
    fn light_scheme_white_family_readable_on_bg() {
        // 2026-09-17 可读性修订护栏:白/亮白族在浅底上不得隐身——
        // 原 WT Solarized Light 官方盘 7=EEE8D5(近底)/15=FDF6E3(=bg
        // 同色),cmd 亮白文本(文件名族)整列不可见(用户实机反馈)。
        let pal = palette(SCHEME_LIGHT).unwrap();
        let bg = pal.def_bg;
        let lum = |c: u32| {
            let (r, g, b) = (c >> 16, (c >> 8) & 0xFF, c & 0xFF);
            0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32
        };
        for slot in [7usize, 8, 15] {
            let d = (lum(pal.base16[slot]) - lum(bg)).abs();
            assert!(d > 60.0, "亮白族槽 {} 与 bg 亮度差不足: {:06X}", slot, pal.base16[slot]);
        }
        assert_ne!(pal.base16[15], bg, "亮白不得与 bg 同色");
    }
}
