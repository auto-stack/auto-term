# PLAN-022 探针 kbd:AppActivate 前台 + Ctrl+Shift+E 横分 + 滚轮 + 拖拽
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class M28 {
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint f);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, int d, UIntPtr e);
  public struct RECT { public int L, T, R, B; }
}
'@
[void][M28]::SetProcessDPIAware()

$exe = "D:\autostack\auto-term\target\debug\auto-term.exe"
$errLog = "D:\autostack\auto-term\docs\plans\evidence\022\t05-slotlog.err"
$ev = "D:\autostack\auto-term\docs\plans\evidence\022"
$env:AUTOTERM_ENGINE_DLL = "D:\autostack\auto-term\target\debug\autoterm_core.dll"
$env:AUTO_MA_DBG = "1"
$p = Start-Process -FilePath $exe -PassThru -RedirectStandardError $errLog
Start-Sleep -Seconds 4
$p.Refresh()
$hw = $p.MainWindowHandle

$ws = New-Object -ComObject WScript.Shell
[void]$ws.AppActivate($p.Id)
Start-Sleep -Milliseconds 600
$ws.SendKeys("^+e")
"SENT ^+e"
Start-Sleep -Seconds 3

$rc = New-Object M28+RECT
[M28]::GetWindowRect($hw, [ref]$rc) | Out-Null
$winW = $rc.R - $rc.L; $winH = $rc.B - $rc.T; $ox = $rc.L; $oy = $rc.T

function Shot([string]$path) {
  $rr = New-Object M28+RECT
  [M28]::GetWindowRect($hw,[ref]$rr) | Out-Null
  $bw = $rr.R - $rr.L; $bh = $rr.B - $rr.T
  if ($bw -le 0 -or $bh -le 0) { return }
  $bmp = New-Object System.Drawing.Bitmap($bw, $bh)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $dc = $g.GetHdc()
  [M28]::PrintWindow($hw,$dc,2) | Out-Null
  $g.ReleaseHdc($dc);$g.Dispose()
  $bmp.Save($path,[System.Drawing.Imaging.ImageFormat]::Png)
  $bmp.Dispose()
  "SHOT $path"
}

Shot "$ev\k-split.png"
# 滚轮回看(左 pane)
[M28]::SetCursorPos([int]($ox + 0.15*$winW), [int]($oy + 0.45*$winH)) | Out-Null
Start-Sleep -Milliseconds 400
for ($i = 0; $i -lt 10; $i++) { [M28]::mouse_event(0x0800,0,0,120,[UIntPtr]::Zero); Start-Sleep -Milliseconds 80 }
Start-Sleep -Milliseconds 900
Shot "$ev\k-wheelup.png"
# 分隔条拖拽
$dy = [int]($oy + 0.45*$winH); $x1 = [int]($ox + 0.52*$winW); $x2 = [int]($ox + 0.68*$winW)
[M28]::SetCursorPos($x1, $dy) | Out-Null
Start-Sleep -Milliseconds 350
[M28]::mouse_event(0x0002,0,0,0,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 150
for ($i = 1; $i -le 18; $i++) {
  $nx = $x1 + [int](($x2 - $x1) * $i / 18)
  [M28]::SetCursorPos($nx, $dy) | Out-Null
  Start-Sleep -Milliseconds 40
}
Start-Sleep -Milliseconds 400
[M28]::mouse_event(0x0004,0,0,0,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 800
Shot "$ev\k-drag.png"

Stop-Process -Id $p.Id -Force
Start-Sleep -Milliseconds 600
"DONE"
