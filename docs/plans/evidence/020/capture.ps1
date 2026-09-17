# capture-window.ps1 — PLAN-019 实机取证:PrintWindow 抓图(018 同款)
# 用法: powershell -File capture-window.ps1 -ProcName auto-term -OutPath <png>
param(
    [string]$ProcName = "auto-term",
    [string]$OutPath = "D:\autostack\auto-term\docs\plans\evidence\019\shot.png"
)
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Win32Cap {
    [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr hwnd, IntPtr hdc, uint flags);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hwnd);
    public struct RECT { public int Left, Top, Right, Bottom; }
}
"@
$procs = Get-Process $ProcName -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne 0 }
if (-not $procs) { Write-Output "NO-WINDOW"; exit 1 }
$p = $procs[0]
$hwnd = $p.MainWindowHandle
# 最小化则还原(014 教训:最小化=退化几何 157x25)
Add-Type @"
using System;using System.Runtime.InteropServices;
public class WinRestore { [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd); [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h); }
"@
if ([WinRestore]::IsIconic($hwnd)) { [void][WinRestore]::ShowWindow($hwnd, 9); Start-Sleep -Milliseconds 800 }
[Win32Cap]::SetForegroundWindow($hwnd) | Out-Null
Start-Sleep -Milliseconds 600
$rect = New-Object Win32Cap+RECT
[Win32Cap]::GetWindowRect($hwnd, [ref]$rect) | Out-Null
$w = $rect.Right - $rect.Left
$h = $rect.Bottom - $rect.Top
if ($w -le 0 -or $h -le 0) { Write-Output "BAD-RECT"; exit 1 }
$bmp = New-Object System.Drawing.Bitmap($w, $h)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$hdc = $g.GetHdc()
# PW_RENDERFULLCONTENT=2(含 GPU 合成面)
[Win32Cap]::PrintWindow($hwnd, $hdc, 2) | Out-Null
$g.ReleaseHdc($hdc)
$g.Dispose()
$bmp.Save($OutPath, [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()
Write-Output "SAVED $OutPath ${w}x${h} pid=$($p.Id)"
