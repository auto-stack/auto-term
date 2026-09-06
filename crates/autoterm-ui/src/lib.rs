//! autoterm-ui — AutoTerm 单窗口终端(iced,PLAN-002 正式架构)
//!
// SPDX-License-Identifier: Apache-2.0
//! 事件驱动:reader 线程经唤醒通道 → iced 订阅 → `Message::PtyBytes`,
//! 无轮询定时器(PLAN-001 spike 的 16ms tick 已移除;dev 钩子激活时
//! 才有粗粒度 dev timer)。

pub mod metrics;
pub mod palette;
pub mod widget;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use iced::keyboard::{Key, Modifiers};
use iced::stream;
use iced::{Color, Element, Size, Subscription, Task, time};
use std::hash::{Hash, Hasher};

use autoterm_core::PtySession;
use autoterm_core::{Damage, StyledChar};
pub use autoterm_core::{CursorShape, SelectionRange, SelectionType, Side};
pub use widget::TermGrid;
use metrics::GridMetrics;

/// 运行配置(bin 解析后传入)。
#[derive(Clone, Debug)]
pub struct AppConfig {
    pub shell: String,
    /// 选中高亮色(005 T6;默认 e8e8e8@25%)。
    pub selection_color: Color,
    /// Ctrl+C 投递模式(008;默认 Auto)。
    pub ctrl_c_mode: CtrlCMode,
    /// dev 取证:自动键入(可多段,"ms:text" 语法同 spike)。
    #[cfg(feature = "dev-tools")]
    pub dev_autotype: Vec<String>,
    /// dev 取证:到时注入拖选序列("<ms>:<r1>:<c1>-<r2>:<c2>",
    /// 视口相对格;PLAN-004 T5 起支持可选类型前缀)。
    #[cfg(feature = "dev-tools")]
    pub dev_select: Option<String>,
    /// dev 取证:到时注入粘贴("<ms>:<文本>",走 Pasted 真实路径;
    /// 不依赖系统剪贴板状态)。
    #[cfg(feature = "dev-tools")]
    pub dev_paste: Option<String>,
    /// dev 取证:到时注入 IME 预编辑("<ms>:<文本>",走 SetPreedit
    /// 真实路径;覆盖层像素取证用)。
    #[cfg(feature = "dev-tools")]
    pub dev_preedit: Option<String>,
    /// dev 取证:到时转储并退出(秒;0=不退出)。
    #[cfg(feature = "dev-tools")]
    pub dev_exit_after: u64,
    /// dev 取证:退出前回滚的行数(正=上翻;转储回滚后视图)。
    #[cfg(feature = "dev-tools")]
    pub dev_scroll: Option<i32>,
    /// dev 取证:退出时转储目标文件。
    #[cfg(feature = "dev-tools")]
    pub dev_dump: Option<PathBuf>,
    /// dev 取证:到时注入右键菜单事件(可多段:"<ms>:<x>:<y>"=在
    /// widget 本地像素开菜单,"<ms>:close"=关菜单;与真实右键/外点
    /// 同一 ContextMenu 消息路径;菜单浮层像素取证用,005 T5)。
    #[cfg(feature = "dev-tools")]
    pub dev_menu: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// PTY 有字节可读(唤醒,不载荷——update 里 drain)。
    PtyBytes,
    Key(Key, Modifiers),
    Resized(Size),
    WindowId(Option<iced::window::Id>),
    /// 滚轮回滚(正=上翻历史,同 Scroll::Delta 约定;行为行)。
    Scrolled(i32),
    /// 拖选自动滚动的节拍(005 T4):仅 drag_scroll 挂起时存在;
    /// 方向行数取自 App 状态(订阅 map 禁捕获,经状态中转)。
    DragScrollTick,
    /// 窗口关闭:同步杀子进程再退出(T8)。
    Closed(iced::window::Id),
    /// 选中消息族(PLAN-004 T2;widget 鼠标事件或 dev 注入)。
    Select(SelectMsg),
    /// 粘贴(Ctrl+Shift+V / 右键;PLAN-004 T4)。
    Paste,
    /// 右键菜单开合(005 T5):Some(widget 本地像素锚点)=打开,
    /// None=关闭。
    ContextMenu(Option<(f32, f32)>),
    /// 右键菜单项动作(005 T5;复制/粘贴走 004 同路径)。
    MenuAction(MenuItem),
    /// 剪贴板读取完成(粘贴流第二拍;PLAN-004 T4)。
    Pasted(String),
    /// IME 预编辑变化(挂起显示,不写 PTY;PLAN-004 T8)。
    SetPreedit(String),
    /// IME 提交(清 preedit + 直写 PTY;PLAN-004 T8)。
    CommitIme(String),
    /// dev 钩子的粗定时(仅 dev-tools 构建存在;常态不存在)。
    #[cfg(feature = "dev-tools")]
    DevTick,
    /// 窗口聚焦变化(005 T8;true=聚焦)。
    Focused(bool),
    /// 光标闪烁相位拍(005 T8):500ms 取反;仅聚焦+可见时挂。
    BlinkTick,
    /// 显式空操作:订阅里非键盘/非滚轮事件的归宿(替代 PtyBytes
    /// 空唤醒复用,002 复审瑕疵清偿)。
    NoOp,
}

/// 选中交互消息(PLAN-004 T2)。`cell` 为视口相对 (row, col),
/// App 侧换算绝对网格行(`- display_offset`)后驱动 core。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectMsg {
    /// 按下起点(单/双/三击 → Simple/Semantic/Lines)。
    Begin {
        ty: SelectionType,
        cell: (usize, usize),
        side: Side,
    },
    /// 拖动终点(widget 已 clamp 到边缘格);`at_edge` = 指针停在
    /// 视口上/下边缘带(拖选自动滚动信号,005 T4)。
    Extend {
        cell: (usize, usize),
        side: Side,
        at_edge: Option<Vertical>,
    },
    /// 释放收尾(copy-on-select 在此触发;空选清除)。
    Finish,
}

/// 拖选越界方向(视口上/下边缘带;005 T4)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vertical {
    Up,
    Down,
}

/// 右键菜单项(005 T5)。动作映射:Copy=copy_selection(与
/// Ctrl+Shift+C 同函数)、Paste=paste_from_clipboard(与快捷键
/// 同路径)、SelectAll=视口 Lines 全选。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    Copy,
    Paste,
    SelectAll,
}

/// 右键菜单状态(005 T5):`at` = 菜单左上角(widget 本地像素);
/// 菜单几何与命中检测在 widget(与 pixel_to_cell 同域)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuState {
    pub at: (f32, f32),
}

/// 指针 y 是否落在视口上/下边缘带(纯函数,可单测):越过上缘
/// 即 Up、越过下缘即 Down;视口内为 None(widget 已把格 clamp 到
/// 边缘行,这里只补"方向"信号)。
fn edge_band(pos_y: f32, bounds_y: f32, bounds_height: f32) -> Option<Vertical> {
    if pos_y < bounds_y {
        Some(Vertical::Up)
    } else if pos_y > bounds_y + bounds_height {
        Some(Vertical::Down)
    } else {
        None
    }
}

/// 自动滚动步长(行/拍,50ms;待澄清#5 采纳默认:距缘不分档)。
pub const AUTO_SCROLL_ROWS: i32 = 2;

/// 选中高亮默认色(005 T6):e8e8e8@25%——004 硬编码值移入配置,
/// 默认视觉不变。
pub const DEFAULT_SELECTION_COLOR: Color = Color {
    r: 232.0 / 255.0,
    g: 232.0 / 255.0,
    b: 232.0 / 255.0,
    a: 0.25,
};

