# PLAN-022 探针 screen v2:前台化 + 屏幕区域捕获 + 内容校验(防误抓他窗)
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class M32 {
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  public struct RECT { public int L, T, R, B; }
}
'@
[void][M32]::SetProcessDPIAware()

$exe = "D:\autostack\auto-term\target\debug\auto-term.exe"
$errLog = "D:\autostack\auto-term\docs\plans\evidence\022\t05-slotlog.err"
$ev = "D:\autostack\auto-term\docs\plans\evidence\022"
$env:AUTOTERM_ENGINE_DLL = "D:\autostack\auto-term\target\debug\autoterm_core.dll"
$env:AUTO_MA_DBG = "1"
$p = Start-Process -FilePath $exe -PassThru -RedirectStandardError $errLog
Start-Sleep -Seconds 3
$p.Refresh()
$hw = $p.MainWindowHandle

$rc = New-Object M32+RECT
[M32]::GetWindowRect($hw, [ref]$rc) | Out-Null
$w = $rc.R - $rc.L; $h = $rc.B - $rc.T

$ok = $false
foreach ($try in 1..4) {
  [M32]::SetForegroundWindow($hw) | Out-Null
  Start-Sleep -Milliseconds 1200
  $rc = New-Object M32+RECT
  [M32]::GetWindowRect($hw, [ref]$rc) | Out-Null
  $w = $rc.R - $rc.L; $h = $rc.B - $rc.T
  $bmp = New-Object System.Drawing.Bitmap($w, $h)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($rc.L, $rc.T, 0, 0, (New-Object System.Drawing.Size($w, $h)))
  $g.Dispose()
  # 校验:窗内中缝应有分隔条灰(#8899aa 族,R≈G>B 且亮度中等)且两侧为终端黑
  $grey = 0
  for ($y = 300; $y -lt 1400; $y += 20) {
    $c = $bmp.GetPixel([int]($w*0.5), $y)
    if ($c.R -gt 100 -and $c.G -gt 120 -and $c.B -gt 140 -and $c.B -ge $c.R) { $grey++ }
  }
  "try=$try grey=$grey"
  if ($grey -ge 20) {
    $bmp.Save("$ev\screen-split.png",[System.Drawing.Imaging.ImageFormat]::Png)
    "SAVED screen-split.png"
    $ok = $true
  }
  $bmp.Dispose()
  if ($ok) { break }
}
Stop-Process -Id $p.Id -Force
Start-Sleep -Milliseconds 600
if ($ok) { "DONE-OK" } else { "DONE-UNVERIFIED" }
