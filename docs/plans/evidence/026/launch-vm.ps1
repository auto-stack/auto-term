$env:AUTO_LANG_CRATE = 'D:\autostack\.wt\lang-025\auto-lang\crates\auto-lang'
$exe = 'D:\autostack\.wt\lang-025\auto-lang\target\debug\auto.exe'
$ev  = 'D:\autostack\auto-term\docs\plans\evidence\026'
$p = Start-Process -FilePath $exe -ArgumentList 'run','-r','vm' `
    -WorkingDirectory 'D:\autostack\auto-term\app' `
    -RedirectStandardOutput "$ev\vm-front-out2.log" `
    -RedirectStandardError  "$ev\vm-front-err2.log" -PassThru
Write-Output ("FRONT_PID=" + $p.Id)