/// 十六进制色解析(纯函数,可单测;005 T6):`RRGGBB[AA]`,可带
/// `#` 前缀。6 位 = RGB(α 取默认 0.25,维持现视觉);8 位末两位
/// = α/255。非法(长度/非十六进制)返回 None,调用方回退默认色。
pub fn parse_hex_color(s: &str) -> Option<Color> {
    let hex = s.strip_prefix('#').unwrap_or(s);
    if hex.is_empty() || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let pair = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    let chan = |i: usize| Some(f32::from(pair(i)?) / 255.0);
    match hex.len() {
        6 => Some(Color {
            r: chan(0)?,
            g: chan(2)?,
            b: chan(4)?,
            a: DEFAULT_SELECTION_COLOR.a,
        }),
        8 => Some(Color {
            r: chan(0)?,
            g: chan(2)?,
            b: chan(4)?,
            a: chan(6)?,
        }),
        _ => None,
    }
}

/// 订阅数据源:唤醒接收端的"一次性槽"(run_with 需 Hash,恒等即可)。
#[derive(Clone)]
struct NotifySlot(Arc<Mutex<Option<Receiver<()>>>>);

impl Hash for NotifySlot {
    fn hash<H: Hasher>(&self, state: &mut H) {
        0u64.hash(state);
    }
}

/// 取证(T8):闪烁相位翻转计数(仅 dev-tools)。
#[cfg(feature = "dev-tools")]
static BLINK_TOGGLES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// 读闪烁相位翻转计数(None=非 dev 构建;005 T8)。
#[cfg(feature = "dev-tools")]
pub fn blink_toggles() -> u64 {
    BLINK_TOGGLES.load(std::sync::atomic::Ordering::Relaxed)
}

pub struct App {
    pub session: PtySession,
    pub config: AppConfig,
    pub metrics: GridMetrics,
    notify_slot: Arc<Mutex<Option<Receiver<()>>>>,
    window_id: Option<iced::window::Id>,
    /// 最近一次窗口视口尺寸(dev 转储 fit_ok 用)。
    pub last_viewport: Option<Size>,

    pub frames: u64,
    pub dirty_updates: u64,
    pub started: Instant,
    pub last_byte_at: Option<Instant>,
    input_sent_at: Option<Instant>,
    /// 损伤门控快照(T6):仅在字节到达/resize/scroll 后重建。
    pub damage: Damage,
    pub snapshot: Vec<Vec<StyledChar>>,
    pub snapshot_rebuilds: u64,
    /// 光标(视口相对;Hidden=None)。随快照一并刷新。
    pub cursor: Option<(usize, usize)>,
    /// 光标形状(DECSCUSR 透传,005 T7;渲染分形用)。
    pub cursor_shape: CursorShape,
    /// 窗口聚焦(005 T8;闪烁门控之一)。初值 false:收到 winit
    /// Focused(true) 才挂闪烁(后台起窗不闪,聚焦即闪)。
    pub focused: bool,
    /// 闪烁相位(005 T8):true=光标亮;门控不满足时常亮。
    pub cursor_blink_on: bool,
    /// 当前选中区间(绝对坐标;高亮渲染用,随交互/内容变化刷新)。
    pub selection_range: Option<SelectionRange>,
    /// 拖选自动滚动(005 T4):Some(每拍行数,上缘正/下缘负)时挂
    /// 50ms 条件订阅;Finish/离缘清除(空闲零唤醒的守门员)。
    pub drag_scroll: Option<i32>,
    /// 右键菜单(005 T5):Some=开着(view 传 TermGrid 画浮层;
    /// ESC/外点关闭;开着时暂停自动滚动)。
    pub menu: Option<MenuState>,
    /// IME 挂起预编辑(空=无;不写 PTY,over-the-spot 显示)。
    pub preedit: Option<String>,

    #[cfg(feature = "dev-tools")]
    queued_input: Vec<(Instant, Vec<u8>)>,
    #[cfg(feature = "dev-tools")]
    queued_select: Option<(Instant, DevSelectSpec)>,
    /// 边缘保持注入已 Begin(防重复;005 T4)。
    #[cfg(feature = "dev-tools")]
    select_edge_holding: bool,
    #[cfg(feature = "dev-tools")]
    queued_paste: Option<(Instant, String)>,
    #[cfg(feature = "dev-tools")]
    queued_preedit: Option<(Instant, String)>,
    /// dev-menu 注入队列(None 项=关菜单;005 T5)。
    #[cfg(feature = "dev-tools")]
    queued_menu: Vec<(Instant, Option<(f32, f32)>)>,
    #[cfg(feature = "dev-tools")]
    exit_at: Option<Instant>,
}

/// dev-select 注入规格(视口相对格;经 handle_select 走真实消息路径)。
/// `edge` = 边缘保持模式(005 T4 取证):到点后每拍补发带 at_edge
/// 的 Extend,驱动自动滚动(无 Finish,转储时选区仍在)。
#[cfg(feature = "dev-tools")]
#[derive(Debug, Clone, Copy)]
struct DevSelectSpec {
    ty: SelectionType,
    start: (usize, usize),
    end: (usize, usize),
    edge: Option<Vertical>,
}

impl App {
    pub fn new(config: AppConfig, cols: usize, rows: usize) -> anyhow::Result<Self> {
        let mut session =
            PtySession::spawn(&config.shell, std::iter::empty::<&str>(), cols, rows)?;
        let notify_slot = Arc::new(Mutex::new(session.take_notify_receiver()));
        let metrics = metrics::measure();
        let now = Instant::now();
        #[cfg(feature = "dev-tools")]
        let queued_input = config
            .dev_autotype
            .iter()
            .map(|s| parse_input(s, now))
            .collect();
        #[cfg(feature = "dev-tools")]
        let queued_select = config
            .dev_select
            .as_deref()
            .and_then(|s| parse_dev_select(s, now));
        #[cfg(feature = "dev-tools")]
        let queued_paste = config
            .dev_paste
            .as_deref()
            .and_then(|s| parse_dev_paste(s, now));
        #[cfg(feature = "dev-tools")]
        let queued_preedit = config
            .dev_preedit
            .as_deref()
            .and_then(|s| parse_dev_paste(s, now));
        #[cfg(feature = "dev-tools")]
        let queued_menu: Vec<_> = config
            .dev_menu
            .iter()
            .filter_map(|s| parse_dev_menu(s, now))
            .collect();
        #[cfg(feature = "dev-tools")]
        let exit_at = (config.dev_exit_after > 0)
            .then(|| now + Duration::from_secs(config.dev_exit_after));
        Ok(Self {
            session,
            config,
            metrics,
            notify_slot,
            window_id: None,
            last_viewport: None,
            frames: 0,
            dirty_updates: 0,
            started: now,
            last_byte_at: None,
            input_sent_at: None,
            damage: Damage::Full,
            snapshot: Vec::new(),
            snapshot_rebuilds: 0,
            cursor: None,
            cursor_shape: CursorShape::Block,
            focused: false,
            cursor_blink_on: true,
            selection_range: None,
            drag_scroll: None,
            menu: None,
            preedit: None,
            #[cfg(feature = "dev-tools")]
            queued_input,
            #[cfg(feature = "dev-tools")]
            queued_select,
            #[cfg(feature = "dev-tools")]
            select_edge_holding: false,
            #[cfg(feature = "dev-tools")]
            queued_paste,
            #[cfg(feature = "dev-tools")]
            queued_preedit,
            #[cfg(feature = "dev-tools")]
            queued_menu,
            #[cfg(feature = "dev-tools")]
            exit_at,
        }
        .initialized())
    }

    pub fn title(&self) -> &str {
        "AutoTerm"
    }

