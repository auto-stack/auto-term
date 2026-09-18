# PLAN-022 探针 pw:PrintWindow 内容捕获 + 帧变化检测(不抢焦点/不抓屏)
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class M31 {
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint f);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  public struct RECT { public int L, T, R, B; }
}
'@
$exe = "D:\autostack\auto-term\target\debug\auto-term.exe"
$errLog = "D:\autostack\auto-term\docs\plans\evidence\022\t05-slotlog.err"
$ev = "D:\autostack\auto-term\docs\plans\evidence\022"
$env:AUTOTERM_ENGINE_DLL = "D:\autostack\auto-term\target\debug\autoterm_core.dll"
$env:AUTO_MA_DBG = "1"
$p = Start-Process -FilePath $exe -PassThru -RedirectStandardError $errLog
Start-Sleep -Seconds 3
$p.Refresh()
$hw = $p.MainWindowHandle

function Shot([string]$path) {
  $rc = New-Object M31+RECT
  [M31]::GetWindowRect($hw, [ref]$rc) | Out-Null
  $bw = $rc.R - $rc.L; $bh = $rc.B - $rc.T
  if ($bw -le 0 -or $bh -le 0) { return $null }
  $bmp = New-Object System.Drawing.Bitmap($bw, $bh)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $dc = $g.GetHdc()
  [M31]::PrintWindow($hw,$dc,2) | Out-Null
  $g.ReleaseHdc($dc);$g.Dispose()
  return $bmp
}
# 等待帧稳定后取两张,做差异检测(窗口内容真在变化才算活帧)
$prev = $null; $attempt = 0; $saved = $false
while ($attempt -lt 12 -and -not $saved) {
  $attempt++
  Start-Sleep -Seconds 3
  $bmp = Shot "x"
  if ($null -eq $bmp) { continue }
  if ($null -ne $prev) {
    # 抽样对比右半区(分裂后右 pane 有字,boot 帧右侧纯黑)
    $diff = 0
    for ($x = 1400; $x -lt 2500; $x += 40) {
      for ($y = 200; $y -lt 800; $y += 40) {
        $a = $prev.GetPixel($x, $y); $b = $bmp.GetPixel($x, $y)
        if ([Math]::Abs($a.R - $b.R) + [Math]::Abs($a.G - $b.G) + [Math]::Abs($a.B - $b.B) -gt 30) { $diff++ }
      }
    }
    "attempt=$attempt diff=$diff"
    if ($diff -gt 3) {
      $bmp.Save("$ev\pw-split.png",[System.Drawing.Imaging.ImageFormat]::Png)
      "SAVED pw-split.png (live frames)"
      $saved = $true
    }
    $prev.Dispose()
  }
  if ($null -ne $prev) { } 
  $prev = $bmp
}
if (-not $saved) { if ($null -ne $prev) { $prev.Save("$ev\pw-split.png",[System.Drawing.Imaging.ImageFormat]::Png); "SAVED (last)" } }
Stop-Process -Id $p.Id -Force
Start-Sleep -Milliseconds 600
"DONE"
