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
    Color, Element, Font, Length, Point, Rectangle, Size, Theme,
    alignment, keyboard, mouse,
};
use iced::advanced::input_method::{InputMethod, Purpose};
use iced::advanced::input_method;

use autoterm_core::{
    Color as TermColor, Damage, NamedColor, SelectionRange, SelectionType, Side, StyledChar,
};

use crate::metrics::GridMetrics;
use crate::palette::to_iced_color;
use crate::{
    DEFAULT_BG, DEFAULT_FG, MenuState, Message, SelectMsg, Vertical, edge_band,
};

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

/// 取证(004 T8):request_ime 调用计数 + 自绘 preedit 帧计数(仅 dev-tools)。
#[cfg(feature = "dev-tools")]
static IME_REQUEST_COUNT: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "dev-tools")]
static PREEDIT_DRAWN: AtomicU64 = AtomicU64::new(0);

/// 读光标绘制状态(None=未画)。
#[cfg(feature = "dev-tools")]
pub fn cursor_drawn() -> Option<(usize, usize)> {
    let v = CURSOR_DRAWN.load(Ordering::Relaxed);
    (v != u64::MAX).then(|| ((v / 8192) as usize, (v % 8192) as usize))
}

/// 读 IME 取证(请求总数,自绘 preedit 帧数)。
#[cfg(feature = "dev-tools")]
pub fn ime_requests() -> (u64, u64) {
    (
        IME_REQUEST_COUNT.load(Ordering::Relaxed),
        PREEDIT_DRAWN.load(Ordering::Relaxed),
    )
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
    /// 选中区间(绝对网格行;配合 `scroll_offset` 回视口)→ 高亮
    /// overlay quad(文本层之下,每帧 emit,不进行缓存 digest)。
    pub selection: Option<SelectionRange>,
    /// IME 挂起预编辑(不写 PTY;over-the-spot 覆盖层由 runtime 绘制,
    /// PLAN-004 T8)。
    pub preedit: Option<String>,
    /// 右键菜单浮层(005 T5):Some=draw 最顶层画;命中检测纯函数。
    pub menu: Option<MenuState>,
    /// 选中高亮色(005 T6;默认 e8e8e8@25%,可 --selection-color)。
    pub selection_color: Color,
}

/// 菜单几何(widget 本地像素;draw 与命中检测同源,纯函数可单测)。
const MENU_W: f32 = 88.0;
const MENU_ITEM_H: f32 = 26.0;
/// 菜单项标签(复制/粘贴/全选;待澄清#4 采纳默认三项)。
const MENU_LABELS: [&str; 3] = ["复制", "粘贴", "全选"];
/// 菜单浮层底色(像素取证判别色)。
const MENU_BG: Color = Color::from_rgb8(0x2a, 0x2e, 0x34);

/// 菜单矩形(widget 本地坐标)。
fn menu_rect(at: (f32, f32)) -> Rectangle {
    Rectangle::new(
        Point::new(at.0, at.1),
        Size::new(MENU_W, MENU_ITEM_H * MENU_LABELS.len() as f32),
    )
}

/// 菜单命中检测(纯函数):返回命中的项索引;矩形外 None。
/// (y 须先作下界判断——负差值 `as usize` 饱和为 0,会误命中首项)
fn menu_item_at(at: (f32, f32), pos: Point) -> Option<usize> {
    let r = menu_rect(at);
    if pos.x < r.x || pos.x >= r.x + r.width || pos.y < r.y {
        return None;
    }
    let idx = ((pos.y - r.y) / MENU_ITEM_H) as usize;
    (idx < MENU_LABELS.len()).then_some(idx)
}

/// 鼠标交互持久状态(经 `Tree` 跨帧存活;PLAN-004 T2)。
/// 拖选进行中标志 + 多击计数(500ms 内同格 2=Semantic、3=Lines)
/// + 键盘修饰(ModifiersChanged 驱动,Alt+拖选块选用;005 T2)。
#[derive(Debug, Default)]
pub struct GridInteraction {
    dragging: bool,
    last_click_at: Option<Instant>,
    last_count: u8,
    last_cell: Option<(usize, usize)>,
    mods: keyboard::Modifiers,
    /// 菜单悬停项(draw 反色;菜单开着时随 CursorMoved 刷新)。
    hover_item: Option<usize>,
}

