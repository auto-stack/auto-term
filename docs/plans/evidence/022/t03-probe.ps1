# PLAN-022 T-03 探针:启动 rust 载体 → 前台化 → Ctrl+Shift+E 横分 → 截屏
# 载体:app/rust-workspace (worktree 运行时 plan-022-dev + mouse-area codegen)
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class M22 {
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint f);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  public struct RECT { public int L, T, R, B; }
}
'@

$exe = "D:\autostack\auto-term\target\debug\auto-term.exe"
$env:AUTOTERM_ENGINE_DLL = "D:\autostack\auto-term\target\debug\autoterm_core.dll"
$p = Start-Process -FilePath $exe -PassThru
Start-Sleep -Seconds 4

$p.Refresh()
$h = $p.MainWindowHandle
if ($h -eq [IntPtr]::Zero) { "NOWIN"; Stop-Process -Id $p.Id -Force; exit 1 }
[M22]::SetForegroundWindow($h) | Out-Null
Start-Sleep -Milliseconds 600

# 单面板基线截屏
function Shot($path) {
  $r = New-Object M22+RECT
  [M22]::GetWindowRect($h,[ref]$r) | Out-Null
  $w = $r.R-$r.L; $ht = $r.B-$r.T
  $bmp = New-Object System.Drawing.Bitmap($w,$ht)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $dc = $g.GetHdc()
  [M22]::PrintWindow($h,$dc,2) | Out-Null
  $g.ReleaseHdc($dc);$g.Dispose()
  $bmp.Save($path,[System.Drawing.Imaging.ImageFormat]::Png)
  $bmp.Dispose()
  "SAVED $path ${w}x${ht}"
}
Shot("D:\autostack\auto-term\docs\plans\evidence\022\t03-vehicle-boot.png")

# Ctrl+Shift+E = 横分(019 捷径,窗口全局语义)
[System.Windows.Forms.SendKeys]::SendWait("^+e")
Start-Sleep -Seconds 2
Shot("D:\autostack\auto-term\docs\plans\evidence\022\t03-vehicle-split.png")

# Ctrl+Shift+O = 纵分(嵌套第二层)
[System.Windows.Forms.SendKeys]::SendWait("^+o")
Start-Sleep -Seconds 2
Shot("D:\autostack\auto-term\docs\plans\evidence\022\t03-vehicle-nested.png")

Stop-Process -Id $p.Id -Force
"DONE"
