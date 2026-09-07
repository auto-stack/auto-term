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

/// PLAN-011 T1 色彩批次的注入命令(与 crates/autoterm-parity/tests/
/// parity_gate.rs 的 COLOR_PS_CMD **逐字节同串**,两处常量互为镜像,
/// 改一处必改另一处)。
///
/// 实测口径(PLAN-011):向 ConPTY 输入管写 SGR 只被 cmd 按字面回显
/// (网格零样式格),SGR 只能经**子进程输出路径**进网格——故由
/// powershell 单行命令([char]27 构造 ESC,避免反斜杠/控制字节过
/// cmd)向 stdout 输出 SGR 批次:命名前景/背景 0-15、索引 16-255
/// 抽样、RGB 前/背景,批前 `ESC[2J ESC[H` 固定行布局(批次恒为锚点
/// 行上方 6 行),末行打 PARITY_COLOR_DONE 锚点。
const COLOR_PS_CMD: &str = r#"powershell -NoProfile -Command "$e=[string][char]27;$n=[string][char]13+[string][char]10;$o=$e+'[2J'+$e+'[H';0..7|%{$o+=$e+'['+(30+$_)+'mX'};8..15|%{$o+=$e+'['+(82+$_)+'mX'};$o+=$e+'[0m'+$n;0..7|%{$o+=$e+'['+(40+$_)+'mX'};8..15|%{$o+=$e+'['+(92+$_)+'mX'};$o+=$e+'[0m'+$n;16,32,64,95,128,160,196,231,255|%{$o+=$e+'[38;5;'+$_+'mX'};$o+=$e+'[0m'+$n;16,32,64,95,128,160,196,231,255|%{$o+=$e+'[48;5;'+$_+'mX'};$o+=$e+'[0m'+$n;$o+=$e+'[38;2;255;0;128mX'+$e+'[38;2;10;20;30mX'+$e+'[38;2;0;255;0mX'+$e+'[0m'+$n;$o+=$e+'[48;2;10;20;30mX'+$e+'[48;2;200;100;50mX'+$e+'[48;2;1;2;3mX'+$e+'[0m'+$n;[Console]::Out.Write($o+'PARITY_COLOR_DONE'+$n)""#;

/// PLAN-011 T1: 逐格样式标量 dump(`STYLE <row> <col> <fg> <bg>` 协议
/// 行,fg|bg 为 kind<<24|value 编码,编码定义见 ffi.rs 模块头)。
fn dump_styles(app: &TermApp) {
    // 先收割刷新 dll 侧快照,再按行回读样式(容量 2×cols)。
    engine::engine_feed_snapshot(app.handle);
    let cols = app.cols as usize;
    let mut buf = vec![0u32; cols * 2];
    for r in 0..app.rows as usize {
        if engine::engine_row_style(app.handle, r, &mut buf) < 0 {
            break;
        }
        for (c, pair) in buf.chunks(2).enumerate() {
            println!("STYLE {r} {c} {} {}", pair[0], pair[1]);
        }
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
        // PLAN-010 T5: 复刻应用窗口标题——不再是 iced 缺省 "Auto Lang - Iced"。
        match auto_lang::ui::iced::run_app_with_title::<shell::AutoTermShell>(Some("AutoTerm")) {
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
        "selection" => {
            // PLAN-010 T7: 选中文本对拍——echo hello world 后,headless
            // 直接驱动 auto-lang 组件注册表面(terminal_registry 核心 +
            // terminal_selection_* 纯函数),Simple 选中 "hello world"
            // 行内区间,SEL 协议输出选中文本;oracle 侧以 TermSession
            // begin/update_selection 同区间选中,parity 门禁对拍。
            let mut app = TermApp::new(80, 24);
            app.send_line("echo hello world");
            wait_prompt(&mut app, 10);
            let lines: Vec<String> = app.lines.clone();

            // 组件注册表面:注册核心,喂入与组件 props 数据面一致的网格。
            let core = auto_lang::ui::terminal::terminal("at-shell", 80, 24);
            auto_lang::ui::terminal::terminal_feed(core, &lines);
            let (row, start) = lines
                .iter()
                .enumerate()
                .find_map(|(i, l)| l.find("hello world").map(|c| (i, c)))
                .expect("selection: hello world 行缺失");
            let end = start + "hello world".len() - 1;
            auto_lang::ui::terminal::terminal_selection_begin(
                core,
                auto_lang::ui::terminal::TermSelectionType::Simple,
                row,
                start,
            );
            auto_lang::ui::terminal::terminal_selection_extend(core, row, end);
            auto_lang::ui::terminal::terminal_selection_finish(core);
            let sel = auto_lang::ui::terminal::terminal_selected_text(core)
                .unwrap_or_default();

            for (i, l) in lines.iter().enumerate() {
                println!("ROW {i} {l}");
            }
            println!("SEL {sel}");
            app.dispose();
            let hit = sel.contains("hello world");
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
        "color" => {
            // PLAN-011 T1: ANSI 注入对拍(输出路径)——同一 powershell
            // 单行命令经 cmd 会话在 stdout 输出 SGR 批次(命名 0-15 /
            // 索引抽样 / RGB,批前 2J+H 固定行布局,批次恒为锚点行上方
            // 6 行)。锚点出现后 dump 全网格文本 + 逐格样式标量,门禁
            // oracle(rlib visible_styled_lines)双侧比对。
            let mut app = TermApp::new(80, 24);
            app.send_line(COLOR_PS_CMD);
            let hit = poll_for(&mut app, "PARITY_COLOR_DONE", 30);
            dump_rows(&app);
            dump_styles(&app);
            app.dispose();
            finish(hit)
        }
        _ => {
            eprintln!("unknown scenario: {name}");
            std::process::ExitCode::FAILURE
        }
    }
}
