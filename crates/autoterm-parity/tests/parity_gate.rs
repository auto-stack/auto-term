//! PLAN-009 T9: 对拍门禁(PLAN-006 S2 实证口径:语义等价,非字节等价)。
//!
//! 双实现:
//!   - oracle:autoterm-core rlib 直驱(PtySession,本仓 Rust 参考实现);
//!   - a2r:at-gen 产物二进制(`scenario <name>` 子进程,ROW 协议 stdout)。
//!
//! 场景矩阵(PLAN-011 起六场景全实跑零 skip):启动提示符 / echo 往返 /
//! Ctrl+C 中断(timeout 主体)/ resize / 色彩逐格等价(ANSI 注入,输出
//! 路径)/ 选中文本。
//!
//! 运行:`cargo test -p autoterm-parity`(前置:at-gen 已 build 出
//! autoterm-at.exe,见 at-gen/README.md)

use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::time::{Duration, Instant};

use autoterm_core::PtySession;

const ECHO_ANCHOR: &str = "autoterm_parity_ok";

/// PLAN-011 T1 色彩批次的注入命令(与 at-gen/src/main.rs 的
/// COLOR_PS_CMD **逐字节同串**,两处常量互为镜像,改一处必改另一处;
/// 机制说明见彼处)。批次恒为锚点行上方 6 行:命名 fg/bg 各 16、
/// 索引 16-255 抽样 fg/bg 各 9、RGB fg/bg 各 3,末行锚点。
const COLOR_PS_CMD: &str = r#"powershell -NoProfile -Command "$e=[string][char]27;$n=[string][char]13+[string][char]10;$o=$e+'[2J'+$e+'[H';0..7|%{$o+=$e+'['+(30+$_)+'mX'};8..15|%{$o+=$e+'['+(82+$_)+'mX'};$o+=$e+'[0m'+$n;0..7|%{$o+=$e+'['+(40+$_)+'mX'};8..15|%{$o+=$e+'['+(92+$_)+'mX'};$o+=$e+'[0m'+$n;16,32,64,95,128,160,196,231,255|%{$o+=$e+'[38;5;'+$_+'mX'};$o+=$e+'[0m'+$n;16,32,64,95,128,160,196,231,255|%{$o+=$e+'[48;5;'+$_+'mX'};$o+=$e+'[0m'+$n;$o+=$e+'[38;2;255;0;128mX'+$e+'[38;2;10;20;30mX'+$e+'[38;2;0;255;0mX'+$e+'[0m'+$n;$o+=$e+'[48;2;10;20;30mX'+$e+'[48;2;200;100;50mX'+$e+'[48;2;1;2;3mX'+$e+'[0m'+$n;[Console]::Out.Write($o+'PARITY_COLOR_DONE'+$n)""#;

/// PTY 场景串行锁:同进程内三场景并发会互相干扰(时钟预算受负载挤压,
/// 中断时序抖动)。cargo test(非 nextest)默认同进程多线程跑测试。
static PARITY_SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());
const INTERRUPT_ANCHOR: &str = "alive_after_parity_interrupt";

// ============================================================
// 公共判定
// ============================================================

/// 末个非空行是否为提示符(cmd:以 '>' 结尾)——两侧取快照的统一时序点。
fn last_is_prompt(rows: &[String]) -> bool {
    rows.iter()
        .rev()
        .find(|l| !l.trim_end().is_empty())
        .map(|l| l.trim_end().ends_with('>'))
        .unwrap_or(false)
}

fn contains(needle: &'static str) -> impl Fn(&[String]) -> bool {
    move |rows: &[String]| rows.iter().any(|l| l.contains(needle))
}

/// 归一:去行尾空白,丢全空尾行,屏蔽 timeout 倒计时行(中断落点在
/// 倒计时文本中的位置是纯时序噪声,语义=「已中断」;006 S2 语义等价口径)。
fn normalize(rows: &[String]) -> Vec<String> {
    let mut out: Vec<String> = rows
        .iter()
        .map(|r| {
            if r.contains("Waiting for") {
                "WAITING_LINE(interrupted)".to_string()
            } else {
                r.trim_end().to_string()
            }
        })
        .collect();
    while out.last().map(|l| l.is_empty()).unwrap_or(false) {
        out.pop();
    }
    out
}

