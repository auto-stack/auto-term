#!/bin/bash
# t04_multitab.sh — PLAN-020 复审 AC-07 补证:多 Tab 布局独立性。
# Tab A 深度 2(3 pane)与 Tab B 深度 1(2 pane)并存,交替激活,
# 断言各自矩形投影按活动 Tab 重算且互不串扰。
B=http://127.0.0.1:18080
echo "== PLAN-020 AC-07 multi-tab independence $(date +%F) =="

echo "--- [1] tab A: depth-2 (split-h, focus right, split-v) ---"
curl -s -m 8 -X POST $B/api/term/tick -H 'Content-Type: application/json' -d '{}' -o /dev/null
P2=$(curl -s -m 8 -X POST $B/api/mux/split -H 'Content-Type: application/json' -d '{"axis":1}')
curl -s -m 8 -X POST $B/api/mux/focus -H 'Content-Type: application/json' -d "{\"pane_id\":$P2}" -o /dev/null
P3=$(curl -s -m 8 -X POST $B/api/mux/split -H 'Content-Type: application/json' -d '{"axis":0}')
echo "tabA panes: 1,$P2,$P3"
curl -s -m 8 $B/api/mux/layout; echo

echo "--- [2] new tab B (single pane, auto-active) ---"
curl -s -m 8 -X POST $B/api/mux/new-tab -H 'Content-Type: application/json' -d '{}' -o /dev/null
sleep 1
curl -s -m 8 $B/api/mux/layout; echo
curl -s -m 8 -X POST $B/api/term/tick -H 'Content-Type: application/json' -d '{}' -o /dev/null
P_B=$(curl -s -m 8 $B/api/mux/focus-id)
echo "tabB focus pane=$P_B (expect single full rect, depth1)"

echo "--- [3] tab B: split once (depth-1 own layout) ---"
PB2=$(curl -s -m 8 -X POST $B/api/mux/split -H 'Content-Type: application/json' -d '{"axis":0}')
echo "tabB split→$PB2"
curl -s -m 8 $B/api/mux/layout; echo

echo "--- [4] activate tab A → A's depth-2 layout restored ---"
TA=$(curl -s -m 8 -X GET "$B/api/mux/tab-id-at" -H 'Content-Type: application/json' -d '{"i":0}')
curl -s -m 8 -X POST $B/api/mux/activate-tab -H 'Content-Type: application/json' -d "{\"tab_id\":$TA}" -o /dev/null
sleep 1
curl -s -m 8 $B/api/mux/layout; echo
echo "--- [5] activate tab B → B's layout intact ---"
TB=$(curl -s -m 8 -X GET "$B/api/mux/tab-id-at" -H 'Content-Type: application/json' -d '{"i":1}')
curl -s -m 8 -X POST $B/api/mux/activate-tab -H 'Content-Type: application/json' -d "{\"tab_id\":$TB}" -o /dev/null
sleep 1
curl -s -m 8 $B/api/mux/layout; echo
echo "== done =="
