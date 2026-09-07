@echo off
rem PLAN-011 T2: one-shot packaging for the at-gen artifact --
rem release build + dist/ trio assembly + layout smoke.
rem Contract: 003 SS5 deploy item -- autoterm-at.exe, autoterm_core.dll
rem and autoterm-ctrlc.exe ship in the SAME directory; the smoke runs
rem `autoterm-at.exe scenario echo` inside dist (SCENARIO_OK proves the
rem trio layout resolves under the real DLL search order).
rem Usage: scripts\package-at.cmd   (run from anywhere; ASCII only --
rem cmd parses batch files in the OEM codepage, keep this file ASCII)
setlocal
cd /d "%~dp0.."

rem 1) engine cdylib + interrupt helper (release)
echo [package-at] step 1: cargo build -p autoterm-core --release
cargo build -p autoterm-core --release
if errorlevel 1 goto :fail_build

rem 2) at-gen artifact crate (own workspace; default release profile)
echo [package-at] step 2: at-gen cargo build --release
cargo build --release --manifest-path at-gen\Cargo.toml
if errorlevel 1 goto :fail_build

rem 3) assemble dist/ (trio; rebuilt idempotently)
echo [package-at] step 3: assemble dist/
if exist dist rmdir /s /q dist
if exist dist goto :fail_dist_lock
mkdir dist
if errorlevel 1 goto :fail_dist
copy /y target\release\autoterm_core.dll dist\ >nul
if errorlevel 1 goto :fail_dist
copy /y target\release\autoterm-ctrlc.exe dist\ >nul
if errorlevel 1 goto :fail_dist
copy /y at-gen\target\release\autoterm-at.exe dist\ >nul
if errorlevel 1 goto :fail_dist

rem 4) layout smoke: run scenario echo from inside dist; the SCENARIO_OK
rem    literal is the only pass signal (findstr /c: -- NOTE: /x exact-line
rem    match fails on LF-only stream output, verified 2026-09-07; ROW
rem    lines never contain the token so substring is sound)
echo [package-at] step 4: dist layout smoke
pushd dist
.\autoterm-at.exe scenario echo 2>&1 | findstr /c:"SCENARIO_OK" >nul
if errorlevel 1 goto :fail_smoke
popd

echo package-at: dist trio ready + layout smoke SCENARIO_OK
dir /b dist
exit /b 0

:fail_build
echo package-at: FAILED at cargo build step
exit /b 1

:fail_dist_lock
echo package-at: FAILED -- dist/ not removable (file lock?)
exit /b 1

:fail_dist
echo package-at: FAILED at dist assembly
dir /b target\release\autoterm_core.dll target\release\autoterm-ctrlc.exe at-gen\target\release\autoterm-at.exe 2>&1
exit /b 1

:fail_smoke
echo package-at: FAILED at dist layout smoke (no exact SCENARIO_OK line)
popd
exit /b 1
