# drive-keys.ps1 — PLAN-019 快捷键/键入驱动 v2(015 t04 口径:Alt 抖动
# 解锁前台 + TOPMOST 钉顶 + IME 强制字母数字 + VK 扫码键入)
# 用法:
#   powershell -File drive-keys.ps1 -Action focus                    # 聚焦+点终端
#   powershell -File drive-keys.ps1 -Action click -X 640 -Y 500      # 客户区坐标点击
#   powershell -File drive-keys.ps1 -Action rclick -X 100 -Y 40
#   powershell -File drive-keys.ps1 -Action keys -Keys "ctrl+shift+e"
#   powershell -File drive-keys.ps1 -Action keys -Keys "type:dir|enter"
param(
    [string]$Action = "keys",
    [int]$X = 0,
    [int]$Y = 0,
    [string]$Keys = ""
)
$ErrorActionPreference = "Stop"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class W2 {
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
}
"@
$procs = Get-Process auto-term -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne 0 }
if (-not $procs) { Write-Output "NO-WINDOW"; exit 1 }
$h = $procs[0].MainWindowHandle

# ── 聚焦(015 FocusWin 同款)────────────────────────────────────────
if ([W2]::IsIconic($h)) { [void][W2]::ShowWindow($h, 9); Start-Sleep -Milliseconds 600 }
[W2]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero)
[void][W2]::SetForegroundWindow($h)
[W2]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero)
[void][W2]::SetWindowPos($h, [IntPtr](-1), 0, 0, 0, 0, 0x3)
Start-Sleep -Milliseconds 250
if ([W2]::GetForegroundWindow() -ne $h) { Write-Output "FOCUS-FAILED"; exit 1 }
$pidOut = [uint32]0

function Click-Client([int]$cx, [int]$cy, [bool]$right) {
    $p = New-Object W2+POINT; $p.X = $cx; $p.Y = $cy
    [void][W2]::ClientToScreen($h, [ref]$p)
    [void][W2]::SetCursorPos($p.X, $p.Y); Start-Sleep -Milliseconds 150
    $down = 0x0002; $up = 0x0004
    if ($right) { $down = 0x0008; $up = 0x0010 }
    [W2]::mouse_event($down, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 80
    [W2]::mouse_event($up, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 150
}

function Tap([byte]$vk) {
    [W2]::keybd_event($vk, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 25
    [W2]::keybd_event($vk, 0, 2, [UIntPtr]::Zero); Start-Sleep -Milliseconds 40
}
function TypeText([string]$s) {
    foreach ($ch in $s.ToCharArray()) {
        if ($ch -eq ' ') { Tap 0x20 }
        elseif ($ch -ge 'a' -and $ch -le 'z') { Tap ([byte](0x41 + ([int][char]$ch) - ([int][char]'a'))) }
        elseif ($ch -ge 'A' -and $ch -le 'Z') { Tap ([byte](0x41 + ([int][char]$ch) - ([int][char]'A'))) }
        elseif ($ch -ge '0' -and $ch -le '9') { Tap ([byte](0x30 + ([int][char]$ch) - ([int][char]'0'))) }
        else { throw "no vk map for '$ch'" }
    }
}

switch ($Action) {
    "focus" {
        # 默认聚焦点 = 内容区中心(客户区约 1293x836;tab 条 ~90px 高)
        Click-Client 640 500 $false
        Write-Output "FOCUSED"
    }
    "click" { Click-Client $X $Y $false; Write-Output "CLICKED $X,$Y" }
    "rclick" { Click-Client $X $Y $true; Write-Output "RCLICKED $X,$Y" }
    "keys" {
        foreach ($combo in $Keys -split '\|') {
            if ($combo.StartsWith("type:")) { TypeText $combo.Substring(5); continue }
            if ($combo -eq "enter") { Tap 0x0D; continue }
            $parts = $combo -split '\+'
            $vks = @()
            foreach ($p in $parts) {
                $vks += switch ($p.ToLower()) {
                    "ctrl" { 0x11 } "shift" { 0x10 } "alt" { 0x12 }
                    "tab" { 0x09 } "left" { 0x25 } "right" { 0x27 }
                    "up" { 0x26 } "down" { 0x28 } "enter" { 0x0D }
                    default { [byte](0x41 + ([int][char]$p[0]) - ([int][char]'a')) }
                }
            }
            foreach ($vk in $vks) { [W2]::keybd_event([byte]$vk, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 25 }
            [array]::Reverse($vks)
            foreach ($vk in $vks) { [W2]::keybd_event([byte]$vk, 0, 2, [UIntPtr]::Zero); Start-Sleep -Milliseconds 25 }
            Start-Sleep -Milliseconds 300
        }
        Write-Output "KEYS-SENT $Keys"
    }
}
