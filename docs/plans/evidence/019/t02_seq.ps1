# t02_seq.ps1 — 单 PS 进程证据序列 v2(带退出码/步骤标记)
$ErrorActionPreference = "Continue"
$out = "D:\autostack\auto-term\docs\plans\evidence\019"
$exe = "D:\autostack\auto-lang\target\debug\auto-term.exe"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class W5 {
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint f);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
}
"@
Add-Type -AssemblyName System.Drawing

function Log([string]$m) { Write-Output "[$(Get-Date -Format HH:mm:ss)] $m" }

function Shot([IntPtr]$h, [string]$name) {
  $r = New-Object W5+RECT
  for ($i = 0; $i -lt 8; $i++) {
    [void][W5]::GetWindowRect($h, [ref]$r)
    if (($r.R - $r.L) -gt 200 -and ($r.B - $r.T) -gt 200) { break }
    if ([W5]::IsIconic($h)) { [void][W5]::ShowWindow($h, 9) }
    Start-Sleep -Milliseconds 400
  }
  $wd = $r.R - $r.L; $ht = $r.B - $r.T
  if ($wd -le 200) { Log "SHOT-SKIP $name (rect ${wd}x${ht})"; return }
  $bmp = New-Object System.Drawing.Bitmap($wd, $ht)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $dc = $g.GetHdc()
  [void][W5]::PrintWindow($h, $dc, 2)
  $g.ReleaseHdc($dc); $g.Dispose()
  $bmp.Save("$out\$name", [System.Drawing.Imaging.ImageFormat]::Png); $bmp.Dispose()
  Log "SHOT $name ${wd}x${ht}"
}

function FocusWin([IntPtr]$h) {
  if ([W5]::IsIconic($h)) { [void][W5]::ShowWindow($h, 9); Start-Sleep -Milliseconds 600 }
  [W5]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero)
  [void][W5]::SetForegroundWindow($h)
  [W5]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero)
  [void][W5]::SetWindowPos($h, [IntPtr](-1), 0, 0, 0, 0, 0x3)
  Start-Sleep -Milliseconds 300
  Log "fg=$([W5]::GetForegroundWindow() -eq $h)"
}

