# PLAN-015 T-04 集成取证脚本(008 口径;证据落同目录 PNG/trace.log)v2
# 判别设计:timeout /t 10 /nobreak 只认 CTRL_C_EVENT、忽略普通按键——
#   ① 字节路径基线:聚焦后 Ctrl+C(0x03 字节)→ 倒计时继续(007 F2 基线);
#   ② 菜单事件路径:右键 → 点第 4 项 "Interrupt"(载荷 3)→ ≤5s 回提示符。
#   ③ AC-02:cmd 内启动 ash(idle)→ Ctrl+C → echo 存活(字节路径不误杀)。
$ErrorActionPreference = "Stop"
$exe = "D:\autostack\auto-lang\target\debug\auto-term.exe"
$ash = "D:\autostack\auto-shell\ash\target\debug\ash.exe"
$out = "D:\autostack\auto-term\docs\plans\evidence\015"
$trace = "$out\trace.log"

Add-Type @"
using System;
using System.Text;
using System.Runtime.InteropServices;
public class W {
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint f);
  public delegate bool CB(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumWindows(CB c, IntPtr l);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr h, [MarshalAs(UnmanagedType.LPWStr)] StringBuilder s, int n);
  [DllImport("imm32.dll")] public static extern IntPtr ImmGetContext(IntPtr h);
  [DllImport("imm32.dll")] public static extern bool ImmSetConversionStatus(IntPtr himc, int conv, int sentence);
  [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint a, uint b, bool attach);
  [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
}
"@
Add-Type -AssemblyName System.Drawing

function FindWindowByPid([uint32]$want) {
  $script:found = [IntPtr]::Zero
  $cb = [W+CB]{ param($h, $l)
    $pid2 = 0; [void][W]::GetWindowThreadProcessId($h, [ref]$pid2)
    if ($pid2 -eq $want -and [W]::IsWindowVisible($h)) {
      $sb = New-Object System.Text.StringBuilder 256
      [void][W]::GetWindowTextW($h, $sb, 256)
      if ($sb.ToString().Length -gt 0) { $script:found = $h; return $false }
    }
    return $true
  }
  [void][W]::EnumWindows($cb, [IntPtr]::Zero)
  $script:found
}

function Shot([IntPtr]$h, [string]$path) {
  foreach ($try in 1..4) {
    $r = New-Object W+RECT; [void][W]::GetClientRect($h, [ref]$r)
    $wd = $r.R - $r.L; $ht = $r.B - $r.T
    if ($wd -gt 0 -and $ht -gt 0) { break }
    Start-Sleep -Milliseconds 300
  }
  if ($wd -le 0 -or $ht -le 0) { throw "client rect empty: ${wd}x${ht}" }
  $bmp = New-Object System.Drawing.Bitmap($wd, $ht)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $dc = $g.GetHdc()
  [void][W]::PrintWindow($h, $dc, 2)
  $g.ReleaseHdc($dc); $g.Dispose()
  $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png); $bmp.Dispose()
}

function FocusWin([IntPtr]$h) {
  # 最小化则还原(014 教训:最小化=退化几何+client rect 0x0),再 Alt 抖动
  # 解锁 SetForegroundWindow,TOPMOST 钉顶防遮挡。
  if ([W]::IsIconic($h)) { [void][W]::ShowWindow($h, 9); Start-Sleep -Milliseconds 600 }  # SW_RESTORE
  [W]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero)
  [void][W]::SetForegroundWindow($h)
  [W]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero)
  # HWND_TOPMOST=-1, SWP_NOMOVE|SWP_NOSIZE=0x3
  [void][W]::SetWindowPos($h, [IntPtr](-1), 0, 0, 0, 0, 0x3)
  Start-Sleep -Milliseconds 250
  if ([W]::GetForegroundWindow() -ne $h) { throw "failed to focus window" }
  # 中文 IME 强制字母数字模式(否则键入被拼音劫持,实测 /t → /它)。
  $pidOut = [uint32]0
  $tid = [W]::GetWindowThreadProcessId($h, [ref]$pidOut)
  [void][W]::AttachThreadInput([W]::GetCurrentThreadId(), $tid, $true)
  $himc = [W]::ImmGetContext($h)
  if ($himc -ne [IntPtr]::Zero) { [void][W]::ImmSetConversionStatus($himc, 0, 0) }  # 0=ALPHANUMERIC
  [void][W]::AttachThreadInput([W]::GetCurrentThreadId(), $tid, $false)
}

