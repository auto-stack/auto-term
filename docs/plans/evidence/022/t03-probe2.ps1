# PLAN-022 T-03/T-05 探针 v2:真实鼠标驱动——横分/滚轮回看/分隔条拖拽
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class M22 {
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint f);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, int d, UIntPtr e);
  public struct RECT { public int L, T, R, B; }
}
'@
[void][M22]::SetProcessDPIAware()

$exe = "D:\autostack\auto-term\target\debug\auto-term.exe"
$env:AUTOTERM_ENGINE_DLL = "D:\autostack\auto-term\target\debug\autoterm_core.dll"
$p = Start-Process -FilePath $exe -PassThru
Start-Sleep -Seconds 4
$p.Refresh()
$h = $p.MainWindowHandle
if ($h -eq [IntPtr]::Zero) { "NOWIN"; Stop-Process -Id $p.Id -Force; exit 1 }
[M22]::SetForegroundWindow($h) | Out-Null
Start-Sleep -Milliseconds 800

$r = New-Object M22+RECT
[M22]::GetWindowRect($h,[ref]$r) | Out-Null
"WIN $($r.L),$($r.T) - $($r.R),$($r.B)"

function Shot($path) {
  $r2 = New-Object M22+RECT
  [M22]::GetWindowRect($h,[ref]$r2) | Out-Null
  $w = $r2.R-$r2.L; $ht = $r2.B-$r2.T
  $bmp = New-Object System.Drawing.Bitmap($w,$ht)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $dc = $g.GetHdc()
  [M22]::PrintWindow($h,$dc,2) | Out-Null
  $g.ReleaseHdc($dc);$g.Dispose()
  $bmp.Save($path,[System.Drawing.Imaging.ImageFormat]::Png)
  $bmp.Dispose()
  "SAVED $path"
}
function Click($x, $y) {
  [M22]::SetCursorPos($x, $y) | Out-Null
  Start-Sleep -Milliseconds 250
  [M22]::mouse_event(0x0002,0,0,0,[UIntPtr]::Zero)
  Start-Sleep -Milliseconds 80
  [M22]::mouse_event(0x0004,0,0,0,[UIntPtr]::Zero)
  Start-Sleep -Milliseconds 400
}
function Drag($x1, $y1, $x2, $y2) {
  [M22]::SetCursorPos($x1, $y1) | Out-Null
  Start-Sleep -Milliseconds 250
  [M22]::mouse_event(0x0002,0,0,0,[UIntPtr]::Zero)
  Start-Sleep -Milliseconds 120
  $steps = 16
  for ($i = 1; $i -le $steps; $i++) {
    $nx = $x1 + [int](($x2 - $x1) * $i / $steps)
    $ny = $y1 + [int](($y2 - $y1) * $i / $steps)
    [M22]::SetCursorPos($nx, $ny) | Out-Null
    Start-Sleep -Milliseconds 40
  }
  Start-Sleep -Milliseconds 350
  [M22]::mouse_event(0x0004,0,0,0,[UIntPtr]::Zero)
  Start-Sleep -Milliseconds 500
}
function Wheel($x, $y, $notches) {
  [M22]::SetCursorPos($x, $y) | Out-Null
  Start-Sleep -Milliseconds 200
  for ($i = 0; $i -lt $notches; $i++) {
    [M22]::mouse_event(0x0800,0,0,120,[UIntPtr]::Zero)
    Start-Sleep -Milliseconds 90
  }
  Start-Sleep -Milliseconds 600
}

$ev = "D:\autostack\auto-term\docs\plans\evidence\022"
Shot("$ev\t05-boot.png")

# ① 横分(按钮物理坐标:窗 1293 宽,横分钮 ≈ (348,100))
Click 348 100
Start-Sleep -Seconds 2
Shot("$ev\t05-split.png")

# ② 左 pane 滚轮回看(滚轮向上 8 格)
Wheel 300 450 8
Shot("$ev\t05-wheelup.png")

# ③ 分隔条拖拽(中缝 ≈ x 646 → 800)
Drag 646 450 800 450
Shot("$ev\t05-divider-drag.png")

Stop-Process -Id $p.Id -Force
"DONE"
