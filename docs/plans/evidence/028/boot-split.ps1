# boot-split.ps1 — PLAN-028 实机:分体轨拉起(026 launch-vm3 范式 + 存值文件钉扎)
$env:AUTOTERM_ENGINE_DLL = 'D:\autostack\.wt\fix-resize-chain\auto-term\target\debug\autoterm_core.dll'
$env:AUTO_VM_STORAGE_FILE = 'D:\autostack\.wt\fix-resize-chain\p028-storage.json'
$env:P028_TRACE = '1'
Remove-Item Env:\AUTO_LANG_CRATE -ErrorAction SilentlyContinue
Remove-Item $env:AUTO_VM_STORAGE_FILE -ErrorAction SilentlyContinue
$exe = 'D:\autostack\.wt\fix-resize-chain\auto-lang\target\debug\auto.exe'
$ev  = 'D:\autostack\.wt\fix-resize-chain\auto-term\docs\plans\evidence\028'
$p = Start-Process -FilePath $exe -ArgumentList 'run','-r','vm','-B','17428' `
    -WorkingDirectory 'D:\autostack\.wt\fix-resize-chain\auto-term\app' `
    -RedirectStandardOutput "$ev\logs\boot-out.log" `
    -RedirectStandardError  "$ev\logs\boot-err.log" -PassThru
Write-Output ("FRONT_PID=" + $p.Id)