/// 多击判定窗口。
const MULTI_CLICK_WINDOW: Duration = Duration::from_millis(500);

/// 按下起始选中类型(纯函数,可单测;005 T2):Alt 按下恒为 Block
/// (块选,Windows Terminal 惯例);否则多击计数 2=Semantic、
/// 3=Lines、1=Simple。
fn begin_selection_type(count: u8, mods: keyboard::Modifiers) -> SelectionType {
    if mods.alt() {
        return SelectionType::Block;
    }
    match count {
        2 => SelectionType::Semantic,
        3 => SelectionType::Lines,
        _ => SelectionType::Simple,
    }
}

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
        // T8:任意事件到达即以当前光标/preedit 刷新 IME 声明(锚点随
        // 光标移动;幂等——runtime 对未变的 (rect,purpose) 去重,
        // preedit 覆盖层仅内容变才重建)
        self.request_ime(shell, bounds, self.preedit.as_deref());
        match event {
            iced::Event::Keyboard(keyboard::Event::ModifiersChanged(mods)) => {
                state.mods = *mods;
            }
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(pos) = cursor.position_over(bounds) else { return };
                // 菜单开着时左键归菜单:命中项→动作,未命中→关闭;
                // 一律吞掉(不得触发选中 Begin,005 T5)
                if let Some(menu) = self.menu {
                    let local = Point::new(pos.x - bounds.x, pos.y - bounds.y);
                    match menu_item_at(menu.at, local) {
                        Some(idx) => {
                            let item = [
                                crate::MenuItem::Copy,
                                crate::MenuItem::Paste,
                                crate::MenuItem::SelectAll,
                            ][idx];
                            shell.publish(Message::MenuAction(item));
                        }
                        None => shell.publish(Message::ContextMenu(None)),
                    }
                    state.hover_item = None;
                    shell.capture_event();
                    return;
                }
                let (cell, side) = self.pixel_to_cell(pos, bounds);
                let now = Instant::now();
                // Alt 按下恒为 Block(005 T2):不参与多击计数(计数视同
                // 1,序列重启),与 1/2/3 击语义正交。
                let alt = state.mods.alt();
                let multi = !alt
                    && state.last_click_at.is_some_and(|t| now - t <= MULTI_CLICK_WINDOW)
                    && state.last_cell == Some(cell)
                    && state.last_count < 3;
                let count = if multi { state.last_count + 1 } else { 1 };
                let ty = begin_selection_type(count, state.mods);
                state.last_click_at = Some(now);
                state.last_count = count;
                state.last_cell = Some(cell);
                state.dragging = true;
                shell.publish(Message::Select(SelectMsg::Begin { ty, cell, side }));
                shell.capture_event();
            }
            iced::Event::InputMethod(ime) => match ime {
                input_method::Event::Preedit(text, _) => {
                    shell.publish(Message::SetPreedit(text.clone()));
                    // 用事件内最新 preedit 即时驱动覆盖层(不等下一帧)
                    self.request_ime(shell, bounds, Some(text));
                    shell.capture_event();
                }
                input_method::Event::Commit(text) => {
                    shell.publish(Message::CommitIme(text.clone()));
                    self.request_ime(shell, bounds, None);
                    shell.capture_event();
                }
                input_method::Event::Closed => {
                    shell.publish(Message::SetPreedit(String::new()));
                }
                _ => {}
            },
            iced::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                // 菜单开着时移动只刷新悬停项(不拖选;005 T5)
                if let Some(menu) = self.menu {
                    state.hover_item = cursor
                        .position_over(bounds)
                        .map(|p| Point::new(p.x - bounds.x, p.y - bounds.y))
                        .and_then(|local| menu_item_at(menu.at, local));
                    return;
                }
                if !state.dragging {
                    return;
                }
                // 拖选越界不动视野,Extend clamp 到边缘格;指针越过
                // 上/下边缘时附 at_edge(自动滚动信号,005 T4)
                let Some(pos) = cursor.position() else { return };
                let (cell, side) = self.pixel_to_cell(pos, bounds);
                let at_edge =
                    edge_band(pos.y, bounds.y, bounds.height);
                shell.publish(Message::Select(SelectMsg::Extend {
                    cell,
                    side,
                    at_edge,
                }));
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
                // 右键 = 开上下文菜单(005 T5;原"直接粘贴"由菜单项
                // 承担)。拖选进行中先收尾(菜单与拖选互斥)。
                if state.dragging {
                    state.dragging = false;
                    shell.publish(Message::Select(SelectMsg::Finish));
                }
                let pos = cursor.position().unwrap_or(bounds.position());
                // 菜单锚点 clamp 进视口(右/下缘不越界)
                let at = (
                    (pos.x - bounds.x).clamp(0.0, (bounds.width - MENU_W).max(0.0)),
                    (pos.y - bounds.y)
                        .clamp(0.0, (bounds.height - menu_rect((0.0, 0.0)).height).max(0.0)),
                );
                state.hover_item = menu_item_at(at, Point::new(
                    pos.x - bounds.x,
                    pos.y - bounds.y,
                ));
                shell.publish(Message::ContextMenu(Some(at)));
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
        tree: &Tree,
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

        // 选中高亮层(T3):overlay quad 每帧 emit——不进行缓存 digest,
        // 避开损伤门控盲区(core 的 damage 不感知 selection 变化);
        // 区间为绝对网格行,+scroll_offset 回视口行,视口外行自然裁掉。
        if let Some(sel) = self.selection {
            let offset = self.scroll_offset as i32;
            let start_row = sel.start.line.0 + offset;
            let end_row = sel.end.line.0 + offset;
            let last_visible = self.lines.len() as i32 - 1;
            if end_row >= 0 && start_row <= last_visible {
                let cols = self.lines.first().map_or(0, |l| l.len());
                let highlight = self.selection_color;
                for row in start_row.max(0)..=end_row.min(last_visible) {
                    // 块选:每行同一列带(矩形对齐);行选:首末行截段、
                    // 中间行整行(005 T2)
                    let (col_begin, col_last) = if sel.is_block {
                        (sel.start.column.0, sel.end.column.0)
                    } else {
                        let begin = if row == start_row {
                            sel.start.column.0
                        } else {
                            0
                        };
                        let last = if row == end_row {
                            sel.end.column.0
                        } else {
                            cols.saturating_sub(1)
                        };
                        (begin, last)
                    };
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle::new(
                                Point::new(
                                    bounds.x + col_begin as f32 * cell_px,
                                    bounds.y + row as f32 * line_px,
                                ),
                                Size::new(
                                    (col_last - col_begin + 1) as f32 * cell_px,
                                    line_px,
                                ),
                            ),
                            ..Default::default()
                        },
                        highlight,
                    );
                }
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
        self.draw_preedit(renderer, bounds, cell_px, line_px, font_px, *viewport);
        self.draw_menu(tree, renderer, bounds, line_px, font_px, *viewport);
    }
}

