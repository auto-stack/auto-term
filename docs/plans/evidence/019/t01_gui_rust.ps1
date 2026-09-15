# t01_gui_rust.ps1 — PLAN-019 rust 轨实机取证(015 t04 单进程口径:
# 聚焦/键入/快捷键/抓图全序列在一个 PS 进程内完成,前台权不失)。
# 判定面:Tab 条可见/新建切换关闭 Tab/横竖分屏/zoom/scheme/快捷键全组。
$ErrorActionPreference = "Stop"
$exe = "D:\autostack\auto-lang\target\debug\auto-term.exe"
$out = "D:\autostack\auto-term\docs\plans\evidence\019"

Add-Type @"
using System;
using System.Runtime.InteropServices;
public class W3 {
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint f);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
}
"@
Add-Type -AssemblyName System.Drawing

function Shot([IntPtr]$h, [string]$path) {
  foreach ($try in 1..4) {
    $r = New-Object W3+RECT; [void][W3]::GetWindowRect($h, [ref]$r)
    $wd = $r.R - $r.L; $ht = $r.B - $r.T
    if ($wd -gt 100 -and $ht -gt 100) { break }
    Start-Sleep -Milliseconds 300
  }
  if ($wd -le 100) { throw "window rect degenerate: ${wd}x${ht}" }
  $bmp = New-Object System.Drawing.Bitmap($wd, $ht)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $dc = $g.GetHdc()
  [void][W3]::PrintWindow($h, $dc, 2)
  $g.ReleaseHdc($dc); $g.Dispose()
  $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png); $bmp.Dispose()
  Write-Output "SHOT $path ${wd}x${ht}"
}

function FocusWin([IntPtr]$h) {
  if ([W3]::IsIconic($h)) { [void][W3]::ShowWindow($h, 9); Start-Sleep -Milliseconds 600 }
  [W3]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero)
  [void][W3]::SetForegroundWindow($h)
  [W3]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero)
  [void][W3]::SetWindowPos($h, [IntPtr](-1), 0, 0, 0, 0, 0x3)
  Start-Sleep -Milliseconds 250
  if ([W3]::GetForegroundWindow() -ne $h) { throw "failed to focus window" }
  $pidOut = [uint32]0
}

