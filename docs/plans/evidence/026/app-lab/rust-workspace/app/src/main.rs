// Auto-generated from Auto language by a2rust-ui

use auto_lang::ui::{Component, View};

// 014 内存哨兵:记账分配器(存活字节 + 大块分配点回溯;超限冻结时
// 自动落盘分配报告到 %TEMP%/auto-term-mem-report.txt)。
#[global_allocator]
static GUARD_ALLOC: auto_lang::ui::mem_guard::GuardAlloc = auto_lang::ui::mem_guard::GuardAlloc;

// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Debug, PartialEq)]
pub enum AppMsg {
    Init,
    Tick,
    KeyIn,
    Menu,
    TabActivate(i32),
    TabClose(i32),
    NewTab,
    NewTabProfile(i32),
    ProfileMenu,
    Split(i32),
    Zoom,
    CloseFocusPane,
    SchemeToggle,
    Shortcut(i32),
    Press(i32),
    Drag(f32, f32),
    Drop,
}

#[derive(Debug)]
pub struct App {
    pub cols: i32,
    pub rows: i32,
    pub tab_ids: Vec<i32>,
    pub tab_labels: Vec<String>,
    pub scheme: i32,
    pub clientW: i32,
    pub clientH: i32,
    pub tabH: i32,
    pub statusH: i32,
    pub slot1: i32,
    pub slot2: i32,
    pub slot3: i32,
    pub slot4: i32,
    pub slot5: i32,
    pub slot6: i32,
    pub key1: String,
    pub key2: String,
    pub key3: String,
    pub key4: String,
    pub key5: String,
    pub key6: String,
    pub x1: i32,
    pub x2: i32,
    pub x3: i32,
    pub x4: i32,
    pub x5: i32,
    pub x6: i32,
    pub y1: i32,
    pub y2: i32,
    pub y3: i32,
    pub y4: i32,
    pub y5: i32,
    pub y6: i32,
    pub w1: i32,
    pub w2: i32,
    pub w3: i32,
    pub w4: i32,
    pub w5: i32,
    pub w6: i32,
    pub h1: i32,
    pub h2: i32,
    pub h3: i32,
    pub h4: i32,
    pub h5: i32,
    pub h6: i32,
    pub lines1: Vec<String>,
    pub lines2: Vec<String>,
    pub lines3: Vec<String>,
    pub lines4: Vec<String>,
    pub lines5: Vec<String>,
    pub lines6: Vec<String>,
    pub g1c: i32,
    pub g1r: i32,
    pub g2c: i32,
    pub g2r: i32,
    pub g3c: i32,
    pub g3r: i32,
    pub g4c: i32,
    pub g4r: i32,
    pub g5c: i32,
    pub g5r: i32,
    pub g6c: i32,
    pub g6r: i32,
    pub cr1: i32,
    pub cc1: i32,
    pub cr2: i32,
    pub cc2: i32,
    pub cr3: i32,
    pub cc3: i32,
    pub cr4: i32,
    pub cc4: i32,
    pub cr5: i32,
    pub cc5: i32,
    pub cr6: i32,
    pub cc6: i32,
    pub dv1: i32,
    pub dv2: i32,
    pub dv3: i32,
    pub dv4: i32,
    pub dv5: i32,
    pub dA1: i32,
    pub dA2: i32,
    pub dA3: i32,
    pub dA4: i32,
    pub dA5: i32,
    pub dX1: i32,
    pub dX2: i32,
    pub dX3: i32,
    pub dX4: i32,
    pub dX5: i32,
    pub dY1: i32,
    pub dY2: i32,
    pub dY3: i32,
    pub dY4: i32,
    pub dY5: i32,
    pub dW1: i32,
    pub dW2: i32,
    pub dW3: i32,
    pub dW4: i32,
    pub dW5: i32,
    pub dH1: i32,
    pub dH2: i32,
    pub dH3: i32,
    pub dH4: i32,
    pub dH5: i32,
    pub dragging: i32,
    pub dragBranch: i32,
    pub dragW: i32,
    pub dragH: i32,
    pub lastVer: i32,
    pub profile_names: Vec<String>,
    pub profile_menu: i32,
}

impl App {
    pub fn new() -> Self {
        let mut __self = Self {
            cols: 100,
            rows: 30,
            tab_ids: vec![],
            tab_labels: vec![],
            scheme: 0,
            clientW: 802,
            clientH: 450,
            tabH: 40,
            statusH: 24,
            slot1: 0,
            slot2: 0,
            slot3: 0,
            slot4: 0,
            slot5: 0,
            slot6: 0,
            key1: "pane-1".to_string(),
            key2: "".to_string(),
            key3: "".to_string(),
            key4: "".to_string(),
            key5: "".to_string(),
            key6: "".to_string(),
            x1: 0,
            x2: 0,
            x3: 0,
            x4: 0,
            x5: 0,
            x6: 0,
            y1: 0,
            y2: 0,
            y3: 0,
            y4: 0,
            y5: 0,
            y6: 0,
            w1: 802,
            w2: 0,
            w3: 0,
            w4: 0,
            w5: 0,
            w6: 0,
            h1: 450,
            h2: 0,
            h3: 0,
            h4: 0,
            h5: 0,
            h6: 0,
            lines1: vec![],
            lines2: vec![],
            lines3: vec![],
            lines4: vec![],
            lines5: vec![],
            lines6: vec![],
            g1c: 100,
            g1r: 30,
            g2c: 100,
            g2r: 30,
            g3c: 100,
            g3r: 30,
            g4c: 100,
            g4r: 30,
            g5c: 100,
            g5r: 30,
            g6c: 100,
            g6r: 30,
            cr1: 0,
            cc1: 0,
            cr2: 0,
            cc2: 0,
            cr3: 0,
            cc3: 0,
            cr4: 0,
            cc4: 0,
            cr5: 0,
            cc5: 0,
            cr6: 0,
            cc6: 0,
            dv1: 0,
            dv2: 0,
            dv3: 0,
            dv4: 0,
            dv5: 0,
            dA1: 0,
            dA2: 0,
            dA3: 0,
            dA4: 0,
            dA5: 0,
            dX1: 0,
            dX2: 0,
            dX3: 0,
            dX4: 0,
            dX5: 0,
            dY1: 0,
            dY2: 0,
            dY3: 0,
            dY4: 0,
            dY5: 0,
            dW1: 0,
            dW2: 0,
            dW3: 0,
            dW4: 0,
            dW5: 0,
            dH1: 0,
            dH2: 0,
            dH3: 0,
            dH4: 0,
            dH5: 0,
            dragging: 0,
            dragBranch: 0,
            dragW: 0,
            dragH: 0,
            lastVer: 0 - 1,
            profile_names: vec![],
            profile_menu: 0,
        };
        __self.on(AppMsg::Init);
        __self
    }
}
impl Default for App {
    fn default() -> Self { Self::new() }
}