    /// 构造尾拍:首帧快照就位(空网格 + Full 损伤)。
    fn initialized(mut self) -> Self {
        self.refresh_after_change();
        self
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![
            // PTY 字节唤醒:转发线程 block 在 core 的唤醒通道上,
            // try_send 进 iced 异步通道——字节到达即醒,无轮询。
            Subscription::run_with(NotifySlot(self.notify_slot.clone()), |slot| {
                let slot = slot.clone();
                stream::channel(
                    16,
                    move |mut sender: iced::futures::channel::mpsc::Sender<Message>| async move {
                        let NotifySlot(slot) = slot;
                        if let Some(rx) = slot.lock().unwrap().take() {
                            std::thread::spawn(move || {
                                loop {
                                    match rx.recv() {
                                        Ok(()) => loop {
                                            // 通道满 = UI 忙:重试而非退出
                                            // (退出会永久丢唤醒,20000 行
                                            // 突发实测暴露过)
                                            match sender.try_send(Message::PtyBytes) {
                                                Ok(()) => break,
                                                Err(e) if e.is_full() => {
                                                    std::thread::sleep(
                                                        std::time::Duration::from_millis(4),
                                                    );
                                                }
                                                Err(_) => return, // 通道关闭:app 退出
                                            }
                                        },
                                        Err(_) => return,
                                    }
                                }
                            });
                        }
                        iced::futures::future::pending::<()>().await
                    },
                )
            }),
            iced::keyboard::listen().map(|event| match event {
                iced::keyboard::Event::KeyPressed { key, modifiers, .. } => {
                    Message::Key(key, modifiers)
                }
                _ => Message::NoOp,
            }),
            iced::window::resize_events()
                .map(|(_id, size)| Message::Resized(size)),
            // 滚轮回滚:wheel 正 y = 上翻(进历史),每格 3 行
            iced::event::listen().map(|event| match event {
                iced::Event::Mouse(iced::mouse::Event::WheelScrolled {
                    delta: iced::mouse::ScrollDelta::Lines { y, .. },
                }) => Message::Scrolled((y * 3.0) as i32),
                _ => Message::NoOp,
            }),
            // 窗口关闭:同步清理子进程(T8)
            iced::window::close_events().map(Message::Closed),
            // 窗口聚焦变化(005 T8;闪烁门控)
            iced::event::listen().map(|event| match event {
                iced::Event::Window(iced::window::Event::Focused) => Message::Focused(true),
                iced::Event::Window(iced::window::Event::Unfocused) => {
                    Message::Focused(false)
                }
                _ => Message::NoOp,
            }),
        ];
        // 拖选自动滚动(005 T4):仅拖选压边时挂 50ms 条件订阅——
        // 空闲/拖选中不压边均零唤醒;菜单开着时暂停(005 T5);
        // 方向行数经 DragScrollTick 从状态取(Subscription::map 禁捕获)。
        if self.drag_scroll.is_some() && self.menu.is_none() {
            subs.push(
                time::every(Duration::from_millis(50)).map(|_| Message::DragScrollTick),
            );
        }
        // 光标闪烁(005 T8):仅聚焦 + 光标可见 + 无菜单 + 无 preedit
        // 时挂 500ms 条件订阅(空闲零唤醒:门控不满足即无定时器)。
        if self.focused
            && self.cursor_shape != CursorShape::Hidden
            && self.menu.is_none()
            && self.preedit.is_none()
        {
            subs.push(
                time::every(Duration::from_millis(500)).map(|_| Message::BlinkTick),
            );
        }
        #[cfg(feature = "dev-tools")]
        {
            if !self.config.dev_autotype.is_empty()
                || self.exit_at.is_some()
                || self.queued_select.is_some()
                || self.queued_paste.is_some()
                || self.queued_preedit.is_some()
                || !self.queued_menu.is_empty()
            {
                subs.push(
                    time::every(Duration::from_millis(50))
                        .map(|_| Message::DevTick),
                );
            }
        }
        // 默认构建:无任何定时器,空闲零唤醒
        Subscription::batch(subs)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PtyBytes => {
                // 键盘非按键事件也走到这里作空唤醒——drain 天然幂等
                self.pump();
                Task::none()
            }
            Message::NoOp => Task::none(),
            #[cfg(feature = "dev-tools")]
            Message::DevTick => {
                let now = Instant::now();
                // 到期字节先收集再处理:孤 0x03 要走 handle_ctrl_c(与
                // 真实按键同路,008 T5),retain 闭包内借不了整个 self。
                let mut due: Vec<Vec<u8>> = Vec::new();
                self.queued_input.retain(|(at, bytes)| {
                    if now >= *at {
                        due.push(bytes.clone());
                        false
                    } else {
                        true
                    }
                });
                for bytes in due {
                    if bytes.as_slice() == [0x03] {
                        self.handle_ctrl_c();
                    } else {
                        self.session.write_input(&bytes);
                    }
                    self.input_sent_at.get_or_insert_with(Instant::now);
                }
                // 注入产生的 Task(如 Finish 的 copy-on-select 剪贴板写)
                // 必须返回给 runtime 执行——丢弃即静默失效(T4 实证)
                let mut tasks: Vec<Task<Message>> = Vec::new();
                if let Some((at, spec)) = self.queued_select {
                    // 自愈注入:到点后若选中缺失(窗口首显 resize 风暴会
                    // 清选——pwsh 冷启动慢时风暴晚于注入)即重注入,
                    // 选中在手则保持(幂等,dev 专用)。
                    // 边缘保持模式(edge=Some):Begin 一次(缺失自愈),
                    // 此后每拍 Extend 压边,自动滚动持续到转储。
                    if now >= at {
                        match spec.edge {
                            Some(edge) => {
                                if !self.select_edge_holding
                                    || self.selection_range.is_none()
                                {
                                    log::info!(
                                        "dev-select edge-hold begin: ty={:?} start={:?}",
                                        spec.ty,
                                        spec.start
                                    );
                                    self.select_edge_holding = true;
                                    tasks.push(self.handle_select(SelectMsg::Begin {
                                        ty: spec.ty,
                                        cell: spec.start,
                                        side: Side::Left,
                                    }));
                                }
                                tasks.push(self.handle_select(SelectMsg::Extend {
                                    cell: spec.end,
                                    side: Side::Right,
                                    at_edge: Some(edge),
                                }));
                            }
                            None => {
                                if self.selection_range.is_none() {
                                    log::info!(
                                        "dev-select inject: ty={:?} start={:?} end={:?}",
                                        spec.ty,
                                        spec.start,
                                        spec.end
                                    );
                                    tasks.push(self.handle_select(SelectMsg::Begin {
                                        ty: spec.ty,
                                        cell: spec.start,
                                        side: Side::Left,
                                    }));
                                    tasks.push(self.handle_select(SelectMsg::Extend {
                                        cell: spec.end,
                                        side: Side::Right,
                                        at_edge: None,
                                    }));
                                    tasks.push(self.handle_select(SelectMsg::Finish));
                                    log::info!(
                                        "dev-select injected: range={:?} text={:?}",
                                        self.selection_range,
                                        self.session.term.selection_text()
                                    );
                                }
                            }
                        }
                    }
                }
                if let Some((at, text)) = self.queued_paste.clone() {
                    if now >= at {
                        self.queued_paste = None;
                        log::info!("dev-paste inject: {text:?}");
                        // 与真实粘贴(Ctrl+Shift+V/右键)同第二拍路径
                        tasks.push(self.update(Message::Pasted(text)));
                    }
                }
                if let Some((at, text)) = self.queued_preedit.clone() {
                    if now >= at {
                        self.queued_preedit = None;
                        log::info!("dev-preedit inject: {text:?}");
                        // 与真实 IME Preedit 事件同路径(挂起显示)
                        tasks.push(self.update(Message::SetPreedit(text)));
                    }
                }
                // dev-menu 注入:到期项逐个走真实 ContextMenu 路径
                let mut menu_fired: Vec<Option<(f32, f32)>> = Vec::new();
                self.queued_menu.retain(|(at, what)| {
                    if now >= *at {
                        menu_fired.push(*what);
                        false
                    } else {
                        true
                    }
                });
                for what in menu_fired {
                    match what {
                        Some(at_menu) => {
                            log::info!("dev-menu inject: open {at_menu:?}");
                            tasks.push(self.update(Message::ContextMenu(Some(at_menu))));
                        }
                        None => {
                            log::info!("dev-menu inject: close");
                            tasks.push(self.update(Message::ContextMenu(None)));
                        }
                    }
                }
                // 边缘保持模式每拍都有 Extend task,转储/退出判定必须
                // 先于 task 返回执行(否则被饿死,应用永不退出——005 T4
                // 首跑实证)
                if let Some(at) = self.exit_at {
                    if now >= at {
                        if let Some(delta) = self.config.dev_scroll {
                            self.session.term.scroll(delta);
                            self.refresh_after_change();
                        }
                        self.dump_state();
                        self.session.kill();
                        return iced::exit();
                    }
                }
                if !tasks.is_empty() {
                    return Task::batch(tasks);
                }
                Task::none()
            }
            Message::Key(key, mods) => {
                // 菜单开着时 ESC 仅关菜单(不写 PTY、不回滚;005 T5)
                if self.menu.is_some()
                    && matches!(
                        &key,
                        Key::Named(iced::keyboard::key::Named::Escape)
                    )
                {
                    self.menu = None;
                    return Task::none();
                }
                // 回滚时键入先回正(终端惯例)
                if self.session.term.display_offset() > 0 {
                    self.session.term.scroll(i32::MIN);
                    self.refresh_after_change();
                }
                // PgUp/PgDn = UI 翻页回滚(消费,不进 PTY)
                match &key {
                    Key::Named(iced::keyboard::key::Named::PageUp) => {
                        let rows = self.session.term.size().1 as i32;
                        self.session.term.scroll(rows);
                        self.refresh_after_change();
                        return Task::none();
                    }
                    Key::Named(iced::keyboard::key::Named::PageDown) => {
                        let rows = self.session.term.size().1 as i32;
                        self.session.term.scroll(-rows);
                        self.refresh_after_change();
                        return Task::none();
                    }
                    _ => {}
                }
                // Ctrl+Shift+C/V(复制/粘贴)在 key_to_bytes 前拦截
                // (否则 ctrl+c 会落 0x03 字节直写 PTY)
                match clipboard_shortcut(&key, &mods) {
                    Some(ClipboardShortcut::Copy) => return self.copy_selection(),
                    Some(ClipboardShortcut::Paste) => return self.paste_from_clipboard(),
                    None => {}
                }
                // 裸 Ctrl+C:双投递拦截(008 T5)——事件管运行中命令,
                // 0x03 字节管 raw 行编辑器;在 key_to_bytes 前截住,
                // 否则只落字节(DEBTS #12 修复前行为)。
                if is_bare_ctrl_c(&key, &mods) {
                    self.handle_ctrl_c();
                    return Task::none();
                }
                if let Some(bytes) = key_to_bytes(&key, &mods) {
                    self.session.write_input(&bytes);
                }
                Task::none()
            }
            Message::Scrolled(delta) => {
                self.session.term.scroll(delta);
                self.refresh_after_change();
                Task::none()
            }
            Message::DragScrollTick => {
                match self.drag_scroll {
                    Some(delta) => self.update(Message::Scrolled(delta)),
                    None => Task::none(),
                }
            }
            Message::Closed(_id) => {
                // 关闭语义(T8):显式杀+等,窗口关闭不留孤儿进程;
                // Drop 安全网仍在(异常路径兜底)。
                self.session.kill();
                let _ = self.session.wait();
                iced::exit()
            }
            Message::WindowId(id) => {
                self.window_id = id;
                Task::none()
            }
            Message::Select(msg) => self.handle_select(msg),
            Message::SetPreedit(text) => {
                // 挂起显示:空串=清除;不写 PTY
                self.preedit = (!text.is_empty()).then_some(text);
                Task::none()
            }
            Message::CommitIme(text) => {
                self.preedit = None;
                if !text.is_empty() {
                    self.session.write_input(text.as_bytes());
                }
                Task::none()
            }
            Message::Paste => self.paste_from_clipboard(),
            Message::ContextMenu(at) => {
                self.menu = at.map(|at| MenuState { at });
                Task::none()
            }
            Message::MenuAction(item) => {
                self.menu = None;
                match item {
                    MenuItem::Copy => self.copy_selection(),
                    MenuItem::Paste => self.paste_from_clipboard(),
                    MenuItem::SelectAll => {
                        // 视口 Lines 全选:左上 Begin、右下 Extend,
                        // 行选语义自动扩满整行(005 T5)
                        let (cols, rows) = self.session.term.size();
                        let begin = self.cell_to_point((0, 0));
                        let end = self.cell_to_point((rows - 1, cols - 1));
                        self.session
                            .term
                            .begin_selection(SelectionType::Lines, begin, Side::Left);
                        self.session.term.update_selection(end, Side::Left);
                        self.selection_range = self.session.term.selection_range();
                        Task::none()
                    }
                }
            }
            Message::Pasted(text) => {
                // \r\n / \n → \r 规整(终端行提交约定),再写 PTY
                let normalized = text.replace("\r\n", "\r").replace('\n', "\r");
                if !normalized.is_empty() {
                    self.session.write_input(normalized.as_bytes());
                }
                Task::none()
            }
            Message::Focused(focused) => {
                self.focused = focused;
                // 相位复位:聚焦即亮,失焦常亮(005 T8)
                self.cursor_blink_on = true;
                Task::none()
            }
            Message::BlinkTick => {
                self.cursor_blink_on = !self.cursor_blink_on;
                #[cfg(feature = "dev-tools")]
                BLINK_TOGGLES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Task::none()
            }
            Message::Resized(size) => {
                self.last_viewport = Some(size);
                let cols =
                    ((size.width / self.metrics.cell_w).floor() as usize).max(10);
                let rows =
                    ((size.height / self.metrics.line_h).floor() as usize).max(4);
                self.session.resize(cols, rows);
                self.refresh_after_change();
                Task::none()
            }
        }
    }

    /// 收割 PTY 字节喂仿真核心;标记重绘。
    fn pump(&mut self) {
        if self.session.drain() {
            self.last_byte_at = Some(Instant::now());
            self.dirty_updates += 1;
            self.refresh_after_change();
        }
        self.frames += 1;
    }

    /// 选中消息族处理(T2):视口格 → 绝对网格点 → 驱动 core;
    /// 高亮区间随之刷新(渲染为 overlay,不进快照/damage)。
    /// Finish = copy-on-select(用户裁定默认开;空选清除)。
    /// Extend 压边时置 drag_scroll(005 T4),离缘/收尾清除。
    fn handle_select(&mut self, msg: SelectMsg) -> Task<Message> {
        match msg {
            SelectMsg::Begin { ty, cell, side } => {
                self.drag_scroll = None;
                self.session.term.begin_selection(ty, self.cell_to_point(cell), side);
            }
            SelectMsg::Extend { cell, side, at_edge } => {
                self.session.term.update_selection(self.cell_to_point(cell), side);
                self.drag_scroll = at_edge.map(|edge| match edge {
                    Vertical::Up => AUTO_SCROLL_ROWS,
                    Vertical::Down => -AUTO_SCROLL_ROWS,
                });
            }
            SelectMsg::Finish => {
                self.drag_scroll = None;
                if let Some(text) = self
                    .session
                    .term
                    .selection_text()
                    .filter(|t| !t.is_empty())
                {
                    self.selection_range = self.session.term.selection_range();
                    return iced::clipboard::write(text);
                }
                self.session.term.clear_selection();
            }
        }
        self.selection_range = self.session.term.selection_range();
        Task::none()
    }

    /// 显式复制(Ctrl+Shift+C):有选中复制选中,否则不动。
    fn copy_selection(&self) -> Task<Message> {
        match self.session.term.selection_text() {
            Some(t) if !t.is_empty() => iced::clipboard::write(t),
            _ => Task::none(),
        }
    }

    /// 粘贴两拍之一:发起剪贴板读取(结果走 [`Message::Pasted`])。
    fn paste_from_clipboard(&self) -> Task<Message> {
        iced::clipboard::read()
            .then(|opt| Task::done(Message::Pasted(opt.unwrap_or_default())))
    }

    /// 裸 Ctrl+C 落地(008,DEBTS #12):按有效模式双投递——
    /// `interrupt()` 广播控制事件(中断运行中命令),0x03 字节供
    /// raw 行编辑器废弃输入行;两消费方不重叠,Both 为推荐默认。
    /// auto 模式下 ash 豁免为仅字节(idle 态事件会整体终止 ash,
    /// 实测见 designs/003 §4.1;#13 落地后撤豁免)。
    fn handle_ctrl_c(&mut self) {
        match resolve_effective_mode(self.config.ctrl_c_mode, &self.config.shell) {
            CtrlCMode::Byte => self.session.write_input(&[0x03]),
            CtrlCMode::Event => {
                self.session.interrupt();
            }
            CtrlCMode::Both | CtrlCMode::Auto => {
                self.session.interrupt();
                self.session.write_input(&[0x03]);
            }
        }
    }

    /// 视口格 (row, col) → core 绝对网格点(历史区为负)。
    fn cell_to_point(&self, (row, col): (usize, usize)) -> autoterm_core::Point {
        let d = self.session.term.display_offset() as i32;
        autoterm_core::Point::new(
            autoterm_core::Line(row as i32 - d),
            autoterm_core::Column(col),
        )
    }

    /// 内容变了(resize/scroll/字节到达)之后:取损伤、按需重建快照。
    /// 损伤门控(T6):快照仅在内容实际变化时重建,空闲帧免全网格遍历;
    /// 绘制级脏行剪裁在 iced 即时模式下会清空未脏行,不做(见
    /// docs/designs/001 的损伤协议节)。
    fn refresh_after_change(&mut self) {
        self.damage = self.session.term.take_damage();
        let content_changed = match &self.damage {
            Damage::Full => true,
            Damage::Lines(lines) => !lines.is_empty(),
        };
        if content_changed {
            self.snapshot = self.session.term.visible_styled_lines();
            self.snapshot_rebuilds += 1;
        }
        // 光标位置/形状每次刷新都取(DECSCUSR 只改形状不产 Wakeup,
        // 快照门控会漏;字节到达必经此处,代价一次 renderable 快照)
        self.cursor = self.session.term.cursor();
        self.cursor_shape = self.session.term.cursor_shape();
        // 选中锚定网格内容:滚动/新增行使绝对行漂移,区间随之重取
        let prev_selection = self.selection_range;
        self.selection_range = self.session.term.selection_range();
        if prev_selection.is_some() && self.selection_range.is_none() {
            log::info!(
                "selection cleared by refresh (damage={:?}, prev={:?})",
                self.damage,
                prev_selection
            );
        }
    }

    /// 光标当前是否可见(005 T8):门控(聚焦+形状可见+无菜单+
    /// 无 preedit)满足时随闪烁相位;否则常亮;Hidden 恒不可见。
    fn cursor_visible(&self) -> bool {
        if self.cursor.is_none() {
            return false;
        }
        let blinking = self.focused
            && self.cursor_shape != CursorShape::Hidden
            && self.menu.is_none()
            && self.preedit.is_none();
        !blinking || self.cursor_blink_on
    }

    pub fn view(&self) -> Element<'_, Message> {
        Element::new(TermGrid {
            lines: self.snapshot.clone(),
            metrics: self.metrics,
            damage: self.damage.clone(),
            scroll_offset: self.session.term.display_offset(),
            cursor: self.cursor,
            cursor_shape: self.cursor_shape,
            cursor_visible: self.cursor_visible(),
            selection: self.selection_range,
            preedit: self.preedit.clone(),
            menu: self.menu,
            selection_color: self.config.selection_color,
        })
    }

    /// 选中区间覆盖的格子数(块选:列带×行数矩形;非块选:行段求和;
    /// 取证 `selection_cells`)。
    fn selection_cell_count(&self) -> usize {
        let Some(r) = self.selection_range else { return 0 };
        if r.is_block {
            let rows = (r.end.line.0 - r.start.line.0 + 1) as usize;
            let cols = r.end.column.0 - r.start.column.0 + 1;
            return rows * cols;
        }
        let cols = self.session.term.size().0;
        (r.start.line.0..=r.end.line.0)
            .map(|line| {
                let begin = if line == r.start.line.0 {
                    r.start.column.0
                } else {
                    0
                };
                let last = if line == r.end.line.0 {
                    r.end.column.0
                } else {
                    cols.saturating_sub(1)
                };
                last.saturating_sub(begin) + 1
            })
            .sum()
    }

    /// dev 转储:网格文本 + 计数(取证;仅 dev-tools 构建)。
    #[cfg(feature = "dev-tools")]
    pub fn dump_state(&mut self) {
        let Some(path) = self.config.dev_dump.clone() else { return };
        let mut out = String::new();
        let (cols, rows) = self.session.term.size();
        let _ = std::fmt::Write::write_fmt(
            &mut out,
            format_args!("shell: {}\ngrid: {cols}x{rows}\n", self.config.shell),
        );
        let _ = std::fmt::Write::write_fmt(
            &mut out,
            format_args!(
                "metrics: font_px={:.3} cell_w={:.4} line_h={:.3} (cell_em={:.4})\n",
                self.metrics.font_px,
                self.metrics.cell_w,
                self.metrics.line_h,
                self.metrics.cell_w / self.metrics.font_px
            ),
        );
        match self.last_viewport {
            Some(v) => {
                let fit = (cols as f32 * self.metrics.cell_w) <= v.width + 0.5;
                let _ = std::fmt::Write::write_fmt(
                    &mut out,
                    format_args!(
                        "viewport: {:.1}x{:.1}\nfit_ok: {} (cols*cell_w={:.1} <= viewport_w={:.1})\n",
                        v.width, v.height, fit, cols as f32 * self.metrics.cell_w, v.width
                    ),
                );
            }
            None => out.push_str("viewport: unknown\nfit_ok: unknown\n"),
        }
        let _ = std::fmt::Write::write_fmt(
            &mut out,
            format_args!(
                "uptime_s: {:.3}\nframes: {}\ndirty_updates: {}\nsnapshot_rebuilds: {}\n",
                self.started.elapsed().as_secs_f64(),
                self.frames,
                self.dirty_updates,
                self.snapshot_rebuilds
            ),
        );
        match &self.damage {
            Damage::Full => out.push_str("damage_last: full\n"),
            Damage::Lines(l) => {
                let _ = std::fmt::Write::write_fmt(
                    &mut out,
                    format_args!("damage_last: lines={}\n", l.len()),
                );
            }
        }
        let _ = std::fmt::Write::write_fmt(
            &mut out,
            format_args!(
                "scroll_offset: {}\n",
                self.session.term.display_offset()
            ),
        );
        match self.session.term.selection_text() {
            Some(t) => {
                let flat = t.replace('\r', "\\r").replace('\n', "\\n");
                let _ = std::fmt::Write::write_fmt(
                    &mut out,
                    format_args!("selection_text: \"{flat}\"\n"),
                );
            }
            None => out.push_str("selection_text: none\n"),
        }
        let _ = std::fmt::Write::write_fmt(
            &mut out,
            format_args!("selection_cells: {}\n", self.selection_cell_count()),
        );
        match self.selection_range {
            Some(r) => {
                let _ = std::fmt::Write::write_fmt(
                    &mut out,
                    format_args!(
                        "selection_range: ({},{})-({},{}) is_block={} (绝对网格行;视口行=绝对+scroll_offset)\n",
                        r.start.line.0,
                        r.start.column.0,
                        r.end.line.0,
                        r.end.column.0,
                        r.is_block
                    ),
                );
            }
            None => out.push_str("selection_range: none\n"),
        }
        match &self.preedit {
            Some(t) => {
                let _ = std::fmt::Write::write_fmt(
                    &mut out,
                    format_args!("preedit: \"{t}\"\n"),
                );
            }
            None => out.push_str("preedit: none\n"),
        }
        let (ime_count, preedit_drawn) = widget::ime_requests();
        let _ = std::fmt::Write::write_fmt(
            &mut out,
            format_args!("ime_requests: {ime_count}\npreedit_drawn: {preedit_drawn}\n"),
        );
        let (rb_prev, rb_last) = widget::paragraph_rebuilds();
        let _ = std::fmt::Write::write_fmt(
            &mut out,
            format_args!(
                "paragraph_rebuilds_prev: {rb_prev}\nparagraph_rebuilds_last: {rb_last}\n"
            ),
        );
        match widget::cursor_drawn() {
            Some((row, col)) => {
                let _ = std::fmt::Write::write_fmt(
                    &mut out,
                    format_args!(
                        "cursor_drawn_at: ({row},{col}) shape={} inverted_block_only=false\n",
                        cursor_shape_name(self.session.term.cursor_shape())
                    ),
                );
            }
            None => out.push_str("cursor_drawn_at: none\n"),
        }
        let _ = std::fmt::Write::write_fmt(
            &mut out,
            format_args!(
                "cursor_shape: {}\n",
                cursor_shape_name(self.session.term.cursor_shape())
            ),
        );
        let _ = std::fmt::Write::write_fmt(
            &mut out,
            format_args!(
                "cursor_visible: {} (focused={} blink_phase={} menu={} preedit={})\n",
                self.cursor_visible(),
                self.focused,
                self.cursor_blink_on,
                self.menu.is_some(),
                self.preedit.is_some()
            ),
        );
        let _ = std::fmt::Write::write_fmt(
            &mut out,
            format_args!("blink_toggles: {}\n", blink_toggles()),
        );
        if let Some(t) = self.last_byte_at {
            let _ = std::fmt::Write::write_fmt(
                &mut out,
                format_args!("last_byte_after_start_s: {:.3}\n", (t - self.started).as_secs_f64()),
            );
        }
        if let Some(t) = self.input_sent_at {
            let _ = std::fmt::Write::write_fmt(
                &mut out,
                format_args!("input_sent_after_start_s: {:.3}\n", (t - self.started).as_secs_f64()),
            );
            if let Some(l) = self.last_byte_at {
                let _ = std::fmt::Write::write_fmt(
                    &mut out,
                    format_args!(
                        "last_byte_after_input_s: {:.3}\n",
                        (l - t).as_secs_f64()
                    ),
                );
            }
        }
        let _ = std::fmt::Write::write_fmt(
            &mut out,
            format_args!("bytes_fed: {}\n", self.session.bytes_fed()),
        );
        out.push_str("=== grid_text_begin ===\n");
        for line in self.session.term.visible_lines() {
            let _ = std::fmt::Write::write_fmt(&mut out, format_args!("{line}\n"));
        }
        out.push_str("=== grid_text_end ===\n");
        let _ = std::fs::write(&path, out);
        log::info!("dumped: {}", path.display());
    }
}