function Click([IntPtr]$h, [int]$cx, [int]$cy, [bool]$right) {
  $p = New-Object W5+POINT; $p.X = $cx; $p.Y = $cy
  [void][W5]::ClientToScreen($h, [ref]$p)
  [void][W5]::SetCursorPos($p.X, $p.Y); Start-Sleep -Milliseconds 150
  $down = 0x0002; $up = 0x0004; if ($right) { $down = 0x0008; $up = 0x0010 }
  [W5]::mouse_event($down, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 80
  [W5]::mouse_event($up, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 200
}

function Tap([byte]$vk) {
  [W5]::keybd_event($vk, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 25
  [W5]::keybd_event($vk, 0, 2, [UIntPtr]::Zero); Start-Sleep -Milliseconds 40
}
function Combo([byte[]]$vks) {
  foreach ($vk in $vks) { [W5]::keybd_event($vk, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 25 }
  [array]::Reverse($vks)
  foreach ($vk in $vks) { [W5]::keybd_event($vk, 0, 2, [UIntPtr]::Zero); Start-Sleep -Milliseconds 25 }
  Start-Sleep -Milliseconds 300
}
function TypeLine([string]$s) {
  foreach ($ch in $s.ToCharArray()) {
    if ($ch -eq ' ') { Tap 0x20 }
    elseif ($ch -ge 'a' -and $ch -le 'z') { Tap ([byte](0x41 + ([int][char]$ch) - ([int][char]'a'))) }
    elseif ($ch -ge 'A' -and $ch -le 'Z') { Tap ([byte](0x41 + ([int][char]$ch) - ([int][char]'A'))) }
    elseif ($ch -ge '0' -and $ch -le '9') { Tap ([byte](0x30 + ([int][char]$ch) - ([int][char]'0'))) }
  }
  Tap 0x0D
}

function ProbeState([string]$tag) {
  try {
    $body = '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"autoui_state","arguments":{}}}'
    $resp = Invoke-RestMethod -Uri "http://127.0.0.1:9247/mcp" -Method Post -ContentType "application/json" -Body $body -TimeoutSec 3
    $txt = $resp.result.content[0].text
    $lines = $txt -split "`n" | Where-Object { $_ -match "axis|slot|zoom|scheme" }
    Log "STATE[$tag] $($lines -join ' | ')"
  } catch { Log "STATE[$tag] FAIL $($_.Exception.Message)" }
}

$CTRL = 0x11; $SHIFT = 0x10; $E = 0x45; $W = 0x57; $Z = 0x5A; $K = 0x4B; $T = 0x54; $TAB = 0x09

Get-Process auto-term -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 1
$env:AUTOTERM_ENGINE_DLL = "D:\autostack\auto-term\target\debug\autoterm_core.dll"
$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = $exe
$psi.UseShellExecute = $false
$psi.RedirectStandardError = $true
$psi.RedirectStandardOutput = $true
$p = [System.Diagnostics.Process]::Start($psi)
Start-Sleep -Seconds 8
$p.Refresh()
$h = $p.MainWindowHandle
Log "start pid=$($p.Id) hwnd=$h exited=$($p.HasExited)"
if ($h -eq [IntPtr]::Zero -or $p.HasExited) { Log "APP-DIED"; exit 1 }

# 1 单 Pane Tab 条
FocusWin $h
Start-Sleep -Seconds 1
ProbeState "boot"
Shot $h "rust-gui-tabbar-single.png"

# 2 键入基线
Click $h 600 500 $false
TypeLine "echo HELLOFROMP1"
Start-Sleep -Seconds 2
Shot $h "rust-gui-typing-baseline.png"

# 3 横分
Combo @($CTRL, $SHIFT, $E)
Start-Sleep -Seconds 2
ProbeState "after-CSE"
Shot $h "rust-gui-split-h.png"
Log "alive1=$(-not $p.HasExited)"
Combo @($CTRL, $SHIFT, $E)
Start-Sleep -Seconds 2
ProbeState "after-CSE-2"
Shot $h "rust-gui-split-h2.png"

# 4 点右 Pane 键入(坐标 = 客户区 75%/50%,由窗口矩形现算)
$r2 = New-Object W5+RECT; [void][W5]::GetWindowRect($h, [ref]$r2)
$wd2 = $r2.R - $r2.L; $ht2 = $r2.B - $r2.T
$cx2 = [int]($wd2 * 0.75); $cy2 = [int]($ht2 * 0.5)
Log "pane2-click client=$cx2,$cy2"
Click $h $cx2 $cy2 $false
TypeLine "echo HELLOFROMP2"
Start-Sleep -Seconds 2
Shot $h "rust-gui-split-typed-right.png"

# 5 Ctrl+Shift+W 关焦点 Pane(=pane-2)
Combo @($CTRL, $SHIFT, $W)
Start-Sleep -Seconds 2
Shot $h "rust-gui-after-close-pane.png"

# 6 "+" 新建 Tab(x≈228 客户区)
Click $h 228 45 $false
Start-Sleep -Seconds 2
Shot $h "rust-gui-tab2-new.png"

# 7 点 Tab1 切回
Click $h 100 45 $false
Start-Sleep -Seconds 2
Shot $h "rust-gui-tab1-switch-back.png"

# 8 zoom
Click $h 600 500 $false
Combo @($CTRL, $SHIFT, $Z)
Start-Sleep -Seconds 2
Shot $h "rust-gui-zoom.png"
Combo @($CTRL, $SHIFT, $Z)
Start-Sleep -Seconds 1

# 9 scheme light
Combo @($CTRL, $SHIFT, $K)
Start-Sleep -Seconds 2
Shot $h "rust-gui-scheme-light.png"

# 10 快捷键新 Tab + 切回
Combo @($CTRL, $SHIFT, $T)
Start-Sleep -Seconds 2
Shot $h "rust-gui-shortcut-newtab.png"
Combo @($CTRL, $SHIFT, $TAB)
Start-Sleep -Seconds 2
Shot $h "rust-gui-shortcut-prevtab.png"

Log "alive-end=$(-not $p.HasExited)"
if ($p.HasExited) { Log "EXITCODE=$($p.ExitCode)" }
try { $err = $p.StandardError.ReadToEnd(); if ($err) { Log "STDERR-TAIL: $($err.Substring([Math]::Max(0,$err.Length-400)))" } } catch {}
Log "DONE"
