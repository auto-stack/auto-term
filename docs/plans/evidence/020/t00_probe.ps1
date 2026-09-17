# t00_probe.ps1 — PLAN-020 T-00 探针实机取证:拖拽捕获层/分数框几何/磁吸。
# 序列:基线 → 拖根竖条至 30%(中途帧 = 捕获层激活布局随动)→ 松手落定
# → 回拖入磁吸带松手(落点应 = 500‰ 即 50%)→ 切 L 形 → 拖右列横条。
$ErrorActionPreference = "Continue"
$out = "D:\autostack\auto-term\docs\plans\evidence\020"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class W0 {
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint f);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
}
"@
Add-Type -AssemblyName System.Drawing

function Log([string]$m) { Write-Output "[$(Get-Date -Format HH:mm:ss)] $m" }

$h = [IntPtr]::Zero
$procs = Get-Process auto -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle -eq "probe020" }
if ($procs) { $h = $procs[0].MainWindowHandle }
if ($h -eq [IntPtr]::Zero) { Log "NO-WINDOW"; exit 1 }
Log "window found hwnd=$h"

if ([W0]::IsIconic($h)) { [void][W0]::ShowWindow($h, 9); Start-Sleep -Milliseconds 800 }
[void][W0]::SetForegroundWindow($h)
Start-Sleep -Milliseconds 800

$wr = New-Object W0+RECT
[void][W0]::GetWindowRect($h, [ref]$wr)
$cr = New-Object W0+RECT
[void][W0]::GetClientRect($h, [ref]$cr)
$cp = New-Object W0+POINT
$cp.X = 0; $cp.Y = 0
[void][W0]::ClientToScreen($h, [ref]$cp)
$cw = $cr.R - $cr.L; $ch = $cr.B - $cr.T
Log "window ($($wr.L),$($wr.T))-($($wr.R),$($wr.B)) client ${cw}x${ch} at ($($cp.X),$($cp.Y))"

function Shot([string]$name) {
  $wd = $wr.R - $wr.L; $ht = $wr.B - $wr.T
  $bmp = New-Object System.Drawing.Bitmap($wd, $ht)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $dc = $g.GetHdc()
  [void][W0]::PrintWindow($h, $dc, 2)
  $g.ReleaseHdc($dc); $g.Dispose()
  $bmp.Save("$out\$name", [System.Drawing.Imaging.ImageFormat]::Png); $bmp.Dispose()
  Log "SHOT $name ${wd}x${ht}"
}

# 内容区原点估计:客户区原点 + Tab 条高度(按钮行 ~40px)
$tabH = 42
$ox = $cp.X; $oy = $cp.Y + $tabH
$cw2 = $cw; $chh = $ch - $tabH

function DragTo([int]$sx, [int]$sy, [int]$tx, [int]$ty, [string]$midShot) {
  [void][W0]::SetCursorPos($sx, $sy)
  Start-Sleep -Milliseconds 250
  [W0]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)  # LEFTDOWN
  Start-Sleep -Milliseconds 150
  $steps = 14
  for ($i = 1; $i -le $steps; $i++) {
    $nx = $sx + [int](($tx - $sx) * $i / $steps)
    $ny = $sy + [int](($ty - $sy) * $i / $steps)
    [void][W0]::SetCursorPos($nx, $ny)
    Start-Sleep -Milliseconds 45
    if ($i -eq 7 -and $midShot -ne "") { Shot $midShot }
  }
  Start-Sleep -Milliseconds 250
  [W0]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)  # LEFTUP
  Start-Sleep -Milliseconds 400
}

# A. 基线(r=500,两分)
Shot "t00-a-baseline.png"

# B. 拖根竖条(50% → 30%):起点 = 内容区中线下 x=cw/2
$sx = $ox + [int]($cw2 * 0.5); $sy = $oy + [int]($chh * 0.6)
$tx = $ox + [int]($cw2 * 0.30)
DragTo $sx $sy $tx $sy "t00-b-middrag.png"
Shot "t00-b-at30.png"

# C. 磁吸:从 30% 回拖至 48%(带内)松手 → 应落 500‰(视觉 50%)
$sx2 = $tx; $tx2 = $ox + [int]($cw2 * 0.48)
DragTo $sx2 $sy $tx2 $sy ""
Shot "t00-c-magnet.png"

# D. 切 L 形
[void][W0]::SetCursorPos($ox + 90, $cp.Y + 14)
Start-Sleep -Milliseconds 200
[W0]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
Start-Sleep -Milliseconds 80
[W0]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
Start-Sleep -Milliseconds 600
Shot "t00-d-lshape.png"

# E. 拖右列横条(分支 11;r2=500 → 650)
$ex = $ox + [int]($cw2 * 0.75); $ey = $oy + [int]($chh * 0.5)
$ty2 = $oy + [int]($chh * 0.65)
DragTo $ex $ey $ex $ty2 "t00-e-middrag2.png"
Shot "t00-e-after.png"

Log "DONE"
