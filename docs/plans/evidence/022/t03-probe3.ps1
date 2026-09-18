# PLAN-022 探针 v4:窗口比例定位(DPI 无关;变量名去冲突)
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class M24 {
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint f);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, int d, UIntPtr e);
  public struct RECT { public int L, T, R, B; }
}
'@
[void][M24]::SetProcessDPIAware()

$script:exe = "D:\autostack\auto-term\target\debug\auto-term.exe"
$env:AUTOTERM_ENGINE_DLL = "D:\autostack\auto-term\target\debug\autoterm_core.dll"
$script:p = Start-Process -FilePath $exe -PassThru
Start-Sleep -Seconds 4
$script:p.Refresh()
$script:hw = $script:p.MainWindowHandle
if ($script:hw -eq [IntPtr]::Zero) { "NOWIN"; Stop-Process -Id $script:p.Id -Force; exit 1 }
[M24]::SetForegroundWindow($script:hw) | Out-Null
Start-Sleep -Milliseconds 800

$rc = New-Object M24+RECT
[M24]::GetWindowRect($script:hw,[ref]$rc) | Out-Null
$winW = $rc.R - $rc.L
$winH = $rc.B - $rc.T
$ox = $rc.L; $oy = $rc.T
"WIN ${winW}x${winH} at $ox,$oy"

function Shot([string]$path, [IntPtr]$wh) {
  $rr = New-Object M24+RECT
  [M24]::GetWindowRect($wh,[ref]$rr) | Out-Null
  $bw = $rr.R - $rr.L
  $bh = $rr.B - $rr.T
  if ($bw -le 0 -or $bh -le 0) { "SKIP $path (${bw}x${bh})"; return }
  $bmp = New-Object System.Drawing.Bitmap($bw, $bh)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $dc = $g.GetHdc()
  [M24]::PrintWindow($wh,$dc,2) | Out-Null
  $g.ReleaseHdc($dc);$g.Dispose()
  $bmp.Save($path,[System.Drawing.Imaging.ImageFormat]::Png)
  $bmp.Dispose()
  "SAVED $path ${bw}x${bh}"
}
function Click([int]$cx, [int]$cy) {
  [M24]::SetCursorPos($cx, $cy) | Out-Null
  Start-Sleep -Milliseconds 300
  [M24]::mouse_event(0x0002,0,0,0,[UIntPtr]::Zero)
  Start-Sleep -Milliseconds 90
  [M24]::mouse_event(0x0004,0,0,0,[UIntPtr]::Zero)
  Start-Sleep -Milliseconds 450
  "CLICK $cx,$cy"
}
function WheelUp([int]$cx, [int]$cy, [int]$n) {
  [M24]::SetCursorPos($cx, $cy) | Out-Null
  Start-Sleep -Milliseconds 300
  for ($i = 0; $i -lt $n; $i++) {
    [M24]::mouse_event(0x0800,0,0,120,[UIntPtr]::Zero)
    Start-Sleep -Milliseconds 80
  }
  Start-Sleep -Milliseconds 700
  "WHEEL $n at $cx,$cy"
}
function DragX([int]$y, [int]$x1, [int]$x2) {
  [M24]::SetCursorPos($x1, $y) | Out-Null
  Start-Sleep -Milliseconds 300
  [M24]::mouse_event(0x0002,0,0,0,[UIntPtr]::Zero)
  Start-Sleep -Milliseconds 150
  $steps = 18
  for ($i = 1; $i -le $steps; $i++) {
    $nx = $x1 + [int](($x2 - $x1) * $i / $steps)
    [M24]::SetCursorPos($nx, $y) | Out-Null
    Start-Sleep -Milliseconds 40
  }
  Start-Sleep -Milliseconds 400
  [M24]::mouse_event(0x0004,0,0,0,[UIntPtr]::Zero)
  Start-Sleep -Milliseconds 600
  "DRAG $x1 -> $x2 at y=$y"
}

$ev = "D:\autostack\auto-term\docs\plans\evidence\022"
Shot "$ev\v3-boot.png" $script:hw

# ① 横分按钮(比例 0.129, 0.056)
Click ([int]($ox + 0.129*$winW)) ([int]($oy + 0.056*$winH))
Start-Sleep -Seconds 2
Shot "$ev\v3-split.png" $script:hw

# ② 左 pane 滚轮上翻(0.15W, 0.45H)
WheelUp ([int]($ox + 0.15*$winW)) ([int]($oy + 0.45*$winH)) 10
Shot "$ev\v3-wheelup.png" $script:hw

# ③ 分隔条拖拽(0.50W → 0.68W, y=0.45H)
DragX ([int]($oy + 0.45*$winH)) ([int]($ox + 0.50*$winW)) ([int]($ox + 0.68*$winW))
Shot "$ev\v3-drag.png" $script:hw

Stop-Process -Id $script:p.Id -Force
"DONE"
