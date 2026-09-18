# PLAN-022 探针 final:分屏/滚轮/拖拽 全链 + 截图 + slot 日志
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class M27 {
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint f);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, int d, UIntPtr e);
  [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr h, int x, int y, int w, int ht, bool repaint);
  public struct RECT { public int L, T, R, B; }
}
'@
[void][M27]::SetProcessDPIAware()

$exe = "D:\autostack\auto-term\target\debug\auto-term.exe"
$errLog = "D:\autostack\auto-term\docs\plans\evidence\022\t05-slotlog.err"
$ev = "D:\autostack\auto-term\docs\plans\evidence\022"
$env:AUTOTERM_ENGINE_DLL = "D:\autostack\auto-term\target\debug\autoterm_core.dll"
$env:AUTO_MA_DBG = "1"
$p = Start-Process -FilePath $exe -PassThru -RedirectStandardError $errLog
Start-Sleep -Seconds 4
$p.Refresh()
$hw = $p.MainWindowHandle
[M27]::SetForegroundWindow($hw) | Out-Null
Start-Sleep -Milliseconds 800

$rc = New-Object M27+RECT
[M27]::GetWindowRect($hw, [ref]$rc) | Out-Null
$winW = $rc.R - $rc.L; $winH = $rc.B - $rc.T
# 固定窗口位置并置顶,避免宿主控制台遮挡点击
[M27]::MoveWindow($hw, 60, 60, $winW, $winH, $true) | Out-Null
Start-Sleep -Milliseconds 500
[M27]::SetForegroundWindow($hw) | Out-Null
Start-Sleep -Milliseconds 400
$rc = New-Object M27+RECT
[M27]::GetWindowRect($hw, [ref]$rc) | Out-Null
$ox = $rc.L; $oy = $rc.T
"MOVED to $ox,$oy ${winW}x${winH}"

function Shot([string]$path) {
  $rr = New-Object M27+RECT
  [M27]::GetWindowRect($hw,[ref]$rr) | Out-Null
  $bw = $rr.R - $rr.L; $bh = $rr.B - $rr.T
  if ($bw -le 0 -or $bh -le 0) { return }
  $bmp = New-Object System.Drawing.Bitmap($bw, $bh)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $dc = $g.GetHdc()
  [M27]::PrintWindow($hw,$dc,2) | Out-Null
  $g.ReleaseHdc($dc);$g.Dispose()
  $bmp.Save($path,[System.Drawing.Imaging.ImageFormat]::Png)
  $bmp.Dispose()
  "SHOT $path"
}
function Click([int]$cx, [int]$cy) {
  [M27]::SetCursorPos($cx, $cy) | Out-Null
  Start-Sleep -Milliseconds 300
  [M27]::mouse_event(0x0002,0,0,0,[UIntPtr]::Zero)
  Start-Sleep -Milliseconds 90
  [M27]::mouse_event(0x0004,0,0,0,[UIntPtr]::Zero)
  Start-Sleep -Milliseconds 450
}

Shot "$ev\f-boot.png"
# 两次尝试横分(0.14W / 0.135W)
Click ([int]($ox + 0.140*$winW)) ([int]($oy + 0.056*$winH))
Start-Sleep -Seconds 2
Shot "$ev\f-after-click1.png"
Click ([int]($ox + 0.115*$winW)) ([int]($oy + 0.056*$winH))
Start-Sleep -Seconds 2
Shot "$ev\f-after-click2.png"
# 滚轮(左 pane 0.15W, 0.45H)上翻 10 格
[M27]::SetCursorPos([int]($ox + 0.15*$winW), [int]($oy + 0.45*$winH)) | Out-Null
Start-Sleep -Milliseconds 300
for ($i = 0; $i -lt 10; $i++) { [M27]::mouse_event(0x0800,0,0,120,[UIntPtr]::Zero); Start-Sleep -Milliseconds 80 }
Start-Sleep -Milliseconds 800
Shot "$ev\f-wheelup.png"
# 分隔条拖拽(y=0.45H; 0.52W → 0.68W)
$dy = [int]($oy + 0.45*$winH); $x1 = [int]($ox + 0.52*$winW); $x2 = [int]($ox + 0.68*$winW)
[M27]::SetCursorPos($x1, $dy) | Out-Null
Start-Sleep -Milliseconds 300
[M27]::mouse_event(0x0002,0,0,0,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 150
for ($i = 1; $i -le 18; $i++) {
  $nx = $x1 + [int](($x2 - $x1) * $i / 18)
  [M27]::SetCursorPos($nx, $dy) | Out-Null
  Start-Sleep -Milliseconds 40
}
Start-Sleep -Milliseconds 400
[M27]::mouse_event(0x0004,0,0,0,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 700
Shot "$ev\f-drag.png"

Stop-Process -Id $p.Id -Force
Start-Sleep -Milliseconds 600
"DONE"
