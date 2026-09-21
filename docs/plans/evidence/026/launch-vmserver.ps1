param([string]$Exe = 'D:\autostack\.wt\inv-026\target\debug\auto.exe', [string]$Tag = 'vmvm')
$env:P024_TRACE = '1'
Remove-Item Env:\AUTO_LANG_CRATE -ErrorAction SilentlyContinue
$ev  = 'D:\autostack\auto-term\docs\plans\evidence\026'
$p = Start-Process -FilePath $Exe -ArgumentList 'run','-r','vm','--server=vm','-B','17403' `
    -WorkingDirectory 'D:\autostack\auto-term\app' `
    -RedirectStandardOutput "$ev\vmvm-$Tag-out.log" `
    -RedirectStandardError  "$ev\vmvm-$Tag-err.log" -PassThru
Write-Output ("FRONT_PID=" + $p.Id)