function Click([IntPtr]$h, [int]$cx, [int]$cy, [bool]$right) {
  $p = New-Object W+POINT; $p.X = $cx; $p.Y = $cy
  [void][W]::ClientToScreen($h, [ref]$p)
  [void][W]::SetCursorPos($p.X, $p.Y); Start-Sleep -Milliseconds 150
  $down = 0x0002; $up = 0x0004; if ($right) { $down = 0x0008; $up = 0x0010 }
  [W]::mouse_event($down, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 80
  [W]::mouse_event($up, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 150
}

function Tap([byte]$vk) {
  [W]::keybd_event($vk, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 25
  [W]::keybd_event($vk, 0, 2, [UIntPtr]::Zero); Start-Sleep -Milliseconds 40
}
function TypeText([string]$s) {
  foreach ($ch in $s.ToCharArray()) {
    if ($ch -eq ' ') { Tap 0x20 }
    elseif ($ch -eq "{ENTER}") { Tap 0x0D }
    elseif ($ch -ge 'a' -and $ch -le 'z') { Tap ([byte](0x41 + ([int][char]$ch) - ([int][char]'a'))) }
    elseif ($ch -ge 'A' -and $ch -le 'Z') { Tap ([byte](0x41 + ([int][char]$ch) - ([int][char]'A'))) }
    elseif ($ch -ge '0' -and $ch -le '9') { Tap ([byte](0x30 + ([int][char]$ch) - ([int][char]'0'))) }
    elseif ($ch -eq ':') { [W]::keybd_event(0x10,0,0,[UIntPtr]::Zero); Tap 0xBA; [W]::keybd_event(0x10,0,2,[UIntPtr]::Zero) }
    elseif ($ch -eq '\') { Tap 0xDC }
    elseif ($ch -eq '/') { Tap 0xBF }
    elseif ($ch -eq '.') { Tap 0xBE }
    elseif ($ch -eq '-') { Tap 0xBD }
    else { throw "no vk map for '$ch'" }
  }
}
function TypeLine([string]$s) { TypeText $s; Tap 0x0D }
function CtrlC { [W]::keybd_event(0x11,0,0,[UIntPtr]::Zero); Tap 0x43; [W]::keybd_event(0x11,0,2,[UIntPtr]::Zero) }

# 1) 启动(stderr→trace.log;DLL 显式指认)。先清残留实例(防 9247 占用
#    与点击串扰)。
Get-Process auto-term -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 1
if (Test-Path $trace) { Remove-Item $trace }
$env:AUTOTERM_ENGINE_DLL = "D:\autostack\auto-term\target\debug\autoterm_core.dll"
# 中断 helper 同布局问题(008 三级解析:exe 同目录扫描在 app exe 住
# auto-lang target 的开发布局下扑空)→ env 人工覆盖(dist 三件套同目录免设)。
$env:AUTOTERM_CTRLC_BIN = "D:\autostack\auto-term\target\debug\autoterm-ctrlc.exe"
$p = Start-Process -FilePath $exe -PassThru -RedirectStandardError $trace
try {
  $h = [IntPtr]::Zero
  foreach ($i in 1..90) {
    Start-Sleep -Milliseconds 500
    $h = FindWindowByPid([uint32]$p.Id)
    if ($h -ne [IntPtr]::Zero) { break }
  }
  if ($h -eq [IntPtr]::Zero) { throw "AutoTerm window not found (pid=$($p.Id))" }
  Start-Sleep -Seconds 3
  FocusWin $h
  Shot $h "$out\t1_boot.png"

  # 2) 聚焦 + 等沉降(IME 英文起步 toggle 落位) + 起跑 /nobreak 倒计时
  Click $h 400 240 $false
  Start-Sleep -Milliseconds 3500
  TypeLine "timeout /t 30 /nobreak"; Start-Sleep -Seconds 2
  Shot $h "$out\t2_countdown.png"

  # 3) 字节路径基线:Ctrl+C(0x03)→ /nobreak 不理,倒计时继续
  CtrlC; Start-Sleep -Seconds 2
  Shot $h "$out\t3_key_ctrlc_ignored.png"

  # 4) 菜单:右键 (100,100) → 4 项浮层;第 4 项中心=(100+40,100+3*16+8)=(140,156)
  Click $h 100 100 $true; Start-Sleep -Milliseconds 500
  Shot $h "$out\t4_menu_open.png"
  Click $h 140 156 $false

  # 5) ≤5s 回提示符(008 口径)
  Start-Sleep -Seconds 5
  Shot $h "$out\t5_after_menu_interrupt.png"

  # 6) 会话存活:echo 往返
  FocusWin $h
  TypeLine "echo alive"; Start-Sleep -Seconds 2
  Shot $h "$out\t6_echo_alive.png"

  # 7) AC-02:ash idle + Ctrl+C → 会话存活
  TypeLine "$ash"; Start-Sleep -Seconds 8
  Shot $h "$out\t7_ash_prompt.png"
  CtrlC; Start-Sleep -Milliseconds 800
  TypeLine "echo alive2"; Start-Sleep -Seconds 2
  Shot $h "$out\t8_ash_alive_after_ctrlc.png"
}
finally {
  if (!$p.HasExited) { $p.Kill() }
  Start-Sleep -Seconds 1
}
Write-Host "=== trace.log ==="
Get-Content $trace -ErrorAction SilentlyContinue | Select-String "menu|pump|resize"