impl Component for App {
    type Msg = AppMsg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            AppMsg::CloseFocusPane => {
                let mut fid = mux_focus_id();
                mux_enqueue(3, fid);
            }
            AppMsg::Drag(x, y) => {
                mux_resize_branch(self.dragBranch, (x as i32), (y as i32));
            }
            AppMsg::Drop => {
                self.dragging = 0;
                self.dragBranch = 0;
                self.dragW = 0;
                self.dragH = 0;
            }
            AppMsg::KeyIn => {
                term_pump_input();
                let mut p1 = mux_rect_pane(1);
                self.lines1 = mux_pane_lines(p1);
                self.cr1 = mux_pane_cursor_row(p1);
                self.cc1 = mux_pane_cursor_col(p1);
                let mut p2 = mux_rect_pane(2);
                self.lines2 = mux_pane_lines(p2);
                self.cr2 = mux_pane_cursor_row(p2);
                self.cc2 = mux_pane_cursor_col(p2);
                let mut p3 = mux_rect_pane(3);
                self.lines3 = mux_pane_lines(p3);
                self.cr3 = mux_pane_cursor_row(p3);
                self.cc3 = mux_pane_cursor_col(p3);
                let mut p4 = mux_rect_pane(4);
                self.lines4 = mux_pane_lines(p4);
                self.cr4 = mux_pane_cursor_row(p4);
                self.cc4 = mux_pane_cursor_col(p4);
                let mut p5 = mux_rect_pane(5);
                self.lines5 = mux_pane_lines(p5);
                self.cr5 = mux_pane_cursor_row(p5);
                self.cc5 = mux_pane_cursor_col(p5);
                let mut p6 = mux_rect_pane(6);
                self.lines6 = mux_pane_lines(p6);
                self.cr6 = mux_pane_cursor_row(p6);
                self.cc6 = mux_pane_cursor_col(p6);
            }
            AppMsg::Menu => {
                let mut item = term_menu_take();
                if item == 3 { term_interrupt(); };
            }
            AppMsg::NewTab => {
                mux_enqueue(1, 0);
            }
            AppMsg::NewTabProfile(i) => {
                mux_enqueue(1, i + 1);
                self.profile_menu = 0;
            }
            AppMsg::Press(k) => {
                let mut b = mux_rect_branch(k);
                self.dragBranch = b;
                self.dragging = 1;
                self.dragW = self.clientW;
                self.dragH = self.clientH - self.tabH - self.statusH;
            }
            AppMsg::ProfileMenu => {
                if self.profile_menu == 1 { self.profile_menu = 0; } else { self.profile_menu = 1; };
            }
            AppMsg::SchemeToggle => {
                if self.scheme == 1 { self.scheme = 0; } else { self.scheme = 1; };
            }
            AppMsg::Shortcut(n) => {
                if n == 1 { mux_enqueue(1, 0); };
                if n == 2 { let mut fid = mux_focus_id(); mux_enqueue(3, fid); };
                if n == 3 { mux_enqueue(2, 1); };
                if n == 4 { mux_enqueue(2, 0); };
                if n == 5 { let mut fid = mux_focus_id(); mux_enqueue(4, fid); };
                if n == 6 { let mut cnt = mux_tab_count(); let mut cur = 0; let mut i = 0; loop { if i >= cnt { break; }; if mux_tab_is_active_at(i) == 1 { cur = i; }; i = i + 1; }; let mut prv = cur - 1; if prv < 0 { prv = cnt - 1; }; let mut ptid = mux_tab_id_at(prv); mux_enqueue(5, ptid); };
                if n == 7 { let mut cnt2 = mux_tab_count(); let mut cur2 = 0; let mut i2 = 0; loop { if i2 >= cnt2 { break; }; if mux_tab_is_active_at(i2) == 1 { cur2 = i2; }; i2 = i2 + 1; }; let mut prv2 = cur2 - 1; if prv2 < 0 { prv2 = cnt2 - 1; }; let mut ptid2 = mux_tab_id_at(prv2); mux_enqueue(5, ptid2); };
                if n == 8 { let mut cnt3 = mux_tab_count(); let mut cur3 = 0; let mut i3 = 0; loop { if i3 >= cnt3 { break; }; if mux_tab_is_active_at(i3) == 1 { cur3 = i3; }; i3 = i3 + 1; }; let mut nxt = cur3 + 1; if nxt >= cnt3 { nxt = 0; }; let mut ntid = mux_tab_id_at(nxt); mux_enqueue(5, ntid); };
                if n == 9 { if self.scheme == 1 { self.scheme = 0; } else { self.scheme = 1; }; };
                if n == 10 { mux_focus_dir(0); };
                if n == 11 { mux_focus_dir(1); };
                if n == 12 { mux_focus_dir(2); };
                if n == 13 { mux_focus_dir(3); };
            }
            AppMsg::Split(ax) => {
                mux_enqueue(2, ax);
            }
            AppMsg::TabActivate(i) => {
                let mut tid = mux_tab_id_at(i);
                mux_enqueue(5, tid);
            }
            AppMsg::TabClose(i) => {
                let mut tid = mux_tab_id_at(i);
                mux_enqueue(6, tid);
            }
            AppMsg::Tick => {
                term_apply_resize();
                get_lines();
                let mut n = mux_tab_count();
                let mut ids = vec![];
                let mut labels = vec![];
                let mut i = 0;
                loop { if i >= n { break; }; ids.push(mux_tab_id_at(i)); let mut lbl = mux_tab_title_at(i); if mux_tab_is_active_at(i) == 1 { lbl = format!("{}{}", "● ", lbl); }; labels.push(lbl.clone()); i = i + 1; };
                self.tab_ids = ids;
                self.tab_labels = labels;
                let mut ver = mux_layout_version();
                let mut nw = mux_window_width();
                let mut nh = mux_window_height();
                let mut cw = self.clientW;
                let mut ch = self.clientH - self.tabH - self.statusH;
                let mut geomChanged = 0;
                if ver != self.lastVer { geomChanged = 1; };
                if nw != self.clientW { geomChanged = 1; };
                if nh != self.clientH { geomChanged = 1; };
                if geomChanged == 1 { self.clientW = nw; self.clientH = nh; cw = nw; ch = nh - self.tabH; };
                if geomChanged == 1 { self.slot1 = mux_rect_kind(1); let mut p1 = mux_rect_pane(1); self.key1 = mux_rect_key(1).to_string(); self.x1 = mux_rect_x(1) * cw / 1000; self.y1 = mux_rect_y(1) * ch / 1000; self.w1 = mux_rect_w(1) * cw / 1000; self.h1 = mux_rect_h(1) * ch / 1000; self.g1c = mux_pane_cols(p1); self.g1r = mux_pane_rows(p1); self.lines1 = mux_pane_lines(p1); self.cr1 = mux_pane_cursor_row(p1); self.cc1 = mux_pane_cursor_col(p1); self.slot2 = mux_rect_kind(2); let mut p2 = mux_rect_pane(2); self.key2 = mux_rect_key(2).to_string(); self.x2 = mux_rect_x(2) * cw / 1000; self.y2 = mux_rect_y(2) * ch / 1000; self.w2 = mux_rect_w(2) * cw / 1000; self.h2 = mux_rect_h(2) * ch / 1000; self.g2c = mux_pane_cols(p2); self.g2r = mux_pane_rows(p2); self.lines2 = mux_pane_lines(p2); self.cr2 = mux_pane_cursor_row(p2); self.cc2 = mux_pane_cursor_col(p2); self.slot3 = mux_rect_kind(3); let mut p3 = mux_rect_pane(3); self.key3 = mux_rect_key(3).to_string(); self.x3 = mux_rect_x(3) * cw / 1000; self.y3 = mux_rect_y(3) * ch / 1000; self.w3 = mux_rect_w(3) * cw / 1000; self.h3 = mux_rect_h(3) * ch / 1000; self.g3c = mux_pane_cols(p3); self.g3r = mux_pane_rows(p3); self.lines3 = mux_pane_lines(p3); self.cr3 = mux_pane_cursor_row(p3); self.cc3 = mux_pane_cursor_col(p3); self.slot4 = mux_rect_kind(4); let mut p4 = mux_rect_pane(4); self.key4 = mux_rect_key(4).to_string(); self.x4 = mux_rect_x(4) * cw / 1000; self.y4 = mux_rect_y(4) * ch / 1000; self.w4 = mux_rect_w(4) * cw / 1000; self.h4 = mux_rect_h(4) * ch / 1000; self.g4c = mux_pane_cols(p4); self.g4r = mux_pane_rows(p4); self.lines4 = mux_pane_lines(p4); self.cr4 = mux_pane_cursor_row(p4); self.cc4 = mux_pane_cursor_col(p4); self.slot5 = mux_rect_kind(5); let mut p5 = mux_rect_pane(5); self.key5 = mux_rect_key(5).to_string(); self.x5 = mux_rect_x(5) * cw / 1000; self.y5 = mux_rect_y(5) * ch / 1000; self.w5 = mux_rect_w(5) * cw / 1000; self.h5 = mux_rect_h(5) * ch / 1000; self.g5c = mux_pane_cols(p5); self.g5r = mux_pane_rows(p5); self.lines5 = mux_pane_lines(p5); self.cr5 = mux_pane_cursor_row(p5); self.cc5 = mux_pane_cursor_col(p5); self.slot6 = mux_rect_kind(6); let mut p6 = mux_rect_pane(6); self.key6 = mux_rect_key(6).to_string(); self.x6 = mux_rect_x(6) * cw / 1000; self.y6 = mux_rect_y(6) * ch / 1000; self.w6 = mux_rect_w(6) * cw / 1000; self.h6 = mux_rect_h(6) * ch / 1000; self.g6c = mux_pane_cols(p6); self.g6r = mux_pane_rows(p6); self.lines6 = mux_pane_lines(p6); self.cr6 = mux_pane_cursor_row(p6); self.cc6 = mux_pane_cursor_col(p6); self.dv1 = mux_rect_branch(7); self.dA1 = mux_rect_axis(7); self.dX1 = mux_rect_x(7) * cw / 1000; self.dY1 = mux_rect_y(7) * ch / 1000; self.dW1 = mux_rect_w(7) * cw / 1000; self.dH1 = mux_rect_h(7) * ch / 1000; self.dv2 = mux_rect_branch(8); self.dA2 = mux_rect_axis(8); self.dX2 = mux_rect_x(8) * cw / 1000; self.dY2 = mux_rect_y(8) * ch / 1000; self.dW2 = mux_rect_w(8) * cw / 1000; self.dH2 = mux_rect_h(8) * ch / 1000; self.dv3 = mux_rect_branch(9); self.dA3 = mux_rect_axis(9); self.dX3 = mux_rect_x(9) * cw / 1000; self.dY3 = mux_rect_y(9) * ch / 1000; self.dW3 = mux_rect_w(9) * cw / 1000; self.dH3 = mux_rect_h(9) * ch / 1000; self.dv4 = mux_rect_branch(10); self.dA4 = mux_rect_axis(10); self.dX4 = mux_rect_x(10) * cw / 1000; self.dY4 = mux_rect_y(10) * ch / 1000; self.dW4 = mux_rect_w(10) * cw / 1000; self.dH4 = mux_rect_h(10) * ch / 1000; self.dv5 = mux_rect_branch(11); self.dA5 = mux_rect_axis(11); self.dX5 = mux_rect_x(11) * cw / 1000; self.dY5 = mux_rect_y(11) * ch / 1000; self.dW5 = mux_rect_w(11) * cw / 1000; self.dH5 = mux_rect_h(11) * ch / 1000; self.lastVer = ver; };
                let mut q1 = mux_rect_pane(1);
                if self.slot1 == 1 { self.lines1 = mux_pane_lines(q1); self.cr1 = mux_pane_cursor_row(q1); self.cc1 = mux_pane_cursor_col(q1); self.g1c = mux_pane_cols(q1); self.g1r = mux_pane_rows(q1); };
                let mut q2 = mux_rect_pane(2);
                if self.slot2 == 1 { self.lines2 = mux_pane_lines(q2); self.cr2 = mux_pane_cursor_row(q2); self.cc2 = mux_pane_cursor_col(q2); self.g2c = mux_pane_cols(q2); self.g2r = mux_pane_rows(q2); };
                let mut q3 = mux_rect_pane(3);
                if self.slot3 == 1 { self.lines3 = mux_pane_lines(q3); self.cr3 = mux_pane_cursor_row(q3); self.cc3 = mux_pane_cursor_col(q3); self.g3c = mux_pane_cols(q3); self.g3r = mux_pane_rows(q3); };
                let mut q4 = mux_rect_pane(4);
                if self.slot4 == 1 { self.lines4 = mux_pane_lines(q4); self.cr4 = mux_pane_cursor_row(q4); self.cc4 = mux_pane_cursor_col(q4); self.g4c = mux_pane_cols(q4); self.g4r = mux_pane_rows(q4); };
                let mut q5 = mux_rect_pane(5);
                if self.slot5 == 1 { self.lines5 = mux_pane_lines(q5); self.cr5 = mux_pane_cursor_row(q5); self.cc5 = mux_pane_cursor_col(q5); self.g5c = mux_pane_cols(q5); self.g5r = mux_pane_rows(q5); };
                let mut q6 = mux_rect_pane(6);
                if self.slot6 == 1 { self.lines6 = mux_pane_lines(q6); self.cr6 = mux_pane_cursor_row(q6); self.cc6 = mux_pane_cursor_col(q6); self.g6c = mux_pane_cols(q6); self.g6r = mux_pane_rows(q6); };
                self.cols = term_cols();
                self.rows = term_rows();
            }
            AppMsg::Zoom => {
                let mut fid = mux_focus_id();
                mux_enqueue(4, fid);
            }
            AppMsg::Init => {
                term_apply_resize();
                get_lines();
                let mut n = mux_tab_count();
                let mut ids = vec![];
                let mut labels = vec![];
                let mut i = 0;
                loop { if i >= n { break; }; ids.push(mux_tab_id_at(i)); let mut lbl = mux_tab_title_at(i); if mux_tab_is_active_at(i) == 1 { lbl = format!("{}{}", "● ", lbl); }; labels.push(lbl.clone()); i = i + 1; };
                self.tab_ids = ids;
                self.tab_labels = labels;
                self.clientW = mux_window_width();
                self.clientH = mux_window_height();
                self.slot1 = mux_rect_kind(1);
                let mut p1 = mux_rect_pane(1);
                self.key1 = mux_rect_key(1).to_string();
                self.x1 = mux_rect_x(1) * self.clientW / 1000;
                self.y1 = mux_rect_y(1) * (self.clientH - self.tabH - self.statusH) / 1000;
                self.w1 = mux_rect_w(1) * self.clientW / 1000;
                self.h1 = mux_rect_h(1) * (self.clientH - self.tabH - self.statusH) / 1000;
                self.g1c = mux_pane_cols(p1);
                self.g1r = mux_pane_rows(p1);
                self.lines1 = mux_pane_lines(p1);
                self.cr1 = mux_pane_cursor_row(p1);
                self.cc1 = mux_pane_cursor_col(p1);
                self.slot2 = mux_rect_kind(2);
                self.key2 = mux_rect_key(2).to_string();
                self.x2 = mux_rect_x(2) * self.clientW / 1000;
                self.y2 = mux_rect_y(2) * (self.clientH - self.tabH - self.statusH) / 1000;
                self.w2 = mux_rect_w(2) * self.clientW / 1000;
                self.h2 = mux_rect_h(2) * (self.clientH - self.tabH - self.statusH) / 1000;
                self.slot3 = mux_rect_kind(3);
                self.key3 = mux_rect_key(3).to_string();
                self.slot4 = mux_rect_kind(4);
                self.key4 = mux_rect_key(4).to_string();
                self.slot5 = mux_rect_kind(5);
                self.key5 = mux_rect_key(5).to_string();
                self.slot6 = mux_rect_kind(6);
                self.key6 = mux_rect_key(6).to_string();
                self.lastVer = mux_layout_version();
                self.cols = term_cols();
                self.rows = term_rows();
                self.profile_names = term_profile_names();
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().style("w-full h-screen bg-background").child(View::row().style("w-full").children(self.tab_labels.iter().enumerate().map(|(i, label)| { let i = i as i32; View::button(format!("{}", label)).style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4").on_click(|_| AppMsg::TabActivate(i)).on_right_click(|_| AppMsg::TabClose(i)).build() }).collect::<Vec<_>>()).child(View::button("+").style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4").on_click(|_| AppMsg::NewTab).on_right_click(|_| AppMsg::ProfileMenu).build()).child(if self.profile_menu == 1 { View::col().child(View::col().children(self.profile_names.iter().enumerate().map(|(j, pname)| { let j = j as i32; View::button(format!("{}", pname)).style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4").on_click(|_| AppMsg::NewTabProfile(j)).build() }).collect::<Vec<_>>()).build()).child(View::button("✕").style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4").on_click(|_| AppMsg::ProfileMenu).build()).build() } else { View::Empty }).child(View::button("横分").style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4").on_click(|_| AppMsg::Split(1)).build()).child(View::button("竖分").style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4").on_click(|_| AppMsg::Split(0)).build()).child(View::button("关Pane").style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4").on_click(|_| AppMsg::CloseFocusPane).build()).child(View::button("Zoom").style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4").on_click(|_| AppMsg::Zoom).build()).child(View::button("◐").style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4").on_click(|_| AppMsg::SchemeToggle).build()).build()).child(View::col().style("relative w-full flex-1").child(View::col().style("w-full h-full bg-background").build()).child(if self.slot1 == 1 { View::col().style(format!("absolute z-10 top-[{}px] left-[{}px] w-[{}px] h-[{}px]", self.y1, self.x1, self.w1, self.h1).as_str()).child(View::Terminal { key: self.key1.clone(), cols: (self.g1c) as u16, rows: (self.g1r) as u16, lines: self.lines1.clone(), scroll_offset: 0u16, preedit: None, on_select: None, on_menu: Some(AppMsg::Menu), on_input: Some(AppMsg::KeyIn), cursor_row: (self.cr1) as u16, cursor_col: (self.cc1) as u16, scheme: (self.scheme) as i32, shortcuts: vec![("alt.w".to_string(), AppMsg::Shortcut(10)), ("alt.s".to_string(), AppMsg::Shortcut(11)), ("ctrl.shift.z".to_string(), AppMsg::Shortcut(5)), ("ctrl.shift.w".to_string(), AppMsg::Shortcut(2)), ("ctrl.shift.tab".to_string(), AppMsg::Shortcut(6)), ("ctrl.shift.t".to_string(), AppMsg::Shortcut(1)), ("ctrl.shift.right".to_string(), AppMsg::Shortcut(8)), ("alt.d".to_string(), AppMsg::Shortcut(13)), ("ctrl.shift.left".to_string(), AppMsg::Shortcut(7)), ("ctrl.shift.k".to_string(), AppMsg::Shortcut(9)), ("ctrl.shift.o".to_string(), AppMsg::Shortcut(4)), ("ctrl.shift.e".to_string(), AppMsg::Shortcut(3)), ("alt.a".to_string(), AppMsg::Shortcut(12))], style: None }).build() } else { View::Empty }).child(if self.slot2 == 1 { View::col().style(format!("absolute z-10 top-[{}px] left-[{}px] w-[{}px] h-[{}px]", self.y2, self.x2, self.w2, self.h2).as_str()).child(View::Terminal { key: self.key2.clone(), cols: (self.g2c) as u16, rows: (self.g2r) as u16, lines: self.lines2.clone(), scroll_offset: 0u16, preedit: None, on_select: None, on_menu: Some(AppMsg::Menu), on_input: Some(AppMsg::KeyIn), cursor_row: (self.cr2) as u16, cursor_col: (self.cc2) as u16, scheme: (self.scheme) as i32, shortcuts: vec![("alt.d".to_string(), AppMsg::Shortcut(13)), ("alt.w".to_string(), AppMsg::Shortcut(10)), ("alt.a".to_string(), AppMsg::Shortcut(12)), ("alt.s".to_string(), AppMsg::Shortcut(11))], style: None }).build() } else { View::Empty }).child(if self.slot3 == 1 { View::col().style(format!("absolute z-10 top-[{}px] left-[{}px] w-[{}px] h-[{}px]", self.y3, self.x3, self.w3, self.h3).as_str()).child(View::Terminal { key: self.key3.clone(), cols: (self.g3c) as u16, rows: (self.g3r) as u16, lines: self.lines3.clone(), scroll_offset: 0u16, preedit: None, on_select: None, on_menu: Some(AppMsg::Menu), on_input: Some(AppMsg::KeyIn), cursor_row: (self.cr3) as u16, cursor_col: (self.cc3) as u16, scheme: (self.scheme) as i32, shortcuts: vec![("alt.a".to_string(), AppMsg::Shortcut(12)), ("alt.s".to_string(), AppMsg::Shortcut(11)), ("alt.d".to_string(), AppMsg::Shortcut(13)), ("alt.w".to_string(), AppMsg::Shortcut(10))], style: None }).build() } else { View::Empty }).child(if self.slot4 == 1 { View::col().style(format!("absolute z-10 top-[{}px] left-[{}px] w-[{}px] h-[{}px]", self.y4, self.x4, self.w4, self.h4).as_str()).child(View::Terminal { key: self.key4.clone(), cols: (self.g4c) as u16, rows: (self.g4r) as u16, lines: self.lines4.clone(), scroll_offset: 0u16, preedit: None, on_select: None, on_menu: Some(AppMsg::Menu), on_input: Some(AppMsg::KeyIn), cursor_row: (self.cr4) as u16, cursor_col: (self.cc4) as u16, scheme: (self.scheme) as i32, shortcuts: vec![("alt.a".to_string(), AppMsg::Shortcut(12)), ("alt.d".to_string(), AppMsg::Shortcut(13)), ("alt.s".to_string(), AppMsg::Shortcut(11)), ("alt.w".to_string(), AppMsg::Shortcut(10))], style: None }).build() } else { View::Empty }).child(if self.slot5 == 1 { View::col().style(format!("absolute z-10 top-[{}px] left-[{}px] w-[{}px] h-[{}px]", self.y5, self.x5, self.w5, self.h5).as_str()).child(View::Terminal { key: self.key5.clone(), cols: (self.g5c) as u16, rows: (self.g5r) as u16, lines: self.lines5.clone(), scroll_offset: 0u16, preedit: None, on_select: None, on_menu: Some(AppMsg::Menu), on_input: Some(AppMsg::KeyIn), cursor_row: (self.cr5) as u16, cursor_col: (self.cc5) as u16, scheme: (self.scheme) as i32, shortcuts: vec![("alt.d".to_string(), AppMsg::Shortcut(13)), ("alt.s".to_string(), AppMsg::Shortcut(11)), ("alt.a".to_string(), AppMsg::Shortcut(12)), ("alt.w".to_string(), AppMsg::Shortcut(10))], style: None }).build() } else { View::Empty }).child(if self.slot6 == 1 { View::col().style(format!("absolute z-10 top-[{}px] left-[{}px] w-[{}px] h-[{}px]", self.y6, self.x6, self.w6, self.h6).as_str()).child(View::Terminal { key: self.key6.clone(), cols: (self.g6c) as u16, rows: (self.g6r) as u16, lines: self.lines6.clone(), scroll_offset: 0u16, preedit: None, on_select: None, on_menu: Some(AppMsg::Menu), on_input: Some(AppMsg::KeyIn), cursor_row: (self.cr6) as u16, cursor_col: (self.cc6) as u16, scheme: (self.scheme) as i32, shortcuts: vec![("alt.w".to_string(), AppMsg::Shortcut(10)), ("alt.s".to_string(), AppMsg::Shortcut(11)), ("alt.a".to_string(), AppMsg::Shortcut(12)), ("alt.d".to_string(), AppMsg::Shortcut(13))], style: None }).build() } else { View::Empty }).child(if self.dv1 != 0 { if self.dA1 == 1 { View::col().style(format!("absolute z-20 top-[{}px] left-[{}px] w-[8px] h-[{}px] bg-[#8899aa]", self.dY1, self.dX1, self.dH1).as_str()).child(View::MouseArea { content: Box::new(auto_lang::ui::view::View::Empty), on_enter: None, on_exit: None, on_double_click: None, on_click: Some(AppMsg::Press(7)), on_context_menu: None, on_release: Some(AppMsg::Drop), on_move: None, logical_extent: None, style: auto_lang::ui::style::Style::parse("w-full h-full").ok() }).build() } else { View::col().style(format!("absolute z-20 top-[{}px] left-[{}px] w-[{}px] h-[8px] bg-[#8899aa]", self.dY1, self.dX1, self.dW1).as_str()).child(View::MouseArea { content: Box::new(auto_lang::ui::view::View::Empty), on_enter: None, on_exit: None, on_double_click: None, on_click: Some(AppMsg::Press(7)), on_context_menu: None, on_release: Some(AppMsg::Drop), on_move: None, logical_extent: None, style: auto_lang::ui::style::Style::parse("w-full h-full").ok() }).build() } } else { View::Empty }).child(if self.dv2 != 0 { if self.dA2 == 1 { View::col().style(format!("absolute z-20 top-[{}px] left-[{}px] w-[8px] h-[{}px] bg-[#8899aa]", self.dY2, self.dX2, self.dH2).as_str()).child(View::MouseArea { content: Box::new(auto_lang::ui::view::View::Empty), on_enter: None, on_exit: None, on_double_click: None, on_click: Some(AppMsg::Press(8)), on_context_menu: None, on_release: Some(AppMsg::Drop), on_move: None, logical_extent: None, style: auto_lang::ui::style::Style::parse("w-full h-full").ok() }).build() } else { View::col().style(format!("absolute z-20 top-[{}px] left-[{}px] w-[{}px] h-[8px] bg-[#8899aa]", self.dY2, self.dX2, self.dW2).as_str()).child(View::MouseArea { content: Box::new(auto_lang::ui::view::View::Empty), on_enter: None, on_exit: None, on_double_click: None, on_click: Some(AppMsg::Press(8)), on_context_menu: None, on_release: Some(AppMsg::Drop), on_move: None, logical_extent: None, style: auto_lang::ui::style::Style::parse("w-full h-full").ok() }).build() } } else { View::Empty }).child(if self.dv3 != 0 { if self.dA3 == 1 { View::col().style(format!("absolute z-20 top-[{}px] left-[{}px] w-[8px] h-[{}px] bg-[#8899aa]", self.dY3, self.dX3, self.dH3).as_str()).child(View::MouseArea { content: Box::new(auto_lang::ui::view::View::Empty), on_enter: None, on_exit: None, on_double_click: None, on_click: Some(AppMsg::Press(9)), on_context_menu: None, on_release: Some(AppMsg::Drop), on_move: None, logical_extent: None, style: auto_lang::ui::style::Style::parse("w-full h-full").ok() }).build() } else { View::col().style(format!("absolute z-20 top-[{}px] left-[{}px] w-[{}px] h-[8px] bg-[#8899aa]", self.dY3, self.dX3, self.dW3).as_str()).child(View::MouseArea { content: Box::new(auto_lang::ui::view::View::Empty), on_enter: None, on_exit: None, on_double_click: None, on_click: Some(AppMsg::Press(9)), on_context_menu: None, on_release: Some(AppMsg::Drop), on_move: None, logical_extent: None, style: auto_lang::ui::style::Style::parse("w-full h-full").ok() }).build() } } else { View::Empty }).child(if self.dv4 != 0 { if self.dA4 == 1 { View::col().style(format!("absolute z-20 top-[{}px] left-[{}px] w-[8px] h-[{}px] bg-[#8899aa]", self.dY4, self.dX4, self.dH4).as_str()).child(View::MouseArea { content: Box::new(auto_lang::ui::view::View::Empty), on_enter: None, on_exit: None, on_double_click: None, on_click: Some(AppMsg::Press(10)), on_context_menu: None, on_release: Some(AppMsg::Drop), on_move: None, logical_extent: None, style: auto_lang::ui::style::Style::parse("w-full h-full").ok() }).build() } else { View::col().style(format!("absolute z-20 top-[{}px] left-[{}px] w-[{}px] h-[8px] bg-[#8899aa]", self.dY4, self.dX4, self.dW4).as_str()).child(View::MouseArea { content: Box::new(auto_lang::ui::view::View::Empty), on_enter: None, on_exit: None, on_double_click: None, on_click: Some(AppMsg::Press(10)), on_context_menu: None, on_release: Some(AppMsg::Drop), on_move: None, logical_extent: None, style: auto_lang::ui::style::Style::parse("w-full h-full").ok() }).build() } } else { View::Empty }).child(if self.dv5 != 0 { if self.dA5 == 1 { View::col().style(format!("absolute z-20 top-[{}px] left-[{}px] w-[8px] h-[{}px] bg-[#8899aa]", self.dY5, self.dX5, self.dH5).as_str()).child(View::MouseArea { content: Box::new(auto_lang::ui::view::View::Empty), on_enter: None, on_exit: None, on_double_click: None, on_click: Some(AppMsg::Press(11)), on_context_menu: None, on_release: Some(AppMsg::Drop), on_move: None, logical_extent: None, style: auto_lang::ui::style::Style::parse("w-full h-full").ok() }).build() } else { View::col().style(format!("absolute z-20 top-[{}px] left-[{}px] w-[{}px] h-[8px] bg-[#8899aa]", self.dY5, self.dX5, self.dW5).as_str()).child(View::MouseArea { content: Box::new(auto_lang::ui::view::View::Empty), on_enter: None, on_exit: None, on_double_click: None, on_click: Some(AppMsg::Press(11)), on_context_menu: None, on_release: Some(AppMsg::Drop), on_move: None, logical_extent: None, style: auto_lang::ui::style::Style::parse("w-full h-full").ok() }).build() } } else { View::Empty }).child(View::MouseArea { content: Box::new(auto_lang::ui::view::View::Empty), on_enter: None, on_exit: None, on_double_click: None, on_click: None, on_context_menu: None, on_release: Some(AppMsg::Drop), on_move: Some(auto_lang::ui::view::PointerMoveHandler::new(move |x: f32, y: f32| AppMsg::Drag(x, y))), logical_extent: Some((1000.0, 1000.0)), style: None }).build()).child(View::row().style("w-full h-[24px] rounded-b-[15px] bg-[#e9e4d8] px-3 items-center justify-between").child(View::text_styled("◉ AutoTerm mux".to_string(), "text-xs")).child(View::text_styled(format!("{}×{}", self.cols, self.rows), "text-xs")).build()).build()
    }

    fn state_snapshot(&self) -> std::collections::HashMap<String, auto_lang::ui::auto_val::Value> {
        let mut m = std::collections::HashMap::new();
        m.insert("cols".to_string(), auto_lang::ui::auto_val::Value::Int(self.cols));
        m.insert("rows".to_string(), auto_lang::ui::auto_val::Value::Int(self.rows));
        m.insert("scheme".to_string(), auto_lang::ui::auto_val::Value::Int(self.scheme));
        m.insert("clientW".to_string(), auto_lang::ui::auto_val::Value::Int(self.clientW));
        m.insert("clientH".to_string(), auto_lang::ui::auto_val::Value::Int(self.clientH));
        m.insert("tabH".to_string(), auto_lang::ui::auto_val::Value::Int(self.tabH));
        m.insert("statusH".to_string(), auto_lang::ui::auto_val::Value::Int(self.statusH));
        m.insert("slot1".to_string(), auto_lang::ui::auto_val::Value::Int(self.slot1));
        m.insert("slot2".to_string(), auto_lang::ui::auto_val::Value::Int(self.slot2));
        m.insert("slot3".to_string(), auto_lang::ui::auto_val::Value::Int(self.slot3));
        m.insert("slot4".to_string(), auto_lang::ui::auto_val::Value::Int(self.slot4));
        m.insert("slot5".to_string(), auto_lang::ui::auto_val::Value::Int(self.slot5));
        m.insert("slot6".to_string(), auto_lang::ui::auto_val::Value::Int(self.slot6));
        m.insert("key1".to_string(), auto_lang::ui::auto_val::Value::str(&self.key1));
        m.insert("key2".to_string(), auto_lang::ui::auto_val::Value::str(&self.key2));
        m.insert("key3".to_string(), auto_lang::ui::auto_val::Value::str(&self.key3));
        m.insert("key4".to_string(), auto_lang::ui::auto_val::Value::str(&self.key4));
        m.insert("key5".to_string(), auto_lang::ui::auto_val::Value::str(&self.key5));
        m.insert("key6".to_string(), auto_lang::ui::auto_val::Value::str(&self.key6));
        m.insert("x1".to_string(), auto_lang::ui::auto_val::Value::Int(self.x1));
        m.insert("x2".to_string(), auto_lang::ui::auto_val::Value::Int(self.x2));
        m.insert("x3".to_string(), auto_lang::ui::auto_val::Value::Int(self.x3));
        m.insert("x4".to_string(), auto_lang::ui::auto_val::Value::Int(self.x4));
        m.insert("x5".to_string(), auto_lang::ui::auto_val::Value::Int(self.x5));
        m.insert("x6".to_string(), auto_lang::ui::auto_val::Value::Int(self.x6));
        m.insert("y1".to_string(), auto_lang::ui::auto_val::Value::Int(self.y1));
        m.insert("y2".to_string(), auto_lang::ui::auto_val::Value::Int(self.y2));
        m.insert("y3".to_string(), auto_lang::ui::auto_val::Value::Int(self.y3));
        m.insert("y4".to_string(), auto_lang::ui::auto_val::Value::Int(self.y4));
        m.insert("y5".to_string(), auto_lang::ui::auto_val::Value::Int(self.y5));
        m.insert("y6".to_string(), auto_lang::ui::auto_val::Value::Int(self.y6));
        m.insert("w1".to_string(), auto_lang::ui::auto_val::Value::Int(self.w1));
        m.insert("w2".to_string(), auto_lang::ui::auto_val::Value::Int(self.w2));
        m.insert("w3".to_string(), auto_lang::ui::auto_val::Value::Int(self.w3));
        m.insert("w4".to_string(), auto_lang::ui::auto_val::Value::Int(self.w4));
        m.insert("w5".to_string(), auto_lang::ui::auto_val::Value::Int(self.w5));
        m.insert("w6".to_string(), auto_lang::ui::auto_val::Value::Int(self.w6));
        m.insert("h1".to_string(), auto_lang::ui::auto_val::Value::Int(self.h1));
        m.insert("h2".to_string(), auto_lang::ui::auto_val::Value::Int(self.h2));
        m.insert("h3".to_string(), auto_lang::ui::auto_val::Value::Int(self.h3));
        m.insert("h4".to_string(), auto_lang::ui::auto_val::Value::Int(self.h4));
        m.insert("h5".to_string(), auto_lang::ui::auto_val::Value::Int(self.h5));
        m.insert("h6".to_string(), auto_lang::ui::auto_val::Value::Int(self.h6));
        m.insert("g1c".to_string(), auto_lang::ui::auto_val::Value::Int(self.g1c));
        m.insert("g1r".to_string(), auto_lang::ui::auto_val::Value::Int(self.g1r));
        m.insert("g2c".to_string(), auto_lang::ui::auto_val::Value::Int(self.g2c));
        m.insert("g2r".to_string(), auto_lang::ui::auto_val::Value::Int(self.g2r));
        m.insert("g3c".to_string(), auto_lang::ui::auto_val::Value::Int(self.g3c));
        m.insert("g3r".to_string(), auto_lang::ui::auto_val::Value::Int(self.g3r));
        m.insert("g4c".to_string(), auto_lang::ui::auto_val::Value::Int(self.g4c));
        m.insert("g4r".to_string(), auto_lang::ui::auto_val::Value::Int(self.g4r));
        m.insert("g5c".to_string(), auto_lang::ui::auto_val::Value::Int(self.g5c));
        m.insert("g5r".to_string(), auto_lang::ui::auto_val::Value::Int(self.g5r));
        m.insert("g6c".to_string(), auto_lang::ui::auto_val::Value::Int(self.g6c));
        m.insert("g6r".to_string(), auto_lang::ui::auto_val::Value::Int(self.g6r));
        m.insert("cr1".to_string(), auto_lang::ui::auto_val::Value::Int(self.cr1));
        m.insert("cc1".to_string(), auto_lang::ui::auto_val::Value::Int(self.cc1));
        m.insert("cr2".to_string(), auto_lang::ui::auto_val::Value::Int(self.cr2));
        m.insert("cc2".to_string(), auto_lang::ui::auto_val::Value::Int(self.cc2));
        m.insert("cr3".to_string(), auto_lang::ui::auto_val::Value::Int(self.cr3));
        m.insert("cc3".to_string(), auto_lang::ui::auto_val::Value::Int(self.cc3));
        m.insert("cr4".to_string(), auto_lang::ui::auto_val::Value::Int(self.cr4));
        m.insert("cc4".to_string(), auto_lang::ui::auto_val::Value::Int(self.cc4));
        m.insert("cr5".to_string(), auto_lang::ui::auto_val::Value::Int(self.cr5));
        m.insert("cc5".to_string(), auto_lang::ui::auto_val::Value::Int(self.cc5));
        m.insert("cr6".to_string(), auto_lang::ui::auto_val::Value::Int(self.cr6));
        m.insert("cc6".to_string(), auto_lang::ui::auto_val::Value::Int(self.cc6));
        m.insert("dv1".to_string(), auto_lang::ui::auto_val::Value::Int(self.dv1));
        m.insert("dv2".to_string(), auto_lang::ui::auto_val::Value::Int(self.dv2));
        m.insert("dv3".to_string(), auto_lang::ui::auto_val::Value::Int(self.dv3));
        m.insert("dv4".to_string(), auto_lang::ui::auto_val::Value::Int(self.dv4));
        m.insert("dv5".to_string(), auto_lang::ui::auto_val::Value::Int(self.dv5));
        m.insert("dA1".to_string(), auto_lang::ui::auto_val::Value::Int(self.dA1));
        m.insert("dA2".to_string(), auto_lang::ui::auto_val::Value::Int(self.dA2));
        m.insert("dA3".to_string(), auto_lang::ui::auto_val::Value::Int(self.dA3));
        m.insert("dA4".to_string(), auto_lang::ui::auto_val::Value::Int(self.dA4));
        m.insert("dA5".to_string(), auto_lang::ui::auto_val::Value::Int(self.dA5));
        m.insert("dX1".to_string(), auto_lang::ui::auto_val::Value::Int(self.dX1));
        m.insert("dX2".to_string(), auto_lang::ui::auto_val::Value::Int(self.dX2));
        m.insert("dX3".to_string(), auto_lang::ui::auto_val::Value::Int(self.dX3));
        m.insert("dX4".to_string(), auto_lang::ui::auto_val::Value::Int(self.dX4));
        m.insert("dX5".to_string(), auto_lang::ui::auto_val::Value::Int(self.dX5));
        m.insert("dY1".to_string(), auto_lang::ui::auto_val::Value::Int(self.dY1));
        m.insert("dY2".to_string(), auto_lang::ui::auto_val::Value::Int(self.dY2));
        m.insert("dY3".to_string(), auto_lang::ui::auto_val::Value::Int(self.dY3));
        m.insert("dY4".to_string(), auto_lang::ui::auto_val::Value::Int(self.dY4));
        m.insert("dY5".to_string(), auto_lang::ui::auto_val::Value::Int(self.dY5));
        m.insert("dW1".to_string(), auto_lang::ui::auto_val::Value::Int(self.dW1));
        m.insert("dW2".to_string(), auto_lang::ui::auto_val::Value::Int(self.dW2));
        m.insert("dW3".to_string(), auto_lang::ui::auto_val::Value::Int(self.dW3));
        m.insert("dW4".to_string(), auto_lang::ui::auto_val::Value::Int(self.dW4));
        m.insert("dW5".to_string(), auto_lang::ui::auto_val::Value::Int(self.dW5));
        m.insert("dH1".to_string(), auto_lang::ui::auto_val::Value::Int(self.dH1));
        m.insert("dH2".to_string(), auto_lang::ui::auto_val::Value::Int(self.dH2));
        m.insert("dH3".to_string(), auto_lang::ui::auto_val::Value::Int(self.dH3));
        m.insert("dH4".to_string(), auto_lang::ui::auto_val::Value::Int(self.dH4));
        m.insert("dH5".to_string(), auto_lang::ui::auto_val::Value::Int(self.dH5));
        m.insert("dragging".to_string(), auto_lang::ui::auto_val::Value::Int(self.dragging));
        m.insert("dragBranch".to_string(), auto_lang::ui::auto_val::Value::Int(self.dragBranch));
        m.insert("dragW".to_string(), auto_lang::ui::auto_val::Value::Int(self.dragW));
        m.insert("dragH".to_string(), auto_lang::ui::auto_val::Value::Int(self.dragH));
        m.insert("lastVer".to_string(), auto_lang::ui::auto_val::Value::Int(self.lastVer));
        m.insert("profile_menu".to_string(), auto_lang::ui::auto_val::Value::Int(self.profile_menu));
        m
    }
    fn tick_interval_ms(&self) -> Option<u32> { Some(50) }
    fn tick_msg(&self) -> Option<AppMsg> { Some(AppMsg::Tick) }
}



// API functions (auto-generated, in-process merged mode — no HTTP)


// PLAN-013 T2: db.at 吸收(转译嵌入)——merged 模式的后端真实现;勿手改
pub mod db {
#![allow(unused)]
// Auto-generated by a2r transpiler

use crate::term::{engine_spawn_ex, engine_free, engine_write_line, engine_rows_for, engine_pump_for, engine_apply_resize_for, engine_viewport_cols, engine_viewport_rows, engine_cursor_row, engine_cursor_col, engine_backlog_pending_mb, engine_backlog_paused, engine_backlog_take_alerts, engine_backlog_dropped, engine_interrupt, engine_is_exited, engine_menu_take, window_width, window_height, config_profiles, config_default_profile, config_profiles_full, config_spawn_program, config_spawn_argv, config_spawn_cwd, engine_scroll_pending_for};
use once_cell::sync::Lazy;
use std::sync::Mutex;

static PANE_IDS: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static PANE_HANDLES: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static PANE_KEYS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(vec![]));

static PANE_TAB_IDS: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static PANE_PROGRAMS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(vec![]));

static PANE_CWDS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(vec![]));

static PANE_COLS: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static PANE_ROWS: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static PANE_EXITED: Lazy<Mutex<Vec<bool>>> = Lazy::new(|| Mutex::new(vec![]));

static TAB_IDS: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static TAB_ROOT_NODE: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static TAB_ACTIVE_PANE: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static TAB_ZOOMED_PANE: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static NODE_IDS: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static NODE_TAB_IDS: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static NODE_AXIS: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static NODE_RATIO: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static NODE_FIRST: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static NODE_SECOND: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static NODE_PANE: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static WS_ACTIVE_TAB: Mutex<i64> = Mutex::new(0);

static NEXT_PANE_ID: Mutex<i64> = Mutex::new(1);

static NEXT_TAB_ID: Mutex<i64> = Mutex::new(1);

static NEXT_NODE_ID: Mutex<i64> = Mutex::new(1);

static MAX_PANES: Mutex<i64> = Mutex::new(6);

static MAGNET_TARGET: Mutex<i64> = Mutex::new(500);

static MAGNET_THRESH: Mutex<i64> = Mutex::new(40);

static MIN_PANE: Mutex<i64> = Mutex::new(100);

static LAYOUT_EPOCH: Mutex<i64> = Mutex::new(0);

static RECT_CACHE_EPOCH: Lazy<Mutex<i64>> = Lazy::new(|| Mutex::new(-1));

static RECT_KIND: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static RECT_PANE: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static RECT_KEY: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(vec![]));

static RECT_BRANCH: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static RECT_AXIS: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static RECT_X: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static RECT_Y: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static RECT_W: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static RECT_H: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static ACT_CODES: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static ACT_ARGS: Lazy<Mutex<Vec<i64>>> = Lazy::new(|| Mutex::new(vec![]));

static COLS: Mutex<i64> = Mutex::new(100);

static ROWS: Mutex<i64> = Mutex::new(30);

static EXITED: Mutex<bool> = Mutex::new(false);

static BOOTED: Mutex<bool> = Mutex::new(false);

static BL_ALERTS_TOTAL: Mutex<i64> = Mutex::new(0);

static BL_PENDING_MB: Mutex<i64> = Mutex::new(0);

static BL_PAUSED: Mutex<i64> = Mutex::new(0);

static BL_DROPPED: Mutex<i64> = Mutex::new(0);

/// 首调判定(banner 短轮询只挂 boot 拍)。
/// 首调判定(banner 短轮询只挂 boot 拍)。
fn list_remove_int(mut xs: Vec<i64>, idx: i64) -> Vec<i64> {
    let mut out: Vec<i64> = vec![];
    let mut i: i64 = 0;
    let mut n: i64 = (xs.len() as i64);
    loop {
        if i >= n {
            break;
        }
        if i != idx {
            out.push(xs[(i) as usize].clone());
        }
        i = i + 1;
    }
    return out;
}

fn list_remove_str(mut xs: Vec<String>, idx: i64) -> Vec<String> {
    let mut out: Vec<String> = vec![];
    let mut i: i64 = 0;
    let mut n: i64 = (xs.len() as i64);
    loop {
        if i >= n {
            break;
        }
        if i != idx {
            out.push(xs[(i) as usize].clone());
        }
        i = i + 1;
    }
    return out;
}

fn list_remove_bool(mut xs: Vec<bool>, idx: i64) -> Vec<bool> {
    let mut out: Vec<bool> = vec![];
    let mut i: i64 = 0;
    let mut n: i64 = (xs.len() as i64);
    loop {
        if i >= n {
            break;
        }
        if i != idx {
            out.push(xs[(i) as usize].clone());
        }
        i = i + 1;
    }
    return out;
}

fn pane_index(pane_id: i64) -> i64 {
    let mut i: i64 = 0;
    let mut n: i64 = ((*PANE_IDS.lock().unwrap()).len() as i64);
    let mut hit: i64 = -1;
    loop {
        if i >= n {
            break;
        }
        if (*PANE_IDS.lock().unwrap())[(i) as usize].clone() == pane_id {
            hit = i;
        }
        i = i + 1;
    }
    return hit;
}

fn node_index(node_id: i64) -> i64 {
    let mut i: i64 = 0;
    let mut n: i64 = ((*NODE_IDS.lock().unwrap()).len() as i64);
    let mut hit: i64 = -1;
    loop {
        if i >= n {
            break;
        }
        if (*NODE_IDS.lock().unwrap())[(i) as usize].clone() == node_id {
            hit = i;
        }
        i = i + 1;
    }
    return hit;
}

fn tab_index(tab_id: i64) -> i64 {
    let mut i: i64 = 0;
    let mut n: i64 = ((*TAB_IDS.lock().unwrap()).len() as i64);
    let mut hit: i64 = -1;
    loop {
        if i >= n {
            break;
        }
        if (*TAB_IDS.lock().unwrap())[(i) as usize].clone() == tab_id {
            hit = i;
        }
        i = i + 1;
    }
    return hit;
}

/// List<int> 成员判定(泵可见性集合用;V1 规模个位数)。
fn contains_int(mut xs: Vec<i64>, v: i64) -> bool {
    let mut i: i64 = 0;
    let mut n: i64 = (xs.len() as i64);
    loop {
        if i >= n {
            break;
        }
        if xs[(i) as usize].clone() == v {
            return true;
        }
        i = i + 1;
    }
    return false;
}

/// 活动下标(未初始化 = -1)。
fn active_tab_index() -> i64 {
    if (*WS_ACTIVE_TAB.lock().unwrap()) == 0 {
        return -1;
    }
    return tab_index((*WS_ACTIVE_TAB.lock().unwrap()));
}

/// 找 pane 所在叶节点 id(0 = 无)。
fn leaf_node_of_pane(pane_id: i64) -> i64 {
    let mut i: i64 = 0;
    let mut n: i64 = ((*NODE_IDS.lock().unwrap()).len() as i64);
    let mut hit: i64 = 0;
    loop {
        if i >= n {
            break;
        }
        if (*NODE_PANE.lock().unwrap())[(i) as usize].clone() == pane_id {
            hit = (*NODE_IDS.lock().unwrap())[(i) as usize].clone();
        }
        i = i + 1;
    }
    return hit;
}

/// 子树内最左叶的 pane id(close 后焦点回落用;0 = 空)。
fn leftmost_pane(node_id: i64) -> i64 {
    let mut ni: i64 = node_index(node_id);
    if ni < 0 {
        return 0;
    }
    let mut pane: i64 = (*NODE_PANE.lock().unwrap())[(ni) as usize].clone();
    if pane != 0 {
        return pane;
    }
    let mut left: i64 = leftmost_pane((*NODE_FIRST.lock().unwrap())[(ni) as usize].clone());
    if left != 0 {
        return left;
    }
    return leftmost_pane((*NODE_SECOND.lock().unwrap())[(ni) as usize].clone());
}

/// 把树中 old_child 的挂点改挂 new_child(父分支槽或 tab 根)。
fn replace_child(tab_id: i64, old_child: i64, new_child: i64) -> i64 {
    let mut ti: i64 = tab_index(tab_id);
    if ti >= 0 {
        if (*TAB_ROOT_NODE.lock().unwrap())[(ti) as usize].clone() == old_child {
            (*TAB_ROOT_NODE.lock().unwrap())[(ti) as usize] = new_child;
            return 1;
        }    }
    let mut i: i64 = 0;
    let mut n: i64 = ((*NODE_IDS.lock().unwrap()).len() as i64);
    loop {
        if i >= n {
            break;
        }
        

        if (*NODE_IDS.lock().unwrap())[(i) as usize].clone() != new_child {
            if (*NODE_FIRST.lock().unwrap())[(i) as usize].clone() == old_child {
                (*NODE_FIRST.lock().unwrap())[(i) as usize] = new_child;
            }            if (*NODE_SECOND.lock().unwrap())[(i) as usize].clone() == old_child {
                (*NODE_SECOND.lock().unwrap())[(i) as usize] = new_child;
            }        }
        i = i + 1;
    }
    return 1;
}

/// 摘除节点(按 id 重建全部节点表)。
fn drop_node(node_id: i64) -> i64 {
    let mut ni: i64 = node_index(node_id);
    if ni < 0 {
        return 0;
    }
    { let __a2r_gv = list_remove_int((*NODE_IDS.lock().unwrap()).clone(), ni); *NODE_IDS.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = list_remove_int((*NODE_TAB_IDS.lock().unwrap()).clone(), ni); *NODE_TAB_IDS.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = list_remove_int((*NODE_AXIS.lock().unwrap()).clone(), ni); *NODE_AXIS.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = list_remove_int((*NODE_RATIO.lock().unwrap()).clone(), ni); *NODE_RATIO.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = list_remove_int((*NODE_FIRST.lock().unwrap()).clone(), ni); *NODE_FIRST.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = list_remove_int((*NODE_SECOND.lock().unwrap()).clone(), ni); *NODE_SECOND.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = list_remove_int((*NODE_PANE.lock().unwrap()).clone(), ni); *NODE_PANE.lock().unwrap() = __a2r_gv; };
    return 1;
}

/// 摘除 Pane(按下标重建全部 pane 表;句柄释放由调用方先行)。
fn remove_pane_at(pi: i64) -> i64 {
    { let __a2r_gv = list_remove_int((*PANE_IDS.lock().unwrap()).clone(), pi); *PANE_IDS.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = list_remove_int((*PANE_HANDLES.lock().unwrap()).clone(), pi); *PANE_HANDLES.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = list_remove_str((*PANE_KEYS.lock().unwrap()).clone(), pi); *PANE_KEYS.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = list_remove_int((*PANE_TAB_IDS.lock().unwrap()).clone(), pi); *PANE_TAB_IDS.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = list_remove_str((*PANE_PROGRAMS.lock().unwrap()).clone(), pi); *PANE_PROGRAMS.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = list_remove_str((*PANE_CWDS.lock().unwrap()).clone(), pi); *PANE_CWDS.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = list_remove_int((*PANE_COLS.lock().unwrap()).clone(), pi); *PANE_COLS.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = list_remove_int((*PANE_ROWS.lock().unwrap()).clone(), pi); *PANE_ROWS.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = list_remove_bool((*PANE_EXITED.lock().unwrap()).clone(), pi); *PANE_EXITED.lock().unwrap() = __a2r_gv; };
    return 1;
}

/// 结构变异后失效矩形投影缓存(PLAN-020 D1;纯计数,廉价)。
fn bump_layout_epoch() -> i64 {
    { let __a2r_gv = (*LAYOUT_EPOCH.lock().unwrap()) + 1; *LAYOUT_EPOCH.lock().unwrap() = __a2r_gv; };
    return (*LAYOUT_EPOCH.lock().unwrap());
}

/// 矩形投影(PLAN-020 D1):对活动 Tab 可见子树做层序(BFS)遍历,
/// 产出 pane 槽(≤MAX_PANES)与 divider 槽(≤MAX_PANES−1)的归一化
/// 矩形(‰)。zoom 投影 = 单 pane 槽满幅、零 divider。纯循环 + 平行
/// List 队列(018 平面表惯例;树深不限)。结果落缓存,getter 消费。
fn rects_recompute() -> i64 {
    { let __a2r_gv = vec![]; *RECT_KIND.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = vec![]; *RECT_PANE.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = vec![]; *RECT_KEY.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = vec![]; *RECT_BRANCH.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = vec![]; *RECT_AXIS.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = vec![]; *RECT_X.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = vec![]; *RECT_Y.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = vec![]; *RECT_W.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = vec![]; *RECT_H.lock().unwrap() = __a2r_gv; };
    let mut ti: i64 = active_tab_index();
    if ti < 0 {
        { let __a2r_gv = (*LAYOUT_EPOCH.lock().unwrap()); *RECT_CACHE_EPOCH.lock().unwrap() = __a2r_gv; };
        return 0;
    }
    let mut zoomed: i64 = (*TAB_ZOOMED_PANE.lock().unwrap())[(ti) as usize].clone();
    if zoomed != 0 {
        

        (*RECT_KIND.lock().unwrap()).push(1);
        (*RECT_PANE.lock().unwrap()).push(zoomed);
        let mut pi: i64 = pane_index(zoomed);
        if pi >= 0 {
            (*RECT_KEY.lock().unwrap()).push((*PANE_KEYS.lock().unwrap())[(pi) as usize].clone());
        } else {
            (*RECT_KEY.lock().unwrap()).push("".to_string());
        }
        (*RECT_BRANCH.lock().unwrap()).push(0);
        (*RECT_AXIS.lock().unwrap()).push(-1);
        (*RECT_X.lock().unwrap()).push(0);
        (*RECT_Y.lock().unwrap()).push(0);
        (*RECT_W.lock().unwrap()).push(1000);
        (*RECT_H.lock().unwrap()).push(1000);
        { let __a2r_gv = (*LAYOUT_EPOCH.lock().unwrap()); *RECT_CACHE_EPOCH.lock().unwrap() = __a2r_gv; };
        return 1;
    }


    let mut p_kind: Vec<i64> = vec![];
    let mut p_pane: Vec<i64> = vec![];
    let mut p_key: Vec<String> = vec![];
    let mut p_x: Vec<i64> = vec![];
    let mut p_y: Vec<i64> = vec![];
    let mut p_w: Vec<i64> = vec![];
    let mut p_h: Vec<i64> = vec![];
    let mut d_kind: Vec<i64> = vec![];
    let mut d_branch: Vec<i64> = vec![];
    let mut d_axis: Vec<i64> = vec![];
    let mut d_x: Vec<i64> = vec![];
    let mut d_y: Vec<i64> = vec![];
    let mut d_w: Vec<i64> = vec![];
    let mut d_h: Vec<i64> = vec![];

    let mut q_node: Vec<i64> = vec![];
    let mut q_x: Vec<i64> = vec![];
    let mut q_y: Vec<i64> = vec![];
    let mut q_w: Vec<i64> = vec![];
    let mut q_h: Vec<i64> = vec![];
    q_node.push((*TAB_ROOT_NODE.lock().unwrap())[(ti) as usize].clone());
    q_x.push(0);
    q_y.push(0);
    q_w.push(1000);
    q_h.push(1000);



    let mut head: i64 = 0;
    let mut steps: i64 = 0;
    loop {
        if head >= q_node.len() as i64 {
            break;
        }
        if steps >= 512 {
            break;
        }
        if (q_node.len() as i64) > 256 {
            break;
        }
        steps = steps + 1;
        let mut nid: i64 = q_node[(head) as usize].clone();
        let mut nx: i64 = q_x[(head) as usize].clone();
        let mut ny: i64 = q_y[(head) as usize].clone();
        let mut nw: i64 = q_w[(head) as usize].clone();
        let mut nh: i64 = q_h[(head) as usize].clone();
        head = head + 1;
        let mut ni: i64 = node_index(nid);
        if ni >= 0 {
            let mut pane: i64 = (*NODE_PANE.lock().unwrap())[(ni) as usize].clone();
            if pane != 0 {
                

                p_kind.push(1);
                p_pane.push(pane);
                p_key.push((*PANE_KEYS.lock().unwrap())[(pane_index(pane)) as usize].clone());
                p_x.push(nx);
                p_y.push(ny);
                p_w.push(nw);
                p_h.push(nh);
            } else {
                


                let mut axis: i64 = (*NODE_AXIS.lock().unwrap())[(ni) as usize].clone();
                let mut ratio: i64 = (*NODE_RATIO.lock().unwrap())[(ni) as usize].clone();
                d_kind.push(2);
                d_branch.push(nid);
                d_axis.push(axis);
                if axis == 1 {
                    d_x.push(nx + nw * ratio / 1000);
                    d_y.push(ny);
                    d_w.push(0);
                    d_h.push(nh);
                } else {
                    d_x.push(nx);
                    d_y.push(ny + nh * ratio / 1000);
                    d_w.push(nw);
                    d_h.push(0);
                }
                let mut fw: i64 = nw;
                let mut fh: i64 = nh;
                let mut sx: i64 = nx;
                let mut sy: i64 = ny;
                if axis == 1 {
                    fw = nw * ratio / 1000;
                    sx = nx + fw;
                } else {
                    fh = nh * ratio / 1000;
                    sy = ny + fh;
                }
                q_node.push((*NODE_FIRST.lock().unwrap())[(ni) as usize].clone());
                q_x.push(nx);
                q_y.push(ny);
                q_w.push(fw);
                q_h.push(fh);
                q_node.push((*NODE_SECOND.lock().unwrap())[(ni) as usize].clone());
                q_x.push(sx);
                q_y.push(sy);
                


                if axis == 1 {
                    q_w.push(nw - fw);
                    q_h.push(nh);
                } else {
                    q_w.push(nw);
                    q_h.push(nh - fh);
                }
            }
        }
    }


    let mut i: i64 = 0;
    loop {
        if i >= (*MAX_PANES.lock().unwrap()) {
            break;
        }
        if i < p_kind.len() as i64 {
            (*RECT_KIND.lock().unwrap()).push(p_kind[(i) as usize].clone());
            (*RECT_PANE.lock().unwrap()).push(p_pane[(i) as usize].clone());
            (*RECT_KEY.lock().unwrap()).push(p_key[(i) as usize].clone());
            (*RECT_X.lock().unwrap()).push(p_x[(i) as usize].clone());
            (*RECT_Y.lock().unwrap()).push(p_y[(i) as usize].clone());
            (*RECT_W.lock().unwrap()).push(p_w[(i) as usize].clone());
            (*RECT_H.lock().unwrap()).push(p_h[(i) as usize].clone());
        } else {
            (*RECT_KIND.lock().unwrap()).push(0);
            (*RECT_PANE.lock().unwrap()).push(0);
            (*RECT_KEY.lock().unwrap()).push("".to_string());
            (*RECT_X.lock().unwrap()).push(0);
            (*RECT_Y.lock().unwrap()).push(0);
            (*RECT_W.lock().unwrap()).push(0);
            (*RECT_H.lock().unwrap()).push(0);
        }

        (*RECT_BRANCH.lock().unwrap()).push(0);
        (*RECT_AXIS.lock().unwrap()).push(-1);
        i = i + 1;
    }
    i = 0;
    loop {
        if i >= d_kind.len() as i64 {
            break;
        }
        (*RECT_KIND.lock().unwrap()).push(d_kind[(i) as usize].clone());
        (*RECT_PANE.lock().unwrap()).push(0);
        (*RECT_KEY.lock().unwrap()).push("".to_string());
        (*RECT_BRANCH.lock().unwrap()).push(d_branch[(i) as usize].clone());
        (*RECT_AXIS.lock().unwrap()).push(d_axis[(i) as usize].clone());
        (*RECT_X.lock().unwrap()).push(d_x[(i) as usize].clone());
        (*RECT_Y.lock().unwrap()).push(d_y[(i) as usize].clone());
        (*RECT_W.lock().unwrap()).push(d_w[(i) as usize].clone());
        (*RECT_H.lock().unwrap()).push(d_h[(i) as usize].clone());
        i = i + 1;
    }
    { let __a2r_gv = (*LAYOUT_EPOCH.lock().unwrap()); *RECT_CACHE_EPOCH.lock().unwrap() = __a2r_gv; };
    return 1;
}

/// 投影缓存就绪(惰性重建;epoch 失效)。
fn rects_ready() -> i64 {
    if (*RECT_CACHE_EPOCH.lock().unwrap()) != (*LAYOUT_EPOCH.lock().unwrap()) {
        rects_recompute();
    }
    return 1;
}

/// 第 k 槽字段读取(k 1 起;越界 = 缺省)。plan:窗格/分隔条统一槽表。
fn rect_at(mut kind: Vec<i64>, k: i64) -> i64 {
    if k < 1 {
        return 0;
    }
    if k > kind.len() as i64 {
        return 0;
    }
    return kind[(k - 1) as usize].clone();
}

pub fn mux_rect_kind(k: i64) -> i64 {
    rects_ready();
    return rect_at((*RECT_KIND.lock().unwrap()).clone(), k);
}

pub fn mux_rect_pane(k: i64) -> i64 {
    rects_ready();
    return rect_at((*RECT_PANE.lock().unwrap()).clone(), k);
}

pub fn mux_rect_key(k: i64) -> String {
    rects_ready();
    if k < 1 {
        return "".to_string();
    }
    if k > (*RECT_KEY.lock().unwrap()).len() as i64 {
        return "".to_string();
    }
    return (*RECT_KEY.lock().unwrap())[(k - 1) as usize].clone();
}

pub fn mux_rect_branch(k: i64) -> i64 {
    rects_ready();
    return rect_at((*RECT_BRANCH.lock().unwrap()).clone(), k);
}

pub fn mux_rect_axis(k: i64) -> i64 {
    rects_ready();
    return rect_at((*RECT_AXIS.lock().unwrap()).clone(), k);
}

pub fn mux_rect_x(k: i64) -> i64 {
    rects_ready();
    return rect_at((*RECT_X.lock().unwrap()).clone(), k);
}

pub fn mux_rect_y(k: i64) -> i64 {
    rects_ready();
    return rect_at((*RECT_Y.lock().unwrap()).clone(), k);
}

pub fn mux_rect_w(k: i64) -> i64 {
    rects_ready();
    return rect_at((*RECT_W.lock().unwrap()).clone(), k);
}

pub fn mux_rect_h(k: i64) -> i64 {
    rects_ready();
    return rect_at((*RECT_H.lock().unwrap()).clone(), k);
}

pub fn mux_layout_version() -> i64 {
    return (*LAYOUT_EPOCH.lock().unwrap());
}

pub fn mux_window_width() -> i64 {
    return window_width();
}

pub fn mux_window_height() -> i64 {
    return window_height();
}

/// 第 k 槽类型(0=空 1=pane 2=divider)。
/// 第 k 槽 pane 模型 key(pane 槽;空 = 非 pane 槽)。
/// 投影版本号(PLAN-020 复审 F1:layout_version 门控)——结构变异时
/// 递增的廉价读;前端每拍先读本值,变化才拉完整 rect-*(把稳态请求量
/// 压回 019 水位,降低 axum 并发与引擎 FFI 争用窗口)。
/// 窗口客户区逻辑 px(PLAN-020 T-00b 窗口尺寸面;矩形投影 px 类
/// 几何的标定源。VM = shim 读渲染器每帧全局;rust = 侧车同名;vue
/// back 降级缺省槽位值)。
/// 当前 Tab 根分支的矩形(‰;(0,0,1000,1000) = 无分屏/未初始化)。
/// mux_resize_branch 的比例换算基准(D2;与投影同一 BFS 语义)。
fn rect_of_node(node_id: i64) -> Vec<i64> {
    let mut out: Vec<i64> = vec![0, 0, 1000, 1000];
    let mut ni: i64 = node_index(node_id);
    if ni < 0 {
        return out;
    }
    let mut ti: i64 = tab_index((*NODE_TAB_IDS.lock().unwrap())[(ni) as usize].clone());
    if ti < 0 {
        return out;
    }

    let mut s_node: Vec<i64> = vec![];
    let mut s_x: Vec<i64> = vec![];
    let mut s_y: Vec<i64> = vec![];
    let mut s_w: Vec<i64> = vec![];
    let mut s_h: Vec<i64> = vec![];
    s_node.push((*TAB_ROOT_NODE.lock().unwrap())[(ti) as usize].clone());
    s_x.push(0);
    s_y.push(0);
    s_w.push(1000);
    s_h.push(1000);
    let mut guard: i64 = 0;
    loop {
        if (s_node.len() as i64) == 0 {
            break;
        }
        if guard >= 512 {
            break;
        }
        if (s_node.len() as i64) > 256 {
            break;
        }
        guard = guard + 1;
        let mut nid: i64 = s_node[((s_node.len() as i64) - 1) as usize].clone();
        let mut nx: i64 = s_x[((s_x.len() as i64) - 1) as usize].clone();
        let mut ny: i64 = s_y[((s_y.len() as i64) - 1) as usize].clone();
        let mut nw: i64 = s_w[((s_w.len() as i64) - 1) as usize].clone();
        let mut nh: i64 = s_h[((s_h.len() as i64) - 1) as usize].clone();
        s_node = list_remove_int(s_node.clone(), (s_node.len() as i64) - 1);
        s_x = list_remove_int(s_x.clone(), (s_x.len() as i64) - 1);
        s_y = list_remove_int(s_y.clone(), (s_y.len() as i64) - 1);
        s_w = list_remove_int(s_w.clone(), (s_w.len() as i64) - 1);
        s_h = list_remove_int(s_h.clone(), (s_h.len() as i64) - 1);
        let mut ci: i64 = node_index(nid);
        if nid == node_id {
            out = vec![nx, ny, nw, nh];
            return out;
        }
        if ci >= 0 {
            if (*NODE_PANE.lock().unwrap())[(ci) as usize].clone() == 0 {
                let mut axis: i64 = (*NODE_AXIS.lock().unwrap())[(ci) as usize].clone();
                let mut ratio: i64 = (*NODE_RATIO.lock().unwrap())[(ci) as usize].clone();
                let mut fw: i64 = nw;
                let mut fh: i64 = nh;
                let mut sx: i64 = nx;
                let mut sy: i64 = ny;
                if axis == 1 {
                    fw = nw * ratio / 1000;
                    sx = nx + fw;
                } else {
                    fh = nh * ratio / 1000;
                    sy = ny + fh;
                }
                s_node.push((*NODE_FIRST.lock().unwrap())[(ci) as usize].clone());
                s_x.push(nx);
                s_y.push(ny);
                s_w.push(fw);
                s_h.push(fh);
                s_node.push((*NODE_SECOND.lock().unwrap())[(ci) as usize].clone());
                s_x.push(sx);
                s_y.push(sy);
                if axis == 1 {
                    s_w.push(nw - fw);
                    s_h.push(nh);
                } else {
                    s_w.push(nw);
                    s_h.push(nh - fh);
                }
            }        }
    }
    return out;
}

pub fn mux_resize_branch(branch_id: i64, px: i64, py: i64) -> i64 {
    let mut ni: i64 = node_index(branch_id);
    if ni < 0 {
        return -1;
    }
    if (*NODE_PANE.lock().unwrap())[(ni) as usize].clone() != 0 {
        return -1;
    }
    let mut rect: Vec<i64> = rect_of_node(branch_id);
    let mut bx: i64 = rect[(0) as usize].clone();
    let mut by: i64 = rect[(1) as usize].clone();
    let mut bw: i64 = rect[(2) as usize].clone();
    let mut bh: i64 = rect[(3) as usize].clone();
    let mut axis: i64 = (*NODE_AXIS.lock().unwrap())[(ni) as usize].clone();
    let mut ratio: i64 = 0;
    if axis == 1 {
        if bw <= 0 {
            return -1;
        }        ratio = (px - bx) * 1000 / bw;
    } else {
        if bh <= 0 {
            return -1;
        }        ratio = (py - by) * 1000 / bh;
    }

    if ratio < (*MIN_PANE.lock().unwrap()) {
        ratio = (*MIN_PANE.lock().unwrap());
    }
    if ratio > 1000 - (*MIN_PANE.lock().unwrap()) {
        ratio = 1000 - (*MIN_PANE.lock().unwrap());
    }
    let mut d: i64 = ratio - (*MAGNET_TARGET.lock().unwrap());
    if d < 0 {
        d = 0 - d;
    }
    if d <= (*MAGNET_THRESH.lock().unwrap()) {
        ratio = (*MAGNET_TARGET.lock().unwrap());
    }
    (*NODE_RATIO.lock().unwrap())[(ni) as usize] = ratio;
    bump_layout_epoch();
    return ratio;
}

/// 分支级调比例(PLAN-020 D2;拖拽唯一写入口,键盘委托同此)。
/// (px,py) = 指针在内容区的 ‰ 坐标(前端捕获层 coords 归一,取整后
/// 传入;1‰ ≈ 1 格距,精度充分)。比例 = 指针沿分支轴在父矩形内的
/// 位置;钳位 [MIN_PANE, 1000−MIN_PANE];磁吸 |r−500|≤40 → 500。
/// 返回落定比例(负 = 未找到/叶无比例)。
/// 活动焦点 Pane id(0 = 无)。
fn mux_focus_pane_id() -> i64 {
    let mut ti: i64 = active_tab_index();
    if ti < 0 {
        return 0;
    }
    return (*TAB_ACTIVE_PANE.lock().unwrap())[(ti) as usize].clone();
}

/// 焦点 Pane 的引擎柄(0 = 无/未初始化)。
fn mux_focus_handle() -> i64 {
    let mut pid: i64 = mux_focus_pane_id();
    if pid == 0 {
        return 0;
    }
    let mut pi: i64 = pane_index(pid);
    if pi < 0 {
        return 0;
    }
    return (*PANE_HANDLES.lock().unwrap())[(pi) as usize].clone();
}

/// 焦点 Pane 的模型 key(pane-<id>;"" = 无)。
fn mux_focus_key_inner() -> String {
    let mut pid: i64 = mux_focus_pane_id();
    if pid == 0 {
        return "".to_string();
    }
    let mut pi: i64 = pane_index(pid);
    if pi < 0 {
        return "".to_string();
    }
    return (*PANE_KEYS.lock().unwrap())[(pi) as usize].clone();
}

/// 当前 Tab 可见 pane id 表(PLAN-019 D1 泵三档判据源)。
/// zoom 投影:zoom 中仅 zoomed Pane 可见;否则 = 当前 Tab 全部 Pane。
fn mux_visible_pane_ids() -> Vec<i64> {
    let mut out: Vec<i64> = vec![];
    let mut ti: i64 = active_tab_index();
    if ti < 0 {
        return out;
    }
    let mut zoomed: i64 = (*TAB_ZOOMED_PANE.lock().unwrap())[(ti) as usize].clone();
    if zoomed != 0 {
        out.push(zoomed);
        return out;
    }
    let mut i: i64 = 0;
    let mut n: i64 = ((*PANE_IDS.lock().unwrap()).len() as i64);
    loop {
        if i >= n {
            break;
        }
        if (*PANE_TAB_IDS.lock().unwrap())[(i) as usize].clone() == (*WS_ACTIVE_TAB.lock().unwrap()) {
            out.push((*PANE_IDS.lock().unwrap())[(i) as usize].clone());
        }
        i = i + 1;
    }
    return out;
}

/// 建新 Pane(引擎 spawn + 表落账)→ 返回其叶节点 id(0 = 失败)。
/// PLAN-025 T-07:profile 参数("" = default_profile)——命中经
/// SpawnSpec(program/argv/cwd)spawn_ex;未命中回落缺省 shell
/// (COMSPEC,引擎 FFI default_shell 兜底,G6)。
fn spawn_pane_into_tab(tab_id: i64, axis: i64, profile: &str) -> i64 {
    let mut pid: i64 = (*NEXT_PANE_ID.lock().unwrap());
    { let __a2r_gv = (*NEXT_PANE_ID.lock().unwrap()) + 1; *NEXT_PANE_ID.lock().unwrap() = __a2r_gv; };
    let mut key: String = format!("{}{}", "pane-", pid.to_string());
    let mut program: String = config_spawn_program(profile);
    let mut h: i64 = 0;
    if program != "" {
        let mut argv: Vec<String> = config_spawn_argv(profile);
        let mut cwd: String = config_spawn_cwd(profile);
        h = engine_spawn_ex(program.as_str(), argv, cwd.as_str(), 100, 30);
    } else {
        h = engine_spawn_ex("", vec![], "", 100, 30);
    }

    if h == 0 {
        return 0;
    }
    (*PANE_IDS.lock().unwrap()).push(pid);
    (*PANE_HANDLES.lock().unwrap()).push(h);
    (*PANE_KEYS.lock().unwrap()).push(key.to_string());
    (*PANE_TAB_IDS.lock().unwrap()).push(tab_id);
    (*PANE_PROGRAMS.lock().unwrap()).push(program.to_string());
    (*PANE_CWDS.lock().unwrap()).push("".to_string());
    (*PANE_COLS.lock().unwrap()).push(100);
    (*PANE_ROWS.lock().unwrap()).push(30);
    (*PANE_EXITED.lock().unwrap()).push(false);

    let mut leaf: i64 = (*NEXT_NODE_ID.lock().unwrap());
    { let __a2r_gv = (*NEXT_NODE_ID.lock().unwrap()) + 1; *NEXT_NODE_ID.lock().unwrap() = __a2r_gv; };
    (*NODE_IDS.lock().unwrap()).push(leaf);
    (*NODE_TAB_IDS.lock().unwrap()).push(tab_id);
    (*NODE_AXIS.lock().unwrap()).push(axis);
    (*NODE_RATIO.lock().unwrap()).push(500);
    (*NODE_FIRST.lock().unwrap()).push(0);
    (*NODE_SECOND.lock().unwrap()).push(0);
    (*NODE_PANE.lock().unwrap()).push(pid);
    return leaf;
}

pub fn mux_enqueue(code: i64, arg: i64) -> i64 {
    (*ACT_CODES.lock().unwrap()).push(code);
    (*ACT_ARGS.lock().unwrap()).push(arg);
    return 1;
}

pub fn mux_focus_dir(d: i64) -> i64 {
    let mut cur: i64 = mux_focus_pane_id();
    if cur == 0 {
        return 0;
    }
    let mut ci: i64 = -1;
    let mut cx: i64 = 0;
    let mut cy: i64 = 0;
    let mut i: i64 = 0;
    loop {
        if i >= (*MAX_PANES.lock().unwrap()) {
            break;
        }
        if (*RECT_KIND.lock().unwrap())[(i) as usize].clone() == 1 && (*RECT_PANE.lock().unwrap())[(i) as usize].clone() == cur {
            ci = i;
            cx = (*RECT_X.lock().unwrap())[(i) as usize].clone() + (*RECT_W.lock().unwrap())[(i) as usize].clone() / 2;
            cy = (*RECT_Y.lock().unwrap())[(i) as usize].clone() + (*RECT_H.lock().unwrap())[(i) as usize].clone() / 2;
        }
        i = i + 1;
    }
    if ci < 0 {
        return 0;
    }
    let mut best: i64 = -1;
    let mut best_score: i64 = 0;
    i = 0;
    loop {
        if i >= (*MAX_PANES.lock().unwrap()) {
            break;
        }
        if i != ci && (*RECT_KIND.lock().unwrap())[(i) as usize].clone() == 1 {
            let mut ox: i64 = (*RECT_X.lock().unwrap())[(i) as usize].clone() + (*RECT_W.lock().unwrap())[(i) as usize].clone() / 2 - cx;
            let mut oy: i64 = (*RECT_Y.lock().unwrap())[(i) as usize].clone() + (*RECT_H.lock().unwrap())[(i) as usize].clone() / 2 - cy;
            let mut ok: i64 = 0;
            let mut primary: i64 = 0;
            let mut cross: i64 = 0;
            if d == 0 && oy < 0 {
                ok = 1;
                primary = 0 - oy;
                cross = ox;
            }            if d == 1 && oy > 0 {
                ok = 1;
                primary = oy;
                cross = ox;
            }            if d == 2 && ox < 0 {
                ok = 1;
                primary = 0 - ox;
                cross = oy;
            }            if d == 3 && ox > 0 {
                ok = 1;
                primary = ox;
                cross = oy;
            }            if ok == 1 {
                if cross < 0 {
                    cross = 0 - cross;
                }                let mut score: i64 = primary * 1000 + cross;
                if best < 0 || score < best_score {
                    best = (*RECT_PANE.lock().unwrap())[(i) as usize].clone();
                    best_score = score;
                }            }        }
        i = i + 1;
    }
    if best >= 0 {
        return mux_focus(best);
    }
    return 0;
}

/// 用户动作入队(UI 线程安全)。
/// 1=NewTab(arg0) 2=Split(arg=axis) 3=CloseFocusPane(arg=pane id)
/// 4=Zoom(arg=pane id) 5=ActivateTab(arg=tab id) 6=CloseTab(arg=tab id)
/// 方向聚焦导航(PLAN-022 T-06 用户指令:Alt+WASD 切换 split 子窗口)。
/// d 0=上 1=下 2=左 3=右;按 pane 槽位矩形中心选方向半平面内最近邻
/// (主轴×1000 + 正交绝对值评分),命中即切焦;贴边无邻居保持原焦点。
/// zoom 态仅一槽可见,无邻居,自然保持。
/// 动作排水(tick 专用,单线程;每拍至多 8 条防饿死视图)。
fn mux_drain_actions() -> i64 {
    let mut guard: i64 = 0;
    loop {
        if guard >= 8 {
            break;
        }
        if ((*ACT_CODES.lock().unwrap()).len() as i64) == 0 {
            break;
        }
        let mut code: i64 = (*ACT_CODES.lock().unwrap())[(0) as usize].clone();
        let mut arg: i64 = (*ACT_ARGS.lock().unwrap())[(0) as usize].clone();
        { let __a2r_gv = list_remove_int((*ACT_CODES.lock().unwrap()).clone(), 0); *ACT_CODES.lock().unwrap() = __a2r_gv; };
        { let __a2r_gv = list_remove_int((*ACT_ARGS.lock().unwrap()).clone(), 0); *ACT_ARGS.lock().unwrap() = __a2r_gv; };
        if code == 1 {
            

            mux_new_tab(profile_name_of_arg(arg).as_str());
        }
        if code == 2 {
            mux_split(arg);
        }
        if code == 3 {
            let mut fid: i64 = mux_focus_pane_id();
            mux_close_pane(fid);
        }
        if code == 4 {
            mux_zoom(arg);
        }
        if code == 5 {
            mux_activate_tab(arg);
        }
        if code == 6 {
            mux_close_tab(arg);
        }
        guard = guard + 1;
    }
    return 0;
}

pub fn mux_init() -> i64 {
    if (*WS_ACTIVE_TAB.lock().unwrap()) != 0 {
        return 0;
    }
    let mut tab: i64 = mux_new_tab("");
    { let __a2r_gv = tab; *WS_ACTIVE_TAB.lock().unwrap() = __a2r_gv; };
    return tab;
}

/// V1 单 Workspace + 初始 Tab + 初始 Pane(缺省 shell,继承宿主 cwd;
/// 100×30 存档几何)。已初始化 = no-op(0)。
/// PLAN-025 T-07:初始 Pane 按 default_profile spawn(配置缺席回落
/// 缺省 shell,G4/G6)。
/// profile 槽位 → 名字(动作队列编码面:1 起;0/越界 = "" = default)。
fn profile_name_of_arg(arg: i64) -> String {
    if arg <= 0 {
        return "".to_string();
    }
    let mut names: Vec<String> = config_profiles();
    if arg > names.len() as i64 {
        return "".to_string();
    }
    return names[(arg - 1) as usize].clone();
}

pub fn mux_split(axis: i64) -> i64 {
    let mut ti: i64 = active_tab_index();
    if ti < 0 {
        return 0;
    }
    let mut tab_id: i64 = (*TAB_IDS.lock().unwrap())[(ti) as usize].clone();

    let mut cnt: i64 = 0;
    let mut pi: i64 = 0;
    let mut pn: i64 = ((*PANE_IDS.lock().unwrap()).len() as i64);
    loop {
        if pi >= pn {
            break;
        }
        if (*PANE_TAB_IDS.lock().unwrap())[(pi) as usize].clone() == tab_id {
            cnt = cnt + 1;
        }
        pi = pi + 1;
    }
    if cnt >= (*MAX_PANES.lock().unwrap()) {
        return -2;
    }
    let mut focus: i64 = (*TAB_ACTIVE_PANE.lock().unwrap())[(ti) as usize].clone();
    let mut old_leaf: i64 = leaf_node_of_pane(focus);
    if old_leaf == 0 {
        return 0;
    }
    let mut new_leaf: i64 = spawn_pane_into_tab(tab_id, axis, "");
    if new_leaf == 0 {
        return 0;
    }
    let mut branch: i64 = (*NEXT_NODE_ID.lock().unwrap());
    { let __a2r_gv = (*NEXT_NODE_ID.lock().unwrap()) + 1; *NEXT_NODE_ID.lock().unwrap() = __a2r_gv; };
    (*NODE_IDS.lock().unwrap()).push(branch);
    (*NODE_TAB_IDS.lock().unwrap()).push(tab_id);
    (*NODE_AXIS.lock().unwrap()).push(axis);
    (*NODE_RATIO.lock().unwrap()).push(500);
    (*NODE_FIRST.lock().unwrap()).push(0);
    (*NODE_SECOND.lock().unwrap()).push(0);
    (*NODE_PANE.lock().unwrap()).push(0);






    replace_child(tab_id, old_leaf, branch);
    let mut bi: i64 = node_index(branch);
    (*NODE_FIRST.lock().unwrap())[(bi) as usize] = old_leaf;
    (*NODE_SECOND.lock().unwrap())[(bi) as usize] = new_leaf;
    bump_layout_epoch();
    let mut ni: i64 = node_index(new_leaf);
    return (*NODE_PANE.lock().unwrap())[(ni) as usize].clone();
}

pub fn mux_close_pane(pane_id: i64) -> i64 {
    let mut pi: i64 = pane_index(pane_id);
    if pi < 0 {
        return -1;
    }
    let mut tab_id: i64 = (*PANE_TAB_IDS.lock().unwrap())[(pi) as usize].clone();
    let mut leaf: i64 = leaf_node_of_pane(pane_id);
    if leaf == 0 {
        return -1;
    }
    let mut ti2: i64 = tab_index(tab_id);
    if (*TAB_ROOT_NODE.lock().unwrap())[(ti2) as usize].clone() == leaf {
        

        return mux_close_tab(tab_id);
    }

    let mut parent: i64 = 0;
    let mut sibling: i64 = 0;
    let mut i: i64 = 0;
    let mut n: i64 = ((*NODE_IDS.lock().unwrap()).len() as i64);
    loop {
        if i >= n {
            break;
        }
        if (*NODE_FIRST.lock().unwrap())[(i) as usize].clone() == leaf {
            parent = (*NODE_IDS.lock().unwrap())[(i) as usize].clone();
            sibling = (*NODE_SECOND.lock().unwrap())[(i) as usize].clone();
        }
        if (*NODE_SECOND.lock().unwrap())[(i) as usize].clone() == leaf {
            parent = (*NODE_IDS.lock().unwrap())[(i) as usize].clone();
            sibling = (*NODE_FIRST.lock().unwrap())[(i) as usize].clone();
        }
        i = i + 1;
    }
    if parent == 0 {
        return 0;
    }

    replace_child(tab_id, parent, sibling);
    drop_node(parent);
    drop_node(leaf);
    let mut h: i64 = (*PANE_HANDLES.lock().unwrap())[(pi) as usize].clone();
    engine_free(h);
    remove_pane_at(pi);

    if (*TAB_ACTIVE_PANE.lock().unwrap())[(ti2) as usize].clone() == pane_id {
        let mut next_focus: i64 = leftmost_pane(sibling);
        if next_focus != 0 {
            (*TAB_ACTIVE_PANE.lock().unwrap())[(ti2) as usize] = next_focus;
        }    }
    bump_layout_epoch();
    return 1;
}

pub fn mux_focus(pane_id: i64) -> i64 {
    let mut pi: i64 = pane_index(pane_id);
    if pi < 0 {
        return -1;
    }
    let mut tab_id: i64 = (*PANE_TAB_IDS.lock().unwrap())[(pi) as usize].clone();
    let mut ti: i64 = tab_index(tab_id);
    if ti < 0 {
        return -1;
    }
    (*TAB_ACTIVE_PANE.lock().unwrap())[(ti) as usize] = pane_id;
    return 1;
}

pub fn mux_zoom(pane_id: i64) -> i64 {
    let mut ti: i64 = active_tab_index();
    if ti < 0 {
        return -1;
    }
    let mut cur: i64 = (*TAB_ZOOMED_PANE.lock().unwrap())[(ti) as usize].clone();
    if cur == pane_id {
        (*TAB_ZOOMED_PANE.lock().unwrap())[(ti) as usize] = 0;
        bump_layout_epoch();
        return 0;
    }
    (*TAB_ZOOMED_PANE.lock().unwrap())[(ti) as usize] = pane_id;
    bump_layout_epoch();
    return 1;
}

pub fn mux_resize_pane(pane_id: i64, ratio: i64) -> i64 {
    let mut leaf: i64 = leaf_node_of_pane(pane_id);
    if leaf == 0 {
        return -1;
    }
    let mut i: i64 = 0;
    let mut n: i64 = ((*NODE_IDS.lock().unwrap()).len() as i64);
    let mut hit: i64 = -1;
    loop {
        if i >= n {
            break;
        }
        if (*NODE_FIRST.lock().unwrap())[(i) as usize].clone() == leaf {
            hit = i;
        }
        if (*NODE_SECOND.lock().unwrap())[(i) as usize].clone() == leaf {
            hit = i;
        }
        i = i + 1;
    }
    if hit < 0 {
        return -1;
    }
    let mut branch: i64 = (*NODE_IDS.lock().unwrap())[(hit) as usize].clone();
    let mut rect: Vec<i64> = rect_of_node(branch);
    let mut px: i64 = rect[(0) as usize].clone() + rect[(2) as usize].clone() * ratio / 1000;
    let mut py: i64 = rect[(1) as usize].clone() + rect[(3) as usize].clone() * ratio / 1000;
    return mux_resize_branch(branch, px, py);
}

pub fn mux_new_tab(profile: &str) -> i64 {
    let mut tab: i64 = (*NEXT_TAB_ID.lock().unwrap());
    { let __a2r_gv = (*NEXT_TAB_ID.lock().unwrap()) + 1; *NEXT_TAB_ID.lock().unwrap() = __a2r_gv; };
    let mut leaf: i64 = spawn_pane_into_tab(tab, 0, profile);
    if leaf == 0 {
        return 0;
    }
    (*TAB_IDS.lock().unwrap()).push(tab);
    (*TAB_ROOT_NODE.lock().unwrap()).push(leaf);
    let mut ni: i64 = node_index(leaf);
    (*TAB_ACTIVE_PANE.lock().unwrap()).push((*NODE_PANE.lock().unwrap())[(ni) as usize].clone());
    (*TAB_ZOOMED_PANE.lock().unwrap()).push(0);
    { let __a2r_gv = tab; *WS_ACTIVE_TAB.lock().unwrap() = __a2r_gv; };
    bump_layout_epoch();
    return tab;
}

pub fn mux_close_tab(tab_id: i64) -> i64 {
    let mut ti: i64 = tab_index(tab_id);
    if ti < 0 {
        return -1;
    }
    if ((*TAB_IDS.lock().unwrap()).len() as i64) == 1 {
        return 0;
    }

    let mut i: i64 = ((*PANE_IDS.lock().unwrap()).len() as i64) - 1;
    loop {
        if i < 0 {
            break;
        }
        if (*PANE_TAB_IDS.lock().unwrap())[(i) as usize].clone() == tab_id {
            let mut h: i64 = (*PANE_HANDLES.lock().unwrap())[(i) as usize].clone();
            engine_free(h);
            remove_pane_at(i);
        }
        i = i - 1;
    }

    i = ((*NODE_IDS.lock().unwrap()).len() as i64) - 1;
    loop {
        if i < 0 {
            break;
        }
        if (*NODE_TAB_IDS.lock().unwrap())[(i) as usize].clone() == tab_id {
            { let __a2r_gv = list_remove_int((*NODE_IDS.lock().unwrap()).clone(), i); *NODE_IDS.lock().unwrap() = __a2r_gv; };
            { let __a2r_gv = list_remove_int((*NODE_TAB_IDS.lock().unwrap()).clone(), i); *NODE_TAB_IDS.lock().unwrap() = __a2r_gv; };
            { let __a2r_gv = list_remove_int((*NODE_AXIS.lock().unwrap()).clone(), i); *NODE_AXIS.lock().unwrap() = __a2r_gv; };
            { let __a2r_gv = list_remove_int((*NODE_RATIO.lock().unwrap()).clone(), i); *NODE_RATIO.lock().unwrap() = __a2r_gv; };
            { let __a2r_gv = list_remove_int((*NODE_FIRST.lock().unwrap()).clone(), i); *NODE_FIRST.lock().unwrap() = __a2r_gv; };
            { let __a2r_gv = list_remove_int((*NODE_SECOND.lock().unwrap()).clone(), i); *NODE_SECOND.lock().unwrap() = __a2r_gv; };
            { let __a2r_gv = list_remove_int((*NODE_PANE.lock().unwrap()).clone(), i); *NODE_PANE.lock().unwrap() = __a2r_gv; };
        }
        i = i - 1;
    }
    { let __a2r_gv = list_remove_int((*TAB_IDS.lock().unwrap()).clone(), ti); *TAB_IDS.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = list_remove_int((*TAB_ROOT_NODE.lock().unwrap()).clone(), ti); *TAB_ROOT_NODE.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = list_remove_int((*TAB_ACTIVE_PANE.lock().unwrap()).clone(), ti); *TAB_ACTIVE_PANE.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = list_remove_int((*TAB_ZOOMED_PANE.lock().unwrap()).clone(), ti); *TAB_ZOOMED_PANE.lock().unwrap() = __a2r_gv; };
    if (*WS_ACTIVE_TAB.lock().unwrap()) == tab_id {
        if ((*TAB_IDS.lock().unwrap()).len() as i64) > 0 {
            { let __a2r_gv = (*TAB_IDS.lock().unwrap())[(0) as usize].clone(); *WS_ACTIVE_TAB.lock().unwrap() = __a2r_gv; };
        } else {
            { let __a2r_gv = 0; *WS_ACTIVE_TAB.lock().unwrap() = __a2r_gv; };
        }
    }
    bump_layout_epoch();
    return 1;
}

pub fn mux_activate_tab(tab_id: i64) -> i64 {
    let mut ti: i64 = tab_index(tab_id);
    if ti < 0 {
        return -1;
    }
    { let __a2r_gv = tab_id; *WS_ACTIVE_TAB.lock().unwrap() = __a2r_gv; };
    bump_layout_epoch();
    return 1;
}

pub fn mux_snapshot() -> String {
    let mut at: i64 = (*WS_ACTIVE_TAB.lock().unwrap());
    let mut out: String = format!("{}{}", "{\"workspace\":{\"active_tab\":", at.to_string());
    out = format!("{}{}", out, "},\"tabs\":[");
    let mut i: i64 = 0;
    let mut n: i64 = ((*TAB_IDS.lock().unwrap()).len() as i64);
    loop {
        if i >= n {
            break;
        }
        if i > 0 {
            out = format!("{}{}", out, ",");
        }
        out = format!("{}{}", format!("{}{}", out, "{\"id\":"), (*TAB_IDS.lock().unwrap())[(i) as usize].clone().to_string());
        out = format!("{}{}", format!("{}{}", out, ",\"root\":"), (*TAB_ROOT_NODE.lock().unwrap())[(i) as usize].clone().to_string());
        out = format!("{}{}", format!("{}{}", out, ",\"active_pane\":"), (*TAB_ACTIVE_PANE.lock().unwrap())[(i) as usize].clone().to_string());
        out = format!("{}{}", format!("{}{}", format!("{}{}", out, ",\"zoomed_pane\":"), (*TAB_ZOOMED_PANE.lock().unwrap())[(i) as usize].clone().to_string()), "}");
        i = i + 1;
    }
    out = format!("{}{}", out, "],\"nodes\":[");
    i = 0;
    n = ((*NODE_IDS.lock().unwrap()).len() as i64);
    loop {
        if i >= n {
            break;
        }
        if i > 0 {
            out = format!("{}{}", out, ",");
        }
        out = format!("{}{}", format!("{}{}", out, "{\"id\":"), (*NODE_IDS.lock().unwrap())[(i) as usize].clone().to_string());
        out = format!("{}{}", format!("{}{}", out, ",\"tab\":"), (*NODE_TAB_IDS.lock().unwrap())[(i) as usize].clone().to_string());
        out = format!("{}{}", format!("{}{}", out, ",\"axis\":"), (*NODE_AXIS.lock().unwrap())[(i) as usize].clone().to_string());
        out = format!("{}{}", format!("{}{}", out, ",\"ratio\":"), (*NODE_RATIO.lock().unwrap())[(i) as usize].clone().to_string());
        out = format!("{}{}", format!("{}{}", out, ",\"first\":"), (*NODE_FIRST.lock().unwrap())[(i) as usize].clone().to_string());
        out = format!("{}{}", format!("{}{}", out, ",\"second\":"), (*NODE_SECOND.lock().unwrap())[(i) as usize].clone().to_string());
        out = format!("{}{}", format!("{}{}", format!("{}{}", out, ",\"pane\":"), (*NODE_PANE.lock().unwrap())[(i) as usize].clone().to_string()), "}");
        i = i + 1;
    }
    out = format!("{}{}", out, "],\"panes\":[");
    i = 0;
    n = ((*PANE_IDS.lock().unwrap()).len() as i64);
    loop {
        if i >= n {
            break;
        }
        if i > 0 {
            out = format!("{}{}", out, ",");
        }
        out = format!("{}{}", format!("{}{}", out, "{\"id\":"), (*PANE_IDS.lock().unwrap())[(i) as usize].clone().to_string());
        out = format!("{}{}", format!("{}{}", out, ",\"handle\":"), (*PANE_HANDLES.lock().unwrap())[(i) as usize].clone().to_string());
        out = format!("{}{}", format!("{}{}", format!("{}{}", out, ",\"key\":\""), (*PANE_KEYS.lock().unwrap())[(i) as usize].clone()), "\"");
        out = format!("{}{}", format!("{}{}", out, ",\"tab\":"), (*PANE_TAB_IDS.lock().unwrap())[(i) as usize].clone().to_string());
        out = format!("{}{}", out, ",\"exited\":");
        if (*PANE_EXITED.lock().unwrap())[(i) as usize].clone() {
            out = format!("{}{}", out, "true");
        } else {
            out = format!("{}{}", out, "false");
        }

        out = format!("{}{}", out, "}");
        i = i + 1;
    }
    out = format!("{}{}", format!("{}{}", out, "],\"focus\":{\"pane\":"), mux_focus_pane_id().to_string());
    out = format!("{}{}", format!("{}{}", format!("{}{}", out, ",\"handle\":"), mux_focus_handle().to_string()), "}}");
    return out;
}

pub fn get_lines() -> Vec<String> {
    mux_init();
    mux_drain_actions();
    let mut handle: i64 = mux_focus_handle();
    if handle == 0 {
        let mut empty: Vec<String> = vec![];
        return empty;
    }
    drain_invisible();
    let mut focus_pid: i64 = mux_focus_pane_id();
    let mut vis: Vec<i64> = mux_visible_pane_ids();
    let mut focus_snapshot: Vec<String> = vec![];
    let mut i: i64 = 0;
    let mut n: i64 = (vis.len() as i64);
    loop {
        if i >= n {
            break;
        }
        let mut pid: i64 = vis[(i) as usize].clone();
        let mut pi: i64 = pane_index(pid);
        if pi >= 0 {
            let mut h: i64 = (*PANE_HANDLES.lock().unwrap())[(pi) as usize].clone();
            let mut key: String = (*PANE_KEYS.lock().unwrap())[(pi) as usize].clone();
            let mut applied: i64 = engine_apply_resize_for(h, key.as_str());
            if applied > 0 {
                (*PANE_COLS.lock().unwrap())[(pi) as usize] = engine_viewport_cols(h);
                (*PANE_ROWS.lock().unwrap())[(pi) as usize] = engine_viewport_rows(h);
                if pid == focus_pid {
                    { let __a2r_gv = engine_viewport_cols(h); *COLS.lock().unwrap() = __a2r_gv; };
                    { let __a2r_gv = engine_viewport_rows(h); *ROWS.lock().unwrap() = __a2r_gv; };
                }            }            let mut snap: Vec<String> = engine_rows_for(h, key.as_str());
            if pid == focus_pid {
                focus_snapshot = snap.clone();
            }        }
        i = i + 1;
    }


    if (*BOOTED.lock().unwrap()) == false {
        let mut tries: i64 = 0;
        loop {
            if tries >= 400 {
                break;
            }
            if (focus_snapshot.len() as i64) > 0 {
                break;
            }
            focus_snapshot = engine_rows_for(handle, mux_focus_key_inner().as_str());
            tries = tries + 1;
        }
        { let __a2r_gv = true; *BOOTED.lock().unwrap() = __a2r_gv; };
    }
    { let __a2r_gv = engine_is_exited(handle); *EXITED.lock().unwrap() = __a2r_gv; };
    return focus_snapshot;
}

/// 分屏:焦点 Pane 的叶位替换为分支(原叶 first、新 Pane second;
/// axis 0=纵向 1=横向;比例 500‰)。返回新 Pane id(0 = 失败;-2 =
/// 超过 MAX_PANES 槽位帽)。深度不限(PLAN-020:矩形投影消费)。
/// 关 Pane:兄弟收编父位(wezterm remove_pane/kitty collapse 同语义);
/// 被关者引擎柄 free(所有权铁律);关的是焦点 → 焦点落兄弟子树最左叶。
/// 关末 Pane = 关其 Tab(PLAN-019 D1,018 §10.4 裁定承接);关的是
/// 末 Tab → mux_close_tab 内拒绝(0,app 常驻)。
/// 1 = 已关(含转关 Tab);0 = 末 Tab 拒绝;-1 = 未找到。
/// 焦点切换(1 = 成功;-1 = 未找到)。
/// 缩放开关(活动 Tab;zoomed_pane 0=无)。返回缩放中 1 / 取消 0。
/// 调比例(PLAN-020 D5 键盘兼容面):pane 所在叶的父分支比例,经
/// mux_resize_branch 同一落定段(钳位 + 磁吸;G6 同一比例真相源)。
/// 返回落定比例(负 = 未找到/无父分支)。
/// 新 Tab(单 Pane;自动激活;profile 名 "" = default_profile,未命中
/// 回落缺省 shell)。返回 tab id(0 = spawn 失败)。
/// 关 Tab:该 tab 全部 Pane 引擎柄 free、pane/节点/表行全量摘除。
/// 1 = 已关;0 = 末 Tab 拒绝;-1 = 未找到。
/// 激活 Tab(1/-1)。
/// 结构快照(JSON 串;/api/mux/snapshot 断言面)。
/// 每拍驱动(首调 mux_init:单 Workspace + 初始 Tab/Pane spawn)。
/// 泵三档(PLAN-019 D1):焦点 Pane = 定向几何泵 + 定向快照(旁路投喂
/// pane key 槽)+ 键入泵目标;可见非焦点 Pane = 同等全量投喂(各自 pane
/// key,分屏后画面不冻结);不可见 Pane(zoom 掩盖/他 Tab)= drain-only
/// (收割引擎防 014 积压,不投喂 widget)。返回焦点 Pane 快照
/// (焦点门面:vue/tick 消费面零变)。
/// 不可见 Pane drain-only:收割引擎输出(快照刷新)不投喂任何 widget
/// (key "-hidden-" 不命中注册表;014 积压护栏维持)。
fn drain_invisible() -> i64 {
    let mut vis: Vec<i64> = mux_visible_pane_ids();
    let mut i: i64 = 0;
    let mut n: i64 = ((*PANE_IDS.lock().unwrap()).len() as i64);
    loop {
        if i >= n {
            break;
        }
        let mut pid: i64 = (*PANE_IDS.lock().unwrap())[(i) as usize].clone();
        if contains_int(vis.clone(), pid) == false {
            let mut h: i64 = (*PANE_HANDLES.lock().unwrap())[(i) as usize].clone();
            let mut sink: Vec<String> = engine_rows_for(h, "-hidden-");
            let mut keep: i64 = (sink.len() as i64);
            if keep > 0 {
                keep = 1;
            }        }
        i = i + 1;
    }
    return 0;
}

pub fn term_send(line: &str) {
    let mut handle: i64 = mux_focus_handle();
    engine_write_line(handle, line);
}

pub fn term_interrupt() -> i64 {
    let mut handle: i64 = mux_focus_handle();
    return engine_interrupt(handle);
}

pub fn term_menu_take() -> i64 {
    return engine_menu_take();
}

pub fn term_pump_input() -> i64 {
    let mut vis: Vec<i64> = mux_visible_pane_ids();
    let mut total: i64 = 0;
    let mut i: i64 = 0;
    let mut n: i64 = (vis.len() as i64);
    loop {
        if i >= n {
            break;
        }
        let mut pid: i64 = vis[(i) as usize].clone();
        let mut pi: i64 = pane_index(pid);
        if pi >= 0 {
            let mut h: i64 = (*PANE_HANDLES.lock().unwrap())[(pi) as usize].clone();
            let mut key: String = (*PANE_KEYS.lock().unwrap())[(pi) as usize].clone();
            let mut pumped: i64 = engine_pump_for(h, key.as_str());
            if pumped > 0 {
                total = total + pumped;
                set_focus_if_active_tab(pid);
            }        }
        i = i + 1;
    }
    return total;
}

/// 宿主 → 子进程一行输入(胶水侧补 \r\n;焦点 Pane 门面)。
/// Ctrl+C 中断(Break→C 双发语义,引擎契约 003 §4.1;焦点 Pane)。
/// 菜单动作载荷取走(PLAN-015 D4;on_menu 消息后由 .Menu 处理器消费):
/// 0=Copy 1=Paste 2=SelectAll 3=Interrupt;-1=无载荷(读取即取走)。
/// 载荷 3 → term_interrupt()(显式中断,不误伤 idle ash;0/1/2 暂忽略)。
/// 直键入泵(PLAN-019 D1 键入路由):逐可见 Pane 排空各自 queue 裸写
/// 各自引擎(无 \r\n 补缀;Enter 已是 VT 串里的 \r)——键入恒落用户
/// 点击聚焦的那个 terminal 槽,Pane 间零串线。本次有泵出的 Pane 记为
/// 其 Tab 焦点(打字随动焦点;V1 近似:纯点击不打字不迁移焦点)。
/// 返回泵送键数。
/// 打字随动焦点:pid 记为其 Tab 焦点(仅当该 Tab = 当前活动 Tab;
/// 不可见 Pane 队列无键,本函数只被可见泵面调用)。
fn set_focus_if_active_tab(pid: i64) -> i64 {
    let mut ti: i64 = active_tab_index();
    if ti < 0 {
        return 0;
    }
    let mut pi: i64 = pane_index(pid);
    if pi >= 0 {
        if (*PANE_TAB_IDS.lock().unwrap())[(pi) as usize].clone() == (*WS_ACTIVE_TAB.lock().unwrap()) {
            (*TAB_ACTIVE_PANE.lock().unwrap())[(ti) as usize] = pid;
        }    }
    return 1;
}

pub fn term_apply_resize() -> i64 {
    let mut pid: i64 = mux_focus_pane_id();
    if pid == 0 {
        return 0;
    }
    let mut pi: i64 = pane_index(pid);
    if pi < 0 {
        return 0;
    }
    let mut key: String = (*PANE_KEYS.lock().unwrap())[(pi) as usize].clone();
    let mut h: i64 = (*PANE_HANDLES.lock().unwrap())[(pi) as usize].clone();
    let mut applied: i64 = engine_apply_resize_for(h, key.as_str());
    if applied > 0 {
        (*PANE_COLS.lock().unwrap())[(pi) as usize] = engine_viewport_cols(h);
        (*PANE_ROWS.lock().unwrap())[(pi) as usize] = engine_viewport_rows(h);
        { let __a2r_gv = engine_viewport_cols(h); *COLS.lock().unwrap() = __a2r_gv; };
        { let __a2r_gv = engine_viewport_rows(h); *ROWS.lock().unwrap() = __a2r_gv; };
    }
    return applied;
}

pub fn term_cols() -> i64 {
    return (*COLS.lock().unwrap());
}

pub fn term_rows() -> i64 {
    return (*ROWS.lock().unwrap());
}

pub fn term_cursor_row() -> i64 {
    let mut handle: i64 = mux_focus_handle();
    if handle == 0 {
        return 0;
    }
    return engine_cursor_row(handle);
}

pub fn term_cursor_col() -> i64 {
    let mut handle: i64 = mux_focus_handle();
    if handle == 0 {
        return 0;
    }
    return engine_cursor_col(handle);
}

pub fn term_backlog_sample() -> i64 {
    let mut handle: i64 = mux_focus_handle();
    if handle == 0 {
        return 0;
    }
    let mut fresh: i64 = engine_backlog_take_alerts(handle);
    { let __a2r_gv = (*BL_ALERTS_TOTAL.lock().unwrap()) + fresh; *BL_ALERTS_TOTAL.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = engine_backlog_pending_mb(handle); *BL_PENDING_MB.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = engine_backlog_paused(handle); *BL_PAUSED.lock().unwrap() = __a2r_gv; };
    { let __a2r_gv = engine_backlog_dropped(handle); *BL_DROPPED.lock().unwrap() = __a2r_gv; };
    return fresh;
}

pub fn term_backlog_alerts_total() -> i64 {
    return (*BL_ALERTS_TOTAL.lock().unwrap());
}

pub fn term_backlog_pending_mb() -> i64 {
    return (*BL_PENDING_MB.lock().unwrap());
}

pub fn term_backlog_paused() -> i64 {
    return (*BL_PAUSED.lock().unwrap());
}

pub fn term_backlog_dropped() -> i64 {
    return (*BL_DROPPED.lock().unwrap());
}

pub fn term_exited() -> bool {
    return (*EXITED.lock().unwrap());
}

pub fn mux_cols() -> i64 {
    return (*COLS.lock().unwrap());
}

pub fn mux_rows() -> i64 {
    return (*ROWS.lock().unwrap());
}

pub fn mux_focus_key() -> String {
    return mux_focus_key_inner();
}

pub fn mux_focus_id() -> i64 {
    return mux_focus_pane_id();
}

static TICK_COUNTER: Mutex<i64> = Mutex::new(0);

static LAST_FULL_TICK: Mutex<i64> = Mutex::new(0);

pub fn term_scroll_pending() -> i64 {
    let mut vis: Vec<i64> = mux_visible_pane_ids();
    let mut i: i64 = 0;
    let mut n: i64 = (vis.len() as i64);
    loop {
        if i >= n {
            break;
        }
        let mut pi: i64 = pane_index(vis[(i) as usize].clone());
        if pi >= 0 {
            if engine_scroll_pending_for((*PANE_HANDLES.lock().unwrap())[(pi) as usize].clone(), (*PANE_KEYS.lock().unwrap())[(pi) as usize].clone()) != 0 {
                return 1;
            }        }
        i = i + 1;
    }
    return 0;
}

pub fn tick_gate() -> i64 {
    { let __a2r_gv = (*TICK_COUNTER.lock().unwrap()) + 1; *TICK_COUNTER.lock().unwrap() = __a2r_gv; };
    if (*TICK_COUNTER.lock().unwrap()) - (*LAST_FULL_TICK.lock().unwrap()) >= 3 {
        { let __a2r_gv = (*TICK_COUNTER.lock().unwrap()); *LAST_FULL_TICK.lock().unwrap() = __a2r_gv; };
        return 1;
    }
    if term_scroll_pending() == 1 {
        { let __a2r_gv = (*TICK_COUNTER.lock().unwrap()); *LAST_FULL_TICK.lock().unwrap() = __a2r_gv; };
        return 1;
    }
    return 0;
}

pub fn term_profile_names() -> Vec<String> {
    return config_profiles();
}

pub fn term_profiles() -> Vec<String> {
    return config_profiles_full();
}

pub fn mux_tabs() -> Vec<String> {
    let mut out: Vec<String> = vec![];
    let mut i: i64 = 0;
    let mut n: i64 = ((*TAB_IDS.lock().unwrap()).len() as i64);
    loop {
        if i >= n {
            break;
        }
        let mut rec: String = format!("{}{}", (*TAB_IDS.lock().unwrap())[(i) as usize].clone().to_string(), "|");
        if (*TAB_IDS.lock().unwrap())[(i) as usize].clone() == (*WS_ACTIVE_TAB.lock().unwrap()) {
            rec = format!("{}{}", rec, "1|");
        } else {
            rec = format!("{}{}", rec, "0|");
        }

        rec = format!("{}{}", rec, mux_tab_title_at(i));
        out.push(rec.to_string());
        i = i + 1;
    }
    return out;
}

pub fn mux_tab_count() -> i64 {
    return ((*TAB_IDS.lock().unwrap()).len() as i64);
}

pub fn mux_tab_id_at(i: i64) -> i64 {
    if i < 0 {
        return 0;
    }
    if i >= (*TAB_IDS.lock().unwrap()).len() as i64 {
        return 0;
    }
    return (*TAB_IDS.lock().unwrap())[(i) as usize].clone();
}

pub fn mux_tab_is_active_at(i: i64) -> i64 {
    if mux_tab_id_at(i) == 0 {
        return 0;
    }
    if (*TAB_IDS.lock().unwrap())[(i) as usize].clone() == (*WS_ACTIVE_TAB.lock().unwrap()) {
        return 1;
    }
    return 0;
}

pub fn mux_tab_title_at(i: i64) -> String {
    let mut n: i64 = i + 1;
    return format!("{}{}", "shell ", n.to_string());
}

pub fn mux_pane_lines(pane_id: i64) -> Vec<String> {
    let mut pi: i64 = pane_index(pane_id);
    if pi < 0 {
        let mut empty: Vec<String> = vec![];
        return empty;
    }
    let mut h: i64 = (*PANE_HANDLES.lock().unwrap())[(pi) as usize].clone();
    let mut key: String = (*PANE_KEYS.lock().unwrap())[(pi) as usize].clone();
    return engine_rows_for(h, key.as_str());
}

pub fn mux_pane_cols(pane_id: i64) -> i64 {
    let mut pi: i64 = pane_index(pane_id);
    if pi < 0 {
        return 100;
    }
    return (*PANE_COLS.lock().unwrap())[(pi) as usize].clone();
}

pub fn mux_pane_rows(pane_id: i64) -> i64 {
    let mut pi: i64 = pane_index(pane_id);
    if pi < 0 {
        return 30;
    }
    return (*PANE_ROWS.lock().unwrap())[(pi) as usize].clone();
}

pub fn mux_pane_cursor_row(pane_id: i64) -> i64 {
    let mut pi: i64 = pane_index(pane_id);
    if pi < 0 {
        return 0;
    }
    return engine_cursor_row((*PANE_HANDLES.lock().unwrap())[(pi) as usize].clone());
}

pub fn mux_pane_cursor_col(pane_id: i64) -> i64 {
    let mut pi: i64 = pane_index(pane_id);
    if pi < 0 {
        return 0;
    }
    return engine_cursor_col((*PANE_HANDLES.lock().unwrap())[(pi) as usize].clone());
}

pub fn mux_visible_pane_count() -> i64 {
    return (mux_visible_pane_ids().len() as i64);
}

pub fn mux_split_axis() -> i64 {
    let mut ti: i64 = active_tab_index();
    if ti < 0 {
        return -1;
    }
    let mut root: i64 = (*TAB_ROOT_NODE.lock().unwrap())[(ti) as usize].clone();
    let mut ri: i64 = node_index(root);
    if ri < 0 {
        return -1;
    }
    if (*NODE_PANE.lock().unwrap())[(ri) as usize].clone() != 0 {
        return -1;
    }
    return (*NODE_AXIS.lock().unwrap())[(ri) as usize].clone();
}

pub fn mux_slot_pane_id(slot: i64) -> i64 {
    rects_ready();
    let mut k: i64 = slot;
    if k < 1 {
        k = 1;
    }
    if k > (*MAX_PANES.lock().unwrap()) {
        return 0;
    }
    if mux_rect_kind(k) == 1 {
        return (*RECT_PANE.lock().unwrap())[(k - 1) as usize].clone();
    }
    return 0;
}

pub fn mux_slot_pane_key(slot: i64) -> String {
    rects_ready();
    let mut k: i64 = slot;
    if k < 1 {
        k = 1;
    }
    if k > (*MAX_PANES.lock().unwrap()) {
        return "".to_string();
    }
    if mux_rect_kind(k) == 1 {
        return (*RECT_KEY.lock().unwrap())[(k - 1) as usize].clone();
    }
    return "".to_string();
}

pub fn mux_zoom_active() -> i64 {
    let mut ti: i64 = active_tab_index();
    if ti < 0 {
        return 0;
    }
    if (*TAB_ZOOMED_PANE.lock().unwrap())[(ti) as usize].clone() != 0 {
        return 1;
    }
    return 0;
}

pub fn mux_layout() -> String {
    let mut ti: i64 = active_tab_index();
    if ti < 0 {
        return "{\"tab\":0,\"axis\":-1,\"slots\":[0,0],\"zoom\":0,\"visible\":0,\"focus\":0,\"rects\":[],\"dividers\":[]}".to_string();
    }
    let mut zoomed: i64 = (*TAB_ZOOMED_PANE.lock().unwrap())[(ti) as usize].clone();
    rects_ready();
    let mut out: String = format!("{}{}", "{\"tab\":", (*TAB_IDS.lock().unwrap())[(ti) as usize].clone().to_string());
    out = format!("{}{}", format!("{}{}", out, ",\"axis\":"), mux_split_axis().to_string());
    out = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", out, ",\"slots\":["), mux_slot_pane_id(1).to_string()), ","), mux_slot_pane_id(2).to_string()), "]");
    out = format!("{}{}", out, ",\"zoom\":");
    if zoomed != 0 {
        out = format!("{}{}", out, zoomed.to_string());
    } else {
        out = format!("{}{}", out, "0");
    }

    out = format!("{}{}", format!("{}{}", out, ",\"visible\":"), mux_visible_pane_count().to_string());
    out = format!("{}{}", format!("{}{}", out, ",\"focus\":"), mux_focus_pane_id().to_string());

    out = format!("{}{}", out, ",\"rects\":[");
    let mut i: i64 = 0;
    let mut n: i64 = ((*RECT_KIND.lock().unwrap()).len() as i64);
    let mut first: i64 = 1;
    loop {
        if i >= n {
            break;
        }
        if (*RECT_KIND.lock().unwrap())[(i) as usize].clone() == 1 {
            if first == 1 {
                first = 0;
            } else {
                out = format!("{}{}", out, ",");
            }
            out = format!("{}{}", format!("{}{}", out, "{\"pane\":"), (*RECT_PANE.lock().unwrap())[(i) as usize].clone().to_string());
            out = format!("{}{}", format!("{}{}", out, ",\"x\":"), (*RECT_X.lock().unwrap())[(i) as usize].clone().to_string());
            out = format!("{}{}", format!("{}{}", out, ",\"y\":"), (*RECT_Y.lock().unwrap())[(i) as usize].clone().to_string());
            out = format!("{}{}", format!("{}{}", out, ",\"w\":"), (*RECT_W.lock().unwrap())[(i) as usize].clone().to_string());
            out = format!("{}{}", format!("{}{}", format!("{}{}", out, ",\"h\":"), (*RECT_H.lock().unwrap())[(i) as usize].clone().to_string()), "}");
        }
        i = i + 1;
    }
    out = format!("{}{}", out, "]");

    out = format!("{}{}", out, ",\"dividers\":[");
    i = 0;
    first = 1;
    loop {
        if i >= n {
            break;
        }
        if (*RECT_KIND.lock().unwrap())[(i) as usize].clone() == 2 {
            if first == 1 {
                first = 0;
            } else {
                out = format!("{}{}", out, ",");
            }
            out = format!("{}{}", format!("{}{}", out, "{\"branch\":"), (*RECT_BRANCH.lock().unwrap())[(i) as usize].clone().to_string());
            out = format!("{}{}", format!("{}{}", out, ",\"axis\":"), (*RECT_AXIS.lock().unwrap())[(i) as usize].clone().to_string());
            out = format!("{}{}", format!("{}{}", out, ",\"x\":"), (*RECT_X.lock().unwrap())[(i) as usize].clone().to_string());
            out = format!("{}{}", format!("{}{}", out, ",\"y\":"), (*RECT_Y.lock().unwrap())[(i) as usize].clone().to_string());
            out = format!("{}{}", format!("{}{}", out, ",\"w\":"), (*RECT_W.lock().unwrap())[(i) as usize].clone().to_string());
            out = format!("{}{}", format!("{}{}", format!("{}{}", out, ",\"h\":"), (*RECT_H.lock().unwrap())[(i) as usize].clone().to_string()), "}");
        }
        i = i + 1;
    }
    out = format!("{}{}", out, "]}");
    return out;
}

}

