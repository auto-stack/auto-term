//! PLAN-009 T9: 对拍门禁(PLAN-006 S2 实证口径:语义等价,非字节等价)。
//!
//! 双实现:
//!   - oracle:autoterm-core rlib 直驱(PtySession,本仓 Rust 参考实现);
//!   - a2r:at-gen 产物二进制(`scenario <name>` 子进程,ROW 协议 stdout)。
//!
//! 场景矩阵(007/008 门禁子集):启动提示符 / echo 往返 / Ctrl+C 中断
//! (timeout 主体)/ resize;色彩自证与选中文本显式 skip(ash 缺席 /
//! UI 级取证归 T10)。
//!
//! 运行:`cargo test -p autoterm-parity`(前置:at-gen 已 build 出
//! autoterm-at.exe,见 at-gen/README.md)

use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::time::{Duration, Instant};

use autoterm_core::PtySession;

const ECHO_ANCHOR: &str = "autoterm_parity_ok";

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
        let rows: Vec<String> = BufRead::lines(std::io::BufReader::new(stdout))
            .map_while(|l| l.ok())
            .take_while(|l| l != "SCENARIO_OK" && l != "SCENARIO_FAIL")
            .filter_map(|l| {
                // ROW 协议:ROW <idx> <text> —— 剥掉行号,只留文本。
                l.strip_prefix("ROW ")
                    .and_then(|rest| rest.split_once(' '))
                    .map(|(_, text)| text.to_string())
            })
            .collect();
        let status = child.wait().expect("wait a2r");
        assert!(status.success(), "a2r scenario '{scenario}' 非零退出");
        (Self { child, stdin }, rows)
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

/// 场景 5(显式 skip):色彩自证——ash 侧事务,ash 缺席即 skip
/// (007/008 惯例;AUTOTERM_ASH_BIN 可指路)。
#[test]
fn parity_color_attestation() {
    let ash = std::env::var("AUTOTERM_ASH_BIN")
        .ok()
        .filter(|p| PathBuf::from(p).is_file());
    if ash.is_none() {
        eprintln!("SKIP 色彩自证: ash 缺席(AUTOTERM_ASH_BIN 未指路)——007/008 显式 skip 惯例");
        return;
    }
    panic!("ash 在案但色彩对拍场景未实现——本相位留白,见 PLAN-009 待澄清4");
}

/// 场景 6(显式 skip):选中文本——UI 级取证归 T10 冒烟。
#[test]
fn parity_selection_text() {
    eprintln!("SKIP 选中文本: UI 级取证归 T10(组件选中文本 headless 面待接)");
}
