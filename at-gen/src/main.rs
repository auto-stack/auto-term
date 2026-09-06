//! PLAN-009 T8/T9: Auto 复刻应用入口。
//!   默认:窗口壳(terminal 组件真渲染,GUI 冒烟 T10 消费);
//!   `scenario echo`:headless 对拍驱动(T9 门禁消费)——转译逻辑
//!   驱动引擎,网格文本按 `ROW <n> <text>` 协议打 stdout。

#[allow(dead_code)]
mod app_logic;
mod engine;
mod shell;

use app_logic::TermApp;
use auto_lang::ui::Component;

/// T10 程序化 UI 冒烟:构造真实窗口壳组件(terminal 组件挂载),驱动
/// 引擎至锚点回显,经 headless 管线(view_to_vtree)dump 视图树取证。
/// 不开事件循环——GUI 真窗口归交互冒烟/截图补充。
fn ui_smoke() -> std::process::ExitCode {
    use auto_lang::ui::vnode_converter::view_to_vtree;

    let mut shell = shell::AutoTermShell::default();
    shell.app.send_line("echo autoterm_smoke_ok");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while std::time::Instant::now() < deadline {
        shell.on(shell::ShellMsg::Tick);
        if shell.app.lines.iter().any(|l| l.contains("autoterm_smoke_ok")) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
    let view = shell.view();
    let vtree = view_to_vtree(view);
    let dump = format!("{vtree:?}");
    // 取证判据:terminal 节点挂载(几何可见)AND 引擎回显进入喂入行
    // (组件 props 数据面)。
    let node = dump.contains("terminal key=at-shell cols=80 rows=24 lines=24");
    let echo = shell.app.lines.iter().any(|l| l.contains("autoterm_smoke_ok"));
    println!("{dump}");
    println!("terminal-node-mounted: {node}");
    println!("echo-in-fed-lines: {echo}");
    let hit = node && echo;
    println!(
        "UI_SMOKE_{}",
        if hit { "OK (terminal 组件挂载 + 引擎回显入 props)" } else { "FAIL" }
    );
    if hit {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    }
}

fn wait_for(app: &mut TermApp, needle: &str, secs: u64) -> bool {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(secs);
    while std::time::Instant::now() < deadline {
        app.tick();
        if app.lines.iter().any(|l| l.contains(needle)) {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
    false
}

fn poll_for(app: &mut TermApp, needle: &str, secs: u64) -> bool {
    wait_for(app, needle, secs)
}

/// 等提示符回归(末个非空行以 '>' 结尾)——与 oracle 侧统一的取快照时序点。
fn wait_prompt(app: &mut TermApp, secs: u64) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(secs);
    loop {
        app.tick();
        if app
            .lines
            .iter()
            .rev()
            .find(|l| !l.trim_end().is_empty())
            .map(|l| l.trim_end().ends_with('>'))
            .unwrap_or(false)
        {
            return;
        }
        if std::time::Instant::now() > deadline {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
}

fn dump_rows(app: &TermApp) {
    for (i, l) in app.lines.iter().enumerate() {
        println!("ROW {i} {l}");
    }
}

fn finish(hit: bool) -> std::process::ExitCode {
    if hit {
        println!("SCENARIO_OK");
        std::process::ExitCode::SUCCESS
    } else {
        println!("SCENARIO_FAIL");
        std::process::ExitCode::FAILURE
    }
}

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("scenario") {
        return run_scenario(args.get(2).map(String::as_str).unwrap_or("echo"));
    }
    if args.get(1).map(String::as_str) == Some("smoke") {
        return ui_smoke();
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
            let hit = poll_for(&mut app, "autoterm_parity_ok", 20);
            wait_prompt(&mut app, 10);
            dump_rows(&app);
            app.dispose();
            finish(hit)
        }
        "resize" => {
            // T9: pre → resize(100x30) → parity echo;几何变更后等价。
            let mut app = TermApp::new(80, 24);
            app.send_line("echo pre_resize");
            wait_for(&mut app, "pre_resize", 20);
            app.resize(100, 30);
            app.send_line("echo autoterm_parity_ok");
            let hit = poll_for(&mut app, "autoterm_parity_ok", 20);
            wait_prompt(&mut app, 10);
            dump_rows(&app);
            app.dispose();
            finish(hit)
        }
        "interrupt" => {
            // T9: timeout 主体 → interrupt(008)→ 存活 echo。
            let mut app = TermApp::new(80, 24);
            app.send_line("timeout /t 60");
            wait_for(&mut app, "60", 10);
            let _ = engine::engine_interrupt(app.handle);
            app.send_line("echo alive_after_parity_interrupt");
            let hit = poll_for(&mut app, "alive_after_parity_interrupt", 20);
            wait_prompt(&mut app, 10);
            dump_rows(&app);
            app.dispose();
            finish(hit)
        }
        _ => {
            eprintln!("unknown scenario: {name}");
            std::process::ExitCode::FAILURE
        }
    }
}