use std::sync::{LazyLock, Mutex};
use serde_json::Value;

static API_DATA: LazyLock<Mutex<Vec<Value>>> = LazyLock::new(|| {
    Mutex::new(vec![])
});
static API_NEXT_ID: LazyLock<Mutex<i64>> = LazyLock::new(|| Mutex::new(100));

fn get_lines() -> Vec<String> {
    db::get_lines()
}

fn tick_gate() -> i32 {
    db::tick_gate() as i32
}

fn term_send(line: String) {
    db::term_send(&line)
}

fn term_interrupt() -> i32 {
    db::term_interrupt() as i32
}

fn term_menu_take() -> i32 {
    db::term_menu_take() as i32
}

fn term_pump_input() -> i32 {
    db::term_pump_input() as i32
}

fn term_apply_resize() -> i32 {
    db::term_apply_resize() as i32
}

fn term_cols() -> i32 {
    db::term_cols() as i32
}

fn term_rows() -> i32 {
    db::term_rows() as i32
}

fn term_cursor_row() -> i32 {
    db::term_cursor_row() as i32
}

fn term_cursor_col() -> i32 {
    db::term_cursor_col() as i32
}

fn term_backlog_sample() -> i32 {
    db::term_backlog_sample() as i32
}

fn term_backlog_alerts_total() -> i32 {
    db::term_backlog_alerts_total() as i32
}

