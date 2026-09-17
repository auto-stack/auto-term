# probe_ma.ps1 — PLAN-021 T-05 双区探针驱动(020 spikes/020-split-probe 复跑)
# 流程:起 VM 探针(AUTO_MA_DBG=1)→ 等窗 → 截前照 → 合成点击大区+细条
# → 截后照。计数读数由人工/审查从 PNG 顶行 "hitsBig=N hitsStrip=M" 判读。
# 用法:
#   powershell -NoProfile -File scripts/repro/probe_ma.ps1 -Tag baseline `
#     -AutoCli D:\autostack\auto-lang\target\debug\auto.exe
# 退出码:0=探针跑完(无论计数),1=环境失败
param(
  [string]$Tag = "run",
  [string]$AutoCli = "D:\autostack\auto-lang\target\debug\auto.exe",
  [string]$RepoRoot = "D:\autostack\auto-term",
  [int]$BigX = 150, [int]$BigY = 200,
  [int]$StripX = 303, [int]$StripY = 200
)
$ErrorActionPreference = "Continue"
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class M5 {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint f);
  public struct RECT { public int L, T, R, B; }
}
'@
[void][M5]::SetProcessDPIAware()
$evid = Join-Path $RepoRoot "docs\plans\evidence\021"
New-Item -ItemType Directory -Force $evid | Out-Null

# 0) 预清:只杀标题含 probe020 的旧探针
Get-Process auto -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle -like "*probe020*" } | Stop-Process -Force
Start-Sleep 1

# 1) 起探针(VM 形态;AUTO_MA_DBG=1 开仪器)。stdout/stderr 走文件——
# 管道不排空会写满阻塞 auto(auto-man 横幅+构建进度全进 stdout)。
$log = Join-Path $evid "probe-$Tag-run.log"
$outLog = Join-Path $evid "probe-$Tag-out.log"
$env:AUTO_MA_DBG = "1"  # Start-Process 继承当前会话环境,须先置
$runner = Start-Process -FilePath $AutoCli -ArgumentList "run", "-r", "vm" `
  -WorkingDirectory (Join-Path $RepoRoot "spikes\020-split-probe") `
  -RedirectStandardOutput $outLog -RedirectStandardError $log `
  -PassThru -NoNewWindow
Write-Output "[$Tag] probe pid=$($runner.Id) started $(Get-Date -Format 'HH:mm:ss') cli=$AutoCli"

# 2) 等窗
$win = $null
$sw = [Diagnostics.Stopwatch]::StartNew()
while ($sw.Elapsed.TotalSeconds -lt 180) {
  Start-Sleep 2
  $win = Get-Process auto -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle -like "*probe020*" -and $_.MainWindowHandle -ne 0 } | Select-Object -First 1
  if ($win) { break }
}
if (-not $win) {
  Write-Output "[$Tag] RESULT SETUP_FAIL no probe020 window in 180s"
  Get-Content $log -Tail 30 -ErrorAction SilentlyContinue
  Stop-Process -Id $runner.Id -Force -ErrorAction SilentlyContinue
  exit 1
}
Start-Sleep 2  # 首帧稳定
[M5]::SetForegroundWindow($win.MainWindowHandle) | Out-Null
Start-Sleep -Milliseconds 500

function Shot([string]$name) {
  $r = New-Object M5+RECT
  [M5]::GetWindowRect($win.MainWindowHandle, [ref]$r) | Out-Null
  $w = $r.R - $r.L; $h = $r.B - $r.T
  $bmp = New-Object System.Drawing.Bitmap($w, $h)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $dc = $g.GetHdc()
  [M5]::PrintWindow($win.MainWindowHandle, $dc, 2) | Out-Null
  $g.ReleaseHdc($dc); $g.Dispose()
  $path = Join-Path $evid $name
  $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
  $bmp.Dispose()
  Write-Host "[$Tag] shot $name ${w}x${h} rect=($($r.L),$($r.T))"
  return @(0 + $r.L, 0 + $r.T, 0 + $w, 0 + $h)
}

$before = Shot "probe-$Tag-before.png"
Write-Output "[$Tag] window origin=($($before[0]),$($before[1])) size=$($before[2])x$($before[3])"

function Click([int]$x, [int]$y, [string]$what) {
  [M5]::SetCursorPos($x, $y) | Out-Null
  Start-Sleep -Milliseconds 250
  [M5]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)  # down
  Start-Sleep -Milliseconds 90
  [M5]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)  # up
  Start-Sleep -Milliseconds 500
  Write-Output "[$Tag] clicked $what at ($x,$y)"
}

# 3) 点击(窗口物理坐标 = origin + 相对偏移)
Click ($before[0] + $BigX) ($before[1] + $BigY) "big"
Click ($before[0] + $StripX) ($before[1] + $StripY) "strip"

$null = Shot "probe-$Tag-after.png"

# 4) 收尾:落仪器日志,杀探针
Start-Sleep 1
Get-Content $log -ErrorAction SilentlyContinue | Select-String "MA_BUILD|UI_EVENT|TERM_PRESS|TERM_WHEEL" | Select-Object -First 40 | ForEach-Object { Write-Output "[$Tag] LOG: $($_.Line)" }
Stop-Process -Id $runner.Id -Force -ErrorAction SilentlyContinue
Write-Output "[$Tag] RESULT DONE tag=$Tag"
exit 0
