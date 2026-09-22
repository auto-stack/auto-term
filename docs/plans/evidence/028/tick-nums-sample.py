#!/usr/bin/env python
"""PLAN-028 tick-nums 采样:POST /api/mux/tick-nums 解析几何面 + history 尾段。
用法: python tick-nums-sample.py <port> [n_samples]
输出: 每拍一行 ver win_w win_h cols rows ntabs slots npanes + 尾段 history"""
import json, sys, time, urllib.request

port = int(sys.argv[1]) if len(sys.argv) > 1 else 17403
n = int(sys.argv[2]) if len(sys.argv) > 2 else 3
url = f"http://127.0.0.1:{port}/api/mux/tick-nums"
for i in range(n):
    req = urllib.request.Request(url, data=b"{}", headers={"Content-Type": "application/json"}, method="POST")
    with urllib.request.urlopen(req, timeout=5) as r:
        ns = json.loads(r.read().decode())
        if isinstance(ns, dict):
            ns = ns.get("result") or ns.get("data") or []
    if not ns:
        print(f"[{i}] EMPTY"); continue
    ver, w, h, cols, rows, ntabs, slots, npanes = ns[0], ns[1], ns[2], ns[3], ns[4], ns[5], ns[6], ns[7]
    pbase = 107
    hist = ns[pbase + 7 * npanes:] if len(ns) >= pbase + 7 * npanes else []
    print(f"[{i}] ver={ver} win={w}x{h} grid={cols}x{rows} tabs={ntabs} npanes={npanes} hist_tail={hist} len={len(ns)}")
    if i < n - 1:
        time.sleep(0.5)