fn term_backlog_pending_mb() -> i32 {
    db::term_backlog_pending_mb() as i32
}

fn term_backlog_paused() -> i32 {
    db::term_backlog_paused() as i32
}

fn term_backlog_dropped() -> i32 {
    db::term_backlog_dropped() as i32
}

fn term_exited() -> bool {
    db::term_exited()
}

fn mux_split(axis: i32) -> i32 {
    db::mux_split(axis as i64) as i32
}

fn mux_close_pane(pane_id: i32) -> i32 {
    db::mux_close_pane(pane_id as i64) as i32
}

fn mux_focus(pane_id: i32) -> i32 {
    db::mux_focus(pane_id as i64) as i32
}

fn mux_focus_dir(d: i32) -> i32 {
    db::mux_focus_dir(d as i64) as i32
}

fn mux_zoom(pane_id: i32) -> i32 {
    db::mux_zoom(pane_id as i64) as i32
}

fn mux_new_tab(profile: String) -> i32 {
    db::mux_new_tab(&profile) as i32
}

fn term_profiles() -> Vec<String> {
    db::term_profiles()
}

fn term_profile_names() -> Vec<String> {
    db::term_profile_names()
}

fn mux_close_tab(tab_id: i32) -> i32 {
    db::mux_close_tab(tab_id as i64) as i32
}

