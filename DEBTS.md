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

## 009 新增观察(PLAN-009 T11 收账,2026-09-06;PLAN-010 T9 逐条销账)

1. **~~a2r 快照存量编译债(74 例)~~ 已清偿达标(PLAN-010 T2,
   2026-09-07)**:74 → 37(恰 50%);首批 question 族 21 例、批2
   say/std 前导/委托缺省/空集合/mut 注册表/Box 免克隆/Option-get/
   comptime/闭包块尾 15 例(理由逐条在案);遗留 37 例
   (智能指针自动解包族/interop 外部框架族/语法未编码族/ownership-闭包
   耦合族/singles)留 ledger 继续;实编门 0 unexpected,失败清单已
   逐例输出(门禁设施补齐)。
2. **~~a2r `List.new()` 路径限定 bug~~ 已修(PLAN-010 T1)**:根因=
   Type.method() 表达式 source_crate 兜底限定误伤内置集合映射;修=
   auto_type_to_rust 命中者(List/Map/Set→Vec/HashMap/HashSet)豁免
   use.rs 前缀;快照 a2r/10_collections/007_list_new 先红后绿入库;
   `[]` 字面量绕开写法不再必要。
3. **~~CLI `auto trans` 挂起~~ 已修(PLAN-010 T3)**:根因=单文件 CLI
   三处兄弟 .at 预扫无界(crate-root 无条件 parent().parent()),任意
   路径遍历整棵无关树逐个 parse;修=src 字面名才按 crate 根扫+深度/
   数量/大小/垃圾目录统一有界;60 行样本 13.3s 完成,产物编译绿,
   与 in-process 逐字节一致。
4. **Vue/web terminal 后端留白**(维持留白):未来虚拟桌面需 web 形态
   时另立调查(xterm.js 类渲染 + 引擎桥可行性;PLAN-009 待澄清4)。
5. **terminal 组件数据面性能余量**(维持观察):形态甲(props-feed)
   2000 行流式基准 <0.1s(debug,帧预算 16ms),无降级需要;超大规模
   流式如遇帧预算压力再评估形态乙(预授权规则在案)。
6. **~~at-gen 窗口标题 cosmetic~~ 已修(PLAN-010 T5)**:auto-lang 侧
   run_app_with_title(Option<&str>) application 链标题面(待澄清④
   最小改,不动 VM 轨);at-gen main 接 Some("AutoTerm")。
7. **~~E0080~~ 已复核销账(PLAN-010 T4)**:ui_counter/ui_accordion +
   tick_interval_ms(Some(16)) 临时变体两例 --features ui-iced 双绿
   (build 级验证,变体验证后撤销)——TickWrap 变体构造器根修实证
   有效,002 §4 "两示例同炸"解除。
8. **~~色彩对拍 ash 依赖~~ 已销账(PLAN-011 T1,2026-09-07)**:
   对拍门禁色彩场景由「ash 缺席显式 skip」改为 **ANSI 注入实跑**
   ——同一 powershell 单行命令经 cmd 会话向 stdout 输出 SGR 批次
   (命名 0-15/索引 16-255 抽样/RGB;输入管注入实测证伪:SGR 仅被
   cmd 字面回显、网格零样式格,故走子进程输出路径),oracle 与
   a2r 两侧标量色面(`kind<<24|value`)批次 7 行逐格等价 + 非空转
   哨兵;门禁六场景全实跑零 skip,ash 不再是任何场景的前置。
   (计划 011 原引「009 新增观察 #4 色彩自证 ash 依赖」实为此处
   补账:#4 原条为 Vue/web 留白,无关色彩;归档 009 正文的 skip
   表述为历史记录,不改。)
9. **桌面集成面回执(auto-os OS-013,2026-09-07)**:003 §5 契约执行核对
   ——①cdylib+ctrlc 同目录分发 ✓(auto-os scripts/deploy-autoterm.sh,
   幂等,来源解析 env→组兄弟→主检出);②中断可用 ✓(桌面会话 Control-
   Break+中文统计证据在案,smoke 重试口径);③遗留观察:宿主桌面窗口
   直接关闭时的引擎子进程清理未显式验证(Close=free 在案);④auto-lang
   侧配套已落地待折:VM 动态轨 Terminal 真渲染两臂(convert_view_messages
   显式臂+convert_terminal 堆 ListData 物化,`.wt/auto-os-013/auto-lang`
   分支 auto-os-config-dev)。引擎契约面(12 符号 FFI/语义)零改动。
