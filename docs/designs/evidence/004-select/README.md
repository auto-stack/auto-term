# PLAN-004 选中/复制/粘贴取证归档

日期:2026-09-05 · 构建:dev-tools · shell:pwsh 7.6.5 · 网格 106x32 ·
度量 font_px=16 cell_w=9.375 line_h=20(逻辑;本机显示缩放 200%)。

## 文件清单

| 文件 | 内容 |
|---|---|
| `scan-select.ps1` | 取证脚本:PrintWindow(PW_RENDERFULLCONTENT)+ SetProcessDPIAware + 背景直通带自校验 + 带色扫描 |
| `select-before.png` | 注入前窗口(逻辑尺寸裁剪版本,作对照) |
| `select-highlight.png` | 选中后窗口(DPI 感知全尺寸 2026x1371) |
| `select-scan.txt` | 像素扫描结果(见下) |
| `select-dump.txt` | dev 转储:selection_text 含标记行,selection_cells 484 |
| `copy-paste-dump.txt` | 粘贴回显(PASTED_004 上屏)+ 选中态 |
| `copy-paste-clipboard-assert.txt` | 外部 Get-Clipboard == selection_text(MATCH) |
| `semantic-dump.txt` | 词选(2..9 行域):整词 SELECT_MARKER_LINE 入选 |
| `semantic-boundary-dump.txt` | 词选窄域:左边界恰在 `:` 语义转义符(`\Users\zhaop>`) |
| `lines-dump.txt` | 行选:318 cells = 3×106 恰三整行 |

## 像素扫描结论(select-scan.txt)

- 窗口 2026x1371(物理);
- 高亮带 **y138-337,高 200px = 恰好 5 行 × 40px 物理行距**(注入行 2..6);
- 命中色均值 **(70.0, 73.0, 76.0)**,理论混合色
  `DEFAULT_FG(e8e8e8) 25% α over DEFAULT_BG(10 14 18) = (71,73,76)`,
  偏差 ≤1/通道;
- 命中采样 178657(step2)≈ 全带宽 × 5 行 − 字形遮挡。

## 交叉断言链

1. core:selection_range/selection_text(sim_regression 11 用例,T1);
2. ui 注入:dev-select(自愈式)→ 转储 selection_text/selection_cells;
3. 像素:带色扫描(无视觉模型依赖)证明高亮上屏且几何正确;
4. 剪贴板:copy-on-select 写入 → 进程退出后外部 Get-Clipboard 读回
   与转储 selection_text 逐字节一致;
5. 粘贴:Pasted 路径 → write_input → PSReadLine 回显上屏(转储可见)。

## 执行期方法学发现(003 证据政策的推进)

- 003 判定 PrintWindow 对 wgpu 不稳 → 本次以 **PW_RENDERFULLCONTENT**
  标志复验:本机可用(不稳结论限定于默认标志/错窗口工况);
- 高缩放屏(200%)必须 **SetProcessDPIAware**,否则只取到左上象限;
- 内容自校验(≥300px DEFAULT_BG 直通带)替代"窗口标题 -like"类脆弱匹配,
  宁失败不出伪证据(曾实测抓到浏览器窗口的伪高亮带)。