fn mux_activate_tab(tab_id: i32) -> i32 {
    db::mux_activate_tab(tab_id as i64) as i32
}

fn mux_snapshot() -> String {
    db::mux_snapshot()
}

fn mux_cols() -> i32 {
    db::mux_cols() as i32
}

fn mux_rows() -> i32 {
    db::mux_rows() as i32
}

fn mux_focus_key() -> String {
    db::mux_focus_key()
}

fn mux_tabs() -> Vec<String> {
    db::mux_tabs()
}

fn mux_tab_count() -> i32 {
    db::mux_tab_count() as i32
}

fn mux_tab_id_at(i: i32) -> i32 {
    db::mux_tab_id_at(i as i64) as i32
}

fn mux_tab_is_active_at(i: i32) -> i32 {
    db::mux_tab_is_active_at(i as i64) as i32
}

fn mux_tab_title_at(i: i32) -> String {
    db::mux_tab_title_at(i as i64)
}

fn mux_pane_lines(pane_id: i32) -> Vec<String> {
    db::mux_pane_lines(pane_id as i64)
}

fn mux_pane_cols(pane_id: i32) -> i32 {
    db::mux_pane_cols(pane_id as i64) as i32
}

fn mux_pane_rows(pane_id: i32) -> i32 {
    db::mux_pane_rows(pane_id as i64) as i32
}

