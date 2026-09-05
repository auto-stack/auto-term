//! autoterm — AutoTerm 单窗口终端入口(PLAN-002 T4)
//!
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::Parser;
use iced::{Font, Size, Task};

use autoterm_ui::{App, AppConfig, Message};

#[derive(Parser, Debug)]
#[command(name = "autoterm", about = "AutoTerm — AutoOS 通用终端")]
struct Args {
    /// shell 可执行文件(默认 pwsh)
    #[arg(long, default_value = "pwsh")]
    shell: String,

    /// [dev 取证] 自动键入("<延迟毫秒>:<文本>",可多段;转义同 unescape)
    #[arg(long = "dev-autotype")]
    #[cfg(feature = "dev-tools")]
    dev_autotype: Vec<String>,

    /// [dev 取证] 到时注入拖选("<ms>:<r1>:<c1>-<r2>:<c2>",
    /// 可选类型前缀 simple/semantic/lines)
    #[arg(long = "dev-select")]
    #[cfg(feature = "dev-tools")]
    dev_select: Option<String>,

    /// [dev 取证] 到时注入粘贴("<ms>:<文本>",走 Pasted 真实路径)
    #[arg(long = "dev-paste")]
    #[cfg(feature = "dev-tools")]
    dev_paste: Option<String>,

    /// [dev 取证] 到时注入 IME 预编辑("<ms>:<文本>",走 SetPreedit 路径)
    #[arg(long = "dev-preedit")]
    #[cfg(feature = "dev-tools")]
    dev_preedit: Option<String>,

    /// [dev 取证] 到时转储并退出的秒数(0 = 不自动退出)
    #[arg(long, default_value = "0")]
    #[cfg(feature = "dev-tools")]
    dev_exit_after: u64,

    /// [dev 取证] 退出前回滚行数(正=上翻历史;转储回滚后视图)
    #[arg(long, allow_hyphen_values = true)]
    #[cfg(feature = "dev-tools")]
    dev_scroll: Option<i32>,

    /// [dev 取证] 退出时转储网格与指标到该文件
    #[arg(long = "dev-dump")]
    #[cfg(feature = "dev-tools")]
    dev_dump: Option<std::path::PathBuf>,
}

fn main() -> Result<()> {
    #[cfg(feature = "dev-tools")]
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args = Args::parse();
    let config = AppConfig {
        shell: args.shell.clone(),
        #[cfg(feature = "dev-tools")]
        dev_autotype: args.dev_autotype,
        #[cfg(feature = "dev-tools")]
        dev_select: args.dev_select,
        #[cfg(feature = "dev-tools")]
        dev_paste: args.dev_paste,
        #[cfg(feature = "dev-tools")]
        dev_preedit: args.dev_preedit,
        #[cfg(feature = "dev-tools")]
        dev_exit_after: args.dev_exit_after,
        #[cfg(feature = "dev-tools")]
        dev_scroll: args.dev_scroll,
        #[cfg(feature = "dev-tools")]
        dev_dump: args.dev_dump,
    };

    let window = iced::window::Settings {
        size: Size::new(1000.0, 650.0),
        ..Default::default()
    };

    iced::application(
        move || {
            let app = App::new(config.clone(), 110, 36).expect("autoterm init");
            (app, Task::batch([iced::window::oldest().map(Message::WindowId)]))
        },
        App::update,
        App::view,
    )
    .title(|app: &App| app.title().to_string())
    .window(window)
    .default_font(Font::MONOSPACE)
    .subscription(App::subscription)
    .run()
    .map_err(|e| anyhow::anyhow!("iced run: {e:?}"))?;
    Ok(())
}
