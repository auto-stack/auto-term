# light-selfcheck.ps1 — 019 冒烟自证:前台发 Ctrl+Shift+K 切 scheme → 抓帧 → 像素分析
param([int]$TargetPid = 0)
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;using System.Runtime.InteropServices;
public class W {
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  public struct R { public int L, T, Rt, B; }
}
"@
$p = Get-Process auto -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1
if (-not $p) { Write-Output "NO-WINDOW"; exit 1 }
$h = $p.MainWindowHandle
if ([W]::IsIconic($h)) { [void][W]::ShowWindow($h, 9); Start-Sleep -Milliseconds 900 }
[void][W]::SetForegroundWindow($h)
Start-Sleep -Milliseconds 900
[System.Windows.Forms.SendKeys]::SendWait('^+k')
Start-Sleep -Milliseconds 1500
$r = New-Object W+R
[void][W]::GetWindowRect($h, [ref]$r)
$w = $r.Rt - $r.L; $ht = $r.B - $r.T
Write-Output ("WINDOW {0}x{1}" -f $w, $ht)
if ($w -lt 300) { Write-Output "DEGENERATE-GEOMETRY"; exit 1 }

# PrintWindow 抓帧
$bmp = New-Object System.Drawing.Bitmap($w, $ht)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($r.L, $r.T, 0, 0, $bmp.Size)
$bmp.Save("D:\autostack\auto-term\docs\plans\evidence\019\t06-vm-light-selfcheck2.png")
$g.Dispose()

$bg = $bmp.GetPixel(600, 500)
Write-Output ("BG(600,500): R={0} G={1} B={2}  (new=238,232,213 / old=253,246,227)" -f $bg.R, $bg.G, $bg.B)
$minSum = 999999; $dx = 0; $dy = 0
for ($x = 10; $x -lt [Math]::Min(700, $w - 10); $x += 2) {
  for ($y = 120; $y -lt [Math]::Min(300, $ht - 10); $y += 2) {
    $q = $bmp.GetPixel($x, $y)
    $s = $q.R + $q.G + $q.B
    if ($s -lt $minSum) { $minSum = $s; $dx = $x; $dy = $y }
  }
}
$dp = $bmp.GetPixel($dx, $dy)
Write-Output ("DARKEST text-px: R={0} G={1} B={2} at {3},{4}  (new fg=7,54,66 / old fg=88,110,117)" -f $dp.R, $dp.G, $dp.B, $dx, $dy)
$bmp.Dispose()
