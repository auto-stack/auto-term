param(
    [Parameter(Mandatory=$true)][string]$Tag,
    [Parameter(Mandatory=$true)][string]$Exe
)
# PLAN-026 C-lab: isolated app copy (app-lab) on port 17404, private rust-workspace
# target (no shared app-back.exe lock), engine DLL pinned via env.
$ev = 'D:\autostack\auto-term\docs\plans\evidence\026'
$env:AUTOTERM_ENGINE_DLL = 'D:\autostack\auto-term\target\debug\autoterm_core.dll'
$env:P024_TRACE = '1'
Remove-Item Env:\AUTO_LANG_CRATE -ErrorAction SilentlyContinue

$out = "$ev\lab-$Tag-out.log"; $err = "$ev\lab-$Tag-err.log"
Remove-Item $out,$err -ErrorAction SilentlyContinue
$p = Start-Process -FilePath $Exe -ArgumentList 'run','-r','vm','-B','17404' `
    -WorkingDirectory "$ev\app-lab" `
    -RedirectStandardOutput $out -RedirectStandardError $err -PassThru
Write-Output "FRONT_PID=$($p.Id) TAG=$Tag"

$ready = $false
for ($i = 0; $i -lt 96; $i++) {
    Start-Sleep 5
    if (Test-Path $out) {
        if (Select-String -Path $out -Pattern 'API server is ready|Server running on' -Quiet) { $ready = $true; break }
    }
    if ($p.HasExited) { Write-Output "FRONT_EXITED_EARLY"; break }
}
if (-not $ready) { Write-Output 'BACKEND_NOT_READY'; exit 1 }
Write-Output 'BACKEND_READY'

Start-Sleep 25   # settle into steady state
$prev = (Get-Process -Id $p.Id).CPU
$c1 = (Get-Item $err).Length
Start-Sleep 20
$proc = Get-Process -Id $p.Id -ErrorAction SilentlyContinue
$c2 = (Get-Item $err).Length
Write-Output ("CPU_20S=" + [math]::Round($proc.CPU - $prev,2) + " RESP=" + $proc.Responding + " ERR_GROWTH=" + ($c2-$c1))

foreach ($ep in @('/api/mux/tab-count','/api/mux/layout-version')) {
    $r = & curl.exe -s -o NUL -w '%{http_code} %{time_total}' "http://127.0.0.1:17404$ep" 2>$null
    Write-Output "CURL $ep -> $r"
}
foreach ($i in 1,2,3) {
    $r = & curl.exe -s -X POST -H 'Content-Type: application/json' -d '{\"k\":1}' -o NUL -w '%{http_code} %{time_total}' "http://127.0.0.1:17404/api/mux/rect-kind" 2>$null
    Write-Output "CURL rect-kind -> $r"
}
Write-Output 'LAB_DONE'
