# PLAN-005 T3 取证:块选高亮**列带几何**像素扫描(程序化)。
# 前提:autoterm.exe(dev-tools)已运行 ~StartedBeforeMs,--dev-select
# 已排定块选注入(SelectMs 为其注入时刻,自 app 启动起算)。
# 用法(编排示例):
#   autoterm.exe --dev-autotype '4000:cls\r' --dev-autotype '4500:echo AAAABBBB;...' `
#     --dev-select '6000:block:1:2-3:6' --dev-exit-after 12 --dev-dump block-dump.txt &
#   Start-Sleep -Milliseconds 3500; ./scan-block.ps1
# 产物:block-before.png / block-highlight.png + block-scan.txt(几何报告):
#   - 命中行的高亮像素须构成**同一列带**(各行 x 起止 ±3px 一致,
#     即矩形对齐,不跨全行)——block_aligned 即本取证断言;
#   - band_cells = 带宽物理px / (cell_w × scale),应≈注入列数差。
param(
    [int]$SelectMs = 6000,
    [int]$StartedBeforeMs = 3500,
    [int]$CaptureAfterMs = 1200,
    [double]$CellW = 9.375,
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

# 高亮行检测:行内最长**连续**命中 run(2px 采样,4px 断缝容忍)≥30px。
# 文本字形反锯齿的散点灰色过不了连续 run 门;整行选会给出全宽 run。
function Find-HighlightRuns($bmp) {
    $d = Lock-Bytes $bmp
    $target = @(71, 73, 76); $tol = 14
    $runs = @{}
    for ($y = 0; $y -lt $d.h; $y++) {
        $row = $y * $d.stride
        $bestLen = 0; $bestStart = -1; $bestEnd = -1
        $runStart = -1; $lastHit = -100
        for ($x = 0; $x -lt $d.w; $x += 2) {
            $i = $row + $x * 4
            $hit = ([Math]::Abs($d.b[$i] - $target[2]) -le $tol -and
                [Math]::Abs($d.b[$i+1] - $target[1]) -le $tol -and
                [Math]::Abs($d.b[$i+2] - $target[0]) -le $tol)
            if ($hit) {
                if ($runStart -lt 0) { $runStart = $x }
                elseif ($x - $lastHit -gt 4) {
                    if ($lastHit - $runStart -gt $bestLen) {
                        $bestLen = $lastHit - $runStart; $bestStart = $runStart; $bestEnd = $lastHit
                    }
                    $runStart = $x
                }
                $lastHit = $x
            }
        }
        if ($runStart -ge 0 -and $lastHit - $runStart -gt $bestLen) {
            $bestLen = $lastHit - $runStart; $bestStart = $runStart; $bestEnd = $lastHit
        }
        if ($bestLen -ge 30) { $runs[$y] = @($bestStart, $bestEnd) }
    }
    return $runs
}

$beforePath = Join-Path $OutDir 'block-before.png'
$afterPath = Join-Path $OutDir 'block-highlight.png'

# 注入前先拍一帧(期望无高亮);再等到注入时刻后拍第二帧。
$wait0 = $SelectMs - $StartedBeforeMs
if ($wait0 -gt 0) {
    # 注入前拍 before:再留 800ms 余量避免撞上注入瞬间
    $beforeWait = $wait0 - 800
    if ($beforeWait -gt 0) { Start-Sleep -Milliseconds $beforeWait }
    $null = Capture-Window $beforePath
    Start-Sleep -Milliseconds (800 + $CaptureAfterMs)
} else {
    Start-Sleep -Milliseconds $CaptureAfterMs
}
$after = Capture-Window $afterPath

$scale = $after.Width / $LogicalWidth
$runs = Find-HighlightRuns $after

# 连续 y 行聚带;带内统计 x 起止一致性(±3px)。
$ys = $runs.Keys | Sort-Object
$report = @()
$report += "window_px: $($after.Width)x$($after.Height)  scale=$scale"
$report += "highlight_rows: $($ys.Count)"
if ($ys.Count -eq 0) {
    $report += "block_aligned: false (no highlight rows)"
} else {
    # 聚连续行带
    $bands = New-Object System.Collections.Generic.List[object]
    $start = $ys[0]; $prev = $ys[0]
    for ($k = 1; $k -le $ys.Count; $k++) {
        $cur = if ($k -lt $ys.Count) { $ys[$k] } else { -999 }
        if ($cur -ne $prev + 1) {
            $bands.Add(@{ y0 = $start; y1 = $prev })
            $start = $cur
        }
        $prev = $cur
    }
    foreach ($b in $bands) {
        $xs0 = ($b.y0..$b.y1 | ForEach-Object { $runs[$_][0] })
        $xs1 = ($b.y0..$b.y1 | ForEach-Object { $runs[$_][1] })
        $x0 = ($xs0 | Measure-Object -Minimum).Minimum
        $x0max = ($xs0 | Measure-Object -Maximum).Maximum
        $x1 = ($xs1 | Measure-Object -Minimum).Minimum
        $x1max = ($xs1 | Measure-Object -Maximum).Maximum
        $aligned = ([Math]::Abs($x0max - $x0) -le 3) -and ([Math]::Abs($x1max - $x1) -le 3)
        $wpx = $x1max - $x0 + 1
        $cells = $wpx / ($CellW * $scale)
        $report += "band: y $($b.y0)-$($b.y1) height=$(($b.y1-$b.y0+1)) x ${x0}-${x1max} width_px=$wpx"
        $report += ("band_cells: {0:F2} (cell_w={1} x scale={2})" -f $cells, $CellW, $scale)
        $report += "band_x_spread: start=$([Math]::Abs($x0max-$x0)) end=$([Math]::Abs($x1max-$x1)) (tol 3)"
        $report += "block_aligned: $aligned"
    }
}
$report | Set-Content (Join-Path $OutDir 'block-scan.txt')
Write-Output ($report -join "`n")
