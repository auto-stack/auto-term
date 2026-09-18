# PLAN-022 探针 auto:Init 自动横分(DBG 门控),纯捕获无交互
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class M29 {
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint f);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  public struct RECT { public int L, T, R, B; }
}
'@
$exe = "D:\autostack\auto-term\target\debug\auto-term.exe"
$errLog = "D:\autostack\auto-term\docs\plans\evidence\022\t05-slotlog.err"
$ev = "D:\autostack\auto-term\docs\plans\evidence\022"
$env:AUTOTERM_ENGINE_DLL = "D:\autostack\auto-term\target\debug\autoterm_core.dll"
$env:AUTO_MA_DBG = "1"
$p = Start-Process -FilePath $exe -PassThru -RedirectStandardError $errLog
Start-Sleep -Seconds 8
$p.Refresh()
$hw = $p.MainWindowHandle
$rc = New-Object M29+RECT
[M29]::GetWindowRect($hw, [ref]$rc) | Out-Null
$bw = $rc.R - $rc.L; $bh = $rc.B - $rc.T
$bmp = New-Object System.Drawing.Bitmap($bw, $bh)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$dc = $g.GetHdc()
[M29]::PrintWindow($hw,$dc,2) | Out-Null
$g.ReleaseHdc($dc);$g.Dispose()
$bmp.Save("$ev\auto-split.png",[System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()
"SHOT $bw x $bh"
Stop-Process -Id $p.Id -Force
Start-Sleep -Milliseconds 600
"DONE"