/// 光标形状的转储名(dev 取证,005 T7)。
#[cfg(feature = "dev-tools")]
fn cursor_shape_name(shape: CursorShape) -> &'static str {
    match shape {
        CursorShape::Block => "block",
        CursorShape::Underline => "underline",
        CursorShape::Beam => "beam",
        CursorShape::HollowBlock => "hollow_block",
        CursorShape::Hidden => "hidden",
    }
}

/// dev-autotype 语法:"<延迟毫秒>:<文本>";无前缀即立即。转义
/// \r \n \t \xHH(仅 dev-tools 构建)。
#[cfg(feature = "dev-tools")]
fn parse_input(s: &str, start: Instant) -> (Instant, Vec<u8>) {
    if let Some((n, rest)) = s.split_once(':') {
        if let Ok(ms) = n.parse::<u64>() {
            return (start + Duration::from_millis(ms), unescape(rest));
        }
    }
    (start, unescape(s))
}

/// dev-select 语法(T3):"<ms>:<r1>:<c1>-<r2>:<c2>"(视口相对格)。
/// T5 起支持可选类型前缀:"<ms>:<simple|semantic|lines|block>:<r1>:<c1>-..."
/// (block=块选,005 T2);后缀可选 ":up"/":down" = 边缘保持注入
/// (每拍 Extend 压边驱动自动滚动,005 T4)。
#[cfg(feature = "dev-tools")]
fn parse_dev_select(s: &str, start: Instant) -> Option<(Instant, DevSelectSpec)> {
    let (ms, rest) = s.split_once(':')?;
    let at = start + Duration::from_millis(ms.parse::<u64>().ok()?);
    // 边缘后缀先摘(取最后一个 ':'),无后缀则原样
    let (rest, edge) = match rest.rsplit_once(':') {
        Some((r, "up")) => (r, Some(Vertical::Up)),
        Some((r, "down")) => (r, Some(Vertical::Down)),
        _ => (rest, None),
    };
    let (ty, rest) = match rest.split_once(':') {
        Some(("simple", r)) => (SelectionType::Simple, r),
        Some(("semantic", r)) => (SelectionType::Semantic, r),
        Some(("lines", r)) => (SelectionType::Lines, r),
        Some(("block", r)) => (SelectionType::Block, r),
        _ => (SelectionType::Simple, rest),
    };
    let (a, b) = rest.split_once('-')?;
    let (r1, c1) = a.split_once(':')?;
    let (r2, c2) = b.split_once(':')?;
    Some((
        at,
        DevSelectSpec {
            ty,
            start: (r1.parse().ok()?, c1.parse().ok()?),
            end: (r2.parse().ok()?, c2.parse().ok()?),
            edge,
        },
    ))
}

