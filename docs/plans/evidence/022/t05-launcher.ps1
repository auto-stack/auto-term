# PLAN-022 T-05 用户实点载具启动器
# 载具 = app/rust-workspace(022 运行时 lang-022)+ autoterm_core.dll
# 用法:powershell -ExecutionPolicy Bypass -File t05-launcher.ps1
# 实点清单(三联合场景,PLAN-022 §8 T-05 / AC-05/07):
#  ① 拖分隔条:按住中缝灰条拖动——比例跟随、磁吸({500}±40 回吸)、
#     钳位 [100,900];松手落定。
#  ② 拖 thumb:左 pane 先敲几条命令造滚转历史,右缘出现官方滚动条
#     thumb(3px 半透明)——拖到最顶(能到最早内容)、滚轮全程可滚、
#     thumb 随滚轮/键入贴底跟随。
#  ③ 接缝点击:在分隔条 8px 带上点击/拖拽——命中归属分隔条(结构
#     操作优先),不误触两侧终端选区;滚动条带不外溢 pane 槽。
$exe = "D:\autostack\auto-term\target\debug\auto-term.exe"
$env:AUTOTERM_ENGINE_DLL = "D:\autostack\auto-term\target\debug\autoterm_core.dll"
$p = Start-Process -FilePath $exe -PassThru
"STARTED pid=$($p.Id)"
"注意:AUTO_MA_DBG=1 时 Init 会自动横分一次(探针补丁);关掉该 env 即正常手动流程。"
