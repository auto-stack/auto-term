//! 仿真回归矩阵:纯 VT 字节(不经 PTY)→ 网格/事件断言。
//! (PLAN-002 T3;PLAN-001 留下的"回归种子"兑现——六类语义各至少一例)

use autoterm_core::{
    Color, Column, Damage, Line, NamedColor, Point, Rgb, SelectionType, Side, TermSession,
};

fn lines(session: &TermSession) -> Vec<String> {
    session.visible_lines()
}

#[test]
fn cursor_positioning_and_ed_el() {
    let mut s = TermSession::new(20, 5);
    // 先写满再清屏:验证 ED(2) 清空 + 光标归位 + CUP 定位
    s.feed(b"JUNK");
    s.feed(b"\x1b[2J\x1b[H");
    s.feed(b"HELLO");
    let l = lines(&s);
    assert_eq!(l[0].trim_end(), "HELLO", "清屏归位后 HELLO 应在 0 行");
    assert!(!l.iter().any(|x| x.contains("JUNK")), "ED(2) 应清屏");

    // CUP:1 基坐标 → 第 4 行第 7 列写 X
    s.feed(b"\x1b[4;7HX");
    let styled = s.visible_styled_lines();
    assert_eq!(styled[3][6].c, 'X', "CUP(4,7) 后 X 应在 [3][6]");

    // EL(2):清第 4 行整行
    s.feed(b"\x1b[4;1H\x1b[2K");
    assert_eq!(lines(&s)[3].trim_end(), "", "EL(2) 应清整行");
}

#[test]
fn sgr_truecolor_fg() {
    let mut s = TermSession::new(10, 2);
    s.feed(b"\x1b[38;2;255;0;128mA");
    let cell = &s.visible_styled_lines()[0][0];
    assert_eq!(cell.c, 'A');
    assert_eq!(
        cell.fg,
        Color::Spec(Rgb { r: 255, g: 0, b: 128 }),
        "SGR 38;2 真彩应落 Spec(Rgb)"
    );
}

#[test]
fn sgr_256color_fg() {
    let mut s = TermSession::new(10, 2);
    s.feed(b"\x1b[38;5;208mB");
    let cell = &s.visible_styled_lines()[0][0];
    assert_eq!(cell.c, 'B');
    assert_eq!(cell.fg, Color::Indexed(208), "SGR 38;5 应落 Indexed");
    // SGR 0 复位
    s.feed(b"\x1b[0mC");
    assert_eq!(
        s.visible_styled_lines()[0][2].fg,
        Color::Named(NamedColor::Foreground),
        "SGR 0 应回默认前景"
    );
}

#[test]
fn alternate_screen_1049_roundtrip() {
    let mut s = TermSession::new(20, 4);
    s.feed(b"MAIN");
    assert!(lines(&s)[0].contains("MAIN"));

    s.feed(b"\x1b[?1049h"); // 切入备用屏(保存光标+清屏)
    s.feed(b"ALT");
    let alt = lines(&s).join("\n");
    assert!(alt.contains("ALT"), "备用屏应见 ALT");
    assert!(!alt.contains("MAIN"), "备用屏不应见主屏内容");

    s.feed(b"\x1b[?1049l"); // 切回主屏
    let main = lines(&s).join("\n");
    assert!(main.contains("MAIN"), "切回后主屏内容保留");
    assert!(!main.contains("ALT"), "备用屏内容不应泄漏到主屏");
}

#[test]
fn cjk_wide_char_occupies_two_cells() {
    let mut s = TermSession::new(10, 2);
    // 中(宽字符)占两格:其后紧接的 X 应落在第 3 列(下标 2)
    s.feed("中X".as_bytes());
    let row = &s.visible_styled_lines()[0];
    assert_eq!(row[0].c, '中', "首格是宽字符");
    assert_eq!(row[2].c, 'X', "X 应跳过 spacer 落在下标 2(宽字符占两格)");
}

#[test]
fn scrollback_display_offset() {
    let mut s = TermSession::new(20, 5);
    // 10 行进 5 行视口:L6..L10 可见,其余进回滚
    let payload: String = (1..=10).map(|i| format!("L{i}\r\n")).collect();
    s.feed(payload.as_bytes());
    let view = lines(&s).join("\n");
    assert!(view.contains("L10"), "贴底应见末行;实际:\n{view}");
    assert!(
        !view.lines().any(|l| l.trim_end() == "L1"),
        "首行 L1 应已滚出视口(整行匹配,避免 L10 子串误判);实际:\n{view}"
    );

    s.scroll(i32::MAX); // 正 delta 上翻,夹到历史最上(= history_size)
    assert_eq!(
        s.display_offset(),
        s.history_size(),
        "顶到头偏移应等于历史行数"
    );
    let scrolled = lines(&s).join("\n");
    assert!(
        scrolled.lines().any(|l| l.trim_end() == "L1"),
        "回滚到顶应见首行 L1(整行匹配);实际:\n{scrolled}"
    );

    s.scroll(i32::MIN); // 负向大步回底
    assert_eq!(s.display_offset(), 0, "回底后偏移归零");
    assert!(lines(&s).join("\n").contains("L10"));
}

