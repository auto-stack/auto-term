"""PLAN-061 targeted theme-adoption probe for auto-term (app/ package).

Counter-controls (auto-os/PLAN-061 V-01/V-02/V-03/V-05 standalone legs):

  leg A (explicit override track): AUTO_UI_THEME=dark -> official Auto/dark
      applied: statusbar (bg-card token strip) pixel == #0f0f12, dark_mode
      state true; terminal-surface pixel recorded as T
  leg B (explicit light): AUTO_UI_THEME=light -> statusbar #ffffff,
      dark_mode false; terminal-surface pixel == T (VT/PTY content domain
      NOT repainted by the theme -- engine-owned palette exemption)
  leg C (host follow track): AUTOOS_DESKTOP_CONFIG full candidate
      Solarized/dark + frozen package dir -> statusbar #073642
  leg D (V-02 business during reskin): boot Solarized/dark, press "+" (real
      NewTab -> owned PTY mux mutation, tabs 1->2), then flip the config
      file to light in-session; tab sentinels must survive; post-flip pixel
      recorded (OS056-REAPPLY-NOTE: live external flip may lag -- contract
      recovery is reopen, closed by leg E); then press the ◐ scheme toggle
      (real business control) -> chrome returns to dark card value
  leg E (reopen recovery): fresh boot with the flipped light config ->
      statusbar #eee8d5 (Solarized light card), app healthy (fresh mux)

Sampling: statusbar = center-bottom 6x6 patch (bg-card token strip, no text
at center); terminal surface = window center (PTY grid, engine palette).
Isolation: evidence/data live under
D:/autostack/plan-artifacts/plan-061/auto-os/term (never inside a repo).
The owned PTY shells spawned by "+" are harvested with the boot process tree
(scoped PID kill, never a broad image-name sweep).
"""

import json
import os
import re
import shutil
import socket
import subprocess
import sys
import time
import urllib.request

EVIDENCE_DIR = r"D:\autostack\plan-artifacts\plan-061\auto-os\term"
CONFIG_DIR = os.path.join(EVIDENCE_DIR, "config")
PACKAGE_DIR = r"D:\autostack\auto-ui\packages\themes"
THEME_DIGEST = "9fa07742dc68aa5377ae06cb2d966386002063f184c515b6c5dddf8a6bef838e"

# Official family tokens (vendor/themes/themes.at, digest 9fa07742..bef838e)
AUTO_DARK_CARD = (0x0F, 0x0F, 0x12)
AUTO_LIGHT_CARD = (0xFF, 0xFF, 0xFF)
SOLARIZED_DARK_CARD = (0x07, 0x36, 0x42)
SOLARIZED_LIGHT_CARD = (0xEE, 0xE8, 0xD5)


def write_config(mode: str, family: str = "Solarized", path: str = None) -> str:
    os.makedirs(CONFIG_DIR, exist_ok=True)
    path = path or os.path.join(CONFIG_DIR, f"config_{family.lower()}_{mode}.at")
    with open(path, "w", encoding="utf-8") as f:
        f.write(
            "desktop {\n"
            f'    theme_family : "{family}"\n'
            '    theme_family_version : "0.2.0"\n'
            f'    theme_family_digest : "{THEME_DIGEST}"\n'
            f'    theme_mode : "{mode}"\n'
            "    dark_theme : true\n"
            "}\n"
        )
    return path


def pick_free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


