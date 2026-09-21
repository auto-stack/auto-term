"""PLAN-024 T-00 现行为取证驱动:MCP bus 快照/状态/截图采集。

用法:python _mcp.py <port> <cmd> [args...]
  state                       -> autoui_state 文本
  snap                        -> autoui_snapshot 文本
  shot <out.png>              -> autoui_screenshot 存盘
  find <substr>               -> snapshot 中含 substr 的行
  type <text>                 -> autoui_type
  key <key>                   -> autoui_keyboard(如 ctrl+c / enter / a)
  action <element_id> <act>   -> autoui_action
  wait <substr> <timeout_s>   -> 轮询 snapshot 直到含 substr
"""
import json
import sys
import time
import urllib.request


def mcp(port, name, args=None, timeout=30):
    req = {"jsonrpc": "2.0", "id": 1, "method": "tools/call",
           "params": {"name": name, "arguments": args or {}}}
    r = urllib.request.Request(f"http://127.0.0.1:{port}/mcp",
                               data=json.dumps(req).encode(),
                               headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(r, timeout=timeout) as resp:
        return json.loads(resp.read().decode())


def text(resp):
    try:
        return resp["result"]["content"][0]["text"]
    except Exception:
        return json.dumps(resp, ensure_ascii=False)


def main():
    port = int(sys.argv[1])
    cmd = sys.argv[2]
    if cmd == "state":
        print(text(mcp(port, "autoui_state")))
    elif cmd == "snap":
        print(text(mcp(port, "autoui_snapshot")))
    elif cmd == "shot":
        out = sys.argv[3]
        resp = mcp(port, "autoui_screenshot", {}, timeout=60)
        src = None
        t = text(resp)
        for token in t.replace('"', " ").split():
            if token.lower().endswith(".png") or token.lower().endswith(".png:"):
                src = token.rstrip(":")
        print(t)
        if src and src != out:
            import shutil
            shutil.copyfile(src, out)
            print(f"copied -> {out}")
    elif cmd == "find":
        sub = sys.argv[3]
        for line in text(mcp(port, "autoui_snapshot")).splitlines():
            if sub in line:
                print(line)
    elif cmd == "type":
        print(text(mcp(port, "autoui_type", {"text": sys.argv[3]})))
    elif cmd == "key":
        a = sys.argv[3].split("+")
        args = {"key": a[-1]}
        if len(a) > 1:
            args["modifiers"] = a[:-1]
        print(text(mcp(port, "autoui_keyboard", args)))
    elif cmd == "action":
        print(text(mcp(port, "autoui_action",
                       {"element_id": sys.argv[3], "action": sys.argv[4]})))
    elif cmd == "desktop":
        args = {"action": sys.argv[3]}
        keys = ["verb", "kind", "text", "vk", "modifiers", "app", "handler", "dx", "dy"]
        for j, v in enumerate(sys.argv[4:]):
            if j < len(keys):
                args[keys[j]] = v
        print(text(mcp(port, "autoui_desktop", args)))
    elif cmd == "wait":
        sub, tmo = sys.argv[3], float(sys.argv[4])
        dl = time.time() + tmo
        while time.time() < dl:
            try:
                if sub in text(mcp(port, "autoui_snapshot")):
                    print(f"OK: found {sub!r}")
                    return
            except Exception:
                pass
            time.sleep(0.5)
        print(f"TIMEOUT: {sub!r} not found in {tmo}s")
        sys.exit(1)
    else:
        print(__doc__)
        sys.exit(2)


if __name__ == "__main__":
    main()
