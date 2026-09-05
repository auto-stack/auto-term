# PLAN-004 T3/T6 取证:选中高亮像素扫描(程序化,不依赖视觉模型)
# 用法:scan-select.ps1 -BeforeMs 3500 -SelectMs 6000 -CaptureAfterMs 4500
#   前提:autoterm.exe 已在运行,--dev-select 已排定(SelectMs 为其注入时刻)
# 产物:select-highlight.png(选中后窗口截屏)+ select-scan.txt(差分报告)
param(
    [int]$BeforeMs = 3500,
    [int]$SelectMs = 6000,
    [int]$CaptureAfterMs = 4500,
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
# DPI 感知:否则在高缩放屏(本机 200%)只拿到左上象限的裁剪截屏
[Win]::SetProcessDPIAware() | Out-Null

function Find-AutoTerm {
    # iced 窗口在首帧渲染后才可见(pwsh 冷启动可能拖慢数秒)——轮询等待
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
    # 窗口首显动画/迟到渲染期间矩形会变——等两次探测一致且够大
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
    # 强判别:存在 ≥300px 连续 DEFAULT_BG(16,20,24±3) 的纯背景行
    # (终端空白行特征;暗色网页不会有精确的该色长直通带)
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
    # PrintWindow(PW_RENDERFULLCONTENT) 内容捕获:遮挡/后台免疫
    # (实测本机 wgpu 窗口 OK;仍留内容自校验,黑帧/空帧即重试)
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

$beforePath = Join-Path $OutDir 'select-before.png'
$afterPath = Join-Path $OutDir 'select-highlight.png'

Start-Sleep -Milliseconds ($BeforeMs - 3500)
$null = Capture-Window $beforePath
Start-Sleep -Milliseconds ($SelectMs - $BeforeMs)
Start-Sleep -Milliseconds $CaptureAfterMs
$after = Capture-Window $afterPath

# 带色扫描(确定性,不依赖前后差分——pwsh 冷启动会拖内容漂移):
# 高亮 = DEFAULT_FG(e8e8e8) 25% alpha 叠 DEFAULT_BG(10 14 18) ≈ (71,73,76)。
# 逐行统计命中该色的像素占比,≥25% 行宽 = 高亮行;报告连续行带。
$pa = Lock-Bytes $after
$stride = $pa.stride; $width = $pa.w; $height = $pa.h
$target = @(71, 73, 76); $tol = 14

$highlightRows = New-Object System.Collections.Generic.List[int]
$sumR = 0L; $sumG = 0L; $sumB = 0L; $matched = 0L
for ($y = 0; $y -lt $height; $y++) {
    $row = $y * $stride
    $rowHits = 0
    for ($x = 0; $x -lt $width; $x += 2) {
        $i = $row + $x * 4
        if ([Math]::Abs($pa.b[$i] - $target[2]) -le $tol -and
            [Math]::Abs($pa.b[$i+1] - $target[1]) -le $tol -and
            [Math]::Abs($pa.b[$i+2] - $target[0]) -le $tol) {
            $rowHits++
            $matched++
            $sumB += $pa.b[$i]; $sumG += $pa.b[$i+1]; $sumR += $pa.b[$i+2]
        }
    }
    if ($rowHits * 2 -ge [int]($width * 0.25)) { $highlightRows.Add($y) }
}

# 连续行带分组
$bands = New-Object System.Collections.Generic.List[string]
if ($highlightRows.Count -gt 0) {
    $start = $highlightRows[0]; $prev = $highlightRows[0]
    for ($k = 1; $k -le $highlightRows.Count; $k++) {
        $cur = if ($k -lt $highlightRows.Count) { $highlightRows[$k] } else { -999 }
        if ($cur -ne $prev + 1) {
            $bands.Add("y ${start}-${prev}  height=$(($prev-$start+1))")
            $start = $cur
        }
        $prev = $cur
    }
}

$report = @()
$report += "window: ${width}x${height}"
$report += "highlight_pixels: $matched"
if ($matched -gt 0) {
    $report += ("avg_highlight_color_rgb: {0:F1},{1:F1},{2:F1} (target 71,73,76)" -f ($sumR/$matched), ($sumG/$matched), ($sumB/$matched))
}
$report += "highlight_rows_total: $($highlightRows.Count)"
$report += "highlight_bands: $($bands -join ' | ')"
$report | Set-Content (Join-Path $OutDir 'select-scan.txt')
Write-Output ($report -join "`n")