function Click([IntPtr]$h, [int]$cx, [int]$cy, [bool]$right) {
  $p = New-Object W3+POINT; $p.X = $cx; $p.Y = $cy
  [void][W3]::ClientToScreen($h, [ref]$p)
  [void][W3]::SetCursorPos($p.X, $p.Y); Start-Sleep -Milliseconds 150
  $down = 0x0002; $up = 0x0004; if ($right) { $down = 0x0008; $up = 0x0010 }
  [W3]::mouse_event($down, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 80
  [W3]::mouse_event($up, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 150
}

function Tap([byte]$vk) {
  [W3]::keybd_event($vk, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 25
  [W3]::keybd_event($vk, 0, 2, [UIntPtr]::Zero); Start-Sleep -Milliseconds 40
}
function Combo([byte[]]$vks) {
  foreach ($vk in $vks) { [W3]::keybd_event($vk, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 25 }
  [array]::Reverse($vks)
  foreach ($vk in $vks) { [W3]::keybd_event($vk, 0, 2, [UIntPtr]::Zero); Start-Sleep -Milliseconds 25 }
  Start-Sleep -Milliseconds 300
}
function TypeText([string]$s) {
  foreach ($ch in $s.ToCharArray()) {
    if ($ch -eq ' ') { Tap 0x20 }
    elseif ($ch -ge 'a' -and $ch -le 'z') { Tap ([byte](0x41 + ([int][char]$ch) - ([int][char]'a'))) }
    elseif ($ch -ge '0' -and $ch -le '9') { Tap ([byte](0x30 + ([int][char]$ch) - ([int][char]'0'))) }
    elseif ($ch -eq '=') { Tap 0xBB }
    else { throw "no vk map for '$ch'" }
  }
}
function TypeLine([string]$s) { TypeText $s; Tap 0x0D }

$CTRL = 0x11; $SHIFT = 0x10; $E = 0x45; $W = 0x57; $O = 0x4F; $Z = 0x5A; $K = 0x4B; $T = 0x54; $TAB = 0x09; $LEFT = 0x25; $RIGHT = 0x27

# ── 序列 ────────────────────────────────────────────────────────────
Get-Process auto-term -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 1
$env:AUTOTERM_ENGINE_DLL = "D:\autostack\auto-term\target\debug\autoterm_core.dll"
$p = Start-Process -FilePath $exe -PassThru -RedirectStandardError "$out\rust-gui-trace.log"
Start-Sleep -Seconds 8
$h = (Get-Process -Id $p.Id | Where-Object { $_.MainWindowHandle -ne 0 }).MainWindowHandle
if ($h -eq [IntPtr]::Zero) { throw "no main window" }

# 1) 单 Pane + Tab 条(AC-01 基础帧)
FocusWin $h
Shot $h "$out\rust-gui-tabbar-single.png"

# 2) 键入基线(AC-07 直键入零回归):echo P019PANE1
Click $h 640 500 $false
TypeLine "echo P019PANE1"
Start-Sleep -Seconds 2
Shot $h "$out\rust-gui-typing-baseline.png"

# 3) Ctrl+Shift+E 横分(AC-02/04)
Combo @($CTRL, $SHIFT, $E)
Start-Sleep -Seconds 2
Shot $h "$out\rust-gui-split-h.png"

# 4) Ctrl+Shift+O 纵分(在 pane-2 焦点…当前焦点 pane-1 → 纵分 pane-1)
#    注:V1 树深 1,已分屏 Tab 再分拒绝 → 本帧应与上帧同构(cap 判定)
Combo @($CTRL, $SHIFT, $O)
Start-Sleep -Seconds 2
Shot $h "$out\rust-gui-split-cap.png"

# 5) 非焦点 Pane 键入路由:点击右 Pane 键入(AC-02 路由面)
$cl = New-Object W3+RECT; [void][W3]::GetClientRect($h, [ref]$cl)
$cx = [int](($cl.R - $cl.L) * 0.75); $cy = [int](($cl.B - $cl.T) * 0.5)
Click $h $cx $cy $false
TypeLine "echo P019PANE2"
Start-Sleep -Seconds 2
Shot $h "$out\rust-gui-split-typed-right.png"

# 6) Ctrl+Shift+W 关焦点 Pane(pane-2)→ 回单 Pane(AC-04/05)
Combo @($CTRL, $SHIFT, $W)
Start-Sleep -Seconds 2
Shot $h "$out\rust-gui-after-close-pane.png"

# 7) "+" 点击新建 Tab(AC-01):tab 条 "+ " 按钮约在 x=225,y=45(客户区)
Click $h 228 45 $false
Start-Sleep -Seconds 2
Shot $h "$out\rust-gui-tab2-new.png"

# 8) 点击 Tab1 切回(约 x=100,y=45)
Click $h 100 45 $false
Start-Sleep -Seconds 2
Shot $h "$out\rust-gui-tab1-switch-back.png"

# 9) Ctrl+Shift+Z zoom(AC-03 zoom 掩盖)
Click $h 640 500 $false
Combo @($CTRL, $SHIFT, $Z)
Start-Sleep -Seconds 2
Shot $h "$out\rust-gui-zoom.png"
Combo @($CTRL, $SHIFT, $Z)
Start-Sleep -Seconds 1

# 10) Ctrl+Shift+K scheme → light(AC-06)
Combo @($CTRL, $SHIFT, $K)
Start-Sleep -Seconds 2
Shot $h "$out\rust-gui-scheme-light.png"

# 11) Ctrl+Shift+T 新 Tab(快捷键面)+ Ctrl+Shift+Tab 切回(AC-04)
Combo @($CTRL, $SHIFT, $T)
Start-Sleep -Seconds 2
Shot $h "$out\rust-gui-shortcut-newtab.png"
Combo @($CTRL, $SHIFT, $TAB)
Start-Sleep -Seconds 2
Shot $h "$out\rust-gui-shortcut-prevtab.png"

# 12) 右键 Tab2 关闭(x=新 tab 位置;切到 tab2 后右键其标签)
Combo @($CTRL, $SHIFT, $TAB)
Start-Sleep -Seconds 1
Click $h 300 45 $true
Start-Sleep -Seconds 2
Shot $h "$out\rust-gui-after-tab-close.png"

Write-Output "DONE"
