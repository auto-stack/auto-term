# PLAN-005 T5 取证:右键菜单浮层像素扫描(开/合两拍,程序化)。
# 前提:autoterm.exe(dev-tools)已运行 ~StartedBeforeMs;
#   --dev-menu '<ms>:<x>:<y>' 已排定(菜单浮层出现);
#   --dev-autotype '<ms>:\x1b' 已排定(ESC 关闭)。
# 产物:menu-open.png / menu-closed.png + menu-scan.txt:
#   - 面板色 MENU_BG(42,46,52)±4 的像素 bbox;
#   - open: bbox ≈ 88×78 逻辑 px × scale,menu_panel_visible: true;
#   - closed: 命中像素 ≈ 0,menu_panel_visible: false(ESC 关闭证)。
param(
    [int]$MenuMs = 5000,
    [int]$EscMs = 6300,
    [int]$StartedBeforeMs = 3500,
    [int]$CaptureAfterMs = 800,
    [double]$PanelW = 88.0,
    [double]$PanelH = 78.0,
    [int]$LogicalWidth = 1000,
    [string]$OutDir = $PSScriptRoot
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public struct RECT { public int Left, Top, Right, Bottom; }
public class Win {
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint flags);
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
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

function Wait-StableWindow([IntPtr]$h) {
    $prev = $null
    for ($i = 0; $i -lt 40; $i++) {
        $r = New-Object RECT
        [Win]::GetWindowRect($h, [ref]$r) | Out-Null
        $w = $r.Right - $r.Left; $h2 = $r.Bottom - $r.Top
        if ($w -ge 800 -and $h2 -ge 500 -and $prev -and
            $prev.w -eq $w -and $prev.h -eq $h2) {
            return $r
        }
        $prev = @{ w = $w; h = $h2 }
        Start-Sleep -Milliseconds 250
    }
    throw "window size never stabilized"
}

function Test-TerminalBackground($bmp) {
    $d = Lock-Bytes $bmp
    for ($y = 60; $y -lt $d.h - 10; $y += 4) {
        $row = $y * $d.stride
        $run = 0; $best = 0
        for ($x = 8; $x -lt $d.w - 8; $x += 2) {
            $i = $row + $x * 4
            if ([Math]::Abs($d.b[$i] - 24) -le 3 -and
                [Math]::Abs($d.b[$i+1] - 20) -le 3 -and
                [Math]::Abs($d.b[$i+2] - 16) -le 3) {
                $run += 2
                if ($run -gt $best) { $best = $run }
            } else {
                $run = 0
            }
        }
        if ($best -ge 300) { return $true }
    }
    return $false
}

function Capture-Window([string]$path) {
    for ($attempt = 1; $attempt -le 4; $attempt++) {
        $p = Find-AutoTerm
        $r = Wait-StableWindow $p.MainWindowHandle
        $w = $r.Right - $r.Left; $h = $r.Bottom - $r.Top
        $bmp = New-Object System.Drawing.Bitmap($w, $h)
        $g = [System.Drawing.Graphics]::FromImage($bmp)
        $hdc = $g.GetHdc()
        $ok = [Win]::PrintWindow($p.MainWindowHandle, $hdc, 2)
        $g.ReleaseHdc($hdc)
        $g.Dispose()
        if ($ok -and (Test-TerminalBackground $bmp)) {
            $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
            return $bmp
        }
        Write-Output "attempt ${attempt}: PrintWindow invalid (ok=$ok), saving debug frame"
        $bmp.Save((Join-Path $OutDir "menu-debug-$attempt.png"), [System.Drawing.Imaging.ImageFormat]::Png)
        $bmp.Dispose()
        Start-Sleep -Milliseconds 300
    }
    throw "PrintWindow capture failed after 4 attempts"
}

function Lock-Bytes($bmp) {
    $rect = New-Object System.Drawing.Rectangle(0, 0, $bmp.Width, $bmp.Height)
    $fmt = [System.Drawing.Imaging.PixelFormat]::Format32bppArgb
    $data = $bmp.LockBits($rect, 'ReadOnly', $fmt)
    $bytes = New-Object byte[] ($data.Stride * $data.Height)
    [System.Runtime.InteropServices.Marshal]::Copy($data.Scan0, $bytes, 0, $bytes.Length)
    $bmp.UnlockBits($data)
    return @{ b = $bytes; stride = $data.Stride; w = $bmp.Width; h = $bmp.Height }
}

# 面板色扫描:MENU_BG(42,46,52)±4 的像素 bbox。
# 跳过窗顶 TopSkipPx(标题栏深色主题色同域);文本反锯齿灰阶散点
# 会污染 bbox——bbox 只统计命中数≥阈值的行/列(面板整行整列命中
# 密集,散点行列被滤掉)。
function Scan-Panel($bmp) {
    $d = Lock-Bytes $bmp
    $topSkip = 100
    $rowCount = New-Object int[] $d.h
    $colCount = New-Object int[] $d.w
    for ($y = $topSkip; $y -lt $d.h; $y++) {
        $row = $y * $d.stride
        for ($x = 0; $x -lt $d.w; $x++) {
            $i = $row + $x * 4
            if ([Math]::Abs($d.b[$i] - 52) -le 4 -and
                [Math]::Abs($d.b[$i+1] - 46) -le 4 -and
                [Math]::Abs($d.b[$i+2] - 42) -le 4) {
                $rowCount[$y]++
                $colCount[$x]++
            }
        }
    }
    $count = ($rowCount | Measure-Object -Sum).Sum
    $y0 = -1; $y1 = -1; $bestLen = 0; $best0 = -1; $best1 = -1
    for ($y = $topSkip; $y -lt $d.h; $y++) {
        if ($rowCount[$y] -ge 40) {
            if ($y0 -lt 0) { $y0 = $y }
            if ($y1 -ne $y - 1 -and $y0 -ge 0) {
                # 断带:结一段
                if ($y1 - $y0 -gt $bestLen) { $bestLen = $y1 - $y0; $best0 = $y0; $best1 = $y1 }
                $y0 = $y
            }
            $y1 = $y
        }
    }
    if ($y0 -ge 0 -and $y1 - $y0 -gt $bestLen) { $best0 = $y0; $best1 = $y1 }
    $x0 = -1; $x1 = -1
    for ($x = 0; $x -lt $d.w; $x++) {
        if ($colCount[$x] -ge 40) {
            if ($x0 -lt 0) { $x0 = $x }
            $x1 = $x
        }
    }
    return @{ count = $count; x0 = $x0; x1 = $x1; y0 = $best0; y1 = $best1 }
}

$openPath = Join-Path $OutDir 'menu-open.png'
$closedPath = Join-Path $OutDir 'menu-closed.png'

# 拍位 1:菜单注入后;拍位 2:ESC 之后。
$scale = 2.0  # 由首拍回填
$report = @()
$t1 = $MenuMs + $CaptureAfterMs - $StartedBeforeMs
if ($t1 -gt 0) { Start-Sleep -Milliseconds $t1 }
$open = Capture-Window $openPath
$scale = $open.Width / $LogicalWidth
$sOpen = Scan-Panel $open
$report += "window_px: $($open.Width)x$($open.Height)  scale=$scale"
$report += ("open_panel_pixels: {0}  bbox: ({1},{2})-({3},{4})" -f $sOpen.count, $sOpen.x0, $sOpen.y0, $sOpen.x1, $sOpen.y1)
$openW = [Math]::Max(0, $sOpen.x1 - $sOpen.x0 + 1)
$openH = [Math]::Max(0, $sOpen.y1 - $sOpen.y0 + 1)
$report += ("open_bbox_size_px: {0}x{1} (expect ~{2}x{3} = panel logical x scale)" -f $openW, $openH, [int]$PanelW, [int]$PanelH)
$visible = ($sOpen.count -ge 500) -and
    ([Math]::Abs($openW - $PanelW * $scale) -le 8) -and
    ([Math]::Abs($openH - $PanelH * $scale) -le 8)
$report += "menu_panel_visible: $visible"

$t2 = $EscMs - $MenuMs
if ($t2 -gt 0) { Start-Sleep -Milliseconds $t2 }
# ESC 拍后再留余量渲染
Start-Sleep -Milliseconds ($CaptureAfterMs + 400)
$closed = Capture-Window $closedPath
$sClosed = Scan-Panel $closed
$report += ("closed_panel_pixels: {0} (expect ~0 after ESC)" -f $sClosed.count)
$report += "menu_closed_by_esc: $($sClosed.count -lt 100)"

$report | Set-Content (Join-Path $OutDir 'menu-scan.txt')
Write-Output ($report -join "`n")