10. **引擎 Rust 形态口径修正与 Auto 化路线(004,2026-09-07 用户裁定)**:
    a2r 可复刻任意 Rust = Auto 语言设计目标;"引擎保持 Rust"(002 Q2/
    009 P2)由终局裁定降格为**分期工程裁定**——现阶段是 Auto/a2r 能力
    不足(外来泛型类型字段/外来 trait impl/trait 对象/裸线程+阻塞 io/
    unsafe 面),非"不支持所以不做",四模块保留未来切 Auto 源选项。
    C ABI 互操作(FFI/win32 同为 C ABI DLL)归宿 = **C 通道**(a2c/
    c_ffi/auto-bindgen;unsafe 消解而非解决,Auto 不设 Rust 式 unsafe
    面)。路线与能力缺口账全文 `docs/designs/004-engine-autoization-
    roadmap.md`(ledger D004-2/D004-4);工作项五条全归 auto-lang 侧:
    ①auto-bindgen 补 kernel32 console manifest ②a2c 复刻 ctrlc+对拍
    (验收复用 ctrl_event/parity_interrupt 门禁)③12 符号面 C 通道驱动
    ④a2r 语言能力四项 ⑤cdylib 导出发射(远期)⑥C 通道 a2r 后端
    (`use.c` 三后端下降:VM/a2c 已在,a2r 生成 Rust FFI 缺——
    auto-bindgen 升格共享 IR,S 静态/D 动态双形态,回调 trampoline
    硬点)。002 原文不改,口径
    以 004 为准(§7 勘读)。
    **①② 已落地清账(auto-lang PLAN-595,2026-09-09)**:FnPtr 变体+
    abi 注记+windows.h manifest 6 函数入 auto-bindgen(6 测绿);a2c
    复刻 autoterm-ctrlc 成真 exe(ctrl_event 门禁产物置换跑 a2c helper
    3 passed 1 ignored 与 Rust 版同绿,静态对拍 3,3/2,2);执行期顺修
    a2c 闭包原型缺陷(真 MSVC 编译 C2065,快照盲区)。回执全文
    004 §5。
    **③ 已落地清账(auto-lang PLAN-597,2026-09-09)**:a2c 全量驱动
    autoterm_core.dll 12 符号面真机通过(CFACE_OK/exit 0;MSVC 须链
    .dll.lib import lib,直接 DLL 输入 LNK1107);VM 标量子集打通
    (use_scanner 潜伏缺口=任何 VM 模式 use.c 均报模块不存在,修复+
    JSON manifest 加载面+三分派臂,VFACE_OK);缓冲出参 4 符号实证
    VM 不可达归 004⑥;路径裁定:独立工具→a2c 链入/宿主内→a2r+侧车/
    VM→标量子集;附带 VM 轨两枚存量缺陷实测定位在案(597 §9)。
    **④ 已落地清账(auto-lang PLAN-599,2026-09-10)**:a2r 外来形态四
    能力(泛型字段/ext-for 外来 trait/dyn Trait+Send 拼写/裸线程 std
    透传)语料五件+rustc 实编门;capstone term.rs 子集与 Rust oracle
    黑盒 stdout 全等(共享 fake_core stub);执行期两修(dyn 字段派生
    语义修订/spawn-move 双发);泛型字段与外来 trait impl 系既有计划
    已原生可用(002 缺口 1.1 实为已清),599 补齐语料/实编/文档;F6
    告警面 env 门控默认静默(全量噪音实测)。回执全文 004 §5。
    **⑤⑥ 已落地清账(auto-lang PLAN-610,2026-09-11)**:C ABI 双面全通。
    ⑤导出面=#[export]/#[export(system)] 兄弟包装模块发射(no_mangle
    extern,符号=fn 名,调用方零改动;int i64↔i32 边界 cast/cstr CString
    边界/句柄空安全解引用/缓冲 kit 收口全部 unsafe 于生成码);⑥ use.c
    双形态下降(manifest 共享 IR 增 link 字段:S 静态 #[link] extern /
    D 动态 libloading env→exe同目录→PATH)。三级 CFACE_OK 实证:
    AC-03 引擎 12 符号 Auto 版 cdylib(005 语料,包真 PtySession)被
    597 a2c 驱动器链接跑通;AC-04 ⑥ S 驱动链真 autoterm_core.dll.lib;
    AC-05 同一 ⑥ 驱动产物改链 ⑤ 产物=**Auto↔Auto 全链零手写胶水**
    (import lib 内嵌 DLL 名换引擎);AC-06 D 形态运行期加载。语料
    27_c_abi 五件+三道 #[ignore] 实编门;trampoline 选型=A' 具名导出
    fn 按名传值(闭包直转证伪 E0308,实作 defer)。回执全文 004 §5。
