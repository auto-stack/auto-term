param(
    [Parameter(Mandatory=$true)][string]$Tag,
    [Parameter(Mandatory=$true)][string]$Exe,
    [string]$LangCrate = ''
)
# PLAN-026 VM hang investigation: controlled quantified run on private port 17402.
# Never touches 17401 (other sessions may own it); PID written to file for cleanup.
$ev = 'D:\autostack\auto-term\docs\plans\evidence\026'
if ($LangCrate -ne '') { $env:AUTO_LANG_CRATE = $LangCrate } else { Remove-Item Env:\AUTO_LANG_CRATE -ErrorAction SilentlyContinue }

$out = "$ev\q-$Tag-out.log"; $err = "$ev\q-$Tag-err.log"
Remove-Item $out,$err -ErrorAction SilentlyContinue
$p = Start-Process -FilePath $Exe -ArgumentList 'run','-r','vm','-B','17402' `
    -WorkingDirectory 'D:\autostack\auto-term\app' `
    -RedirectStandardOutput $out -RedirectStandardError $err -PassThru
$pid_ = $p.Id
"$pid_" | Out-File "$ev\q-$Tag.pid" -Encoding ascii
Write-Output "FRONT_PID=$pid_ TAG=$Tag"

# wait for backend ready (fresh app-back build can take minutes; the runner
# gives up waiting after 60s and prints "continuing anyway" while cargo still
# builds — accept either marker, or direct port listen as final fallback)
$ready = $false
for ($i = 0; $i -lt 96; $i++) {
    Start-Sleep 5
    if (Test-Path $out) {
        if (Select-String -Path $out -Pattern 'API server is ready|Server running on' -Quiet) { $ready = $true; break }
    }
    if ($p.HasExited) { Write-Output "FRONT_EXITED_EARLY code=$($p.ExitCode)"; break }
}
if (-not $ready) { Write-Output 'BACKEND_NOT_READY'; exit 1 }
Write-Output 'BACKEND_READY'

# settle 20s, then sample err-log growth (4 lines per completed tick) for 45s
Start-Sleep 20
$counts = @()
for ($i = 0; $i -lt 45; $i++) { Start-Sleep 1; $counts += (Get-Item $err).Length }
$deltas = for ($i = 1; $i -lt $counts.Count; $i++) { $counts[$i] - $counts[$i-1] }
Write-Output ('BYTES-PER-SEC=' + ($deltas -join ','))

# probe backend latency directly (rules backend slowness in/out)
foreach ($ep in @('/api/mux/tab-count','/api/mux/cols','/api/mux/rows')) {
    $r = & curl.exe -s -o NUL -w '%{http_code} %{time_total}' "http://127.0.0.1:17402$ep" 2>$null
    Write-Output "CURL $ep -> $r"
}
Write-Output 'QUANT_DONE'