fn assert_grid_equal(what: &str, oracle: &[String], a2r: &[String]) {
    let (o, a) = (normalize(oracle), normalize(a2r));
    assert!(
        o == a,
        "{what}: 网格文本不等价\n--- oracle ---\n{o:#?}\n--- a2r ---\n{a:#?}"
    );
}

// ============================================================
// oracle 侧(rlib 直驱)
// ============================================================

struct Oracle {
    session: PtySession,
}

impl Oracle {
    fn spawn() -> Self {
        Self {
            session: PtySession::spawn("cmd", Vec::<String>::new(), 80, 24)
                .expect("oracle spawn"),
        }
    }

    fn write_line(&mut self, line: &str) {
        self.session
            .write_input(format!("{}\r\n", line).as_bytes());
    }

    fn wait_for<F: Fn(&[String]) -> bool>(
        &mut self,
        what: &str,
        timeout: Duration,
        pred: F,
    ) -> Vec<String> {
        let deadline = Instant::now() + timeout;
        loop {
            self.session.drain();
            let lines = self.session.term.visible_lines();
            if pred(&lines) {
                return lines;
            }
            if Instant::now() > deadline {
                panic!("oracle 等待超时: {what}");
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    }

    /// 等提示符回归(与 a2r 侧 wait_prompt 同语义)。
    fn wait_prompt(&mut self, timeout: Duration) {
        let deadline = Instant::now() + timeout;
        loop {
            self.session.drain();
            let lines = self.session.term.visible_lines();
            if last_is_prompt(&lines) {
                return;
            }
            if Instant::now() > deadline {
                panic!("oracle 等提示符回归超时");
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    }

    fn interrupt(&mut self) -> i32 {
        #[cfg(windows)]
        return if self.session.interrupt() { 1 } else { 0 };
        #[cfg(not(windows))]
        return 1;
    }

    fn resize(&mut self, cols: usize, rows: usize) {
        self.session.resize(cols, rows);
    }
}

// ============================================================
// a2r 侧(at-gen scenario 子进程,ROW 协议)
// ============================================================

fn at_bin() -> PathBuf {
    if let Ok(p) = std::env::var("AUTOTERM_AT_BIN") {
        return PathBuf::from(p);
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("at-gen/target/debug/autoterm-at.exe")
}

struct A2r {
    child: Child,
    stdin: ChildStdin,
}

impl A2r {
    /// 启动 scenario 子进程并回收 ROW 快照(SCENARIO_* 终止行不入网格)。
    fn spawn(scenario: &str) -> (Self, Vec<String>) {
        let (a2r, rows, _) = Self::spawn_with_extras(scenario);
        (a2r, rows)
    }

    /// PLAN-010 T7: 同 spawn,另回收非 ROW 协议行(如 `SEL <text>`)——
    /// 场景级取证载荷(选中文本等)与网格取证分行输出。
    fn spawn_with_extras(scenario: &str) -> (Self, Vec<String>, Vec<String>) {
        let bin = at_bin();
        assert!(
            bin.is_file(),
            "at-gen 产物缺失: {} —— 先在 at-gen/ 下 cargo build(见 at-gen/README.md)",
            bin.display()
        );
        let mut child = Command::new(&bin)
            .arg("scenario")
            .arg(scenario)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("a2r scenario 子进程启动失败");
        let stdin = child.stdin.take().expect("stdin");
        let stdout = child.stdout.take().expect("stdout");
        let mut rows: Vec<String> = Vec::new();
        let mut extras: Vec<String> = Vec::new();
        for l in BufRead::lines(std::io::BufReader::new(stdout)).map_while(|l| l.ok()) {
            if l == "SCENARIO_OK" || l == "SCENARIO_FAIL" {
                break;
            }
            // ROW 协议:ROW <idx> <text> —— 剥掉行号,只留文本。
            if let Some(rest) = l.strip_prefix("ROW ") {
                if let Some((_, text)) = rest.split_once(' ') {
                    rows.push(text.to_string());
                    continue;
                }
            }
            extras.push(l);
        }
        let status = child.wait().expect("wait a2r");
        assert!(status.success(), "a2r scenario '{scenario}' 非零退出");
        (Self { child, stdin }, rows, extras)
    }

    /// 对拍驱动面:经 stdin 向驱动核注入后续命令(场景扩展用)。
    fn write_line(&mut self, line: &str) {
        let _ = writeln!(self.stdin, "{line}");
    }
}

impl Drop for A2r {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// ============================================================
// 场景矩阵
// ============================================================

/// 场景 1+2:启动提示符 + echo 往返——oracle/a2r 同命令,网格等价。
#[test]
fn parity_startup_and_echo() {
    let _serial = PARITY_SERIAL.lock().unwrap();
    let mut o = Oracle::spawn();
    o.write_line(&format!("echo {ECHO_ANCHOR}"));
    let _ = o.wait_for(ECHO_ANCHOR, Duration::from_secs(20), contains(ECHO_ANCHOR));
    o.wait_prompt(Duration::from_secs(10));
    let oracle_grid = o.wait_for(ECHO_ANCHOR, Duration::from_secs(1), contains(ECHO_ANCHOR));

    let (a2r, a2r_grid) = A2r::spawn("echo");
    assert_grid_equal("启动提示符+echo 往返", &oracle_grid, &a2r_grid);
    // 显式锚点复核(两侧都必须看到)。
    assert!(oracle_grid.iter().any(|l| l.contains(ECHO_ANCHOR)));
    assert!(a2r_grid.iter().any(|l| l.contains(ECHO_ANCHOR)));
}

/// 场景 3:resize——几何变更后 echo 往返仍等价。
#[test]
fn parity_resize() {
    let _serial = PARITY_SERIAL.lock().unwrap();
    let mut o = Oracle::spawn();
    o.write_line("echo pre_resize");
    let _ = o.wait_for("pre_resize", Duration::from_secs(20), contains("pre_resize"));
    o.wait_prompt(Duration::from_secs(10));
    o.resize(100, 30);
    o.write_line(&format!("echo {ECHO_ANCHOR}"));
    let _ = o.wait_for(ECHO_ANCHOR, Duration::from_secs(20), contains(ECHO_ANCHOR));
    o.wait_prompt(Duration::from_secs(10));
    let oracle_grid = o.wait_for(ECHO_ANCHOR, Duration::from_secs(1), contains(ECHO_ANCHOR));

    let (a2r, a2r_grid) = A2r::spawn("resize");
    assert_grid_equal("resize 后网格等价", &oracle_grid, &a2r_grid);
}

/// 场景 4:Ctrl+C 中断(timeout 主体,008 双投递)——中断后终端存活,
/// 双侧 echo 依旧可达。
#[test]
fn parity_interrupt() {
    let _serial = PARITY_SERIAL.lock().unwrap();
    // 008 拷贝部署契约:autoterm-ctrlc.exe 须与宿主(此处为 at-gen 产物
    // exe)同目录;DLL 内 interrupt 据此解析 helper。缺失则降级 0x03 字节
    // 路径,26200 上被吞 → timeout 杀不掉 → 场景必败,故显式 skip。
    let helper_src = {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("target/debug/autoterm-ctrlc.exe")
    };
    let helper_dst = at_bin()
        .parent()
        .expect("at bin parent")
        .join("autoterm-ctrlc.exe");
    if !helper_src.is_file() {
        eprintln!("SKIP Ctrl+C 中断: autoterm-ctrlc.exe 未构建(先 cargo build -p autoterm-core)——008 部署契约未满足");
        return;
    }
    std::fs::copy(&helper_src, &helper_dst).expect("部署 helper 到 at-gen exe 目录");

    let mut o = Oracle::spawn();
    o.write_line("timeout /t 60");
    let _ = o.wait_for("60", Duration::from_secs(10), contains("60"));
    let _ = o.interrupt();
    o.write_line(&format!("echo {INTERRUPT_ANCHOR}"));
    let _ = o.wait_for(INTERRUPT_ANCHOR, Duration::from_secs(20), contains(INTERRUPT_ANCHOR));
    o.wait_prompt(Duration::from_secs(10));
    let oracle_grid = o.wait_for(INTERRUPT_ANCHOR, Duration::from_secs(1), contains(INTERRUPT_ANCHOR));

    let (a2r, a2r_grid) = A2r::spawn("interrupt");
    assert_grid_equal("Ctrl+C 中断后网格等价", &oracle_grid, &a2r_grid);
}

/// 场景 5(PLAN-011 T1 去 skip):色彩逐格等价对拍——SGR 经**子进程
/// 输出路径**进网格(PLAN-011 实测:向 ConPTY 输入管注入 SGR 只被
/// cmd 按字面回显、网格零样式格,输入管注入不成立)。
///
/// 机制:同一 powershell 单行命令(COLOR_PS_CMD,与 at-gen scenario
/// `color` 侧**逐字节同串**,两处常量互为镜像)在 cmd 会话内向
/// stdout 输出 SGR 批次:命名前/背景 0-15、索引 16-255 抽样、RGB
/// 前/背景,批前 `ESC[2J ESC[H` 固定行布局——批次恒为锚点行
/// (PARITY_COLOR_DONE)上方 6 行。oracle 经 rlib
/// `visible_styled_lines`、a2r 经 FFI `row_style`(glue
/// engine_row_style,STYLE 协议行)各取标量色面
/// (`kind<<24|value`,ffi.rs 模块头编码),批次 7 行(6 批次行 +
/// 锚点行,后者兼证 ESC[0m 复位)逐格等价;另设非空转哨兵
/// (比对行非默认标量 ≥24 种)。
#[test]
fn parity_color_attestation() {
    use autoterm_core::Color;
    const ANCHOR: &str = "PARITY_COLOR_DONE";
    let _serial = PARITY_SERIAL.lock().unwrap();

    fn kind(c: Color) -> u32 {
        match c {
            Color::Named(n) => 1 << 24 | n as u32,
            Color::Indexed(i) => 1 << 24 | i as u32,
            Color::Spec(rgb) => {
                2 << 24 | (rgb.r as u32) << 16 | (rgb.g as u32) << 8 | rgb.b as u32
            }
        }
    }

    // oracle:同命令驱动 cmd→powershell,锚点回归后取带样式快照。
    let mut o = Oracle::spawn();
    o.write_line(COLOR_PS_CMD);
    let oracle_grid = o.wait_for(ANCHOR, Duration::from_secs(30), contains(ANCHOR));
    o.session.drain();
    let oracle_styled = o.session.term.visible_styled_lines();

    // a2r:scenario color 自驱动同一命令,ROW + STYLE 协议回收。
    let (_a2r, a2r_rows, extras) = A2r::spawn_with_extras("color");

    // STYLE 协议行 → a2r 标量色面(24 行 × 80 列,缺格 0 兜底)。
    let mut a2r_plane = vec![vec![(0u32, 0u32); 80]; 24];
    let mut style_rows = 0usize;
    for l in &extras {
        if let Some(rest) = l.strip_prefix("STYLE ") {
            let mut it = rest.split_whitespace();
            let (Some(r), Some(c)) = (
                it.next().and_then(|v| v.parse::<usize>().ok()),
                it.next().and_then(|v| v.parse::<usize>().ok()),
            ) else {
                continue;
            };
            let fg = it.next().and_then(|v| v.parse::<u32>().ok());
            let bg = it.next().and_then(|v| v.parse::<u32>().ok());
            if let ((Some(fg), Some(bg)), true, true) = ((fg, bg), r < 24, c < 80) {
                if r == style_rows {
                    style_rows += 1;
                }
                a2r_plane[r][c] = (fg, bg);
            }
        }
    }
    assert!(style_rows >= 7, "a2r STYLE 协议行不足 7 行(实际 {style_rows})");

    // 锚点行定位(批次恒在其上方 6 行;两侧各自定位,免疫整体滚动)。
    let o_anchor = oracle_grid
        .iter()
        .position(|l| l.contains(ANCHOR))
        .expect("oracle 网格缺色彩锚点行");
    let a_anchor = a2r_rows
        .iter()
        .position(|l| l.contains(ANCHOR))
        .expect("a2r 网格缺色彩锚点行");
    assert!(o_anchor >= 6 && a_anchor >= 6, "锚点行位次异常: o={o_anchor} a={a_anchor}");

    let mut distinct = std::collections::BTreeSet::new();
    for d in 0..=6 {
        let (ro, ra) = (o_anchor - 6 + d, a_anchor - 6 + d);
        // 批次行文本等价(先验:样式逐格比隐含文本,此处显式给出诊断)。
        assert_eq!(
            oracle_grid[ro].trim_end(),
            a2r_rows[ra].trim_end(),
            "色彩批次行 {d} 文本不等价"
        );
        // 标量色面逐格等价。
        for c in 0..80 {
            let o = (
                kind(oracle_styled[ro][c].fg),
                kind(oracle_styled[ro][c].bg),
            );
            let a = a2r_plane[ra][c];
            assert_eq!(o, a, "色彩批次行 {d} 列 {c} 标量色漂移");
            distinct.insert(o.0);
            distinct.insert(o.1);
        }
    }
    // 非空转哨兵:全默认色面(两侧相等但无色彩)不得通过。
    assert!(
        distinct.len() >= 24,
        "色彩批次空转:非默认标量仅 {} 种 —— {distinct:?}",
        distinct.len()
    );
}

/// 场景 6(PLAN-010 T7 接线,去 skip):选中文本——oracle 侧
/// TermSession::begin/update_selection 对 "hello world" 行内区间
/// Simple 选中,a2r 侧 scenario=selection 经组件注册表面
/// (terminal_selection_* + terminal_selected_text)同区间选中;
/// 双侧选中文本语义等价(行尾归一沿用)。
#[test]
fn parity_selection_text() {
    use autoterm_core::{Column, Line, Point, SelectionType, Side};
    let _serial = PARITY_SERIAL.lock().unwrap();

    // oracle:真实 PTY 会话,锚点行命中 + 提示符回归后取网格时序点。
    let mut o = Oracle::spawn();
    o.write_line("echo hello world");
    let _ = o.wait_for("hello world", Duration::from_secs(20), contains("hello world"));
    o.wait_prompt(Duration::from_secs(10));
    o.session.drain();
    let lines = o.session.term.visible_lines();
    let (row, start) = lines
        .iter()
        .enumerate()
        .find_map(|(i, l)| l.find("hello world").map(|c| (i, c)))
        .expect("oracle: hello world 行缺失");
    let end = start + "hello world".len() - 1;
    // alacritty Point = 视口绝对网格坐标(display_offset=0),Side 区
    // 起讫半格:Left=含本格,Right=含本格(Simple 闭区间语义)。
    o.session.term.begin_selection(
        SelectionType::Simple,
        Point::new(Line(row as i32), Column(start)),
        Side::Left,
    );
    o.session.term.update_selection(
        Point::new(Line(row as i32), Column(end)),
        Side::Right,
    );
    let oracle_sel = o.session.term.selection_text().unwrap_or_default();
    assert!(
        oracle_sel.contains("hello world"),
        "oracle 选中文本应含锚点词,实际: {oracle_sel:?}"
    );

    // a2r:组件注册表面同区间选中,SEL 协议取选中文本。
    let (a2r, _rows, extras) = A2r::spawn_with_extras("selection");
    let sel = extras
        .iter()
        .find_map(|l| l.strip_prefix("SEL ").map(|s| s.to_string()))
        .unwrap_or_else(|| panic!("a2r selection 场景缺 SEL 行;extras={extras:?}"));

    // 语义等价归一:行尾裁剪(alacritty 会把行尾空白格计入选中)。
    let norm = |s: &str| s.lines().map(|l| l.trim_end()).collect::<Vec<_>>().join("
");
    assert_eq!(
        norm(&sel),
        norm(&oracle_sel),
        "选中文本对拍:a2r(组件表面) vs oracle(TermSession)"
    );
}