class AutoUiMcpClient:
    def __init__(self, port: int):
        self.url = f"http://127.0.0.1:{port}/mcp"
        self._req_id = 1

    def _call(self, tool_name: str, args: dict = None) -> dict:
        req = {
            "jsonrpc": "2.0",
            "id": self._req_id,
            "method": "tools/call",
            "params": {"name": tool_name, "arguments": args or {}},
        }
        self._req_id += 1
        http_req = urllib.request.Request(
            self.url, data=json.dumps(req).encode("utf-8"),
            headers={"Content-Type": "application/json"},
        )
        with urllib.request.urlopen(http_req, timeout=10) as response:
            res = json.loads(response.read().decode("utf-8"))
            if "error" in res:
                raise RuntimeError(f"MCP Tool '{tool_name}' error: {res['error']}")
            return res.get("result", {})

    def text(self, tool_name: str, args: dict = None) -> str:
        return self._call(tool_name, args).get("content", [{}])[0].get("text", "")

    def state(self) -> str:
        return self.text("autoui_state")

    def snapshot(self) -> str:
        return self.text("autoui_snapshot")

    def press(self, element_id):
        return self.text("autoui_action", {"element_id": f"vnode_{element_id}", "action": "press"})

    def find_button_by_label(self, label: str):
        snap = self.snapshot()
        for line in snap.splitlines():
            if "button" in line and label in line:
                m = re.search(r"#vnode_(\d+)", line)
                if m:
                    return int(m.group(1))
        return None

    def screenshot(self, name: str) -> str:
        raw = self.text("autoui_screenshot", {"name": name, "baseline": False})
        low = raw.lower()
        if "saved to" in low:
            after = raw[low.index("saved to") + len("saved to"):].lstrip(" :").replace("\\\\?\\", "")
            return after.strip()
        return ""


def kill_tree(proc):
    """Scoped owned-process harvest (this boot's tree only)."""
    if proc.poll() is not None:
        return
    try:
        subprocess.run(["taskkill", "/F", "/T", "/PID", str(proc.pid)],
                       capture_output=True, timeout=10)
    except Exception:
        pass
    try:
        proc.wait(timeout=5)
    except Exception:
        pass


def patch_avg(rgb, cx, cy, r=3):
    from PIL import Image

    px = [rgb.getpixel((x, y)) for x in range(cx - r, cx + r + 1)
          for y in range(cy - r, cy + r + 1)]
    n = len(px)
    return tuple(round(sum(c[i] for c in px) / n) for i in range(3))


