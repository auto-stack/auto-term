# t02_vm_seq.ps1 — PLAN-020 T-02 VM 实机取证序列。
# 基线 → 横分 → 拖根竖条至 30%(中途帧)→ 回拖入磁吸带 → L 形
# (点右 Pane 聚焦再竖分)→ 拖右列横条。
$ErrorActionPreference = "Continue"
Add-Type -TypeDefinition "public static class DpiHelper { [System.Runtime.InteropServices.DllImport(\"user32.dll\")] public static extern bool SetProcessDPIAware(); }"
[void][DpiHelper]::SetProcessDPIAware()
$out = "D:\autostack\auto-term\docs\plans\evidence\020"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class W2 {
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint f);
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
  [DllImport("user32.dll")] public static extern uint SendInput(uint n, INPUT[] inputs, int size);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  [StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public InputUnion u; }
  [StructLayout(LayoutKind.Explicit)] public struct InputUnion { [FieldOffset(0)] public MOUSEINPUT mi; }
  [StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx, dy; public uint mouseData, dwFlags, time; public IntPtr dwExtraInfo; }
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, UIntPtr w, IntPtr l);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
}
"@
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms

function Log([string]$m) { Write-Output "[$(Get-Date -Format HH:mm:ss)] $m" }

$h = [IntPtr]::Zero
$procs = Get-Process auto-term -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle -eq "终端" }
if ($procs) { $h = $procs[0].MainWindowHandle }
if ($h -eq [IntPtr]::Zero) { Log "NO-WINDOW"; exit 1 }
Log "window hwnd=$h title=$($procs[0].MainWindowTitle)"

if ([W2]::IsIconic($h)) { [void][W2]::ShowWindow($h, 9); Start-Sleep -Milliseconds 800 }
# ALT 抢前台技巧(019 t02 同款)+ 移位 (1100,80) + 置顶
[W2]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero)
[void][W2]::SetForegroundWindow($h)
[W2]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero)
[void][W2]::SetWindowPos($h, [IntPtr](0), 1100, 80, 0, 0, 0x0001)
Start-Sleep -Milliseconds 900

$wr = New-Object W2+RECT
[void][W2]::GetWindowRect($h, [ref]$wr)
$cr = New-Object W2+RECT
[void][W2]::GetClientRect($h, [ref]$cr)
$cp = New-Object W2+POINT
$cp.X = 0; $cp.Y = 0
[void][W2]::ClientToScreen($h, [ref]$cp)
$cw = $cr.R - $cr.L; $ch = $cr.B - $cr.T
Log "client ${cw}x${ch} at ($($cp.X),$($cp.Y))"

function Shot([string]$name) {
  $wd = $wr.R - $wr.L; $ht = $wr.B - $wr.T
  $bmp = New-Object System.Drawing.Bitmap($wd, $ht)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $dc = $g.GetHdc()
  [void][W2]::PrintWindow($h, $dc, 2)
  $g.ReleaseHdc($dc); $g.Dispose()
  $bmp.Save("$out\$name", [System.Drawing.Imaging.ImageFormat]::Png); $bmp.Dispose()
  Log "SHOT $name"
}

