$env:P024_TRACE = '1'
Remove-Item Env:\AUTO_LANG_CRATE -ErrorAction SilentlyContinue
$exe = 'D:\autostack\auto-lang\target\debug\auto.exe'
$ev  = 'D:\autostack\auto-term\docs\plans\evidence\026'
$p = Start-Process -FilePath $exe -ArgumentList 'run','-r','vm','-B','17403' `
    -WorkingDirectory 'D:\autostack\auto-term\app' `
    -RedirectStandardOutput "$ev\t3-out.log" `
    -RedirectStandardError  "$ev\t3-err.log" -PassThru
Write-Output ("FRONT_PID=" + $p.Id)
