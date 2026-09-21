
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class M1 {
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint f);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  public struct RECT { public int L, T, R, B; }
}
'@
$p = Get-Process auto | Where-Object { $_.MainWindowTitle -like "*probe*" } | Select-Object -First 1
if (-not $p) { "NOWIN"; exit }
$r = New-Object M1+RECT
[M1]::GetWindowRect($p.MainWindowHandle,[ref]$r) | Out-Null
$w = $r.R-$r.L; $ht = $r.B-$r.T
$bmp = New-Object System.Drawing.Bitmap($w,$ht)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$dc = $g.GetHdc()
[M1]::PrintWindow($p.MainWindowHandle,$dc,2) | Out-Null
$g.ReleaseHdc($dc);$g.Dispose()
$bmp.Save("D:\autostack\auto-term\docs\plans\evidence\020\exp-press.png",[System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()
"SAVED ${w}x${ht}"
