# PLAN-005 T8 取证:前台聚焦 blink 探针。App 必须先在跑;本脚本
# SetForegroundWindow 后等待,让 blink 相位在聚焦态累计。
param([int]$HoldMs = 4000)
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class F {
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
}
'@
$p = Get-Process autoterm -ErrorAction SilentlyContinue |
    Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1
if (-not $p) { Write-Output "no autoterm window"; exit 1 }
[F]::SetForegroundWindow($p.MainWindowHandle) | Out-Null
Start-Sleep -Milliseconds 400
$fg = [F]::GetForegroundWindow()
Write-Output ("fg_equals_autoterm: " + ($fg -eq $p.MainWindowHandle))
Start-Sleep -Milliseconds $HoldMs
$fg2 = [F]::GetForegroundWindow()
Write-Output ("fg_held_after_wait: " + ($fg2 -eq $p.MainWindowHandle))