/// dev-paste 语法:"<ms>:<文本>"(原样;粘贴内部自会做换行规整)。
#[cfg(feature = "dev-tools")]
fn parse_dev_paste(s: &str, start: Instant) -> Option<(Instant, String)> {
    let (ms, rest) = s.split_once(':')?;
    let at = start + Duration::from_millis(ms.parse::<u64>().ok()?);
    Some((at, rest.to_string()))
}

/// dev-menu 语法:"<ms>:<x>:<y>"(开,widget 本地像素)或
/// "<ms>:close"(关;005 T5)。
#[cfg(feature = "dev-tools")]
fn parse_dev_menu(s: &str, start: Instant) -> Option<(Instant, Option<(f32, f32)>)> {
    let (ms, rest) = s.split_once(':')?;
    let at = start + Duration::from_millis(ms.parse::<u64>().ok()?);
    if rest == "close" {
        return Some((at, None));
    }
    let (x, y) = rest.split_once(':')?;
    Some((at, Some((x.parse().ok()?, y.parse().ok()?))))
}

fn unescape(s: &str) -> Vec<u8> {
    let mut out = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.peek().copied() {
                Some('r') | Some('n') => {
                    chars.next();
                    out.push(b'\r');
                }
                Some('t') => {
                    chars.next();
                    out.push(b'\t');
                }
                // \xHH:两位十六进制字节(T5,Ctrl+C=\x03 等注入用)
                Some('x') => {
                    chars.next();
                    let c1 = chars.next();
                    let c2 = chars.next();
                    match (
                        c1.and_then(|c| c.to_digit(16)),
                        c2.and_then(|c| c.to_digit(16)),
                    ) {
                        (Some(h), Some(l)) => {
                            out.push((h * 16 + l) as u8);
                        }
                        _ => {
                            // 无效十六进制:原样保留(回吐已消费字符)
                            out.extend_from_slice(b"\\x");
                            for c in [c1, c2].into_iter().flatten() {
                                let mut buf = [0u8; 4];
                                out.extend_from_slice(
                                    c.encode_utf8(&mut buf).as_bytes(),
                                );
                            }
                        }
                    }
                }
                Some(other) => {
                    out.push(other as u8);
                    chars.next();
                }
                None => out.push(b'\\'),
            }
        } else {
            let mut buf = [0u8; 4];
            out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
        }
    }
    out
}

