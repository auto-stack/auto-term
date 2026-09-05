# PLAN-004 T8 IME 取证:dev-preedit 注入 → 鼠标移动事件触发 IME 声明
# → PrintWindow 抓 over-the-spot 覆盖层 → 扫描终端光标下方亮带。
# 用法:ime-smoke.ps1 -MoveAtMs 8500 -CaptureAtMs 10500
param(
    [int]$MoveAtMs = 8500,
    [int]$CaptureAtMs = 10500,
    [string]$OutDir = $PSScriptRoot
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public struct RECT { public int Left, Top, Right, Bottom; }
public class Win {
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint flags);
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
}
'@
[Win]::SetProcessDPIAware() | Out-Null

function Find-AutoTerm {
    for ($i = 0; $i -lt 40; $i++) {
        $p = Get-Process autoterm -ErrorAction SilentlyContinue |
            Where-Object { $_.MainWindowTitle -eq 'AutoTerm' } |
            Select-Object -First 1
        if ($p) { return $p }
        Start-Sleep -Milliseconds 250
    }
    throw "autoterm window not found"
}

$p = Find-AutoTerm
$h = $p.MainWindowHandle
# 前台强制(Alt-tap 取前台权):鼠标移动事件必须落在未遮挡的本窗口上
[Win]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero)
[Win]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero)
[Win]::SetForegroundWindow($h) | Out-Null
for ($i = 0; $i -lt 20; $i++) {
    if ([Win]::GetForegroundWindow() -eq $h) { break }
    Start-Sleep -Milliseconds 100
}
if ([Win]::GetForegroundWindow() -ne $h) { throw "failed to foreground AutoTerm" }
"foreground: ok"

$r = New-Object RECT
[Win]::GetWindowRect($h, [ref]$r) | Out-Null
"rect: $($r.Right-$r.Left)x$($r.Bottom-$r.Top) at ($($r.Left),$($r.Top))"

Start-Sleep -Milliseconds $MoveAtMs
# 鼠标移动进窗口中心:产生交互事件 → widget.update → request_ime
[Win]::SetCursorPos((($r.Left+$r.Right)/2), (($r.Top+$r.Bottom)/2)) | Out-Null
"cursor moved at ${MoveAtMs}ms"

Start-Sleep -Milliseconds ($CaptureAtMs - $MoveAtMs)

$w = $r.Right - $r.Left; $wh = $r.Bottom - $r.Top
$bmp = $null
for ($attempt = 1; $attempt -le 4; $attempt++) {
    $bmp = New-Object System.Drawing.Bitmap($w, $wh)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $hdc = $g.GetHdc()
    $ok = [Win]::PrintWindow($h, $hdc, 2)
    $g.ReleaseHdc($hdc)
    $g.Dispose()
    if ($ok) { break }
    $bmp.Dispose(); $bmp = $null
    Start-Sleep -Milliseconds 400
}
if (-not $bmp) { throw "PrintWindow failed after retries" }

# 校验截屏有效(DEFAULT_BG 直通带)
$rect2 = New-Object System.Drawing.Rectangle(0, 0, $w, $wh)
$fmt = [System.Drawing.Imaging.PixelFormat]::Format32bppArgb
$data = $bmp.LockBits($rect2, 'ReadOnly', $fmt)
$bytes = New-Object byte[] ($data.Stride * $data.Height)
[System.Runtime.InteropServices.Marshal]::Copy($data.Scan0, $bytes, 0, $bytes.Length)
$bmp.UnlockBits($data)

# 自绘 preedit 特征扫描(T8 备选路径):光标行内联文本 + 2px 实心
# 下划线(≥200px 连续亮条——普通文本字形不会有整行实心长条)。
$underlineRows = 0
$underlineBestRun = 0
$textBandBright = 0
for ($y = 60; $y -lt [Math]::Min($wh - 10, 600); $y++) {
    $row = $y * $data.Stride
    $run = 0; $best = 0
    for ($x = 20; $x -lt $w - 20; $x += 2) {
        $i = $row + $x * 4
        $b = $bytes[$i]; $gg = $bytes[$i+1]; $rr = $bytes[$i+2]
        $bright = ($rr -gt 150 -and $gg -gt 150 -and $b -gt 150)
        if ($bright) {
            $run += 2
            if ($run -gt $best) { $best = $run }
            if ($y -lt 300) { $textBandBright++ }
        } else { $run = 0 }
    }
    if ($best -gt $underlineBestRun) { $underlineBestRun = $best }
    if ($best -ge 200) { $underlineRows++ }
}
$out = Join-Path $OutDir 'ime-preedit.png'
$bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)

$report = @()
$report += "window: ${w}x${wh}"
$report += "underline_rows_ge200px: $underlineRows"
$report += "longest_solid_bright_run_px: $underlineBestRun"
$report += "text_band_bright_pixels(y<300): $textBandBright"
$report += "verdict: $(if ($underlineRows -ge 1 -and $textBandBright -gt 300) { 'PREEDIT_SELF_DRAWN_VISIBLE' } else { 'NOT_DETECTED' })"
$report | Set-Content (Join-Path $OutDir 'ime-scan.txt')
Write-Output ($report -join "`n")