impl TermGrid {
    /// 以终端光标格为锚向 runtime 声明 IME 策略(T8):
    /// over-the-spot 首选已试——runtime 覆盖层在本机/此版本不落屏
    /// (main-events 相相位丢弃 input_method;381 次请求实证),
    /// 按计划裁定次序降级为**自绘 preedit**(draw_preedit);
    /// 此处仍请求 Enabled{cursor, Terminal, preedit: None}——
    /// 保 winit 的 IME 启用与组合窗定位(set_ime_cursor_area)。
    fn request_ime(
        &self,
        shell: &mut iced::advanced::Shell<'_, Message>,
        bounds: Rectangle,
        _preedit: Option<&str>,
    ) {
        let (row, col) = self.cursor.unwrap_or((0, 0));
        let cursor = Rectangle::new(
            Point::new(
                bounds.x + col as f32 * self.metrics.cell_w,
                bounds.y + row as f32 * self.metrics.line_h,
            ),
            Size::new(self.metrics.cell_w, self.metrics.line_h),
        );
        shell.request_input_method(&InputMethod::<String>::Enabled {
            cursor,
            purpose: Purpose::Terminal,
            preedit: None,
        });
        #[cfg(feature = "dev-tools")]
        {
            IME_REQUEST_COUNT.fetch_add(1, Ordering::Relaxed);
        }
    }

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

