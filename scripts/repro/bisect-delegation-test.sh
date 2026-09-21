#!/bin/bash
# bisect-delegation-test.sh — VM 轨 api 委托断供的 git-bisect 判据脚本。
# 在 auto-lang worktree(bisect 检出态)内运行:构建 auto.exe →
# 起 auto-term app(VM 形态, AUTO_FFI_TRACE=1)→ 45s 窗口内数 DLL
# feed_ready 调用: >10 = good(委托通), 0-10 = bad(断供)。
# 退出码: 0=good 1=bad 125=跳过(构建失败)
set +e
WT="/d/autostack/.wt/lang-021/auto-lang"
powershell -NoProfile -Command "Get-Process auto,app-back,node -ErrorAction SilentlyContinue | Stop-Process -Force" 2>/dev/null
sleep 1
LOG=/d/autostack/auto-term/docs/plans/evidence/021/bisect-run.log
cargo build -p auto > /tmp/bisect-build.log 2>&1
BUILD_RC=$?
if [ $BUILD_RC -ne 0 ] || [ ! -f "$WT/target/debug/auto.exe" ]; then
  echo "BUILD_FAIL at $(git -C "$WT" rev-parse --short HEAD)"
  tail -3 /tmp/bisect-build.log
  exit 125
fi
powershell -NoProfile -Command "Get-Process auto,app-back,node -ErrorAction SilentlyContinue | Stop-Process -Force" 2>/dev/null
sleep 1
cd /d/autostack/auto-term/app
AUTO_FFI_TRACE=1 "$WT/target/debug/auto.exe" run -r vm > /tmp/bisect-out.log 2> "$LOG" &
APP=$!
sleep 60
N=$(grep -c "DLL_ENTER.*feed_ready" "$LOG" 2>/dev/null)
SPAWN=$(grep -c "DLL_ENTER.*spawn_ex" "$LOG" 2>/dev/null)
powershell -NoProfile -Command "Get-Process auto,app-back,node -ErrorAction SilentlyContinue | Stop-Process -Force" 2>/dev/null
kill $APP 2>/dev/null
HEAD=$(git -C "$WT" rev-parse --short HEAD)
echo "RESULT head=$HEAD feed=$N spawn=$SPAWN"
if [ "$SPAWN" -eq 0 ]; then exit 125   # app 没起来(环境抖动)→ 跳过
elif [ "$N" -gt 10 ]; then exit 0       # good
else exit 1; fi                          # bad
