# DEBTS.md — 已知债务

> PLAN-001 盘点 · PLAN-002 勾账 · PLAN-003(Phase 2 性能与输入)勾账 ·
> PLAN-004(Phase 3 交互完备)勾账 · PLAN-005(Phase 4 交互批)勾账 ·
> PLAN-006(#7 调查关账)· PLAN-007(ash 门禁,#12/#13 新增)·
> PLAN-008(#12 修复双发落地,#7 关账,#13 供三态数据)· 2026-09-06。

## 债务账本(PLAN-004 后状态)

| # | 债务 | PLAN-001 现状 | PLAN-004 后 | 清偿方向 |
| --- | --- | --- | --- | --- |
| 1 | 整帧重拼渲染 | 每帧全网格重画 | **已清偿(003)**:保留式画布——每行 Paragraph 缓存+digest 判异+脏行门控,末帧重建 0 | — |
| 2 | 无字形图集 | 每 run 独立 fill_text | **已清偿(003)**:每行单 Paragraph,shaping 由行缓存持有;GPU 字形缓存 iced 内置 | 图集化(wgpu atlas)留待真有性能需求时 |
| 3 | 固定字形度量 | Consolas 硬编码 + 拟合 | **已清偿**:cosmic-text 实测 cell_w(9.375px@16px),fit_ok 数值断言;宽字符 unicode-width(实测证伪 advance 法) | — |
| 4 | 16ms 轮询驱动 | time::every tick | **已清偿**:唤醒通道事件驱动(6s 59 次更新 vs 372 轮询),常态零定时器 | — |
| 5 | 无滚动回滚 UI | — | **已清偿**:滚轮/PgUp/PgDn/键入回正/↑N(offset=171 回顶实证) | 滚动条可选 |
| 6 | 无光标块/选中/IME | 键盘只回写 | **已清偿(004)**:选中三模式(拖选/双击词/三击行)+ 高亮(像素证据)+ copy-on-select + Ctrl+Shift+C/V/右键粘贴(剪贴板读回断言);IME 管线全通 + 自绘 preedit(像素证据);**残留:人工拼音清单待用户执行**(001 附录跑法) | 人工清单跑完即全关 |
| 7 | Ctrl+C / 关闭语义 | 未验证 | 关闭已清偿(002);**已清偿(008)**:win32 直调立项并落地——`autoterm-ctrlc` 辅助进程 AttachConsole 后广播控制事件(`PtySession::interrupt()`,Break→C 双发),`cmd /c timeout` 5s 内真实中断门禁(`ctrl_event.rs`);稳定版 conhost 复测随事件注入失去必要性(不再依赖 0x03 翻译行为);008 待澄清#5 记录了"双发+残留缺口"形态的用户裁定项 | — |
| 8 | 仅 Windows 基座 | 同 | 未动 | Linux/macOS 计划 |
| 9 | spike 无统一文档 | 同 | **已清偿**:crates/* 正式结构 + 001 设计文档 | — |
| 10 | 颜色表硬编码 | 16 色/xterm256 内置 | **已清偿**:palette.rs 全 NamedColor 映射(Dim×8/Bright/Dim 前景,TDD) | 主题系统可选;选中色已清偿(005 `--selection-color`) |
| 11 | IME over-the-spot 运行时覆盖层不落屏(新) | — | 004 实测:iced_winit main-events 相相位丢弃 `State::Updated{input_method}`(381 次请求埋点实证),redraw 相相位应用链在本机不出画面;已按裁定降级自绘(可用) | 升级 iced 版本时重试 `Enabled{preedit: Some}` 路线,成则删自绘 |
| 12 | **Ctrl+C 无法中断运行中命令(通路层,影响所有 shell)**(007 F2) | — | **已清偿(008,2026-09-06)**:辅助进程 `autoterm-ctrlc.exe`(手写 kernel32 FFI,零新依赖)AttachConsole 进 ConPTY 控制台广播控制事件;**实测(26200.9168)CTRL_C 广播被 ConPTY 客户端吞掉、CTRL_BREAK 可达** → `interrupt()`=Break→C 双发 + helper exit=0xC000013A 同判成功(handler 派发竞态:被事件杀死=事件已广播);UI 裸 Ctrl+C 双投递(事件+0x03),`--ctrl-c-mode auto\|byte\|event\|both`(默认 auto,ash 豁免仅字节);门禁 `tests/ctrl_event.rs`(cmd/timeout 5s 中断、pwsh 回提示符、helper 缺失降级),UI 铁证 cmd 死循环 both=停/byte=继续;机制全记录 `docs/designs/003-ash-compatibility.md` §4.1 | **残留缺口(26200 类 build)**:ping 类对 Break 特殊处理(打统计继续)仍不可中断,`#[ignore]` 复现器留档,健康 build 转红即 C 通道到位;分发约束:autoterm-ctrlc.exe 须与主程序同目录(缺失自动降级) |
| 13 | ash 内建命令不可被 Ctrl+C 中断(ash 侧,跨仓协调) | — | 007 F1:内建全程 raw mode、ash 阻塞在 `std::thread::sleep`(auto-shell `cmd/commands/sleep.rs:35`)不读 stdin,0x03 排队到内建结束;Windows Terminal 下同样如此,与终端无关;外部子进程路径正常(ash `frontend/subprocess.rs:43` 临时退 raw mode)。归属 auto-shell 仓,本仓只记证不修复。**008 三态矩阵补证(003 §4.1)**:idle 态事件整体终止 ash(无 ctrl handler)→ auto 模式暂豁免 ash 为仅字节;builtin 态真事件也惰性(ash 存活至内建自然结束);external 态子进程中断正常 | auto-shell 侧:装 ctrl handler(idle 存活)+内建执行期处理事件即可撤 auto 豁免并打通内建中断;协调请附 003 §4.1 矩阵数据 |

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
5. ~~Ctrl+C 稳定版复测(债务 #7)~~ **已关账(PLAN-010 T6,
   2026-09-07)**:关账证据 = 009 `parity_interrupt` 门禁——26200 真实
   ConPTY 上 timeout 主体 + 双投递(auto/事件+字节)广播成功路径实测
   有效(门禁绿);auto-term 侧 `engine_ffi_integration` interrupt 分支
   在案。#5 与旧账 #7(006 调查项)合并销账,同一事实;win32 立项
   不再需要;ping 类 Break 免疫残留维持 §4.1 已知边界记录;
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
   - **第一相位关账(PLAN-009,2026-09-06,分支 A 单计划)**:#8 的
     P0–P4 落地——a2r F1 修复 + rustc 实编门(auto-lang)、`terminal`
     组件真身迁移(视口/损伤/选中/IME/滚动/菜单,iced 双端测试)、
     引擎 cdylib adapter(12 符号 FFI 面 + libloading 集成测试)、
     `.at` 复刻应用 + **行为对拍门禁**(启动/echo/resize/Ctrl+C 中断
     网格等价 5/5,色彩/选中显式 skip)+ UI 冒烟取证(程序化 vtree +
     真窗口截图);无环断言过(auto-lang 0×autoterm;仅 at-gen→
     auto-lang 运行时 1 边)。关账条件三条全数在案:**对拍门禁在库**
     (crates/autoterm-parity)+ **组件真身迁移完成**(auto-lang
     src/ui/terminal/,本仓 widget.rs 冻结为参考 oracle)+ **本仓转型**
     (README 定位节)。遗留 → 见下方"009 新增观察"。

## 009 新增观察(PLAN-009 T11 收账,2026-09-06)

1. **a2r 快照存量编译债(74 例)**:rustc 实编门(auto-lang
   `a2r_rustc_real_compile_gate`)首跑暴露 006 时代文本金样管线的存量
   编译炸裂(错误码逐例在 auto-lang `test/a2r/compile_gate_known_broken.txt`
   ledger);门对新破坏必红,ledger 变绿即报 obsolete。修复属 a2r codegen
   缺陷账(建议以 question 族为首批)。
2. **a2r `List.new()` 路径限定 bug**:`List.new()` 会被错误限定为
   `<最后 use.rs 符号>::Vec::new()`(T8 实证);绕开:空列表用 `[]`
   字面量(at-gen/README.md)。
3. **CLI `auto trans` 挂起**:master 存量回归(f3185a7 时代可用);
   in-process `transpile_rust` 同源正常,转译规程走库面(at-gen README)。
4. **Vue/web terminal 后端留白**(非目标确认):未来虚拟桌面需 web 形态
   时另立调查(xterm.js 类渲染 + 引擎桥可行性;PLAN-009 待澄清4)。
5. **terminal 组件数据面性能余量**:形态甲(props-feed)2000 行流式
   基准 <0.1s(debug,帧预算 16ms),无降级需要;超大规模流式如遇帧
   预算压力再评估形态乙(预授权规则在案)。
6. **at-gen 窗口标题 cosmetic**:"Auto Lang - Iced" 为 iced application
   缺省标题;复刻应用应暴露自定义标题(iced .title() 一行)。
7. **E0080 已顺手修(T8,R4 预授权内)**:run_app tick 订阅捕获闭包被
   iced 0.14 const 检查必炸——TickWrap 变体构造器零捕获方案
   (auto-lang renderer.rs);002 §4 记录的"两示例同炸"应已解除,
   建议_auto-lang 侧复跑当年两个示例复核后销账。