    /// 自绘 IME 预编辑(T8 备选路径):光标格起内联显示 + 下划线;
    /// 不写 PTY(挂起),Commit 才上屏。CJK 按双格宽估算下划线长度。
    #[allow(clippy::too_many_arguments)]
    fn draw_preedit(
        &self,
        renderer: &mut iced::Renderer,
        bounds: Rectangle,
        cell_px: f32,
        line_px: f32,
        font_px: f32,
        viewport: Rectangle,
    ) {
        #[cfg(feature = "dev-tools")]
        let mut drawn = 0u64;
        if let Some(text) = self.preedit.as_deref().filter(|t| !t.is_empty()) {
            if let Some((row, col)) = self.cursor {
                let x = bounds.x + col as f32 * cell_px;
                let y = bounds.y + row as f32 * line_px;
                let cells = text
                    .chars()
                    .map(|c| if c.is_ascii() { 1.0 } else { 2.0 })
                    .sum::<f32>();
                let w = cells * cell_px;
                renderer.fill_text(
                    plain_text(text.to_string(), w + cell_px, line_px, font_px),
                    Point::new(x, y),
                    DEFAULT_FG,
                    viewport,
                );
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle::new(
                            Point::new(x, y + line_px - 3.0),
                            Size::new(w, 2.0),
                        ),
                        ..Default::default()
                    },
                    DEFAULT_FG,
                );
                #[cfg(feature = "dev-tools")]
                {
                    drawn = 1;
                }
            }
        }
        #[cfg(feature = "dev-tools")]
        PREEDIT_DRAWN.store(drawn, Ordering::Relaxed);
    }

    /// 右键菜单浮层(005 T5,draw 最顶层):底 quad + 边框 + 三项
    /// 文本(复制/粘贴/全选)+ hover 反色;悬停项取自 Tree 态。
    fn draw_menu(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        bounds: Rectangle,
        line_px: f32,
        font_px: f32,
        viewport: Rectangle,
    ) {
        let Some(menu) = self.menu else { return };
        let hover = tree.state.downcast_ref::<GridInteraction>().hover_item;
        let rect = menu_rect(menu.at);
        let rect = Rectangle::new(
            Point::new(bounds.x + rect.x, bounds.y + rect.y),
            rect.size(),
        );
        renderer.fill_quad(
            renderer::Quad {
                bounds: rect,
                border: iced::Border {
                    color: Color { a: 0.4, ..DEFAULT_FG },
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            },
            MENU_BG,
        );
        for (i, label) in MENU_LABELS.iter().enumerate() {
            let item_y = rect.y + i as f32 * MENU_ITEM_H;
            let hovered = hover == Some(i);
            if hovered {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle::new(
                            Point::new(rect.x + 1.0, item_y + 1.0),
                            Size::new(MENU_W - 2.0, MENU_ITEM_H - 2.0),
                        ),
                        ..Default::default()
                    },
                    DEFAULT_FG,
                );
            }
            renderer.fill_text(
                menu_text((*label).to_string(), MENU_W - 12.0, line_px, font_px),
                Point::new(rect.x + 8.0, item_y + (MENU_ITEM_H - line_px) / 2.0),
                if hovered { DEFAULT_BG } else { DEFAULT_FG },
                viewport,
            );
        }
    }

    fn draw_cursor(
        &self,
        renderer: &mut iced::Renderer,
        bounds: Rectangle,
        cell_px: f32,
        line_px: f32,
        font_px: f32,
        viewport: Rectangle,
    ) {        #[cfg(feature = "dev-tools")]
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
    text_with_font(content, width, line_px, font_px, Font::MONOSPACE)
}

/// 菜单标签文本:MONOSPACE(Consolas)无 CJK 字形(004 preedit
/// 取证同款豆腐块),中文标签走系统界面字体(005 T5)。
fn menu_text(
    content: String,
    width: f32,
    line_px: f32,
    font_px: f32,
) -> Text<String, Font> {
    text_with_font(
        content,
        width,
        line_px,
        font_px,
        Font::with_name("Microsoft YaHei UI"),
    )
}

