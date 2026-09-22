# resize-window.ps1 — PLAN-028 实机取证:程序化拉窗(WM_SIZE 路径与用户拖拽同源)
# 用法: powershell -File resize-window.ps1 -ProcName auto -Width 1100 -Height 800
param(
    [string]$ProcName = "auto",
    [int]$Width = 1100,
    [int]$Height = 800
)
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class WinResize {
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr hWnd, IntPtr after, int X, int Y, int cx, int cy, uint uFlags);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
    [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
    public struct RECT { public int Left, Top, Right, Bottom; }
}
"@
$procs = Get-Process $ProcName -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne 0 }
if (-not $procs) { Write-Output "NO-WINDOW"; exit 1 }
$hwnd = ($procs | Select-Object -First 1).MainWindowHandle
if ([WinResize]::IsIconic($hwnd)) { [void][WinResize]::ShowWindow($hwnd, 9); Start-Sleep -Milliseconds 800 }
$rect = New-Object WinResize+RECT
[WinResize]::GetWindowRect($hwnd, [ref]$rect) | Out-Null
# SWP_NOMOVE(0x2) 只改尺寸;客户区略小于窗框(标题/边框),以实测为准
$ok = [WinResize]::SetWindowPos($hwnd, [IntPtr]::Zero, 0, 0, $Width, $Height, 0x2)
Start-Sleep -Milliseconds 900
$rect2 = New-Object WinResize+RECT
[WinResize]::GetWindowRect($hwnd, [ref]$rect2) | Out-Null
Write-Output ("RESIZED=" + $ok + " before=" + ($rect.Right-$rect.Left) + "x" + ($rect.Bottom-$rect.Top) + " after=" + ($rect2.Right-$rect2.Left) + "x" + ($rect2.Bottom-$rect2.Top))
