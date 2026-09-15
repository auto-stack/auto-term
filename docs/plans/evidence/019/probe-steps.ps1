$ErrorActionPreference = "Stop"
$exe = "D:\autostack\auto-lang\target\debug\auto-term.exe"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class W4 {
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
}
"@
Add-Type -AssemblyName System.Drawing
Write-Output "S1 kill+start"
Get-Process auto-term -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 1
$env:AUTOTERM_ENGINE_DLL = "D:\autostack\auto-term\target\debug\autoterm_core.dll"
$p = Start-Process -FilePath $exe -PassThru
Start-Sleep -Seconds 8
$h = (Get-Process -Id $p.Id).MainWindowHandle
Write-Output "S2 hwnd=$h"
if ([W4]::IsIconic($h)) { [void][W4]::ShowWindow($h, 9); Start-Sleep -Milliseconds 600 }
[W4]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero)
[void][W4]::SetForegroundWindow($h)
[W4]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero)
[void][W4]::SetWindowPos($h, [IntPtr](-1), 0, 0, 0, 0, 0x3)
Start-Sleep -Milliseconds 300
Write-Output "S3 fg=$([W4]::GetForegroundWindow() -eq $h)"
$ppt = New-Object W4+POINT; $ppt.X = 640; $ppt.Y = 500
[void][W4]::ClientToScreen($h, [ref]$ppt)
[void][W4]::SetCursorPos($ppt.X, $ppt.Y)
Start-Sleep -Milliseconds 150
Write-Output "S4 cursor at $($ppt.X),$($ppt.Y)"
[W4]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
Start-Sleep -Milliseconds 80
[W4]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
Write-Output "S5 clicked"
Start-Sleep -Milliseconds 500
Write-Output "S6 typing d"
[W4]::keybd_event(0x44, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 30
[W4]::keybd_event(0x44, 0, 2, [UIntPtr]::Zero); Start-Sleep -Milliseconds 200
Write-Output "S7 typed d done"
