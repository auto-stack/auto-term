# PLAN-005 T7 取证:DECSCUSR 光标形状像素证据(程序化)。
# 前提:autoterm.exe(dev-tools)已运行 ~StartedBeforeMs,且已排定
#   --dev-autotype '<Ms>:cls\r' 与 '<Ms>:(20 空格)+ESC[N q'——光标
#   被推到已知列(≈col 39),该列带无文本,亮像素只可能来自光标。
# 产物:cursor-<shape>.png + cursor-<shape>-scan.txt(条带判别):
#   block:     亮像素 ≥500(h≈40px 满格反色)
#   underline: 30..140 亮像素且行跨度 ≤10(格底细条)
#   beam:      100..350 亮像素且列跨度 ≤10(格左细条)
param(
    [string]$Shape = "underline",
    [int]$CaptureMs = 6500,
    [int]$StartedBeforeMs = 4000,
    # 光标格(dev 转储 cursor_drawn_at 同源:命令执行后提示符重画,
    # 光标恒落 (行,列);条带 = 该格 ±8px,行带同域——避免扫到
    # 同列其他行的文本字形)
    [int]$Row = 20,
    [int]$Col = 19,
    [double]$ClientTop = 58.0,
    [double]$CellW = 19.0,
    [double]$LineH = 40.5,
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

$outPng = Join-Path $OutDir "cursor-$Shape.png"
$wait = $CaptureMs - $StartedBeforeMs
if ($wait -gt 0) { Start-Sleep -Milliseconds $wait }
$bmp = Capture-Window $outPng

# 光标格条带(±8px)内亮像素(DEFAULT_FG e8e8e8±30)计数 + bbox。
$X0 = [int]($Col * $CellW) - 8
$X1 = [int]($Col * $CellW + $CellW) + 8
$Y0 = [int]($ClientTop + $Row * $LineH) - 8
$Y1 = [int]($ClientTop + ($Row + 1) * $LineH) + 8
$d = Lock-Bytes $bmp
$minX = $d.w; $maxX = -1; $minY = $d.h; $maxY = -1; $count = 0
for ($y = [Math]::Max(0, $Y0); $y -lt [Math]::Min($d.h, $Y1); $y++) {
    $row = $y * $d.stride
    for ($x = [Math]::Max(0, $X0); $x -lt [Math]::Min($d.w, $X1); $x++) {
        $i = $row + $x * 4
        if ([Math]::Abs($d.b[$i] - 232) -le 30 -and
            [Math]::Abs($d.b[$i+1] - 232) -le 30 -and
            [Math]::Abs($d.b[$i+2] - 232) -le 30) {
            $count++
            if ($x -lt $minX) { $minX = $x }
            if ($x -gt $maxX) { $maxX = $x }
            if ($y -lt $minY) { $minY = $y }
            if ($y -gt $maxY) { $maxY = $y }
        }
    }
}
$bw = [Math]::Max(0, $maxX - $minX + 1)
$bh = [Math]::Max(0, $maxY - $minY + 1)
$ok = switch ($Shape) {
    "block" { $count -ge 500 }
    "underline" { ($count -ge 30) -and ($count -le 140) -and ($bh -le 10) }
    "beam" { ($count -ge 100) -and ($count -le 350) -and ($bw -le 10) }
    default { $false }
}
$report = @()
$report += "cell_band: x ${X0}-${X1} y ${Y0}-${Y1} (row $Row col $Col)  expect_shape: $Shape"
$report += "bright_pixels: $count  bbox: ($minX,$minY)-($maxX,$maxY)  ${bw}x${bh}"
$report += "cursor_shape_ok: $ok"
$report | Set-Content (Join-Path $OutDir "cursor-$Shape-scan.txt")
Write-Output ($report -join "`n")
