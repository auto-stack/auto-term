# vue_crash_repro.ps1 — PLAN-021 T-01 [D1] 无头复现器
# 目标:复刻 020 复审"浏览器页面打开并持续轮询 ~2 分钟必崩"形态,
# 全程无人工交互;崩溃判定 = app-back.exe 消失 + WER Event 1000
# (0xc0000374,时间晚于本轮起点)。附带内存看门狗(>2GB 尝试同用户
# MiniDump;顺带观察 020 遗留的 20GB 内存实录是否同现)。
#
# 用法(任一 PowerShell,无需 admin):
#   powershell -NoProfile -File scripts/repro/vue_crash_repro.ps1 -Round 1
# 退出码:0=本轮复现崩溃  1=超时未崩  2=环境/启动失败
param(
  [int]$Round = 1,
  [int]$TimeoutSec = 300,
  [string]$RepoRoot = "D:\autostack\auto-term",
  [string]$AutoCli = "D:\autostack\auto-lang\target\debug\auto.exe",
  [string]$Browser = "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"
)
$ErrorActionPreference = "Continue"
$evid = Join-Path $RepoRoot "docs\plans\evidence\021"
New-Item -ItemType Directory -Force $evid | Out-Null
$tag = "r$Round"

function Cleanup {
  # 只杀本复现器拉起的东西:headless Edge 按 user-data-dir 命中,runner/back 按名;
  # vite(node)按 17400/17401 端口归属清——不按名杀 node,防误伤无关进程。
  Get-CimInstance Win32_Process -Filter "Name='msedge.exe'" -ErrorAction SilentlyContinue |
    Where-Object { $_.CommandLine -match "repro-edge-$tag" } |
    ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
  foreach ($n in @("app-back", "auto", "cargo")) {
    Get-Process $n -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
  }
  foreach ($p in @(17400, 17401)) {
    Get-NetTCPConnection -LocalPort $p -State Listen -ErrorAction SilentlyContinue |
      Select-Object -ExpandProperty OwningProcess -Unique |
      ForEach-Object { Stop-Process -Id $_ -Force -ErrorAction SilentlyContinue }
  }
}

function Port-Up([int]$p) {
  $c = New-Object Net.Sockets.TcpClient
  try { $c.Connect("127.0.0.1", $p); return $c.Connected } catch { return $false } finally { $c.Close() }
}

# ── 0) 环境预清 ─────────────────────────────────────────────
Cleanup
Start-Sleep 1
if ((Port-Up 17401) -or (Port-Up 17400)) {
  Write-Output "[$tag] RESULT SETUP_FAIL ports 17400/17401 occupied by foreign process"
  exit 2
}

# ── 1) 起栈: auto run -r vue(cwd=app;含 vite 17400 + axum back 17401)──
$runLog = Join-Path $evid "repro-$tag-run.log"
$runner = Start-Process -FilePath $AutoCli -ArgumentList "run", "-r", "vue" `
  -WorkingDirectory (Join-Path $RepoRoot "app") `
  -RedirectStandardOutput $runLog -RedirectStandardError (Join-Path $evid "repro-$tag-run.err.log") `
  -PassThru -NoNewWindow
Write-Output "[$tag] runner pid=$($runner.Id) started $(Get-Date -Format 'HH:mm:ss')"

# ── 2) 等 back 上线(17401;首轮含 sidecar 重编译,预算放宽)──
$boot = [Diagnostics.Stopwatch]::StartNew()
$backPid = 0
while ($boot.Elapsed.TotalSeconds -lt 300) {
  Start-Sleep 2
  $b = @(Get-Process app-back -ErrorAction SilentlyContinue)
  if ($b.Count -gt 0 -and (Port-Up 17401)) { $backPid = $b[0].Id; break }
  if ($runner.HasExited) { break }
}
if (-not $backPid) {
  Write-Output "[$tag] RESULT SETUP_FAIL back not up in 300s (runner exited=$($runner.HasExited))"
  Get-Content $runLog -Tail 30 -ErrorAction SilentlyContinue
  Cleanup; exit 2
}
$bootSec = [int]$boot.Elapsed.TotalSeconds
Write-Output "[$tag] back pid=$backPid up after ${bootSec}s"

# ── 3) 开无头浏览器页面(轮询形态的驱动臂)─────────────────
$prof = Join-Path $evid "repro-edge-$tag"
$browser = Start-Process -FilePath $Browser -ArgumentList `
  "--headless=new", "--disable-gpu", "--no-first-run", `
  "--user-data-dir=$prof", "--window-size=1000,700", `
  "http://localhost:17400" -PassThru -NoNewWindow
