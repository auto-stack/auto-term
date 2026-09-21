# PLAN-027 T-05 · split 前端内存斜率回归钉
# 用法: ./027-mem-slope.ps1 [-Pid <前端进程号>] [-Minutes 6] [-MaxMBperMin 0.5]
# 语义: 对运行中的 VM split 前端(auto.exe)每 30s 采一次 PrivateMemorySize64,
#       线性拟合斜率,超阈值退出码 1(修前基线 +2.23MB/min,evidence/027)。
# 备注: 实例由外部拉起(auto run -r vm + AUTOTERM_ENGINE_DLL 钉 DLL,见
#       记忆 vm-track-launch-runbook);本脚本只管采样断言,免重建。
param(
    [Parameter(Mandatory = $true)][int]$Pid_,
    [int]$Minutes = 6,
    [double]$MaxMBperMin = 0.5
)
$ErrorActionPreference = 'Stop'
$samples = @()
$deadline = (Get-Date).AddMinutes($Minutes)
while ((Get-Date) -lt $deadline) {
    Start-Sleep -Seconds 30
    $p = Get-Process -Id $Pid_ -ErrorAction Stop   # 进程亡 = 立即失败
    $p.Refresh()
    $samples += [pscustomobject]@{ T = (Get-Date); Priv = $p.PrivateMemorySize64 }
}
if ($samples.Count -lt 6) { "样本不足($($samples.Count))"; exit 2 }
# 最小二乘斜率(MB/min)
$n = $samples.Count
$xs = 0..($n - 1) | ForEach-Object { $_ * 0.5 }        # 分钟
$ys = $samples | ForEach-Object { $_.Priv / 1MB }
$mx = ($xs | Measure-Object -Average).Average
$my = ($ys | Measure-Object -Average).Average
$num = 0; $den = 0
for ($i = 0; $i -lt $n; $i++) {
    $num += ($xs[$i] - $mx) * ($ys[$i] - $my)
    $den += [math]::Pow(($xs[$i] - $mx), 2)
}
$slope = $num / $den
"slope = {0:+0.00} MB/min (n=$n, {1}min)  threshold = $MaxMBperMin" -f $slope, $Minutes
if ($slope -gt $MaxMBperMin) {
    "FAIL: 斜率超阈——per-tick 泄漏回归(DEBT #26 修前 +2.23MB/min)"
    exit 1
}
"PASS"
exit 0
