"""PLAN-024 T-01 实机验证驱动:几何源 override 生效性。

序列:launch auto-term → 稳态 cols(全幅投影预期)→ layout grid(rect 变)
→ cols 随动 → layout free(rect 回)→ cols 回。附 calculator 在场
(串扰面:cols 稳定不漂)。
"""
import json
import sys
import time
import urllib.request

PORT = 9251
EVID = "D:/autostack/auto-term/docs/plans/evidence/024"


def mcp(name, args=None, timeout=30):
    req = {"jsonrpc": "2.0", "id": 1, "method": "tools/call",
           "params": {"name": name, "arguments": args or {}}}
    r = urllib.request.Request(f"http://127.0.0.1:{PORT}/mcp",
                               data=json.dumps(req).encode(),
                               headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(r, timeout=timeout) as resp:
        return json.loads(resp.read().decode())


def text(resp):
    return resp["result"]["content"][0]["text"]


def terminal_geom():
    snap = text(mcp("autoui_snapshot"))
    for line in snap.splitlines():
        if "terminal key=" in line:
            return line.strip()
    return None


def wait_terminal(timeout=40):
    dl = time.time() + timeout
    while time.time() < dl:
        g = terminal_geom()
        if g:
            return g
        time.sleep(1)
    return None


def main():
    print("== launch auto-term ==")
    print(text(mcp("autoui_desktop", {"action": "bus", "verb": "launch\tauto-term"})))
    g = wait_terminal()
    print("boot geom:", g)
    time.sleep(6)  # 投影/引擎 resize 收敛
    for round_ in range(3):
        time.sleep(2)
        print(f"steady[{round_}]:", terminal_geom())
    print("== layout grid(rect 变更)==")
    print(text(mcp("autoui_desktop", {"action": "bus", "verb": "layout\tgrid"})))
    time.sleep(6)
    for round_ in range(3):
        time.sleep(2)
        print(f"grid[{round_}]:", terminal_geom())
    print("== layout free(rect 回)==")
    print(text(mcp("autoui_desktop", {"action": "bus", "verb": "layout\tfree"})))
    time.sleep(6)
    for round_ in range(3):
        time.sleep(2)
        print(f"free[{round_}]:", terminal_geom())


if __name__ == "__main__":
    main()
