#!/bin/bash
# t01_model.sh — PLAN-020 T-01/T-03 模型断言剧本(curl;vue back 8080)。
# 序列:基线 → 横分 → 聚焦右→纵分(用户示例,深度2) → rect getters →
# 拖拽模拟(500→300 越磁吸带) → 磁吸(480→500) → 钳位(50→100) →
# 键盘兼容面(mux_resize_pane 700→700) → MAX_PANES 帽(-2) → close 重算
# → 窗口尺寸面(vue back 无窗,缺省 1024x768)。
B=http://127.0.0.1:8080
echo "== PLAN-020 T-01 model assertions $(date +%F %T) =="
curl -s -m 8 -X POST $B/api/term/tick -H 'Content-Type: application/json' -d '{}' > /dev/null
sleep 1
curl -s -m 8 -X POST $B/api/term/tick -H 'Content-Type: application/json' -d '{}' > /dev/null

echo "--- [1] baseline single pane ---"
curl -s -m 8 $B/api/mux/layout; echo

echo "--- [2] split horizontal (axis=1) ---"
SPLIT1=$(curl -s -m 8 -X POST $B/api/mux/split -H 'Content-Type: application/json' -d '{"axis":1}')
echo "split→$SPLIT1"
curl -s -m 8 $B/api/mux/layout; echo

echo "--- [3] focus right pane, split vertical (axis=0) ---"
P2=$(curl -s -m 8 $B/api/mux/focus-id)
curl -s -m 8 -X POST $B/api/mux/focus -H 'Content-Type: application/json' -d "{\"pane_id\":$SPLIT1}" > /dev/null
SPLIT2=$(curl -s -m 8 -X POST $B/api/mux/split -H 'Content-Type: application/json' -d '{"axis":0}')
echo "focus→$SPLIT1 split→$SPLIT2"
curl -s -m 8 $B/api/mux/layout; echo

echo "--- [4] rect getters (11 slots) ---"
for k in 1 2 3 4 5 6 7 8 9 10 11; do
  KIND=$(curl -s -m 8 -X POST $B/api/mux/rect-kind -H 'Content-Type: application/json' -d "{\"k\":$k}")
  PANE=$(curl -s -m 8 -X POST $B/api/mux/rect-pane -H 'Content-Type: application/json' -d "{\"k\":$k}")
  X=$(curl -s -m 8 -X POST $B/api/mux/rect-x -H 'Content-Type: application/json' -d "{\"k\":$k}")
  Y=$(curl -s -m 8 -X POST $B/api/mux/rect-y -H 'Content-Type: application/json' -d "{\"k\":$k}")
  W=$(curl -s -m 8 -X POST $B/api/mux/rect-w -H 'Content-Type: application/json' -d "{\"k\":$k}")
  H=$(curl -s -m 8 -X POST $B/api/mux/rect-h -H 'Content-Type: application/json' -d "{\"k\":$k}")
  BR=$(curl -s -m 8 -X POST $B/api/mux/rect-branch -H 'Content-Type: application/json' -d "{\"k\":$k}")
  AX=$(curl -s -m 8 -X POST $B/api/mux/rect-axis -H 'Content-Type: application/json' -d "{\"k\":$k}")
  echo "slot$k kind=$KIND pane=$PANE x=$X y=$Y w=$W h=$H branch=$BR axis=$AX"
done

echo "--- [5] drag root branch 500→300 (px=300, outside band) ---"
ROOT_BR=$(curl -s -m 8 -X POST $B/api/mux/rect-branch -H 'Content-Type: application/json' -d '{"k":7}')
R=$(curl -s -m 8 -X POST $B/api/mux/resize-branch -H 'Content-Type: application/json' -d "{\"branch_id\":$ROOT_BR,\"px\":300,\"py\":0}")
echo "branch=$ROOT_BR resize(300,0)→$R (expect 300)"
curl -s -m 8 $B/api/mux/layout; echo

echo "--- [6] magnet: drag to 480 (inside ±40 band) → 500 ---"
R=$(curl -s -m 8 -X POST $B/api/mux/resize-branch -H 'Content-Type: application/json' -d "{\"branch_id\":$ROOT_BR,\"px\":480,\"py\":0}")
echo "resize(480,0)→$R (expect 500)"

echo "--- [7] clamp: drag to 50 → 100 (MIN_PANE) ---"
R=$(curl -s -m 8 -X POST $B/api/mux/resize-branch -H 'Content-Type: application/json' -d "{\"branch_id\":$ROOT_BR,\"px\":50,\"py\":0}")
echo "resize(50,0)→$R (expect 100)"

echo "--- [8] keyboard compat: mux_resize_pane(pane1, 700) → 700 ---"
P1=$(curl -s -m 8 $B/api/mux/focus-id)
RP=$(curl -s -m 8 -X POST $B/api/mux/resize-pane -H 'Content-Type: application/json' -d "{\"pane_id\":$P1,\"ratio\":700}")
echo "pane=$P1 resize_pane(700)→$RP (expect 700)"

echo "--- [9] right-column branch drag (axis=0): py drives ---"
# 右列分支 = 深度2 的纵分;找到 axis=0 的 divider
for k in 8 9 10 11; do
  AX=$(curl -s -m 8 -X POST $B/api/mux/rect-axis -H 'Content-Type: application/json' -d "{\"k\":$k}")
  BR=$(curl -s -m 8 -X POST $B/api/mux/rect-branch -H 'Content-Type: application/json' -d "{\"k\":$k}")
  if [ "$AX" == "0" ]; then
    R=$(curl -s -m 8 -X POST $B/api/mux/resize-branch -H 'Content-Type: application/json' -d "{\"branch_id\":$BR,\"px\":750,\"py\":250}")
    echo "right-col branch=$BR resize(750,250)→$R (expect 250)"
    break
  fi
done
curl -s -m 8 $B/api/mux/layout; echo

echo "--- [10] MAX_PANES cap: split until 7th pane → -2 ---"
# 当前 3 pane;再分 3 次到 6,第 6 次(第 7 pane)应 -2
for i in 4 5 6 7; do
  curl -s -m 8 -X POST $B/api/mux/focus -H 'Content-Type: application/json' -d "{\"pane_id\":1}" > /dev/null
  # 交替轴向避免歧义;split 焦点 pane1 的叶
  AX=$([ $i == 7 ] && echo 0 || echo 1)
  curl -s -m 8 -X POST $B/api/mux/focus -H 'Content-Type: application/json' -d "{\"pane_id\":1}" > /dev/null
  R=$(curl -s -m 8 -X POST $B/api/mux/split -H 'Content-Type: application/json' -d "{\"axis\":1}")
  echo "split#$i → $R"
  if [ "$R" == "0" ] || [ "$R" == "-2" ]; then break; fi
done
curl -s -m 8 $B/api/mux/layout; echo

echo "--- [11] window size face (vue back = headless defaults) ---"
WW=$(curl -s -m 8 $B/api/mux/window-width); WH=$(curl -s -m 8 $B/api/mux/window-height)
echo "window=${WW}x${WH} (expect 1024x768 headless defaults)"

echo "--- [12] close pane → rects recompute ---"
curl -s -m 8 -X POST $B/api/mux/close-pane -H 'Content-Type: application/json' -d "{\"pane_id\":3}" > /dev/null
curl -s -m 8 $B/api/mux/layout; echo
echo "== done =="