#[test]
fn dsr_cursor_position_report() {
    let mut s = TermSession::new(20, 5);
    // DSR(6):光标在 (1,1) 时应答 ESC[1;1R
    s.feed(b"\x1b[6n");
    let answers = s.pump();
    assert_eq!(
        answers, b"\x1b[1;1R",
        "DSR 应答应为 ESC[1;1R(应答必须回写 PTY,见 docs/designs/000)"
    );

    // 移动光标后再问:应答坐标随之变化
    s.feed(b"\x1b[3;4H\x1b[6n");
    let answers = s.pump();
    assert_eq!(answers, b"\x1b[3;4R", "光标移到 (3,4) 后应答 ESC[3;4R");
}

#[test]
fn damage_tracking_partial_and_full() {
    let mut s = TermSession::new(20, 5);
    // 首帧:全损伤(启动即覆盖全屏)
    assert_eq!(s.take_damage(), Damage::Full);

    // 单行写入:脏行为少数(光标行 ± ),非 Full
    s.feed(b"HI");
    let d = s.take_damage();
    let Damage::Lines(lines) = d else {
        panic!("单行写入应为行级损伤,实际 {d:?}");
    };
    assert!(!lines.is_empty(), "至少光标行脏");
    assert!(lines.len() < 5, "单行写入不应全屏脏;实际 {lines:?}");

    // 静默一拍:无 feed → 损伤恒含光标行(damage() 语义:Always
    // damage current cursor),除此之外无别的脏行
    assert_eq!(s.take_damage(), Damage::Lines(vec![0]));

    // 清屏(ED 2)→ 全损伤
    s.feed(b"\x1b[2J");
    assert_eq!(s.take_damage(), Damage::Full);
}

// ---- PLAN-004 T1:选中封装(Simple/Semantic/Lines)----

/// 视口行 → 绝对网格行(UI 侧换算契约:绝对行 = 视口行 - display_offset)。
fn abs_point(s: &TermSession, row: i32, col: usize) -> Point {
    Point::new(Line(row - s.display_offset() as i32), Column(col))
}

#[test]
fn selection_simple_range_and_text() {
    let mut s = TermSession::new(20, 5);
    s.feed(b"HELLO WORLD");
    // 拖选 HELLO:锚 (0,0)Left → (0,4)Right(右缘含 0..=4 列)
    s.begin_selection(SelectionType::Simple, abs_point(&s, 0, 0), Side::Left);
    s.update_selection(abs_point(&s, 0, 4), Side::Right);
    let range = s.selection_range().expect("非空 Simple 选中应有 range");
    assert_eq!((range.start.line.0, range.start.column.0), (0, 0));
    assert_eq!((range.end.line.0, range.end.column.0), (0, 4));
    assert_eq!(s.selection_text().as_deref(), Some("HELLO"));

    // 空选(同点同侧)→ range/文本均 None
    s.begin_selection(SelectionType::Simple, abs_point(&s, 0, 2), Side::Left);
    assert!(s.selection_range().is_none(), "空选 range 应为 None");

    // 清除后无任何选中
    s.clear_selection();
    assert!(s.selection_text().is_none(), "清除后无选中文本");
}

#[test]
fn selection_semantic_word_expansion() {
    let mut s = TermSession::new(20, 5);
    s.feed(b"foo bar baz");
    // 双击语义选词:点在 bar 的 'a'(列 5),无需拖动即扩到整词
    s.begin_selection(SelectionType::Semantic, abs_point(&s, 0, 5), Side::Left);
    let range = s.selection_range().expect("语义选词应有 range");
    assert_eq!(
        (range.start.column.0, range.end.column.0),
        (4, 6),
        "应扩到整词 bar(列 4..=6,空格为默认语义边界)"
    );
    assert_eq!(s.selection_text().as_deref(), Some("bar"));
}

#[test]
fn selection_lines_full_rows_with_scrollback() {
    let mut s = TermSession::new(20, 5);
    let payload: String = (1..=10).map(|i| format!("L{i}\r\n")).collect();
    s.feed(payload.as_bytes());
    s.scroll(i32::MAX); // 顶到头:视口行 0 = L1(既有 scrollback 用例语义)
    let view = lines(&s);
    assert_eq!(view[0].trim_end(), "L1", "前置:回滚到顶行 0 应是 L1");

    // 三击行选视口行 1..=2(L2/L3):换算契约经 abs_point(-display_offset)
    s.begin_selection(SelectionType::Lines, abs_point(&s, 1, 1), Side::Left);
    s.update_selection(abs_point(&s, 2, 0), Side::Left);
    let range = s.selection_range().expect("行选应有 range");
    let d = s.display_offset() as i32;
    assert_eq!(range.start.line.0, 1 - d, "range 行坐标为绝对网格行(历史区为负)");
    assert_eq!(range.end.line.0, 2 - d);
    assert_eq!(range.start.column.0, 0, "行选应扩满整行(起始列 0)");
    assert_eq!(range.end.column.0, 19, "行选应扩满整行(终止列 19=last_column)");
    assert_eq!(s.selection_text().as_deref(), Some("L2\nL3\n"), "行选文本带换行尾");
}