/// 剪贴板快捷键决策(纯函数,可单测;PLAN-004 T4)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardShortcut {
    Copy,
    Paste,
}

/// Ctrl+Shift+C/V → 复制/粘贴;其余组合(含 Ctrl+字母裸控制字节路径)
/// 返回 None 交回 key_to_bytes。
pub fn clipboard_shortcut(key: &Key, mods: &Modifiers) -> Option<ClipboardShortcut> {
    if !(mods.control() && mods.shift()) {
        return None;
    }
    let Key::Character(s) = key else { return None };
    let mut chars = s.chars();
    let c = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    match c.to_ascii_lowercase() {
        'c' => Some(ClipboardShortcut::Copy),
        'v' => Some(ClipboardShortcut::Paste),
        _ => None,
    }
}

/// Ctrl+C 投递模式(PLAN-008,DEBTS #12):
/// - `Byte`:仅写 0x03(修复前行为;ash 在 #13 落地前的 auto 豁免);
/// - `Event`:仅 `interrupt()` 广播控制事件;
/// - `Both`:事件 + 字节(默认推荐:事件中断运行中命令,字节供
///   raw 行编辑器废弃输入行——两消费方不重叠);
/// - `Auto`:按 shell 判定——ash(实测 idle 态事件整体终止进程,
///   见 designs/003 §4.1 矩阵)豁免为 Byte,其余 Both。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CtrlCMode {
    Auto,
    Byte,
    Event,
    Both,
}

