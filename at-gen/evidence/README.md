# T10 UI 冒烟取证(PLAN-009)

- `ui_smoke.txt`:程序化取证(主)——真实窗口壳组件(AutoTermShell)经
  引擎驱动至锚点回显,headless 管线 view_to_vtree dump:terminal 节点
  挂载(key=at-shell cols=80 rows=24 lines=24)+ 引擎回显入 props 双验过。
- `ui_window.png`:截图补充——`autoterm-at.exe`(GUI 模式)真窗口,
  terminal 组件实时渲染 cmd 会话:banner/提示符/echo 回显/光标块在案。
  (窗口标题 "Auto Lang - Iced" 为 iced application 缺省,T11 记 cosmetic 债。)

复跑:`autoterm-at.exe smoke`;GUI:`autoterm-at.exe`(需 autoterm_core.dll
在 exe 向上 4 级 target/debug 内)。
