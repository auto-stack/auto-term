# PLAN-022 拖拽单变量探针:仅分隔条一次按压-拖拽-释放,读 dragging 态
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class M35 {
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, int d, UIntPtr e);
  public struct RECT { public int L, T, R, B; }
}
'@
[void][M35]::SetProcessDPIAware()
$exe = "D:\autostack\auto-term\target\debug\auto-term.exe"
$errLog = "D:\autostack\auto-term\docs\plans\evidence\022\t05-slotlog.err"
$env:AUTOTERM_ENGINE_DLL = "D:\autostack\auto-term\target\debug\autoterm_core.dll"
$env:AUTO_MA_DBG = "1"
$p = Start-Process -FilePath $exe -PassThru -RedirectStandardError $errLog
Start-Sleep -Seconds 4
$p.Refresh()
$hw = $p.MainWindowHandle
$rc = New-Object M35+RECT
[M35]::GetWindowRect($hw, [ref]$rc) | Out-Null
$ox = $rc.L; $oy = $rc.T; $w = $rc.R - $rc.L; $h = $rc.B - $rc.T
$dy = [int]($oy + 0.5*$h); $x1 = [int]($ox + 0.50*$w); $x2 = [int]($ox + 0.62*$w)
[M35]::SetCursorPos($x1, $dy) | Out-Null
Start-Sleep -Milliseconds 500
[M35]::mouse_event(0x0002,0,0,0,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 250
for ($i = 1; $i -le 10; $i++) {
  [M35]::SetCursorPos($x1 + [int](($x2-$x1)*$i/10), $dy) | Out-Null
  Start-Sleep -Milliseconds 60
}
Start-Sleep -Milliseconds 500
[M35]::mouse_event(0x0004,0,0,0,[UIntPtr]::Zero)
Start-Sleep -Seconds 2
Stop-Process -Id $p.Id -Force
Start-Sleep -Milliseconds 600
"DONE"
