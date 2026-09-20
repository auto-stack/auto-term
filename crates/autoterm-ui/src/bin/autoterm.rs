//! autoterm — AutoTerm 单窗口终端入口(PLAN-002 T4)
//!
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::Parser;
use iced::{Font, Size, Task};

use autoterm_ui::{
    App, AppConfig, CtrlCMode, Message, DEFAULT_SELECTION_COLOR, parse_ctrl_c_mode, parse_hex_color,
};

#[derive(Parser, Debug)]
#[command(name = "autoterm", about = "AutoTerm — AutoOS 通用终端")]
struct Args {
    /// shell 可执行文件(显式覆盖 --profile 与配置文件;缺省见
    /// --profile 解析链:显式 --shell > --profile > config.toml
    /// default_profile > pwsh,PLAN-025 T-09)
    #[arg(long)]
    shell: Option<String>,

    /// 配置文件 profile 名(config.toml;未命中报错退出 1;空串 =
    /// default_profile)
    #[arg(long)]
    profile: Option<String>,

    /// 选中高亮色(RRGGBB[AA] 十六进制;非法回退默认 e8e8e8@25%)
    #[arg(long = "selection-color", default_value = "e8e8e840")]
    selection_color: String,

    /// Ctrl+C 投递模式(auto/byte/event/both;默认 auto——ash 豁免为
    /// 字节,其余双投递;008,详见 designs/003 §4.1)
    #[arg(long = "ctrl-c-mode", default_value = "auto")]
    ctrl_c_mode: String,

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

    /// [dev 取证] 到时注入右键菜单("<ms>:<x>:<y>",widget 本地像素)
    #[arg(long = "dev-menu")]
    #[cfg(feature = "dev-tools")]
    dev_menu: Vec<String>,
}

fn main() -> Result<()> {
    #[cfg(feature = "dev-tools")]
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args = Args::parse();
    // PLAN-025 T-09 spawn 解析链:显式 --shell > --profile(未命中 =
    // 报错退出 1,信息含配置搜索路径)> 配置 default_profile > 既有
    // pwsh 缺省(COMSPEC 兜底属 app 轨 spawn 面,CLI 维持兼容缺省)。
    let file_config = autoterm_config::Config::load_default();
    let (program, argv, cwd): (String, Vec<String>, Option<String>) = if let Some(shell) = &args.shell {
        (shell.clone(), Vec::new(), None)
    } else {
        let wanted = args.profile.as_deref();
        let profile = match wanted {
            Some(name) if !name.is_empty() => file_config.profile(name),
            // 空串或未给 = default_profile。
            _ => file_config.default_profile(),
        };
        match profile {
            Some(p) => {
                let spec = autoterm_config::Config::spawn_spec(p);
                let cwd = if spec.cwd.is_empty() { None } else { Some(spec.cwd) };
                (spec.program, spec.argv, cwd)
            }
            None => {
                if args.profile.is_some() {
                    eprintln!(
                        "错误:--profile {:?} 未命中任何 profile(配置搜索路径:{})",
                        args.profile.as_deref().unwrap_or(""),
                        autoterm_config::default_path()
                            .map(|p| p.display().to_string())
                            .unwrap_or_else(|| "<exe 目录不可得>".to_owned())
                    );
                    std::process::exit(1);
                }
                ("pwsh".to_owned(), Vec::new(), None)
            }
        }
    };
    let config = AppConfig {
        shell: program,
        argv,
        cwd,
        selection_color: parse_hex_color(&args.selection_color)
            .unwrap_or(DEFAULT_SELECTION_COLOR),
        ctrl_c_mode: parse_ctrl_c_mode(&args.ctrl_c_mode).unwrap_or_else(|| {
            eprintln!(
                "非法 --ctrl-c-mode {:?}(可用 auto|byte|event|both),回退 auto",
                args.ctrl_c_mode
            );
            CtrlCMode::Auto
        }),
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
        #[cfg(feature = "dev-tools")]
        dev_menu: args.dev_menu,
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
