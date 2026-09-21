# PLAN-023 T-04 烟测:启动 → Ctrl+Shift+E 横分 → 点左/右槽 → 键入 → 滚轮 → 拖分隔条
# 断言面 = stderr 的 [TERM_PRESS]/[P22-FOCUS]/[P22-KEY]/[P22-WHEEL](AUTO_MA_DBG=1)
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class M23 {
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint f);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, int d, UIntPtr e);
  public struct RECT { public int L, T, R, B; }
}
'@
[void][M23]::SetProcessDPIAware()

$exe = "D:\autostack\auto-term\target\debug\auto-term.exe"
$ev = "D:\autostack\auto-term\docs\plans\evidence\023"
$env:AUTOTERM_ENGINE_DLL = "D:\autostack\auto-term\target\debug\autoterm_core.dll"
$env:AUTO_MA_DBG = "1"
$errLog = "$ev\t04-smoke.err"
$p = Start-Process -FilePath $exe -PassThru -RedirectStandardError $errLog
Start-Sleep -Seconds 6
$p.Refresh()
$hw = $p.MainWindowHandle
[M23]::SetForegroundWindow($hw) | Out-Null
Start-Sleep -Milliseconds 800

$rc = New-Object M23+RECT
[M23]::GetWindowRect($hw, [ref]$rc) | Out-Null
$w = $rc.R - $rc.L; $h = $rc.B - $rc.T; $ox = $rc.L; $oy = $rc.T
"WIN ${w}x${h} at $ox,$oy"

function Shot([string]$path) {
  $rc2 = New-Object M23+RECT
  [M23]::GetWindowRect($hw,[ref]$rc2) | Out-Null
  $bw = $rc2.R - $rc2.L; $bh = $rc2.B - $rc2.T
  $bmp = New-Object System.Drawing.Bitmap($bw, $bh)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $dc = $g.GetHdc()
  [M23]::PrintWindow($hw,$dc,2) | Out-Null
  $g.ReleaseHdc($dc);$g.Dispose()
  $bmp.Save($path,[System.Drawing.Imaging.ImageFormat]::Png)
  $bmp.Dispose()
  "SHOT $path"
}
function Click([double]$fx, [double]$fy) {
  [M23]::SetCursorPos([int]($ox + $fx*$w), [int]($oy + $fy*$h)) | Out-Null
  Start-Sleep -Milliseconds 300
  [M23]::mouse_event(0x0002,0,0,0,[UIntPtr]::Zero)
  Start-Sleep -Milliseconds 120
  [M23]::mouse_event(0x0004,0,0,0,[UIntPtr]::Zero)
  Start-Sleep -Milliseconds 600
  "CLICK $fx,$fy"
}

# ① boot 单槽
Shot "$ev\t04-s1-boot.png"

# ② 横分按钮(比例 0.129, 0.056;SendKeys 无人值守下不稳)
Click 0.129 0.056
Start-Sleep -Seconds 3
Shot "$ev\t04-s2-split.png"

# ③ 点左槽中心(键入 + 回显)
$ws = New-Object -ComObject WScript.Shell
[void]$ws.AppActivate($p.Id)
Start-Sleep -Milliseconds 500
Click 0.25 0.55
$ws.SendKeys("echo LEFT-OK")
$ws.SendKeys("{ENTER}")
Start-Sleep -Seconds 2
Shot "$ev\t04-s3-left-focus.png"

# ④ 点右槽中心 + 键入(022 死区:聚焦输入)+ 造滚动历史
Click 0.75 0.55
$ws.SendKeys("echo RIGHT-OK")
$ws.SendKeys("{ENTER}")
$ws.SendKeys("for /l {%}i in (1,1,60) do @echo HIST-{%}i")
$ws.SendKeys("{ENTER}")
Start-Sleep -Seconds 3
Shot "$ev\t04-s4-right-focus.png"

# ⑤ 右槽滚轮上翻
[M23]::SetCursorPos([int]($ox + 0.75*$w), [int]($oy + 0.55*$h)) | Out-Null
Start-Sleep -Milliseconds 300
[M23]::mouse_event(0x0800,0,0,120,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 400
[M23]::mouse_event(0x0800,0,0,120,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 600
Shot "$ev\t04-s5-wheel.png"

# ⑥ 拖分隔条 50%→65%
$dy = [int]($oy + 0.5*$h)
[M23]::SetCursorPos([int]($ox + 0.50*$w), $dy) | Out-Null
Start-Sleep -Milliseconds 350
[M23]::mouse_event(0x0002,0,0,0,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 150
for ($i = 1; $i -le 15; $i++) {
  [M23]::SetCursorPos([int]($ox + (0.50 + 0.15*$i/15)*$w), $dy) | Out-Null
  Start-Sleep -Milliseconds 40
}
Start-Sleep -Milliseconds 400
[M23]::mouse_event(0x0004,0,0,0,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 800
Shot "$ev\t04-s6-divider-65.png"

Stop-Process -Id $p.Id -Force
Start-Sleep -Milliseconds 600
"DONE"
