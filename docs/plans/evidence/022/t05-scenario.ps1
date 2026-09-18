# PLAN-022 T-05 三联合场景:auto-split + 造历史 + 拖分隔条 + 接缝拖拽 + 拖 thumb
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class M33 {
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint f);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, int d, UIntPtr e);
  public struct RECT { public int L, T, R, B; }
}
'@
[void][M33]::SetProcessDPIAware()

$exe = "D:\autostack\auto-term\target\debug\auto-term.exe"
$errLog = "D:\autostack\auto-term\docs\plans\evidence\022\t05-slotlog.err"
$ev = "D:\autostack\auto-term\docs\plans\evidence\022"
$env:AUTOTERM_ENGINE_DLL = "D:\autostack\auto-term\target\debug\autoterm_core.dll"
$env:AUTO_MA_DBG = "1"
$p = Start-Process -FilePath $exe -PassThru -RedirectStandardError $errLog
Start-Sleep -Seconds 4
$p.Refresh()
$hw = $p.MainWindowHandle
[M33]::SetForegroundWindow($hw) | Out-Null
Start-Sleep -Milliseconds 800

$rc = New-Object M33+RECT
[M33]::GetWindowRect($hw, [ref]$rc) | Out-Null
$w = $rc.R - $rc.L; $h = $rc.B - $rc.T; $ox = $rc.L; $oy = $rc.T
"WIN ${w}x${h} at $ox,$oy"

function Shot([string]$path) {
  $rc2 = New-Object M33+RECT
  [M33]::GetWindowRect($hw,[ref]$rc2) | Out-Null
  $bw = $rc2.R - $rc2.L; $bh = $rc2.B - $rc2.T
  $bmp = New-Object System.Drawing.Bitmap($bw, $bh)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $dc = $g.GetHdc()
  [M33]::PrintWindow($hw,$dc,2) | Out-Null
  $g.ReleaseHdc($dc);$g.Dispose()
  $bmp.Save($path,[System.Drawing.Imaging.ImageFormat]::Png)
  $bmp.Dispose()
  "SHOT $path"
}
function DragX([int]$y, [int]$x1, [int]$x2) {
  [M33]::SetCursorPos($x1, $y) | Out-Null
  Start-Sleep -Milliseconds 350
  [M33]::mouse_event(0x0002,0,0,0,[UIntPtr]::Zero)
  Start-Sleep -Milliseconds 150
  $steps = 18
  for ($i = 1; $i -le $steps; $i++) {
    $nx = $x1 + [int](($x2 - $x1) * $i / $steps)
    [M33]::SetCursorPos($nx, $y) | Out-Null
    Start-Sleep -Milliseconds 40
  }
  Start-Sleep -Milliseconds 400
  [M33]::mouse_event(0x0004,0,0,0,[UIntPtr]::Zero)
  Start-Sleep -Milliseconds 700
  "DRAG $x1 -> $x2 y=$y"
}
function DragY([int]$x, [int]$y1, [int]$y2) {
  [M33]::SetCursorPos($x, $y1) | Out-Null
  Start-Sleep -Milliseconds 350
  [M33]::mouse_event(0x0002,0,0,0,[UIntPtr]::Zero)
  Start-Sleep -Milliseconds 150
  $steps = 14
  for ($i = 1; $i -le $steps; $i++) {
    $ny = $y1 + [int](($y2 - $y1) * $i / $steps)
    [M33]::SetCursorPos($x, $ny) | Out-Null
    Start-Sleep -Milliseconds 40
  }
  Start-Sleep -Milliseconds 400
  [M33]::mouse_event(0x0004,0,0,0,[UIntPtr]::Zero)
  Start-Sleep -Milliseconds 700
  "DRAGY $y1 -> $y2 x=$x"
}

# ── ① 造滚转历史:焦点 pane(左)跑 60 行循环 ──
$ws = New-Object -ComObject WScript.Shell
[void]$ws.AppActivate($p.Id)
Start-Sleep -Milliseconds 500
$ws.SendKeys("for /l %i in (1,1,60) do @echo HISTORYLINE-%i%")
$ws.SendKeys("{ENTER}")
Start-Sleep -Seconds 3
Shot "$ev\t05-s1-history.png"

# ── ② 拖分隔条:50% → 68%(y=0.5H)──
$dy = [int]($oy + 0.5*$h)
DragX $dy ([int]($ox + 0.50*$w)) ([int]($ox + 0.68*$w))
Shot "$ev\t05-s2-divider-68.png"

# ── ③ 接缝拖拽:从新分隔条位置(68%)拖回 55%——命中归属=分隔条 ──
DragX $dy ([int]($ox + 0.68*$w)) ([int]($ox + 0.55*$w))
Shot "$ev\t05-s3-seam-55.png"

# ── ④ 拖 thumb:左 pane 滚动条轨道(x≈0.66W=左槽右缘内,y 0.3H→0.7H)──
DragY ([int]($ox + 0.662*$w)) ([int]($oy + 0.30*$h)) ([int]($oy + 0.70*$h))
Shot "$ev\t05-s4-thumb-drag.png"

Stop-Process -Id $p.Id -Force
Start-Sleep -Milliseconds 600
"DONE"
