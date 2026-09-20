#!/bin/bash
# t07_profiles.sh — PLAN-025 T-07 profile 契约剧本(curl;vue back)。
# 前置:app-back 以 AUTO_HTTP_PORT + AUTOTERM_CONFIG=evidence/025/config-t07.toml 起。
# 断言:profile 路由两枚 / new-tab(profile) 建_TAB / marker 经 SpawnSpec
# argv 生效 / cwd 经 starting_directory 生效 / 未命中 profile 回落不崩(G6)。
B=${1:-http://127.0.0.1:8125}
echo "== PLAN-025 T-07 profile assertions $(date +%F_%T) =="
curl -s -m 8 -X POST $B/api/term/tick -H 'Content-Type: application/json' -d '{}' > /dev/null
sleep 1
curl -s -m 8 -X POST $B/api/term/tick -H 'Content-Type: application/json' -d '{}' > /dev/null

echo "--- [1] profile-names(声明序) ---"
NAMES=$(curl -s -m 8 $B/api/term/profile-names)
echo "$NAMES"
echo "$NAMES" | grep -q '"cmd"' && echo "$NAMES" | grep -q '"marker"' && echo "PASS names" || echo "FAIL names"

echo "--- [2] profiles 记录面(name|commandline|cwd) ---"
FULL=$(curl -s -m 8 $B/api/term/profiles)
echo "$FULL"
echo "$FULL" | grep -qF 'cmd|cmd.exe|D:\\autostack' && echo "PASS record-cmd" || echo "FAIL record-cmd"
echo "$FULL" | grep -q 'marker|cmd.exe /k echo P025MARK|' && echo "PASS record-marker" || echo "FAIL record-marker"

echo "--- [3] 首_TAB default profile(cmd)+ cwd(prompt 证据) ---"
curl -s -m 8 -X POST $B/api/term/send -H 'Content-Type: application/json' -d '{"line":"echo P025DEFAULT"}' > /dev/null
sleep 1
L=$(curl -s -m 8 -X POST $B/api/term/tick -H 'Content-Type: application/json' -d '{}')
sleep 1
L=$(curl -s -m 8 -X POST $B/api/term/tick -H 'Content-Type: application/json' -d '{}')
echo "$L" | grep -q 'P025DEFAULT' && echo "PASS default-spawn" || echo "FAIL default-spawn"
echo "$L" | grep -q 'autostack' && echo "PASS cwd(D:\\autostack prompt)" || echo "FAIL cwd"

echo "--- [4] new-tab(profile=marker):argv 经 spawn_ex 生效 ---"
T2=$(curl -s -m 8 -X POST $B/api/mux/new-tab -H 'Content-Type: application/json' -d '{"profile":"marker"}')
echo "new-tab(marker)→$T2"
sleep 1
L2=$(curl -s -m 8 -X POST $B/api/term/tick -H 'Content-Type: application/json' -d '{}')
sleep 1
L2=$(curl -s -m 8 -X POST $B/api/term/tick -H 'Content-Type: application/json' -d '{}')
echo "$L2" | grep -q 'P025MARK' && echo "PASS marker-argv" || echo "FAIL marker-argv"

echo "--- [5] new-tab(未命中 ghost):回落缺省 shell 不崩(G6) ---"
T3=$(curl -s -m 8 -X POST $B/api/mux/new-tab -H 'Content-Type: application/json' -d '{"profile":"ghost"}')
echo "new-tab(ghost)→$T3 (expect >0)"
[ "$T3" -gt 0 ] 2>/dev/null && echo "PASS ghost-fallback" || echo "FAIL ghost-fallback"

echo "--- [6] Tab 计数(3) ---"
TABS=$(curl -s -m 8 $B/api/mux/tabs)
echo "$TABS"
echo "$TABS" | grep -o 'shell ' | wc -l | grep -q '^3$' && echo "PASS tabs-3" || echo "FAIL tabs-3"

echo "== done =="