fn mux_pane_cursor_row(pane_id: i32) -> i32 {
    db::mux_pane_cursor_row(pane_id as i64) as i32
}

fn mux_pane_cursor_col(pane_id: i32) -> i32 {
    db::mux_pane_cursor_col(pane_id as i64) as i32
}

fn mux_visible_pane_count() -> i32 {
    db::mux_visible_pane_count() as i32
}

fn mux_split_axis() -> i32 {
    db::mux_split_axis() as i32
}

fn mux_slot_pane_id(slot: i32) -> i32 {
    db::mux_slot_pane_id(slot as i64) as i32
}

fn mux_slot_pane_key(slot: i32) -> String {
    db::mux_slot_pane_key(slot as i64)
}

fn mux_focus_id() -> i32 {
    db::mux_focus_id() as i32
}

fn mux_zoom_active() -> i32 {
    db::mux_zoom_active() as i32
}

fn mux_layout() -> String {
    db::mux_layout()
}

fn mux_enqueue(code: i32, arg: i32) -> i32 {
    db::mux_enqueue(code as i64, arg as i64) as i32
}

fn mux_rect_kind(k: i32) -> i32 {
    db::mux_rect_kind(k as i64) as i32
}

fn mux_rect_pane(k: i32) -> i32 {
    db::mux_rect_pane(k as i64) as i32
}

