# DEBTS.md — 已知债务

> PLAN-001 盘点 · PLAN-002 勾账 · PLAN-003(Phase 2 性能与输入)勾账 ·
> PLAN-004(Phase 3 交互完备)勾账 · PLAN-005(Phase 4 交互批)勾账 ·
> PLAN-006(#7 调查关账)· PLAN-007(ash 门禁,#12/#13 新增)· 2026-09-06。

## 债务账本(PLAN-004 后状态)

| # | 债务 | PLAN-001 现状 | PLAN-004 后 | 清偿方向 |
| --- | --- | --- | --- | --- |
| 1 | 整帧重拼渲染 | 每帧全网格重画 | **已清偿(003)**:保留式画布——每行 Paragraph 缓存+digest 判异+脏行门控,末帧重建 0 | — |
| 2 | 无字形图集 | 每 run 独立 fill_text | **已清偿(003)**:每行单 Paragraph,shaping 由行缓存持有;GPU 字形缓存 iced 内置 | 图集化(wgpu atlas)留待真有性能需求时 |
| 3 | 固定字形度量 | Consolas 硬编码 + 拟合 | **已清偿**:cosmic-text 实测 cell_w(9.375px@16px),fit_ok 数值断言;宽字符 unicode-width(实测证伪 advance 法) | — |
| 4 | 16ms 轮询驱动 | time::every tick | **已清偿**:唤醒通道事件驱动(6s 59 次更新 vs 372 轮询),常态零定时器 | — |
| 5 | 无滚动回滚 UI | — | **已清偿**:滚轮/PgUp/PgDn/键入回正/↑N(offset=171 回顶实证) | 滚动条可选 |
| 6 | 无光标块/选中/IME | 键盘只回写 | **已清偿(004)**:选中三模式(拖选/双击词/三击行)+ 高亮(像素证据)+ copy-on-select + Ctrl+Shift+C/V/右键粘贴(剪贴板读回断言);IME 管线全通 + 自绘 preedit(像素证据);**残留:人工拼音清单待用户执行**(001 附录跑法) | 人工清单跑完即全关 |
| 7 | Ctrl+C / 关闭语义 | 未验证 | 关闭已清偿(002);003 矩阵完结 + 004 裁定:**无稳定版环境,复测待环境**(矩阵脚本就绪,001 附录决策树不变);经典控制台程序真事件仍需 win32 GenerateConsoleCtrlEvent | 稳定版环境到位补测即关;复现则 win32 直调立项;**007 注:事件断裂已由 #12 坐实,win32 直调方向即 #12 修复通路** |
| 8 | 仅 Windows 基座 | 同 | 未动 | Linux/macOS 计划 |
| 9 | spike 无统一文档 | 同 | **已清偿**:crates/* 正式结构 + 001 设计文档 | — |
| 10 | 颜色表硬编码 | 16 色/xterm256 内置 | **已清偿**:palette.rs 全 NamedColor 映射(Dim×8/Bright/Dim 前景,TDD) | 主题系统可选;选中色已清偿(005 `--selection-color`) |
| 11 | IME over-the-spot 运行时覆盖层不落屏(新) | — | 004 实测:iced_winit main-events 相相位丢弃 `State::Updated{input_method}`(381 次请求埋点实证),redraw 相相位应用链在本机不出画面;已按裁定降级自绘(可用) | 升级 iced 版本时重试 `Enabled{preedit: Some}` 路线,成则删自绘 |
| 12 | **Ctrl+C 无法中断运行中命令(通路层,影响所有 shell)**(007 F2) | — | 007 实测坐实:0x03 经 portable-pty ConPTY 主端写入**不触发** CTRL_C_EVENT——`cmd /c ping`+0x03 五秒不退无 ^C、交互 pwsh+ping 同样断裂、UI 级配方 C 复现;portable-pty 0.9.0 spawn 无 CREATE_NEW_PROCESS_GROUP(嫌疑排除),主端写管道无事件注入 API;根因=OS conhost 对 VT 输入 0x03 的控制事件翻译(MS Q&A/wintty#155/winpty#116 灰色地带);证据链+候选修复通路(helper 进程 GenerateConsoleCtrlEvent / conhost 版本调查 / 上游 portable-pty)见 `docs/designs/003-ash-compatibility.md` §4 F2;复现器留档为 `ash_integration.rs` 两个 `#[ignore]` 用例 | 修复另立计划(虚拟桌面"打断挂死命令"依赖此);与 #7 的 win32 GenerateConsoleCtrlEvent 备注同源,007 已从"待复测"坐实为"确认缺陷" |
| 13 | ash 内建命令不可被 Ctrl+C 中断(ash 侧,跨仓协调) | — | 007 F1:内建全程 raw mode、ash 阻塞在 `std::thread::sleep`(auto-shell `cmd/commands/sleep.rs:35`)不读 stdin,0x03 排队到内建结束;Windows Terminal 下同样如此,与终端无关;外部子进程路径正常(ash `frontend/subprocess.rs:43` 临时退 raw mode)。归属 auto-shell 仓,本仓只记证不修复 | auto-shell 侧评估内建执行期读 stdin/轮询;与 #12 叠加构成"ash 长命令全不可中断"现状 |

## 新增观察(PLAN-004 实测,设计输入)

- **pwsh 冷启动拖慢 iced 首显**(visible=false 到首帧):窗口首显
  resize 风暴会清选中(core damage 语义)——dev 注入钩子需自愈式;
- **取证链路定型**:PrintWindow(PW_RENDERFULLCONTENT)+ 
  SetProcessDPIAware(高缩放屏必须,否则只取左上象限)+ 背景直通带
  内容自校验;CopyFromScreen 受遮挡/前台竞态污染,降为辅助;
- **DevTick 注入必须返回 Task**:丢弃返回 Task = 剪贴板写等副作用
  静默失效(T4 实证);
- iced `Shell::request_input_method` 仅 redraw 相相位消费(见 #11)。

## Phase 4 候选清单(优先级已定,用户裁定 2026-09-05 次序)

> **006 结论(2026-09-05)**:#7 调查关账——四问全正面,建议 GO;
> 材料与 #8 拆解草案见 `docs/designs/002-autoize-feasibility.md`。

> **总排序裁定:先把 Rust 版功能做齐全、测试完备(下述 1-5 及收尾),
> 然后才做 Auto 复刻(#7 调研 → #8 复刻)**——参考实现一次性做稳,
> 复刻只做一轮,oracle 不漂移。#6 不在本轨。
>
> **进度更新(PLAN-005,2026-09-06):1-4 已全部清偿**(证据:
> docs/designs/evidence/005-interaction/,README 有索引与手工
> 清单)——**"Rust 功能齐全"启动条件达成,#8 Auto 化可启动**
> (前置 #7 调研仍待做)。

1. ~~拖选到边缘自动滚动~~ **已清偿(005 T4)**:Extend at_edge +
   drag_scroll 条件订阅 50ms×2 行(菜单开着暂停);转储证锚定
   绝对行 + scroll_offset 顶满钳制;
2. ~~块选(Block)+ 右键上下文菜单~~ **已清偿(005 T2/T3/T5)**:
   Alt+拖选恒 Block(与多击正交)+ is_block 列带渲染
   (block_aligned 像素证);右键改菜单(复制/粘贴/全选,动作接
   004 路径,直接粘贴废止);
3. ~~选中主题色配置~~ **已清偿(005 T6)**:`--selection-color
   RRGGBB[AA]`,默认 e8e8e8@25%(理论混色像素级吻合);
4. ~~光标形状(Underline/Beam)与闪烁~~ **已清偿(005 T7/T8)**:
   DECSCUSR 三形分形渲染(像素证据);闪烁四重门控条件订阅
   500ms,失焦/隐藏/菜单/preedit 均不闪;零唤醒回归 frames=5@11s;
5. Ctrl+C 稳定版复测(债务 #7,待环境后关账或立项 win32);
6. Unix 基座适配(独立轨道,不属"功能齐全"批);
7. **已关账(PLAN-006,2026-09-05)**:iced↔AutoUI 生态对齐调查毕——
   `.at` 组件模型**不承载**自定义渲染(View 封闭 28 变体),TermGrid
   唯一表达路径 = auto-lang 内原生组件(自定义 iced Widget,code_editor
   范式四件套接线);证据链与 spike 见
   `docs/designs/002-autoize-feasibility.md` + `spikes/autoize-roundtrip/`;
8. **Auto 化——必须项**(用户裁定 2026-09-05,升级自"候选"):
   虚拟桌面当前**只支持以 AutoUI 代码加载 app**(AutoUI = **auto-lang
   内 ui 模块**:Auto 描述层 + iced/gpui 后端;原独立 auto-ui 仓已废弃
   合并,旧引用 006 已修正),AutoTerm 入驻虚拟桌面的唯一路径就是
   Auto 化;AutoLang 经 a2r(auto-lang)转回 Rust——
   - 本仓 Rust = **参考实现/验收 oracle**;往返口径(006 实证修正):
     **语义等价 + rustc 实编 + 行为对拍**,非字节级一致(构造面 1:1,
     i64/pub/杂点为系统性变形;a2r F1 派生组合缺陷须先修或规约绕开);
   - 复刻范围 = 应用层(autoterm-core 封装 + autoterm-ui App/
     TermGrid,约 2k 行);引擎复用形态(006 Q2 修正)= Rust adapter:
     alacritty_terminal 绑定自动管线不可行(泛型+trait 回调落界外)、
     portable-pty 部分可用、iced 走 View/renderer 集成而非 FFI;
   - **对后续 Rust 侧开发的约束**:新代码保持 a2r 可表达形态
     (idiomatic、避免 Rust 特有奇技),维持往返一致性;
   - 前置依赖 #7:**已关账**——TermGrid 走 auto-lang 原生组件
     (code_editor 范式),系 auto-lang 侧补齐工作(W1,估 8–15 人日),
     连同 a2r 修缮(W2)构成 #8 第一相位(P0/P1,拆解草案见 002 §6);
   - **启动条件**:Rust 版功能齐全 + 测试完备(1-5 及收尾)之后;
     总工作量级 25–50 人日(两仓,002 §5);go/no-go 建议分支 A,
     终裁留给用户。
