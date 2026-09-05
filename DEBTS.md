# DEBTS.md — 已知债务

> PLAN-001 盘点 · PLAN-002 勾账 · PLAN-003(Phase 2 性能与输入)勾账 ·
> PLAN-004(Phase 3 交互完备)勾账 · 2026-09-05。

## 债务账本(PLAN-004 后状态)

| # | 债务 | PLAN-001 现状 | PLAN-004 后 | 清偿方向 |
| --- | --- | --- | --- | --- |
| 1 | 整帧重拼渲染 | 每帧全网格重画 | **已清偿(003)**:保留式画布——每行 Paragraph 缓存+digest 判异+脏行门控,末帧重建 0 | — |
| 2 | 无字形图集 | 每 run 独立 fill_text | **已清偿(003)**:每行单 Paragraph,shaping 由行缓存持有;GPU 字形缓存 iced 内置 | 图集化(wgpu atlas)留待真有性能需求时 |
| 3 | 固定字形度量 | Consolas 硬编码 + 拟合 | **已清偿**:cosmic-text 实测 cell_w(9.375px@16px),fit_ok 数值断言;宽字符 unicode-width(实测证伪 advance 法) | — |
| 4 | 16ms 轮询驱动 | time::every tick | **已清偿**:唤醒通道事件驱动(6s 59 次更新 vs 372 轮询),常态零定时器 | — |
| 5 | 无滚动回滚 UI | — | **已清偿**:滚轮/PgUp/PgDn/键入回正/↑N(offset=171 回顶实证) | 滚动条可选 |
| 6 | 无光标块/选中/IME | 键盘只回写 | **已清偿(004)**:选中三模式(拖选/双击词/三击行)+ 高亮(像素证据)+ copy-on-select + Ctrl+Shift+C/V/右键粘贴(剪贴板读回断言);IME 管线全通 + 自绘 preedit(像素证据);**残留:人工拼音清单待用户执行**(001 附录跑法) | 人工清单跑完即全关 |
| 7 | Ctrl+C / 关闭语义 | 未验证 | 关闭已清偿(002);003 矩阵完结 + 004 裁定:**无稳定版环境,复测待环境**(矩阵脚本就绪,001 附录决策树不变);经典控制台程序真事件仍需 win32 GenerateConsoleCtrlEvent | 稳定版环境到位补测即关;复现则 win32 直调立项 |
| 8 | 仅 Windows 基座 | 同 | 未动 | Linux/macOS 计划 |
| 9 | spike 无统一文档 | 同 | **已清偿**:crates/* 正式结构 + 001 设计文档 | — |
| 10 | 颜色表硬编码 | 16 色/xterm256 内置 | **已清偿**:palette.rs 全 NamedColor 映射(Dim×8/Bright/Dim 前景,TDD) | 主题系统可选;选中色配置见 Phase 4 候选 |
| 11 | IME over-the-spot 运行时覆盖层不落屏(新) | — | 004 实测:iced_winit main-events 相相位丢弃 `State::Updated{input_method}`(381 次请求埋点实证),redraw 相相位应用链在本机不出画面;已按裁定降级自绘(可用) | 升级 iced 版本时重试 `Enabled{preedit: Some}` 路线,成则删自绘 |

## 新增观察(PLAN-004 实测,设计输入)

- **pwsh 冷启动拖慢 iced 首显**(visible=false 到首帧):窗口首显
  resize 风暴会清选中(core damage 语义)——dev 注入钩子需自愈式;
- **取证链路定型**:PrintWindow(PW_RENDERFULLCONTENT)+ 
  SetProcessDPIAware(高缩放屏必须,否则只取左上象限)+ 背景直通带
  内容自校验;CopyFromScreen 受遮挡/前台竞态污染,降为辅助;
- **DevTick 注入必须返回 Task**:丢弃返回 Task = 剪贴板写等副作用
  静默失效(T4 实证);
- iced `Shell::request_input_method` 仅 redraw 相相位消费(见 #11)。

## Phase 4 候选清单(优先级未定)

1. **拖选到边缘自动滚动**(004 裁定#4:与块选/右键菜单同批);
2. **块选(Block)**+ 右键上下文菜单;
3. 选中主题色配置(现 DEFAULT_FG 25% α 硬编码);
4. 光标形状(Underline/Beam)与闪烁;
5. Ctrl+C 稳定版复测(#7 待环境后关账或立项 win32);
6. Unix 基座适配;
7. iced↔auto-ui 生态对齐(`.at` 组件模型承载原生 widget 调查);
8. **Auto 复刻**(用户裁定 2026-09-05:AutoLang 可直调 Rust 库,
   当前 Rust 实现有效;届时将应用层[autoterm-core 封装 +
   autoterm-ui App/TermGrid,约 2k 行]以 Auto 代码复刻,引擎
   crate[alacritty_terminal/iced/portable-pty]经绑定复用——
   前置依赖 #7 的组件模型调查)。