fn mux_rect_key(k: i32) -> String {
    db::mux_rect_key(k as i64)
}

fn mux_rect_branch(k: i32) -> i32 {
    db::mux_rect_branch(k as i64) as i32
}

fn mux_rect_axis(k: i32) -> i32 {
    db::mux_rect_axis(k as i64) as i32
}

fn mux_rect_x(k: i32) -> i32 {
    db::mux_rect_x(k as i64) as i32
}

fn mux_rect_y(k: i32) -> i32 {
    db::mux_rect_y(k as i64) as i32
}

fn mux_rect_w(k: i32) -> i32 {
    db::mux_rect_w(k as i64) as i32
}

fn mux_rect_h(k: i32) -> i32 {
    db::mux_rect_h(k as i64) as i32
}

fn mux_layout_version() -> i32 {
    db::mux_layout_version() as i32
}

fn mux_window_width() -> i32 {
    db::mux_window_width() as i32
}

fn mux_window_height() -> i32 {
    db::mux_window_height() as i32
}

fn mux_resize_pane(pane_id: i32, ratio: i32) -> i32 {
    db::mux_resize_pane(pane_id as i64, ratio as i64) as i32
}

fn mux_resize_branch(branch_id: i32, px: i32, py: i32) -> i32 {
    db::mux_resize_branch(branch_id as i64, px as i64, py as i64) as i32
}

