# PLAN-005 T6 取证:--selection-color 命中色变更(程序化)。
# 前提:autoterm.exe(dev-tools)已运行 ~StartedBeforeMs,带
#   --selection-color <hex> 与排定的 --dev-select(SelectMs 注入)。
# 产物:selcolor-before.png / selcolor-after.png + selcolor-scan.txt:
#   红主导像素(r > g+40 且 r > b+40)行带 = 新选中色;断言
#   selcolor_changed: 命中行≥3(默认灰 71,73,76 不红主导)。
param(
    [int]$SelectMs = 6000,
    [int]$StartedBeforeMs = 4000,
    [int]$CaptureAfterMs = 1200,
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
        Write-Output "attempt ${attempt}: PrintWindow invalid, retrying"
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

$beforePath = Join-Path $OutDir 'selcolor-before.png'
$afterPath = Join-Path $OutDir 'selcolor-after.png'

$wait0 = $SelectMs - 800 - $StartedBeforeMs
if ($wait0 -gt 0) { Start-Sleep -Milliseconds $wait0 }
$null = Capture-Window $beforePath
Start-Sleep -Milliseconds (800 + $CaptureAfterMs)
$after = Capture-Window $afterPath

# 高亮行检测:红主导行(r > g+40 且 r > b+40 的像素 ≥ 200/行,
# 在选中带行区间内)——不预设混合模型,直接测渲染产物主色。
$d = Lock-Bytes $after
$hitRows = New-Object System.Collections.Generic.List[int]
$sumR = 0L; $sumG = 0L; $sumB = 0L; $matched = 0L
for ($y = 100; $y -lt $d.h; $y++) {
    $row = $y * $d.stride
    $rowRed = 0
    for ($x = 0; $x -lt $d.w; $x += 2) {
        $i = $row + $x * 4
        $r = $d.b[$i+2]; $g = $d.b[$i+1]; $b = $d.b[$i]
        if ($r -gt $g + 40 -and $r -gt $b + 40 -and $r -gt 60) {
            $rowRed++
            $matched++
            $sumB += $d.b[$i]; $sumG += $d.b[$i+1]; $sumR += $r
        }
    }
    if ($rowRed -ge 200) { $hitRows.Add($y) }
}

$report = @()
$report += "window_px: $($d.w)x$($d.h)"
$report += "expect: red-dominant wash rows (ff0000 selection-color), not default gray 71,73,76"
$report += "red_dominant_pixels: $matched"
if ($matched -gt 0) {
    $report += ("avg_red_rgb: {0:F1},{1:F1},{2:F1}" -f ($sumR/$matched), ($sumG/$matched), ($sumB/$matched))
}
$report += "highlight_rows_total: $($hitRows.Count)"
$changed = ($hitRows.Count -ge 3) -and ($matched -gt 1000)
$report += "selcolor_changed: $changed"
$report | Set-Content (Join-Path $OutDir 'selcolor-scan.txt')
Write-Output ($report -join "`n")
