//! PLAN-014 T-06:TermSession::resize 护栏单测——退化几何拒绝与大幅
//! 收缩先裁历史的行为钉子(19:13 案根因,DEBTS #15 嵌入契约)。
//! 防未来重构移除护栏时静默复活:护栏在,这里绿;护栏丢,这里红。

use autoterm_core::term::TermSession;

/// 填充滚动历史:喂 N 行短行(不换行折叠,便于精确断言),使视口滚入历史区。
fn fill_history(s: &mut TermSession, lines: usize) {
    for i in 0..lines {
        let chunk = format!("line {i:04}\r\n");
        s.feed(chunk.as_bytes());
    }
}

#[test]
fn degenerate_resize_is_rejected_inertly() {
    let mut s = TermSession::new(80, 24);
    fill_history(&mut s, 50);
    let before = s.history_size();
    assert!(before > 0, "前置:历史已填充({before})");

    // 最小化/零尺寸产物(1x1、0x0、单列)一律拒绝:最小化是可见性
    // 事件,不是几何变化;拒绝必须惰性——不裁历史、不进重排。
    s.resize(1, 1);
    assert_eq!(s.size(), (80, 24), "1x1 必须被拒绝");
    s.resize(0, 0);
    assert_eq!(s.size(), (80, 24), "0x0 必须被拒绝");
    s.resize(1, 24);
    assert_eq!(s.size(), (80, 24), "cols<2 必须被拒绝");
    s.resize(80, 0);
    assert_eq!(s.size(), (80, 24), "rows<1 必须被拒绝");
    assert_eq!(
        s.history_size(),
        before,
        "拒绝必须惰性:历史原样保留,重排从未发生"
    );
}

#[test]
fn major_shrink_clears_history_first() {
    let mut s = TermSession::new(80, 24);
    fill_history(&mut s, 50);
    assert!(s.history_size() > 0);

    // 80 → 40(列数减半及以上):先裁历史拆除重排燃料(短行不发生
    // 折行,清空后历史应恰为 0)。
    s.resize(40, 24);
    assert_eq!(s.size(), (40, 24));
    assert_eq!(s.history_size(), 0, "大幅收缩必须先清空滚动历史");

    // 温和收缩(80 → 50,因子<2)不裁:历史保留。
    let mut s2 = TermSession::new(80, 24);
    fill_history(&mut s2, 50);
    s2.resize(50, 24);
    assert_eq!(s2.size(), (50, 24));
    assert!(s2.history_size() > 0, "温和收缩保留历史");
}

#[test]
fn same_size_resize_is_noop() {
    let mut s = TermSession::new(80, 24);
    fill_history(&mut s, 50);
    s.resize(80, 24);
    assert_eq!(s.size(), (80, 24));
    assert!(s.history_size() > 0, "同尺寸 no-op:历史不动");
}