fn main() -> auto_lang::ui::AppResult<()> {
    #[cfg(feature = "ui-iced")]
    {
        if std::env::var("AUTO_VM_WINDOW").is_err() {
            std::env::set_var("AUTO_VM_WINDOW", "802x482");
        }
        if std::env::var("AUTO_VM_TITLE").is_err() {
            std::env::set_var("AUTO_VM_TITLE", "终端");
        }
        // Plan 020 T-05：孵化参数在册 → native 协议 client 臂（返回即走）；
        // 无标记 → 独立窗（下行 iced_entry 现行行为零变化）。
        let __autodesk_args: Vec<String> = std::env::args().collect();
        let __has_client = __autodesk_args.iter().any(|a| a.starts_with("--autodesk-client="));
        let __has_incubate = __autodesk_args.iter().any(|a| a == "--autodesk-incubate");
        if __has_client || __has_incubate {
            let mut __pipe: Option<String> = None;
            let mut __broker = auto_lang::ui::desktop_protocol::broker::BROKER_PIPE.to_string();
            let mut __render: Option<String> = None;
            let mut __app_name = "auto-term".to_string();
            for __a in &__autodesk_args {
                if let Some(v) = __a.strip_prefix("--autodesk-client=") {
                    __pipe = Some(v.to_string());
                } else if let Some(v) = __a.strip_prefix("--autodesk-broker=") {
                    __broker = v.to_string();
                } else if let Some(v) = __a.strip_prefix("--autodesk-render=") {
                    __render = Some(v.to_string());
                } else if let Some(v) = __a.strip_prefix("--app386=") {
                    __app_name = v.to_string();
                }
            }
            if let Some(arg) = __render.as_deref() {
                if auto_lang::ui::desktop_protocol::coverage::RenderMode::parse(arg).is_none() {
                    eprintln!("[autodesk-client] 未知 --autodesk-render={arg}（auto|queue|independent），回退 auto");
                }
            }
            let __mode = auto_lang::ui::desktop_protocol::coverage::RenderMode::resolve(
                __render.as_deref(),
                None,
            );
            // PLAN-026 T-06 翻转：组件先行构造（auto 裁决 = 覆盖扫描制
            // ——queue 优先 + NotCovered 降级 independent 留痕）。
            let __component = App::default();
            let (__frame_mode, __downgraded, __log) =
                auto_lang::ui::desktop_protocol::client_entry::resolve_native_frame_mode(
                    __mode,
                    "App",
                    &__component.view(),
                );
            if let Some(__l) = &__log {
                eprintln!("[autodesk-client] {__l}");
            }
            let __rqhost = __autodesk_args.iter().any(|a| a == "--autodesk-rqhost");
            let __target = if __rqhost {
                // PLAN-031：rqhost 采纳（well-known rendezvous + exit-on-EOF）。
                auto_lang::ui::desktop_protocol::client_entry::ClientTarget::Rqhost {
                    wellknown: __broker.clone(),
                    app_name: __app_name.clone(),
                }
            } else { match __pipe {
                Some(p) => auto_lang::ui::desktop_protocol::client_entry::ClientTarget::Direct(p),
                None => auto_lang::ui::desktop_protocol::client_entry::ClientTarget::Broker {
                    broker_pipe: __broker,
                },
            } };
            let __opts = auto_lang::ui::desktop_protocol::client_entry::ClientOpts {
                app_name: __app_name.clone(),
                title: __app_name,
                width: 480.0,
                height: 320.0,
                frame_mode: __frame_mode,
                auto_downgraded: __downgraded,
            };
            return auto_lang::ui::desktop_protocol::client_entry::run_native_client(
                __component,
                __opts,
                __target,
            )
            .map_err(Into::into);
        }
        println!("Running with Iced backend");
        return auto_lang::ui::iced::run_app_devtools::<App>();
    }
    #[cfg(feature = "ui-gpui")]
    {
        // Plan 020 §5.5：GPUI 臂不接桌面孵化客户端——参数在册报错退出留痕
        //（v1 限 iced；防静默直跑开窗与孵化预期背离）。
        if std::env::args().any(|a| a == "--autodesk-incubate" || a.starts_with("--autodesk-client=")) {
            return Err("native GPUI 臂不接桌面孵化客户端（Plan 020 v1 限 iced）".into());
        }
        println!("Running with GPUI backend");
        return auto_lang::ui::gpui::run_app::<App>("auto-term");
    }
    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err("No backend enabled! Use --features ui-iced or ui-gpui".into())
    }
}

// rust_sidecar: 用户 .rs 侧车模块声明(PLAN-013 T2;勿手改)
mod term;