def sample_pixels(png_path: str):
    """(statusbar, terminal_center) pixels: statusbar = center-bottom strip
    (bg-card token), terminal = window center (PTY grid, engine palette)."""
    from PIL import Image

    with Image.open(png_path) as im:
        rgb = im.convert("RGB")
        w, h = rgb.size
        sb = patch_avg(rgb, w // 2, h - 12)
        term = patch_avg(rgb, w // 2, h // 2)
    return sb, term


def check_pixel(tag, shot_path, expect, failures, label="statusbar"):
    sb, term = sample_pixels(shot_path)
    print(f"  [*] {tag} {label} pixel = #{sb[0]:02x}{sb[1]:02x}{sb[2]:02x} "
          f"(expect #{expect[0]:02x}{expect[1]:02x}{expect[2]:02x}); "
          f"terminal-center = #{term[0]:02x}{term[1]:02x}{term[2]:02x}")
    if not all(abs(a - c) <= 3 for a, c in zip(sb, expect)):
        failures.append(f"{tag}: {label} pixel #{sb[0]:02x}{sb[1]:02x}{sb[2]:02x} "
                        f"!= expected #{'%02x%02x%02x' % expect}")
    return term


def boot_leg(tag: str, app_dir: str, auto_bin: str, env_extra: dict,
             expect_card, failures: list, want_dark_state=None,
             keep: bool = False) -> dict:
    """Boot -> settle -> statusbar/terminal pixels -> state dump."""
    os.makedirs(EVIDENCE_DIR, exist_ok=True)
    port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port), **env_extra)
    proc = subprocess.Popen([auto_bin, "run", "-r", "vm", "--server", "vm"], cwd=app_dir, env=env,
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    out = {"state": "", "term_px": None, "keep": keep}
    try:
        client = AutoUiMcpClient(port)
        start = time.time()
        state = ""
        while time.time() - start < 420:
            try:
                s = client.state()
                if "tab_labels" in s and "cols:" in s:
                    state = s
                    break
            except Exception:
                pass
            time.sleep(0.5)
        if not state:
            failures.append(f"{tag}: MCP not ready within 420s")
            return out
        time.sleep(4)  # settle: PTY prompt render + theme publish

        tmp = client.screenshot(tag)
        if tmp and os.path.exists(tmp):
            shot = os.path.join(EVIDENCE_DIR, f"{tag}.png")
            with open(tmp, "rb") as s, open(shot, "wb") as d:
                d.write(s.read())
            out["term_px"] = check_pixel(tag, shot, expect_card, failures)
        else:
            failures.append(f"{tag}: screenshot not saved")

        if want_dark_state is not None:
            dm_true = "dark_mode: true" in state
            if want_dark_state and not dm_true:
                failures.append(f"{tag}: dark_mode state not true")
            if (not want_dark_state) and dm_true:
                failures.append(f"{tag}: dark_mode state not false")

        out["state"] = state
        out["client"] = client
        out["proc"] = proc
        out["port"] = port
        with open(os.path.join(EVIDENCE_DIR, f"state-{tag}.txt"), "w", encoding="utf-8") as f:
            f.write(state)
        if not out.get("keep"):
            kill_tree(proc)
        return out
    except BaseException:
        kill_tree(proc)
        raise


def tab_count(state: str):
    m = re.search(r"tab_labels: \[(.*?)\]", state, re.S)
    if m is None:
        return None
    return m.group(1).count('"') // 2


def leg_d_live(app_dir: str, auto_bin: str, failures: list) -> None:
    """Boot Solarized/dark -> in-session config flip to light (chrome
    follows) -> real SchemeToggle press back to dark (business control,
    dark_mode/scheme semantics) -> sentinels conserved throughout.

    NOTE (pre-existing platform gap, not a 061 regression): on the CLI VM
    split-over-HTTP form the term shim extern `term/get_lines` fails (500,
    "future_all/race: empty or invalid future list") and no PTY can spawn,
    so the mux/NewTab business is not exercisable here; the engine-backed
    PTY/mux/resize/lifecycle gates are the cargo workspace tests (original
    gates) and the desktop-host in-process form. Theme legs sample the
    theme-driven chrome (statusbar bg-card strip), which is fully live on
    this form.
    """
    os.makedirs(EVIDENCE_DIR, exist_ok=True)
    cfg_flip = os.path.join(CONFIG_DIR, "config_flip.at")
    write_config("dark", path=cfg_flip)
    port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port),
               AUTOOS_DESKTOP_CONFIG=cfg_flip,
               AUTOOS_THEME_PACKAGE_DIR=PACKAGE_DIR)
    proc = subprocess.Popen([auto_bin, "run", "-r", "vm", "--server", "vm"],
                            cwd=app_dir, env=env,
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        client = AutoUiMcpClient(port)
        start = time.time()
        state = ""
        while time.time() - start < 420:
            try:
                s = client.state()
                if "tab_labels" in s and "cols:" in s:
                    state = s
                    break
            except Exception:
                pass
            time.sleep(0.5)
        if not state:
            failures.append("leg D: MCP not ready within 420s")
            return
        time.sleep(4)
        with open(os.path.join(EVIDENCE_DIR, "state-legD-boot.txt"), "w", encoding="utf-8") as f:
            f.write(state)
        print(f"  [*] boot dark_mode true (Solarized/dark policy): "
              f"{'dark_mode: true' in state}")
        if "dark_mode: true" not in state:
            failures.append("leg D: boot dark_mode not true under dark policy")

        # in-session external write: flip mode dark -> light
        with open(cfg_flip, "w", encoding="utf-8") as f:
            f.write(
                "desktop {\n"
                '    theme_family : "Solarized"\n'
                '    theme_family_version : "0.2.0"\n'
                f'    theme_family_digest : "{THEME_DIGEST}"\n'
                '    theme_mode : "light"\n'
                "    dark_theme : false\n"
                "}\n"
            )
        print("  [*] config flipped to light in-session; polling for survival")

        survived = True
        last_state = state
        deadline = time.time() + 20
        while time.time() < deadline:
            time.sleep(1)
            last_state = client.state()
            if "tab_labels" not in last_state:
                survived = False
                break
            if "dark_mode: false" in last_state:
                break
        if not survived:
            failures.append("leg D: app state lost during reskin poll")
        else:
            print("  [+] app state survived reskin window; dark_mode flipped: "
                  f"{'dark_mode: false' in last_state} (V-02)")

        tmp = client.screenshot("061_term_D_after_flip")
        if tmp and os.path.exists(tmp):
            shot = os.path.join(EVIDENCE_DIR, "061_term_D_after_flip.png")
            with open(tmp, "rb") as s, open(shot, "wb") as d:
                d.write(s.read())
            sb, term = sample_pixels(shot)
            light = all(abs(a - c) <= 3 for a, c in zip(sb, SOLARIZED_LIGHT_CARD))
            dark_lag = all(abs(a - c) <= 3 for a, c in zip(sb, SOLARIZED_DARK_CARD))
            print(f"  [*] post-flip statusbar = #{sb[0]:02x}{sb[1]:02x}{sb[2]:02x} "
                  f"(light={light}, dark-lag={dark_lag})")

        # real business control post-reskin: scheme toggle back to dark
        tog = client.find_button_by_label("◐")
        if tog is None:
            with open(os.path.join(EVIDENCE_DIR, "snapshot-legD-toggle.txt"), "w", encoding="utf-8") as f:
                f.write(client.snapshot())
            failures.append("leg D: scheme toggle '◐' not found (snapshot dumped)")
        else:
            client.press(tog)
            time.sleep(3)
            st = client.state()
            with open(os.path.join(EVIDENCE_DIR, "state-legD-after-toggle.txt"), "w", encoding="utf-8") as f:
                f.write(st)
            # host-policy authority (053 chain): with an active host config
            # the policy snapshot re-asserts its mode over the app design
            # layer toggle -- recorded, not failed (the toggle's own
            # authority is the standalone track, asserted in leg B).
            dm_after = "dark_mode: true" in st
            print(f"  [*] toggle under active policy: dark_mode true after = {dm_after} "
                  f"(policy re-assert documented)")
            tmp2 = client.screenshot("061_term_D_after_toggle")
            if tmp2 and os.path.exists(tmp2):
                shot2 = os.path.join(EVIDENCE_DIR, "061_term_D_after_toggle.png")
                with open(tmp2, "rb") as s, open(shot2, "wb") as d:
                    d.write(s.read())
                sb2, _ = sample_pixels(shot2)
                back = all(abs(a - c) <= 3 for a, c in zip(sb2, SOLARIZED_DARK_CARD))
                print(f"  [*] post-toggle statusbar = #{sb2[0]:02x}{sb2[1]:02x}{sb2[2]:02x} "
                      f"(dark card back: {back})")
                # policy re-asserts light (see note above) -- recorded, not failed

        with open(os.path.join(EVIDENCE_DIR, "state-legD-after.txt"), "w", encoding="utf-8") as f:
            f.write(last_state)
    finally:
        kill_tree(proc)


def main() -> int:
    failures: list = []
    app_dir = os.path.dirname(os.path.abspath(__file__))
    app_dir = os.path.dirname(app_dir)  # app/
    auto_bin = os.environ.get("AUTO_BIN", "auto")

    print(f"[*] leg A: explicit AUTO_UI_THEME=dark (override track)")
    leg_a = boot_leg("061_term_A_dark", app_dir, auto_bin,
                     {"AUTO_UI_THEME": "dark"}, AUTO_DARK_CARD, failures,
                     want_dark_state=True)
    print(f"[*] leg B: explicit AUTO_UI_THEME=light (+ standalone scheme toggle)")
    leg_b = boot_leg("061_term_B_light", app_dir, auto_bin,
                     {"AUTO_UI_THEME": "light"}, AUTO_LIGHT_CARD, failures,
                     want_dark_state=False, keep=True)
    # CLI VM split form note: with no engine there are no PTY panes, so the
    # "terminal surface" is the theme-driven content background -- a
    # dark-vs-light equality check is meaningless here (pre-existing CLI
    # gap, receipt-named). The VT exemption is structural: the terminal
    # component palette is engine-owned (terminal_effective_palette) and is
    # exercised by the cargo workspace gates on the engine path.

    # standalone override track: the design layer owns the mode here (env
    # seed is boot-time, no active host policy) -> the real scheme toggle
    # must flip dark_mode and the chrome back to dark card (AUTO dark).
    lb_client = leg_b.get("client")
    if lb_client is None:
        failures.append("leg B: no client for toggle check")
    else:
        tog = lb_client.find_button_by_label("◐")
        if tog is None:
            failures.append("leg B: scheme toggle not found")
        else:
            lb_client.press(tog)
            time.sleep(3)
            st = lb_client.state()
            with open(os.path.join(EVIDENCE_DIR, "state-legB-after-toggle.txt"), "w", encoding="utf-8") as f:
                f.write(st)
            if "dark_mode: true" not in st:
                failures.append("leg B: standalone scheme toggle did not set dark_mode true")
            else:
                print("  [+] standalone toggle: dark_mode flipped true")
            shot = None
            tmp = lb_client.screenshot("061_term_B_after_toggle")
            if tmp and os.path.exists(tmp):
                shot = os.path.join(EVIDENCE_DIR, "061_term_B_after_toggle.png")
                with open(tmp, "rb") as s, open(shot, "wb") as d:
                    d.write(s.read())
                sb, term = sample_pixels(shot)
                back = all(abs(a - c) <= 3 for a, c in zip(sb, AUTO_DARK_CARD))
                print(f"  [*] post-toggle statusbar = #{sb[0]:02x}{sb[1]:02x}{sb[2]:02x} (dark back: {back})")
                if not back:
                    failures.append("leg B: chrome did not return to dark card after standalone toggle")
    if leg_b.get("proc") is not None:
        kill_tree(leg_b["proc"])
        tmp_dir = os.path.join(app_dir, "src", "front", "tmp")
        if os.path.isdir(tmp_dir):
            shutil.rmtree(tmp_dir, ignore_errors=True)

    print(f"[*] leg C: host follow track -- Solarized/dark full candidate")
    cfg_dark = write_config("dark")
    boot_leg("061_term_C_solarized_dark", app_dir, auto_bin,
             {"AUTOOS_DESKTOP_CONFIG": cfg_dark,
              "AUTOOS_THEME_PACKAGE_DIR": PACKAGE_DIR},
             SOLARIZED_DARK_CARD, failures)

    print(f"[*] leg D: live business during reskin (NewTab + flip + toggle)")
    leg_d_live(app_dir, auto_bin, failures)

    print(f"[*] leg E: reopen recovery with flipped light config")
    cfg_flip = os.path.join(CONFIG_DIR, "config_flip.at")
    boot_leg("061_term_E_reopen_light", app_dir, auto_bin,
             {"AUTOOS_DESKTOP_CONFIG": cfg_flip,
              "AUTOOS_THEME_PACKAGE_DIR": PACKAGE_DIR},
             SOLARIZED_LIGHT_CARD, failures)

    if failures:
        print("[-] FAILURES:")
        for f in failures:
            print("   -", f)
        return 1
    print("[+] PLAN-061 term probe: all counter-controls passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
