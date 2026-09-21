#!/bin/bash
# bisect-codegen-test.sh — rust 轨 codegen 漂移的 git-bisect 判据。
# 在 auto-lang worktree(bisect 检出态)内:构建该提交的 auto.exe →
# 全清 auto-term 生成树 → 用该 exe 从零生成 rust 轨 app → 把生成
# Cargo.toml 的 auto-lang path 指回本检出 → cargo build:能编译 =
# good(0),编译错 = bad(1),环境失败 = skip(125)。
set +e
WT="/d/autostack/.wt/lang-021-p2/auto-lang"
APP="/d/autostack/auto-term/app"
LOG=/d/autostack/auto-term/docs/plans/evidence/021/bisect-codegen.log
powershell -NoProfile -Command "Get-Process auto,auto-term,app-back,node -ErrorAction SilentlyContinue | Stop-Process -Force" 2>/dev/null
sleep 1
cargo build -p auto > /tmp/bc-build.log 2>&1
if [ ! -f "$WT/target/debug/auto.exe" ]; then echo "BUILD_FAIL $(git -C $WT rev-parse --short HEAD)"; tail -3 /tmp/bc-build.log; exit 125; fi
rm -rf "$APP/rust-workspace"
cd "$APP"
"$WT/target/debug/auto.exe" run -r rust > /tmp/bc-gen.log 2>&1
# 生成完成后(编译失败预期),把 path 指回本检出再做裁决性编译
sed -i 's|path = "../../../auto-lang/crates/auto-lang"|path = "D:/autostack/.wt/lang-021-p2/auto-lang/crates/auto-lang"|' "$APP/rust-workspace/Cargo.toml" 2>/dev/null
cd "$APP/rust-workspace"
ERRS=$(cargo build -p auto-term 2>&1 | grep -cE "^error(\[|:)")
HEAD=$(git -C "$WT" rev-parse --short HEAD)
echo "RESULT head=$HEAD errors=$ERRS"
powershell -NoProfile -Command "Get-Process auto,auto-term,app-back,node -ErrorAction SilentlyContinue | Stop-Process -Force" 2>/dev/null
if [ "$ERRS" -eq 0 ]; then exit 0; else exit 1; fi
