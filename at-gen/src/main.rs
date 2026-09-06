//! PLAN-009 T8/T9: Auto 复刻应用入口。
//!   默认:窗口壳(terminal 组件真渲染,GUI 冒烟 T10 消费);
//!   `scenario echo`:headless 对拍驱动(T9 门禁消费)——转译逻辑
//!   驱动引擎,网格文本按 `ROW <n> <text>` 协议打 stdout。

#[allow(dead_code)]
mod app_logic;
mod engine;
mod shell;

use app_logic::TermApp;

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("scenario") {
        return run_scenario(args.get(2).map(String::as_str).unwrap_or("echo"));
    }
    #[cfg(feature = "ui-iced")]
    {
        match auto_lang::ui::HostBackend::Iced.run::<shell::AutoTermShell>() {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("app error: {e}");
                std::process::ExitCode::FAILURE
            }
        }
    }
    #[cfg(not(feature = "ui-iced"))]
    {
        eprintln!("GUI requires the ui-iced feature of auto-lang");
        std::process::ExitCode::FAILURE
    }
}

/// T9 a2r 侧驱动核:同转译逻辑 headless 跑 echo 场景,stdout 打网格。
fn run_scenario(name: &str) -> std::process::ExitCode {
    match name {
        "echo" => {
            let mut app = TermApp::new(80, 24);
            app.send_line("echo autoterm_parity_ok");
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
            let mut hit = false;
            while std::time::Instant::now() < deadline {
                app.tick();
                if app.lines.iter().any(|l| l.contains("autoterm_parity_ok")) {
                    hit = true;
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(250));
            }
            for (i, l) in app.lines.iter().enumerate() {
                println!("ROW {i} {l}");
            }
            app.dispose();
            if hit {
                println!("SCENARIO_OK");
                std::process::ExitCode::SUCCESS
            } else {
                println!("SCENARIO_FAIL");
                std::process::ExitCode::FAILURE
            }
        }
        _ => {
            eprintln!("unknown scenario: {name}");
            std::process::ExitCode::FAILURE
        }
    }
}
