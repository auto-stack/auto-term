# PLAN-022 T-05 v2:MoveWindow+置顶;WM_CHAR 打字(免焦点);三联合
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class M34 {
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint f);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, int d, UIntPtr e);
  [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr h, uint msg, UIntPtr wp, IntPtr lp);
  public struct RECT { public int L, T, R, B; }
}
'@
[void][M34]::SetProcessDPIAware()

$exe = "D:\autostack\auto-term\target\debug\auto-term.exe"
$errLog = "D:\autostack\auto-term\docs\plans\evidence\022\t05-slotlog.err"
$ev = "D:\autostack\auto-term\docs\plans\evidence\022"
$env:AUTOTERM_ENGINE_DLL = "D:\autostack\auto-term\target\debug\autoterm_core.dll"
$env:AUTO_MA_DBG = "1"
$p = Start-Process -FilePath $exe -PassThru -RedirectStandardError $errLog
Start-Sleep -Seconds 4
$p.Refresh()
$hw = $p.MainWindowHandle
$rc = New-Object M34+RECT
[M34]::GetWindowRect($hw, [ref]$rc) | Out-Null
[M34]::SetWindowPos($hw, [IntPtr](-1), 60, 60, ($rc.R-$rc.L), ($rc.B-$rc.T), 0x0040) | Out-Null  # HWND_TOPMOST
Start-Sleep -Milliseconds 300
[M34]::SetWindowPos($hw, [IntPtr](-2), 60, 60, ($rc.R-$rc.L), ($rc.B-$rc.T), 0x0040) | Out-Null  # HWND_NOTOPMOST
Start-Sleep -Milliseconds 500
[M34]::GetWindowRect($hw, [ref]$rc) | Out-Null
$w = $rc.R - $rc.L; $h = $rc.B - $rc.T; $ox = $rc.L; $oy = $rc.T
"WIN ${w}x${h} at $ox,$oy"

function Shot([string]$path) {
  $rc2 = New-Object M34+RECT
  [M34]::GetWindowRect($hw,[ref]$rc2) | Out-Null
  $bw = $rc2.R - $rc2.L; $bh = $rc2.B - $rc2.T
  $bmp = New-Object System.Drawing.Bitmap($bw, $bh)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $dc = $g.GetHdc()
  [M34]::PrintWindow($hw,$dc,2) | Out-Null
  $g.ReleaseHdc($dc);$g.Dispose()
  $bmp.Save($path,[System.Drawing.Imaging.ImageFormat]::Png)
  $bmp.Dispose()
  "SHOT $path"
}
function TypeText([string]$s) {
  foreach ($ch in $s.ToCharArray()) {
    [M34]::PostMessageW($hw, 0x0102, [UIntPtr][uint16][char]$ch, [IntPtr]::Zero) | Out-Null  # WM_CHAR
    Start-Sleep -Milliseconds 12
  }
  Start-Sleep -Milliseconds 200
  [M34]::PostMessageW($hw, 0x0102, [UIntPtr][uint16]13, [IntPtr]::Zero) | Out-Null  # Enter CR
  Start-Sleep -Milliseconds 2500
  "TYPED"
}
function DragX([int]$y, [int]$x1, [int]$x2) {
  [M34]::SetCursorPos($x1, $y) | Out-Null
  Start-Sleep -Milliseconds 400
  [M34]::mouse_event(0x0002,0,0,0,[UIntPtr]::Zero)
  Start-Sleep -Milliseconds 200
  $steps = 18
  for ($i = 1; $i -le $steps; $i++) {
    $nx = $x1 + [int](($x2 - $x1) * $i / $steps)
    [M34]::SetCursorPos($nx, $y) | Out-Null
    Start-Sleep -Milliseconds 45
  }
  Start-Sleep -Milliseconds 500
  [M34]::mouse_event(0x0004,0,0,0,[UIntPtr]::Zero)
  Start-Sleep -Milliseconds 900
  "DRAGX $x1 -> $x2 y=$y"
}
function DragY([int]$x, [int]$y1, [int]$y2) {
  [M34]::SetCursorPos($x, $y1) | Out-Null
  Start-Sleep -Milliseconds 400
  [M34]::mouse_event(0x0002,0,0,0,[UIntPtr]::Zero)
  Start-Sleep -Milliseconds 200
  $steps = 14
  for ($i = 1; $i -le $steps; $i++) {
    $ny = $y1 + [int](($y2 - $y1) * $i / $steps)
    [M34]::SetCursorPos($x, $ny) | Out-Null
    Start-Sleep -Milliseconds 45
  }
  Start-Sleep -Milliseconds 500
  [M34]::mouse_event(0x0004,0,0,0,[UIntPtr]::Zero)
  Start-Sleep -Milliseconds 900
  "DRAGY $y1 -> $y2 x=$x"
}

# ── ① 造历史(WM_CHAR 免焦点直投)──
TypeText "for /l %i in (1,1,60) do @echo HISTORYLINE-%i%"
Shot "$ev\t05-s1-history.png"

# ── ② 拖分隔条 50% → 68% ──
$dy = [int]($oy + 0.5*$h)
DragX $dy ([int]($ox + 0.50*$w)) ([int]($ox + 0.68*$w))
Shot "$ev\t05-s2-divider-68.png"

# ── ③ 接缝归属:从 68% 分隔条上拖回 55% ──
DragX $dy ([int]($ox + 0.68*$w)) ([int]($ox + 0.55*$w))
Shot "$ev\t05-s3-seam-55.png"

# ── ④ 拖 thumb:左槽右缘轨道 y 0.3H → 0.75H ──
DragY ([int]($ox + 0.655*$w)) ([int]($oy + 0.30*$h)) ([int]($oy + 0.75*$h))
Shot "$ev\t05-s4-thumb-drag.png"

Stop-Process -Id $p.Id -Force
Start-Sleep -Milliseconds 600
"DONE"