fn text_with_font(
    content: String,
    width: f32,
    line_px: f32,
    font_px: f32,
    font: Font,
) -> Text<String, Font> {
    Text {
        content,
        bounds: Size::new(width, line_px),
        size: font_px.into(),
        line_height: LineHeight::Absolute(line_px.into()),
        font,
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

#[cfg(test)]
mod edge_band_tests {
    use super::edge_band;
    use crate::Vertical;

    #[test]
    fn inside_viewport_has_no_edge() {
        assert_eq!(edge_band(100.0, 0.0, 650.0), None);
        assert_eq!(edge_band(0.0, 0.0, 650.0), None, "上缘线上不算越界");
        assert_eq!(edge_band(650.0, 0.0, 650.0), None, "下缘线上不算越界");
    }

    #[test]
    fn beyond_edges_reports_direction() {
        assert_eq!(edge_band(-1.0, 0.0, 650.0), Some(Vertical::Up));
        assert_eq!(edge_band(651.0, 0.0, 650.0), Some(Vertical::Down));
    }
}

#[cfg(test)]
mod menu_item_at_tests {
    use super::{MENU_ITEM_H, menu_item_at};
    use iced::Point;

    const AT: (f32, f32) = (100.0, 50.0);

    #[test]
    fn hits_map_to_item_index() {
        // 第 1 项中点 → 复制(idx 0)
        assert_eq!(
            menu_item_at(AT, Point::new(140.0, AT.1 + MENU_ITEM_H / 2.0)),
            Some(0)
        );
        // 第 2 项中点 → 粘贴(idx 1)
        assert_eq!(
            menu_item_at(
                AT,
                Point::new(140.0, AT.1 + MENU_ITEM_H * 1.5)
            ),
            Some(1)
        );
        // 第 3 项中点 → 全选(idx 2)
        assert_eq!(
            menu_item_at(
                AT,
                Point::new(140.0, AT.1 + MENU_ITEM_H * 2.5)
            ),
            Some(2)
        );
    }

    #[test]
    fn outside_rect_and_below_items_miss() {
        // 矩形外(左/右/下)
        assert_eq!(menu_item_at(AT, Point::new(99.0, 60.0)), None);
        assert_eq!(menu_item_at(AT, Point::new(189.0, 60.0)), None);
        assert_eq!(menu_item_at(AT, Point::new(140.0, 49.0)), None);
        // 右缘之下(y 越过三项总高)
        assert_eq!(
            menu_item_at(AT, Point::new(140.0, AT.1 + MENU_ITEM_H * 3.0 + 1.0)),
            None
        );
    }
}

#[cfg(test)]
mod begin_selection_type_tests {
    use super::begin_selection_type;
    use autoterm_core::SelectionType;
    use iced::keyboard::Modifiers;

    #[test]
    fn alt_forces_block_and_restarts_click_count() {
        // Alt 恒 Block:单/双/三击计数一概不参与
        assert_eq!(
            begin_selection_type(1, Modifiers::ALT),
            SelectionType::Block
        );
        assert_eq!(
            begin_selection_type(2, Modifiers::ALT),
            SelectionType::Block,
            "Alt 按下时双击计数不得翻成 Semantic"
        );
        assert_eq!(
            begin_selection_type(3, Modifiers::ALT.union(Modifiers::SHIFT)),
            SelectionType::Block,
            "Alt+Shift 仍为 Block"
        );
    }

    #[test]
    fn plain_clicks_map_multi_count() {
        assert_eq!(
            begin_selection_type(1, Modifiers::empty()),
            SelectionType::Simple
        );
        assert_eq!(
            begin_selection_type(2, Modifiers::empty()),
            SelectionType::Semantic
        );
        assert_eq!(
            begin_selection_type(3, Modifiers::empty()),
            SelectionType::Lines
        );
        assert_eq!(
            begin_selection_type(4, Modifiers::empty()),
            SelectionType::Simple,
            "计数溢出回 Simple(与旧 match _ 分支一致)"
        );
        // 非 Alt 修饰(Shift/Ctrl)不影响多击映射
        assert_eq!(
            begin_selection_type(2, Modifiers::SHIFT),
            SelectionType::Semantic
        );
    }
}