Write-Output "[$tag] headless browser pid=$($browser.Id) page open $(Get-Date -Format 'HH:mm:ss')"

# ── 4) 监控:崩溃判定(WER 1000 + 进程消失)+ 内存看门狗 ──
$startWall = Get-Date
$sw = [Diagnostics.Stopwatch]::StartNew()
$memLog = Join-Path $evid "repro-$tag-mem.log"
"time_s back_count pids ws_mb_list" | Out-File $memLog
$dumped = $false
$crash = $false
while ($sw.Elapsed.TotalSeconds -lt $TimeoutSec) {
  Start-Sleep 5
  $b = @(Get-Process app-back -ErrorAction SilentlyContinue)
  if ($b.Count -eq 0) {
    $t = [int]$sw.Elapsed.TotalSeconds
    Start-Sleep 3   # 给 WER 一点落账时间
    $wer = Get-WinEvent -FilterHashtable @{LogName='Application'; Id=1000} -MaxEvents 30 -ErrorAction SilentlyContinue |
      Where-Object { $_.Message -match 'app-back' -and $_.TimeCreated -ge $startWall.AddSeconds(-30) } |
      Select-Object -First 1
    if ($wer) {
      $code = if ($wer.Message -match 'Exception code: (0x[0-9a-fA-F]+)') { $Matches[1] } else { "?" }
      Write-Output "[$tag] CRASH at ${t}s after page open; WER exception=$code"
      $wer.Message | Out-File (Join-Path $evid "repro-$tag-wer.txt")
      $crash = ($code -eq "0xc0000374")
    } else {
      Write-Output "[$tag] back died at ${t}s but NO WER event (clean exit?)"
    }
    break
  }
  $t = [int]$sw.Elapsed.TotalSeconds
  $wsList = ($b | ForEach-Object { [int]($_.WorkingSet64 / 1MB) }) -join "/"
  $pidList = ($b | ForEach-Object { $_.Id }) -join "/"
  "$t $($b.Count) $pidList $wsList" | Add-Content $memLog
  if (($t % 30) -eq 0) { Write-Output "[$tag] alive ${t}s n=$($b.Count) ws=${wsList}MB" }
  # 看门狗:>2GB 且未转储 → comsvcs MiniDump(同用户免 admin,best effort)
  $wsMax = ($b | ForEach-Object { $_.WorkingSet64 / 1MB } | Measure-Object -Maximum).Maximum
  if (-not $dumped -and $wsMax -gt 2048) {
    $dumped = $true
    $dumpPath = Join-Path $evid "repro-$tag-back.dmp"
    Start-Process -FilePath "rundll32.exe" `
      -ArgumentList "comsvcs.dll,", "MiniDump", "$($b[0].Id)", "`"$dumpPath`"", "full" `
      -NoNewWindow -Wait -ErrorAction SilentlyContinue
    Write-Output "[$tag] watchdog minidump attempt at ws=${wsMax}MB -> $dumpPath"
  }
}

# ── 5) 收尾 ────────────────────────────────────────────────
Cleanup
$verdict = if ($crash) { "CRASH_0xc0000374" } else { "NO_CRASH" }
Write-Output "[$tag] RESULT $verdict round=$Round elapsed=$([int]$sw.Elapsed.TotalSeconds)s boot=${bootSec}s dumped=$dumped"
if ($crash) { exit 0 } else { exit 1 }