# SendInput 绝对坐标(0..65535 虚拟屏归一)——系统级真输入
function SendClick([int]$x, [int]$y) {
  $vs = [System.Windows.Forms.Screen]::VirtualScreen
  $ax = [int](($x - $vs.X) * 65535 / $vs.Width)
  $ay = [int](($y - $vs.Y) * 65535 / $vs.Height)
  $mv = New-Object W2+INPUT
  $mv.type = 0; $mv.u.mi = New-Object W2+MOUSEINPUT
  $mv.u.mi.dx = $ax; $mv.u.mi.dy = $ay; $mv.u.mi.dwFlags = 0x8001 -bor 0x0002
  $dn = New-Object W2+INPUT
  $dn.type = 0; $dn.u.mi = New-Object W2+MOUSEINPUT
  $dn.u.mi.dwFlags = 0x0002
  $up = New-Object W2+INPUT
  $up.type = 0; $up.u.mi = New-Object W2+MOUSEINPUT
  $up.u.mi.dwFlags = 0x0004
  $arr = [W2+INPUT[]]@($mv, $dn, $up)
  [void][W2]::SendInput(3, $arr, [System.Runtime.InteropServices.Marshal]::SizeOf([type][W2+INPUT]))
  Start-Sleep -Milliseconds 500
}
function Click([int]$x, [int]$y) {
  SendClick $x $y
}
function MoveTo([int]$x, [int]$y) {
  [void][W2]::SetCursorPos($x, $y)
  Start-Sleep -Milliseconds 120
}
function DragTo([int]$sx, [int]$sy, [int]$tx, [int]$ty, [string]$midShot) {
  [void][W2]::SetCursorPos($sx, $sy)
  Start-Sleep -Milliseconds 250
  [W2]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
  Start-Sleep -Milliseconds 150
  $steps = 14
  for ($i = 1; $i -le $steps; $i++) {
    $nx = $sx + [int](($tx - $sx) * $i / $steps)
    $ny = $sy + [int](($ty - $sy) * $i / $steps)
    [void][W2]::SetCursorPos($nx, $ny)
    Start-Sleep -Milliseconds 45
    if ($i -eq 7 -and $midShot -ne "") { Shot $midShot }
  }
  Start-Sleep -Milliseconds 250
  [W2]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
  Start-Sleep -Milliseconds 500
}
function MoveTo([int]$x, [int]$y) {
  $lp = [IntPtr](($y -shl 16) -bor ($x -band 0xFFFF))
  [void][W2]::PostMessage($h, 0x0200, [UIntPtr]::Zero, $lp)
  Start-Sleep -Milliseconds 120
}

function DragTo([int]$sx, [int]$sy, [int]$tx, [int]$ty, [string]$midShot) {
  $lp0 = [IntPtr](($sy -shl 16) -bor ($sx -band 0xFFFF))
  [void][W2]::PostMessage($h, 0x0200, [UIntPtr]::Zero, $lp0)
  Start-Sleep -Milliseconds 150
  [void][W2]::PostMessage($h, 0x0201, [UIntPtr]1, $lp0)
  Start-Sleep -Milliseconds 150
  $steps = 14
  for ($i = 1; $i -le $steps; $i++) {
    $nx = $sx + [int](($tx - $sx) * $i / $steps)
    $ny = $sy + [int](($ty - $sy) * $i / $steps)
    $lp = [IntPtr](($ny -shl 16) -bor ($nx -band 0xFFFF))
    [void][W2]::PostMessage($h, 0x0200, [UIntPtr]1, $lp)
    Start-Sleep -Milliseconds 45
    if ($i -eq 7 -and $midShot -ne "") { Shot $midShot }
  }
  Start-Sleep -Milliseconds 250
  [void][W2]::PostMessage($h, 0x0202, [UIntPtr]::Zero, $lp)
  Start-Sleep -Milliseconds 500
}

# 物理比例 = client 物理宽 / 逻辑 802(按钮逻辑中心 × scale)
$scale = $cw / 802.0
Log "scale=$scale"
$btnY = $cp.Y + [int](41 * $scale)
# 基线
Shot "t02-a-baseline.png"

# A2. 哨兵:点 "+" 新建 Tab(验证点击通道)
Click 233 41
Shot "t02-a2-newtab.png"

# B. 横分
Click 335 41
Shot "t02-b-split-h.png"

# C. 拖根竖条(50%→30%)
$sy2 = [int]($ch * 0.6)
DragTo [int]($cw * 0.5) $sy2 [int]($cw * 0.30) $sy2 "t02-c-middrag.png"
Shot "t02-c-at30.png"

# D. 磁吸:30% → 48% 松手 → 应回 50%
DragTo [int]($cw * 0.30) $sy2 [int]($cw * 0.48) $sy2 ""
Shot "t02-d-magnet.png"

# E. 点右 Pane(75%, 50%)聚焦 → 竖分(L 形)
Click [int]($cw * 0.75) [int]($ch * 0.5)
Click 455 41   # 竖分
Shot "t02-e-lshape.png"

# F. 拖右列横条(50%→65%)
$ex = [int]($cw * 0.75)
DragTo $ex [int]($ch * 0.5) $ex [int]($ch * 0.65) "t02-f-middrag2.png"
Shot "t02-f-after.png"

Log "DONE"
