//! T1 PoC — iced Paragraph 公面可行性(PLAN-003)
//!
// SPDX-License-Identifier: Apache-2.0
//! 验证保留式画布的前提:`Plain<P>`(iced 内建的 content 比对 +
//! compare/resize/rebuild 包装)可在 widget 外公面构造、shaping 产生
//! 非零尺寸、update 语义符合脏行剪裁需求。
//! 底层经 iced_graphics 全局字体系统(OnceLock 惰性初始化),
//! headless 测试无需跑起 iced application。

use iced::advanced::text::paragraph::Plain;
use iced::advanced::text::{LineHeight, Paragraph, Shaping, Wrapping};
use iced::advanced::text::Text;
use iced::{Font, Pixels, Size, alignment};

type RowPara = Plain<<iced::Renderer as iced::advanced::text::Renderer>::Paragraph>;

fn owned(content: &str, width: f32) -> Text<String, Font> {
    Text {
        content: content.to_string(),
        bounds: Size::new(width, 20.0),
        size: Pixels(16.0),
        line_height: LineHeight::Absolute(Pixels(20.0)),
        font: Font::MONOSPACE,
        align_x: iced::Alignment::Start.into(),
        align_y: alignment::Vertical::Top,
        shaping: Shaping::Basic,
        wrapping: Wrapping::None,
    }
}

fn borrowed<'a>(content: &'a str, width: f32) -> Text<&'a str, Font> {
    Text {
        content,
        bounds: Size::new(width, 20.0),
        size: Pixels(16.0),
        line_height: LineHeight::Absolute(Pixels(20.0)),
        font: Font::MONOSPACE,
        align_x: iced::Alignment::Start.into(),
        align_y: alignment::Vertical::Top,
        shaping: Shaping::Basic,
        wrapping: Wrapping::None,
    }
}

#[test]
fn paragraph_public_construction_and_shaping() {
    let p = RowPara::new(owned("HELLO", 500.0));
    let min = p.min_bounds();
    assert!(min.width > 0.0, "shaping 应产生非零宽度");
    assert!(min.height > 0.0, "shaping 应产生非零高度");
    assert!(min.width < 500.0, "min 应小于可用宽度");
}

#[test]
fn paragraph_update_semantics_for_damage_gating() {
    let mut p = RowPara::new(owned("SAME", 500.0));

    // 同内容同参 → false(不重建,fill_paragraph 复用)
    assert!(!p.update(borrowed("SAME", 500.0)), "同文本不应重建");

    // 内容变 → true(重建)
    assert!(p.update(borrowed("OTHER", 500.0)), "文本变化应重建");
    assert_eq!(p.content(), "OTHER");

    // 内容同、bounds 变 → true(Bounds 差异,内部走 resize 不重排)
    assert!(p.update(borrowed("OTHER", 600.0)), "bounds 变化应触发更新");

    // 尺寸随内容单调(5 字符宽于 1 字符)
    let mut a = RowPara::new(owned("X", 500.0));
    a.update(borrowed("XXXXX", 500.0));
    let wide = a.min_bounds().width;
    let mut b = RowPara::new(owned("X", 500.0));
    b.update(borrowed("X", 500.0));
    let narrow = b.min_bounds().width;
    assert!(wide > narrow, "5 字符应宽于 1 字符({wide} vs {narrow})");
}

#[test]
fn paragraph_reuse_avoids_reshaping_cost_pattern() {
    // 保留式画布的核心收益模拟:同内容反复 update 不重建
    let mut p = RowPara::new(owned("PS C:\\Users> echo hi", 900.0));
    for _ in 0..100 {
        assert!(!p.update(borrowed("PS C:\\Users> echo hi", 900.0)));
    }
    // raw() 可交给 fill_paragraph 绘制(类型对上即可)
    let _raw: &<iced::Renderer as iced::advanced::text::Renderer>::Paragraph = p.raw();
}
