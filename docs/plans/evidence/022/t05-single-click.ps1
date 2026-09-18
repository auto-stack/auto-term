# PLAN-022 单击定界:程序点一次横分,读模型 pane 数
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class M39 {
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, int d, UIntPtr e);
  [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr h, int x, int y, int w, int ht, bool repaint);
  public struct RECT { public int L, T, R, B; }
}
'@
[M39]::SetProcessDPIAware()
$exe = "D:\autostack\auto-term\target\debug\auto-term.exe"
$errLog = "D:\autostack\auto-term\docs\plans\evidence\022\t05-slotlog.err"
$env:AUTOTERM_ENGINE_DLL = "D:\autostack\auto-term\target\debug\autoterm_core.dll"
$env:AUTO_MA_DBG = "1"
$p = Start-Process -FilePath $exe -PassThru -RedirectStandardError $errLog
Start-Sleep -Seconds 4
$p.Refresh()
$hw = $p.MainWindowHandle
$rc = New-Object M39+RECT
[M39]::GetWindowRect($hw, [ref]$rc) | Out-Null
$w = $rc.R - $rc.L; $h = $rc.B - $rc.T
[M39]::MoveWindow($hw, 60, 60, $w, $h, $true) | Out-Null
Start-Sleep -Milliseconds 600
[M39]::SetForegroundWindow($hw) | Out-Null
Start-Sleep -Milliseconds 500
[M39]::GetWindowRect($hw, [ref]$rc) | Out-Null
$ox = $rc.L; $oy = $rc.T
[M39]::SetCursorPos([int]($ox + 0.129*$w), [int]($oy + 0.055*$h)) | Out-Null
Start-Sleep -Milliseconds 350
[M39]::mouse_event(0x0002,0,0,0,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 90
[M39]::mouse_event(0x0004,0,0,0,[UIntPtr]::Zero)
Start-Sleep -Seconds 3
"CLICKED once"
Stop-Process -Id $p.Id -Force
Start-Sleep -Milliseconds 600
"DONE"
