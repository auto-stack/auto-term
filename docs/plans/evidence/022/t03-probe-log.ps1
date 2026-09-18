# PLAN-022 探针 v6:split + stderr 重定向(AUTO_MA_DBG=1 slot 数值)
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class M26 {
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, int d, UIntPtr e);
  public struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
}
'@
[void][M26]::SetProcessDPIAware()

$exe = "D:\autostack\auto-term\target\debug\auto-term.exe"
$errLog = "D:\autostack\auto-term\docs\plans\evidence\022\t05-slotlog.err"
$env:AUTOTERM_ENGINE_DLL = "D:\autostack\auto-term\target\debug\autoterm_core.dll"
$env:AUTO_MA_DBG = "1"
$p = Start-Process -FilePath $exe -PassThru -RedirectStandardError $errLog
Start-Sleep -Seconds 4
$p.Refresh()
$wh = $p.MainWindowHandle
[M26]::SetForegroundWindow($wh) | Out-Null
Start-Sleep -Milliseconds 800

$rc = New-Object M26+RECT
[M26]::GetWindowRect($wh, [ref]$rc) | Out-Null
$winW = $rc.R - $rc.L; $winH = $rc.B - $rc.T; $ox = $rc.L; $oy = $rc.T
"WINGET ${winW}x${winH} at $ox,$oy"

[M26]::SetCursorPos([int]($ox + 0.129*$winW), [int]($oy + 0.056*$winH)) | Out-Null
Start-Sleep -Milliseconds 300
[M26]::mouse_event(0x0002,0,0,0,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 90
[M26]::mouse_event(0x0004,0,0,0,[UIntPtr]::Zero)
Start-Sleep -Seconds 3
"CLICKED"

Stop-Process -Id $p.Id -Force
Start-Sleep -Milliseconds 800
"DONE log=$((Get-Item $errLog).Length) bytes"
