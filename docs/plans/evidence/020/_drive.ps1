param(
  [int]$cx = 334,
  [int]$cy = 98,
  [int]$tx = -1,
  [int]$ty = -1,
  [string]$mode = "click"
)
$ErrorActionPreference = "Continue"
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class M3 {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
}
'@
[void][M3]::SetProcessDPIAware()
$p = Get-Process auto | Where-Object { $_.MainWindowTitle -eq [char]0x7EC8 + [char]0x7AEF } | Select-Object -First 1
if (-not $p) { Write-Output "NOWIN"; exit 1 }
[M3]::SetForegroundWindow($p.MainWindowHandle) | Out-Null
Start-Sleep -Milliseconds 400
[M3]::SetCursorPos($cx, $cy)
Start-Sleep -Milliseconds 250
[M3]::mouse_event(0x0002,0,0,0,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 80
if ($mode -eq "drag" -and $tx -ge 0) {
  $steps = 14
  for ($i = 1; $i -le $steps; $i++) {
    $nx = $cx + [int](($tx - $cx) * $i / $steps)
    $ny = $cy + [int](($ty - $cy) * $i / $steps)
    [M3]::SetCursorPos($nx, $ny)
    Start-Sleep -Milliseconds 45
  }
  Start-Sleep -Milliseconds 300
}
[M3]::mouse_event(0x0004,0,0,0,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 400
Write-Output "DONE ($cx,$cy) mode=$mode end=($tx,$ty)"
