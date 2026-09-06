# PLAN-005 Phase 4 交互批取证归档

日期:2026-09-05/06 · 构建:dev-tools(debug) · shell:pwsh 7.6.5 ·
窗口 1000x650 逻辑(2026x1371 物理,本机缩放 200%)· 网格 106x32 ·
度量 font_px=16 cell_w=9.375 line_h=20(逻辑)。

## 文件清单

| 文件 | 内容 |
|---|---|
| `scan-block.ps1` | T3:块选高亮**列带几何**扫描(连续 run 门 + 带内 x 一致性断言) |
| `block-before.png` / `block-highlight.png` | 块选注入前/后窗口截图 |
| `block-scan.txt` | 几何结论:4 带全 x52-142(spread=0)、block_aligned=True、band_cells≈5(注入列带 2..=6) |
| `block-dump.txt` | T2:dev-select `block` 注入转储——selection_text `"AABBB\nCCDDD\nEEFFF"`(逐行截断)、selection_cells=15(3×5 矩形) |
| `autoscroll-dump.txt` | T4:边缘保持注入(`:up`)转储——scroll_offset=11(顶满钳制)、selection_range `(-4,0)-(5,4)` 绝对坐标、selection_text L06..L15(锚定+滚动揭露行尽收) |
| `scan-menu.ps1` | T5:菜单浮层扫描(2D 阈值 bbox,标题栏/反锯齿免疫) |
| `menu-open.png` / `menu-closed.png` | 菜单开/关截图(复制/粘贴/全选,YaHei 中文标签) |
| `menu-scan.txt` | open: panel_visible=True(bbox=注入位 88×78 逻辑);close 注入后 34px 残留 → closed=True |
| `menu-dump.txt` | 菜单会话转储 |
| `scan-selcolor.ps1` | T6:`--selection-color ff0000` 命中色扫描(红主导行判别) |
| `selcolor-before.png` / `selcolor-after.png` | 新色选中前/后截图 |
| `selcolor-scan.txt` | avg_red_rgb=76.1,15.1,18.1 = ff0000@25% over bg(16,20,24) 的理论混色;selcolor_changed=True |
| `scan-cursor.ps1` | T7:光标形状格带扫描(±8px 条带,亮像素计数+bbox 判形) |
| `cursor-underline.png/-scan.txt/-dump.txt` | Underline:亮带 19×4(格底细条)、shape=underline |
| `cursor-beam.png/-scan.txt/-dump.txt` | Beam:亮带 4×38(格左细条)、shape=beam |
| `cursor-block.png/-scan.txt/-dump.txt` | Block:亮带 19×38(满格反色)、shape=block |
| `focus-blink.ps1` | T8:前台聚焦探针(SetForegroundWindow + 保持校验) |
| `blink-8s/9s-dump.txt` | 失焦基线:toggles=10/17(初值 focused=true 时期;改初值后见下) |
| `blink-focused-dump.txt` | 聚焦闪烁:focus 2s 内 toggles=4(≈2Hz),失焦回稳常亮;frames=5@11s 静止 |

## 关键结论

1. **块选**:core Block 分支现成(T1 补 3 sim 用例 14/14 绿);
   UI Alt+拖选触发 + `is_block` 列带渲染,像素几何 block_aligned
   全对齐,复制文本逐行截断无拖尾。
2. **拖选自动滚动**:Extend 压边 → drag_scroll → 50ms 条件订阅 →
   Scrolled(±2 行/拍);转储证 scroll_offset 顶满钳制 + 选中锚定
   绝对行(range 绝对坐标不动,视口行随滚动漂移)。
3. **右键菜单**:右键不再直接粘贴(菜单项承担);开合走真实
   ContextMenu 消息路径;命中检测纯函数(menu_item_at,TDD 抓出
   负 y `as usize` 饱和 cast bug);CJK 标签走 YaHei(MONOSPACE
   豆腐块为 004 已知限制,preedit 同)。
4. **选中色**:hex 解析纯函数(6 位维持 25% 默认 α、8 位显式、
   非法回退);ff0000 冒烟混色与理论值逐通道吻合。
5. **光标形状/闪烁**:DECSCUSR 须经 **shell 输出**(键入字节被
   PSReadLine 吞掉——取证方法学发现);闪烁四重门控
   (focused && shape!=Hidden && !menu && !preedit)以条件订阅实现,
   失焦/静态运行零定时器、字节路径零唤醒(frames=5@11s 静止 vs
   基线 116@有内容)。

## 手工清单(自动化未覆盖,5 分钟)

- [ ] 真鼠标 Alt+拖选块选手感(多击计数不误触发);
- [ ] 右键菜单三项动作与 004 快捷键等效(复制→外部粘贴读回);
- [ ] 闪烁不刺眼(真聚焦下 500ms 相位),preedit 挂起时光标常亮;
- [ ] 失焦(点别的窗口)光标常亮、回聚焦恢复闪烁。

## 取证方法学发现(承 004 附录)

- **PSReadLine 吞键入转义**:DECSCUSR/DECTCEM 等设备控制序列
  必须 `Write-Host "`e[4 q" -NoNewline` 由 shell **输出**,autotype
  直写 PTY 主端的字节会被 PSReadLine 当输入吃掉(初跑实证);
- **PrintWindow 竞态**:截图脚本与 app 的相对启动时刻必须以
  "构建先于后台链"编排(cargo build 与 app 同链后台化时,脚本
  时间轴整体前移,拍位错窗——menu 首拍实证);
- **像素 bbox 污染源两则**:标题栏深色主题色与菜单面板色同域
  (跳过窗顶 100px 解决);文本反锯齿灰阶散点(行列 2D 命中数
  阈值过滤解决);
- **订阅 map 禁捕获**:iced `Subscription::map` 要求非捕获闭包,
  载荷经 App 状态中转(DragScrollTick 模式),与 DevTick 同构。