/// 解析 `--ctrl-c-mode` 参数(大小写不敏感;非法返 None 由调用方
/// 回退默认并告警)。
pub fn parse_ctrl_c_mode(s: &str) -> Option<CtrlCMode> {
    match s.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(CtrlCMode::Auto),
        "byte" => Some(CtrlCMode::Byte),
        "event" => Some(CtrlCMode::Event),
        "both" => Some(CtrlCMode::Both),
        _ => None,
    }
}

/// 裸 Ctrl+C(Character 'c' + Ctrl 且**非** Shift;Ctrl+Shift+C 归
/// 剪贴板复制,不得劫持)。
pub fn is_bare_ctrl_c(key: &Key, mods: &Modifiers) -> bool {
    if !mods.control() || mods.shift() {
        return false;
    }
    let Key::Character(s) = key else { return false };
    let mut chars = s.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => c.to_ascii_lowercase() == 'c',
        _ => false,
    }
}

/// auto 模式落地:ash(文件名 stem 判定,#13 落地后撤)→ Byte,
/// 其余 → Both;显式模式原样透传。
pub fn resolve_effective_mode(mode: CtrlCMode, shell_program: &str) -> CtrlCMode {
    match mode {
        CtrlCMode::Auto => {
            let is_ash = std::path::Path::new(shell_program)
                .file_stem()
                .is_some_and(|stem| stem.eq_ignore_ascii_case("ash"));
            if is_ash {
                CtrlCMode::Byte
            } else {
                CtrlCMode::Both
            }
        }
        other => other,
    }
}

/// 键盘 → PTY 字节(承 spike:可打印/Enter/Backspace/Tab/方向键/
/// Home/End/Delete/PgUp/PgDn/Ctrl+字母;Shift 符号映射自理)。
pub fn key_to_bytes(key: &Key, mods: &Modifiers) -> Option<Vec<u8>> {
    const SHIFT_MAP: &[(char, char)] = &[
        ('1', '!'), ('2', '@'), ('3', '#'), ('4', '$'), ('5', '%'),
        ('6', '^'), ('7', '&'), ('8', '*'), ('9', '('), ('0', ')'),
        ('-', '_'), ('=', '+'), ('[', '{'), (']', '}'), ('\\', '|'),
        (';', ':'), ('\'', '"'), (',', '<'), ('.', '>'), ('/', '?'),
        ('`', '~'),
    ];
    match key {
        Key::Character(s) => {
            let mut c = s.chars().next()?;
            if mods.control() {
                c = c.to_ascii_lowercase();
                if c.is_ascii_lowercase() {
                    return Some(vec![(c as u8) - b'a' + 1]);
                }
                return None;
            }
            if mods.shift() && c.is_ascii_lowercase() {
                c = c.to_ascii_uppercase();
            } else if mods.shift() {
                if let Some((_, upper)) = SHIFT_MAP.iter().find(|(low, _)| *low == c) {
                    c = *upper;
                }
            }
            let mut buf = [0u8; 4];
            Some(c.encode_utf8(&mut buf).as_bytes().to_vec())
        }
        Key::Named(named) => match named {
            iced::keyboard::key::Named::Enter => Some(b"\r".to_vec()),
            iced::keyboard::key::Named::Backspace => Some(vec![0x7f]),
            iced::keyboard::key::Named::Tab => Some(b"\t".to_vec()),
            iced::keyboard::key::Named::Escape => Some(vec![0x1b]),
            iced::keyboard::key::Named::ArrowUp => Some(b"\x1b[A".to_vec()),
            iced::keyboard::key::Named::ArrowDown => Some(b"\x1b[B".to_vec()),
            iced::keyboard::key::Named::ArrowRight => Some(b"\x1b[C".to_vec()),
            iced::keyboard::key::Named::ArrowLeft => Some(b"\x1b[D".to_vec()),
            iced::keyboard::key::Named::Home => Some(b"\x1b[H".to_vec()),
            iced::keyboard::key::Named::End => Some(b"\x1b[F".to_vec()),
            iced::keyboard::key::Named::Delete => Some(b"\x1b[3~".to_vec()),
            _ => None,
        },
        _ => None,
    }
}

/// 默认前景/底色(与 widget 底色一致)。
pub const DEFAULT_FG: Color = Color::from_rgb8(0xe8, 0xe8, 0xe8);
pub const DEFAULT_BG: Color = Color::from_rgb8(0x10, 0x14, 0x18);

/// resize 换算用的度量基准(T5 实测化之前的过渡实现:
/// Consolas advance 0.5498em、行高 1.25em、字号 16px)。
pub const CELL_ADVANCE_EM: f32 = 1126.0 / 2048.0;
pub const LINE_HEIGHT_EM: f32 = 1.25;
pub const FONT_PX: f32 = 16.0;

