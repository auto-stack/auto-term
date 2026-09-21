Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class M9 {
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint f);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, UIntPtr e);
  public struct RECT { public int L, T, R, B; }
}
'@
[void][M9]::SetProcessDPIAware()
$p = Get-Process auto | Where-Object { $_.MainWindowTitle -like "*probe020*" } | Select-Object -First 1
if (-not $p) { Write-Output "NOWIN"; exit 1 }
$h = $p.MainWindowHandle
$r = New-Object M9+RECT
[M9]::GetWindowRect($h,[ref]$r) | Out-Null
Write-Output "window $($r.R-$r.L)x$($r.B-$r.T) at ($($r.L),$($r.T))"
# 点击蓝色块:窗口内 (150, 250) 物理
$wl0 = [int]$r.L
$ht0 = [int]$r.T
[M9]::SetCursorPos($wl0 + 150, $ht0 + 250)
Start-Sleep -Milliseconds 250
[M9]::mouse_event(0x0002,0,0,0,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 80
[M9]::mouse_event(0x0004,0,0,0,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 800
$wl = [int]($r.R - $r.L)
$ht2 = [int]($r.B - $r.T)
$bmp = New-Object System.Drawing.Bitmap($wl,$ht2)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$dc = $g.GetHdc()
[M9]::PrintWindow($h,$dc,2) | Out-Null
$g.ReleaseHdc($dc);$g.Dispose()
$bmp.Save("D:\autostack\auto-term\docs\plans\evidence\020\exp-click-result.png",[System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()
Write-Output "clicked+shot"
