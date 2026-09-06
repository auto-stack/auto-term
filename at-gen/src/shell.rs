//! PLAN-009 T8 手写窗口壳(**不转译**):auto-lang ui Component——
//! 挂 `terminal` 组件(View::Terminal,PLAN-009 P1 真身),每拍调
//! 转译逻辑的 `.tick()` 驱动引擎收割。Auto 侧应用状态机在
//! app_logic.rs(转译产物);本壳只做 iced 管线适配。

use auto_lang::ui::{Component, View};
use crate::app_logic::TermApp;

#[derive(Clone, Debug)]
pub enum ShellMsg {
    Tick,
    Submit(String),
}

#[derive(Debug)]
pub struct AutoTermShell {
    pub app: TermApp,
    input: String,
}

impl Default for AutoTermShell {
    fn default() -> Self {
        let mut app = TermApp::new(80, 24);
        app.send_line("echo autoterm-at shell ready");
        Self { app, input: String::new() }
    }
}

impl Component for AutoTermShell {
    type Msg = ShellMsg;

    /// 引擎输出收割节拍(iced tick 订阅)。
    fn tick_interval_ms(&self) -> Option<u32> {
        Some(50)
    }

    fn tick_msg(&self) -> Option<Self::Msg> {
        Some(ShellMsg::Tick)
    }

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            ShellMsg::Tick => self.app.tick(),
            ShellMsg::Submit(line) => self.app.send_line(line.as_str()),
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::Terminal {
            key: "at-shell".into(),
            cols: self.app.cols as u16,
            rows: self.app.rows as u16,
            lines: self.app.lines.clone(),
            scroll_offset: 0,
            preedit: None,
            on_select: None,
            on_menu: None,
            style: None,
        }
    }
}
