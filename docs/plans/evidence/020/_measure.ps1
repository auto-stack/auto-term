$ErrorActionPreference = "Continue"
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class M1 {
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint f);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  public struct RECT { public int L, T, R, B; }
}
'@
[void][M1]::SetProcessDPIAware()
$p = Get-Process auto | Where-Object { $_.MainWindowTitle -eq [char]0x7EC8 + [char]0x7AEF } | Select-Object -First 1
if (-not $p) { Write-Output "NOWIN"; exit 1 }
$h = $p.MainWindowHandle
$r = New-Object M1+RECT
[M1]::GetWindowRect($h,[ref]$r) | Out-Null
$w = $r.R-$r.L; $ht = $r.B-$r.T
Write-Output "window ${w}x${ht} at ($($r.L),$($r.T))"
$bmp = New-Object System.Drawing.Bitmap($w,$ht)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$dc = $g.GetHdc()
[M1]::PrintWindow($h,$dc,2) | Out-Null
$g.ReleaseHdc($dc);$g.Dispose()
$bmp.Save("D:\autostack\auto-term\docs\plans\evidence\020\acc-measure.png",[System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()
Write-Output "SHOT saved"