#[cfg(test)]
mod parse_hex_color_tests {
    use super::{DEFAULT_SELECTION_COLOR, parse_hex_color};

    #[test]
    fn six_digit_takes_default_alpha() {
        let c = parse_hex_color("ff0000").expect("6 位应可解析");
        assert_eq!((c.r, c.g, c.b), (1.0, 0.0, 0.0));
        assert_eq!(c.a, DEFAULT_SELECTION_COLOR.a, "6 位维持默认 25% α");
        // # 前缀与大小写
        let c = parse_hex_color("#E8E8E8").expect("带 # 大写应可解析");
        assert_eq!((c.r, c.g, c.b), (232.0 / 255.0, 232.0 / 255.0, 232.0 / 255.0));
    }

    #[test]
    fn eight_digit_carries_alpha() {
        let c = parse_hex_color("00ff0080").expect("8 位应可解析");
        assert_eq!((c.r, c.g, c.b), (0.0, 1.0, 0.0));
        assert!((c.a - 128.0 / 255.0).abs() < 1e-6);
    }

    #[test]
    fn invalid_falls_back_to_none() {
        assert_eq!(parse_hex_color(""), None);
        assert_eq!(parse_hex_color("ff00"), None, "4 位不合法");
        assert_eq!(parse_hex_color("ff000"), None, "5 位不合法");
        assert_eq!(parse_hex_color("ff00000"), None, "7 位不合法");
        assert_eq!(parse_hex_color("zz0000"), None, "非十六进制不合法");
        assert_eq!(parse_hex_color("ff 000"), None, "空白不合法");
    }
}

#[cfg(test)]
mod unescape_tests {
    use super::unescape;

    #[test]
    fn hex_escape() {
        assert_eq!(unescape(r"\x41"), b"A");
        assert_eq!(unescape(r"\x03"), &[0x03]);
        assert_eq!(unescape(r"\x1b[A"), &[0x1b, b'[', b'A']);
        // 大小写十六进制
        assert_eq!(unescape(r"\x0D"), &[0x0d]);
        // 无效十六进制:原样保留反斜杠与 x
        assert_eq!(unescape(r"\xzz"), br"\xzz");
        // 与既有转义混用
        assert_eq!(unescape(r"a\x09b\r"), b"a\tb\r");
    }
}

#[cfg(test)]
mod clipboard_shortcut_tests {
    use super::{ClipboardShortcut, clipboard_shortcut};
    use iced::keyboard::{Key, Modifiers};

    fn key(s: &str) -> Key {
        Key::Character(s.into())
    }

    #[test]
    fn ctrl_shift_c_and_v_intercepted() {
        let mods = Modifiers::CTRL.union(Modifiers::SHIFT);
        assert_eq!(
            clipboard_shortcut(&key("c"), &mods),
            Some(ClipboardShortcut::Copy)
        );
        // Shift 产生的大写形态同样命中
        assert_eq!(
            clipboard_shortcut(&key("C"), &mods),
            Some(ClipboardShortcut::Copy)
        );
        assert_eq!(
            clipboard_shortcut(&key("v"), &mods),
            Some(ClipboardShortcut::Paste)
        );
        assert_eq!(
            clipboard_shortcut(&key("V"), &mods),
            Some(ClipboardShortcut::Paste)
        );
    }

    #[test]
    fn other_combinations_fall_through() {
        // 裸 Ctrl+C:不拦截(SIGINT 0x03 字节路径,key_to_bytes 负责)
        assert_eq!(
            clipboard_shortcut(&key("c"), &Modifiers::CTRL),
            None,
            "Ctrl+C 必须落 0x03(中断语义),不得被剪贴板劫持"
        );
        // 无修饰 / 仅 Shift / Ctrl+Shift+其他键:均不拦截
        assert_eq!(clipboard_shortcut(&key("c"), &Modifiers::empty()), None);
        assert_eq!(clipboard_shortcut(&key("c"), &Modifiers::SHIFT), None);
        assert_eq!(
            clipboard_shortcut(&key("x"), &Modifiers::CTRL.union(Modifiers::SHIFT)),
            None
        );
        // 多字符输入(死键组合等)不拦截
        assert_eq!(
            clipboard_shortcut(
                &Key::Character("ae".into()),
                &Modifiers::CTRL.union(Modifiers::SHIFT)
            ),
            None
        );
        // 非字符键不拦截
        assert_eq!(
            clipboard_shortcut(
                &Key::Named(iced::keyboard::key::Named::Enter),
                &Modifiers::CTRL.union(Modifiers::SHIFT)
            ),
            None
        );
    }
}

#[cfg(test)]
mod ctrl_c_tests {
    use super::{CtrlCMode, Key, Modifiers, is_bare_ctrl_c, parse_ctrl_c_mode, resolve_effective_mode};

    fn key_mods_test(c: &str, ctrl: bool, shift: bool) -> (Key, Modifiers) {
        let mut m = Modifiers::empty();
        if ctrl {
            m = m.union(Modifiers::CTRL);
        }
        if shift {
            m = m.union(Modifiers::SHIFT);
        }
        (Key::Character(c.into()), m)
    }

    #[test]
    fn bare_ctrl_c_detected() {
        let (k, m) = key_mods_test("c", true, false);
        assert!(is_bare_ctrl_c(&k, &m), "裸 Ctrl+C 必须命中");
    }

    #[test]
    fn ctrl_shift_c_not_bare() {
        let (k, m) = key_mods_test("c", true, true);
        assert!(!is_bare_ctrl_c(&k, &m), "Ctrl+Shift+C 是剪贴板复制,不得劫持");
    }

    #[test]
    fn plain_c_not_bare() {
        let (k, m) = key_mods_test("c", false, false);
        assert!(!is_bare_ctrl_c(&k, &m), "无 Ctrl 不拦截");
        let (k, m) = key_mods_test("d", true, false);
        assert!(!is_bare_ctrl_c(&k, &m), "Ctrl+D 不拦截");
    }

    #[test]
    fn auto_mode_exempt_ash() {
        // T4 实测 idle 态事件整体终止 ash → auto 下 ash 豁免为字节
        assert_eq!(
            resolve_effective_mode(CtrlCMode::Auto, "D:/x/auto-shell/ash/target/release/ash.exe"),
            CtrlCMode::Byte
        );
        assert_eq!(
            resolve_effective_mode(CtrlCMode::Auto, "ash.exe"),
            CtrlCMode::Byte
        );
    }

    #[test]
    fn auto_mode_full_for_others() {
        assert_eq!(resolve_effective_mode(CtrlCMode::Auto, "pwsh"), CtrlCMode::Both);
        assert_eq!(resolve_effective_mode(CtrlCMode::Auto, "cmd"), CtrlCMode::Both);
    }

    #[test]
    fn explicit_mode_passthrough() {
        for m in [CtrlCMode::Byte, CtrlCMode::Event, CtrlCMode::Both] {
            assert_eq!(resolve_effective_mode(m, "ash.exe"), m, "显式模式不判 shell");
        }
    }

    #[test]
    fn parse_accepts_four_modes() {
        assert_eq!(parse_ctrl_c_mode("auto"), Some(CtrlCMode::Auto));
        assert_eq!(parse_ctrl_c_mode("byte"), Some(CtrlCMode::Byte));
        assert_eq!(parse_ctrl_c_mode("event"), Some(CtrlCMode::Event));
        assert_eq!(parse_ctrl_c_mode("both"), Some(CtrlCMode::Both));
        assert_eq!(parse_ctrl_c_mode("AUTO"), Some(CtrlCMode::Auto), "大小写不敏感");
        assert_eq!(parse_ctrl_c_mode("nope"), None);
    }
}
