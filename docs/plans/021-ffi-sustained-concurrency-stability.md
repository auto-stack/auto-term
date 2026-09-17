---
plan_id: PLAN-021
status: drafting
feature_name: VM 视图/事件管线稳定性专项(FFI sustained 堆破坏 + 指针事件派发/scroll/split 交互修复)
author: [zhaopuming/zcode-session]
created_at: 2026-09-17T00:00:00Z
updated_at: 2026-09-17T00:00:00Z
plan_revision: 2
current_step: 0
total_steps: 8
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
---

# PLAN-021 · VM 视图/事件管线稳定性专项(FFI sustained 堆破坏 + 指针事件/scroll/split)

## 0. 变更摘要

PLAN-020 复审发现(P1):vue 形态 back(app-back.exe)在浏览器页面
打开并轮询 ~2 分钟后必然 **STATUS_HEAP_CORRUPTION**(0xc0000374)
崩溃,2/2 复现(vue-run.log 15:31:22 / 15:54:43);用户裁定专项立项
排查修复。

**范围扩充(2026-09-17,020 复审移交)**:除 F1 外,并入 **VM 形态
mouse_area press→消息派发失效**缺陷——双区探针(300×300 大块 +
10px 细条)用户实点 2/2 计数为 0(MCP 开/关、DPI 感知/非感知、
多实例均复现;同窗口 Button 点击正常、700+ 次 tick 正常)。渲染臂
(21162)每帧执行且 on_click 已接线,断点在 iced 事件→widget
publish→VM 派发之间。该缺陷直接阻塞 020 的 AC-04(分隔条拖拽)。
F1 与本缺陷同属"VM 视图/事件管线稳定性"家族,合并排查。
020 移交项在本计划落位为 T-05[D5](勿与 FFI 线 T-02 混淆)。

**范围扩充二(2026-09-17,用户指令,rev2)**:并入 **panel scroll +
AutoUI 官方 scroll 组件接入 + split 拖拽**三题,按**一条任务链**
处理而非三个独立任务。结构性耦合依据(本仓读码 + 020 在案事实):
1. 同链:终端滚动条 thumb 拖拽为 widget 自绘,与分隔条 mouse_area
   同走 VM 指针事件链(auto-lang ui/terminal/iced/widget.rs:513-590
   ButtonPressed→ScrollbarDrag→Moved/Released;wheel 另支 :606)。
   020 已证该链 press 派发断裂(探针 2/2 计数 0);官方 scroll 组件
   的 press/drag 同样必须过此链——换组件换不掉根因,组件有问题
   要修的大概率也是这条派发链;
2. 同层:管线修复、组件接入/修组件、分隔条复验均落在 auto-lang
   ui/iced 事件层,共享同一前置修复;
3. 命中区物理竞争:终端 widget 右缘自划 14px 滚动条命中条
   (SCROLLBAR_HIT_W=14.0,widget.rs:1029),020 重构的分隔条命中区
   为 8px,面板边界处相邻/重叠,归属规则必须联合设计,否则链修通
   后出现"边缘互吞"新回归。
子路径区分:press/drag 与 wheel 是两条子路径(020 探针仅证死
press),取证时一次仪器化全覆盖,死法差异本身是断点定位证据。

已确立的边界事实(020 复审隔离实验):
1. 纯同端点并发(20×tick×3 轮)**不崩**;
2. 混合端点并发(tick+apply-resize+pane-lines+rect-*,含引擎 FFI,
   ×5 轮)**不崩**;
3. boot 短轮询窗并发 **不崩**;
4. **浏览器页面持续轮询形态:2/2 必崩**——纯 HTTP 请求谱无法复现,
   需要"页面长时叠加轮询"形态,指向 sustained 负载下的累积性缺陷
   (内存布局/句柄/线程专有状态类),而非单次请求竞争。

## 1. 目标

- G1 定位根因:确定堆破坏的确切发生面(DLL 内部 / 侧车胶水 /
  a2r 生成代码 / 三者交互),有可复核的证据(仪器日志/转储)。
- G2 修复落地:修复后 vue 形态在页面持续轮询下稳定运行
  (≥10 分钟 ×3 轮浸泡零崩溃),且不回归 rust/vm 形态。
- G3 防回归:确定性复现脚本入库存档,修复以回归/金样钉住。
- G4 交互链修复(rev2):VM 指针事件管线修复后,panel 滚动(改用
  AutoUI 官方 scroll 组件,组件缺陷即修)与 split 拖拽在 VM 形态
  可用;滚动条×分隔条命中区归属规则定稿;020 AC-04 携带项关案。

### 非目标

- vue terminal 交互能力扩展(019 边界:只读视口,维持);
- FFI 根修不得引入 VM/rust 形态行为回归(suite+冒烟守护);
- 019 已记账的 DEBTS #20/#21 的清理(除非根因恰好落在同一处)。

## 2. 架构方案

线 A(FFI 堆破坏)——三条并行取证线,按证据收敛:

1. **复现固化线**:无头化复现器——浏览器(或等效持续轮询驱动)
   + vue 栈,恢复 "2 分钟必崩" 的确定性;失败态保留转储
   (.LocalDumps 或 _NOT_ARRARY)供堆分析。
2. **现场仪器线**:分线程标记——在引擎 FFI 调用点(engine_rows_for/
   apply_resize/pump/spawn/free)与 glue 快照表写入点注线程 ID +
   时序日志;堆破坏首现前的最后调用序列即嫌犯面。
3. **嫌疑面清单**(按先验排序):
   - a. autoterm_core.dll 内部对**同句柄并发调用**的线程安全
     (snapshot/scrollback 重分配段);axum 多 worker + 50ms 轮询
     使同/异句柄 FFI 重叠概率远高于单线程 UI 面;
   - b. 侧车胶水(term.rs)快照表与 DLL 内部状态的生命周期交错
     (free 后悬置/写写冲突);#21 已证锁守卫跨语句,非全期;
   - c. a2r 生成代码的全局互斥粒度(两段锁的 check-then-act),
     至多丢更新,不解释堆破坏——低先验,排除用。

修复候选(按定位结果选):DLL 内部串行化/加锁(autoterm-core);
胶水侧调用面收敛(单引擎互斥);或最小并发窗口消除。**修复不得
以破坏 014 几何随动/泵三档为代价**。

线 B(VM 指针事件管线,rev2 并入)——一条任务链,顺序固定:

1. **管线取证+修复(链共享前置)**:press/drag/wheel 三子路径一次
   仪器化;020 双区探针复跑为基线(现状:press 2/2 计数 0,渲染臂
   正常);断点定位后修复落 auto-lang ui/iced 事件层;
2. **官方 scroll 组件接入(链上消费者一)**:各 panel 滚动统一改用
   AutoUI 官方 scroll 组件(接入面参考 auto-lang
   examples/ui_scroll.rs;终端自绘滚动条是否由官方组件替换由本步
   勘定);组件自身缺陷即修(同层改动)。组件兼作"链是否修通"的
   对照实验——修复后组件仍不可用即断点未穷尽;
3. **命中区联合设计+split 拖拽复验(链上消费者二)**:滚动条命中区
   (自绘 14px 或官方组件等效区)×分隔条 8px 命中区的宽度/优先级/
   z-order 归属规则定稿并实现;三联合场景(带滚动条面板旁拖分隔
   条、拖 thumb、接缝点击归属)闭环后,020 AC-04 携带项关案。

## 3. 技术栈

- auto-term(主):crates/autoterm-core(DLL 引擎)+ app 侧车胶水
  + vue 栈;主检出直落惯例。
- auto-lang(主,线 B):ui/iced 事件层(press/drag/wheel 派发)+
  官方 scroll 组件 + ui/terminal/iced/widget.rs(自绘滚动条);
  按 worktree 流程入役。
- auto-lang(或有,线 A):若定位指向 a2r 生成代码,按 87eba67ab
  裁定走 lang worktree(独立评估,本计划预授权仅限"证据要求时")。

## 4. 需求分析与背景调查

### 4.1 授权记录

- 用户 2026-09-17 裁定(020 复审 F1 三选):**选项 1 vue 臂稳定性
  专项立项**,排查引擎 FFI sustained 并发缺陷。
- 用户 2026-09-17 指令(rev2 扩充):panel scroll + AutoUI 官方
  scroll 组件(有问题则修组件)+ split 拖拽;经问询确认 scroll 与
  split 边界结构性耦合,按一条任务链处理。
- 仓库:auto-term(主);auto-lang 仅在证据指向生成代码时经
  worktree 进入。预算/自动续跑:未指定。

### 4.2 已有证据(全部在库)

- 崩溃实录:app/vue-run.log(0xc0000374 ×2;15:31:22 与 15:54:43);
- 隔离矩阵:020 §9 re-review 记录(事实 1-4,见 §0);
- 20GB 内存实录:020 T-01 联调(同进程家族,归因未定,本计划一并
  观察——若同根因则一并关闭);
- 相关在案债:DEBTS #21(锁守卫跨语句,ABBA 家族)、#24(竞态
  与遍历上界)。

### 4.3 代码事实(读码 2026-09-17)

- 引擎 = crates/autoterm-core(libloading 由 glue 加载;VM/rust
  同一 DLL);glue = app/term.rs(a2r 侧车,SNAPSHOTS/SessionState
  均有 Mutex);
- axum back = 多 tokio worker,每个请求独立 worker → 同句柄
  FFI 可重叠(019 #21 已证锁守卫只护单语句);
- vue 页面 tick 50ms:term_apply_resize + get_lines + pane_lines
  (逐存在槽)+ rect-*/tab 系(纯 db);
- 终端滚动条自绘走 VM 指针事件链:auto-lang
  ui/terminal/iced/widget.rs:513-590(press 起 ScrollbarDrag→
  Moved→Released)、:606(WheelScrolled)、:1029(SCROLLBAR_HIT_W
  =14.0);
- 020 分隔条=定尺寸 div(视觉+命中)+Fill mouse-area(8px),与上述
  14px 命中条在面板边界相邻/重叠;
- 020 隔离事实:mouse_area press 2/2 计数 0,渲染臂每帧正常,
  断点在 iced 事件→widget publish→VM 派发之间;同窗口 Button
  点击正常(对照)。

## 5. 详细设计

| # | 改动 | 文件:符号(仓) | 说明 |
|---|---|---|---|
| D1 | 复现固化 | scripts/repro(新)+ vue 栈驱动 | 页面轮询形态的无头等价驱动;目标 ≤3 分钟稳定复现 |
| D2 | 仪器取证 | crates/autoterm-core + app/term.rs 临时注记 | FFI 调用点线程 ID/句柄/时序日志;首现前的最后序列=嫌犯面 |
| D3 | 根修 | 视 D2 结果:autoterm-core 内部锁 / glue 调用面收敛 | 修复不得回归 014 几何随动/泵三档;单测/金样钉住 |
| D4 | 防回归 | tests + DEBTS | 复现脚本转为回归浸泡;DEBTS #25 关案记录 |
| D5 | 管线取证+根修 | auto-lang ui/iced 事件层(+临时探针) | press/drag/wheel 三子路径一次仪器化;020 双区探针复跑;断点修复(020 移交项落位) |
| D6 | 官方组件接入 | auto-lang scroll 组件 + 各 panel 接入点 | panel 滚动统一换 AutoUI 官方 scroll 组件;组件缺陷即修(同层);兼作链修复对照实验 |
| D7 | 命中区归属+复验 | terminal widget 命中区 × 020 分隔条命中区 | 归属规则(宽度/优先级/z-order)定稿实现;三联合场景;020 AC-04 关案 |

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add(或 modify) | docs/specs/engine-ffi-color-encoding.md(或 engine 契约所在 spec;T-02 定位后定稿) | 前:引擎 FFI 线程安全承诺未明示。后:明示引擎并发调用契约(串行化承诺或线程安全等级) | 崩溃根因落档为契约;调用方(三形态)按契约使用 | AC-02/03 |

| SD-02 | add | docs/specs/…(VM 交互命中区契约;T-07 定稿回填精确路径) | 前:滚动条命中条与分隔条命中区无归属规则。后:面板边界处两命中区的宽度/优先级/z-order 归属规则成文 | 链修通后防"边缘互吞"回归 | AC-08 |

(018 建的 engine spec 文件名以库内实际为准,T-02 定位后回填精确
路径——本表先立语义。SD-02 同理,T-07 定稿后回填。)

## 6. 测试设计

1. **复现器**:D1 脚本,修复前 3/3 崩溃、修复后 3/3 不崩(各
   ≥10 分钟);
2. **金样**:D3 修复的行为钉(如串行化语义的可观测断言);
3. **回归**:cargo test --workspace 全绿;014 几何随动/泵三档
   冒烟;020 的 t01 curl 剧本复跑(11 槽投影面零变);
4. **实机**:vue 页面打开 10 分钟浸泡 + 用户目验 tab 条/布局正常;
5. **交互链(rev2)**:020 双区探针复跑(press 计数 0→非 0);
   官方 scroll 组件 thumb 拖动+滚轮实机可用;三联合场景(带滚动条
   面板旁拖分隔条/拖 thumb/接缝点击归属)用户实点通过。

## 7. 验收标准

- **AC-01** 复现固化:无头复现器 3/3 稳定触发崩溃(修复前基线),
  产出堆分析可用现场。验证:脚本 + 日志。
- **AC-02** 根因定位:仪器证据指认破坏面(DLL 内部/胶水/生成码
  三选一,含调用序列)。验证:取证日志 + 结论段。
- **AC-03** 修复有效:修复后复现器 3/3 通过(≥10 分钟浸泡零崩溃
  ×3 轮)。验证:脚本 + 日志。
- **AC-04** 零回归:cargo test --workspace 全绿;020 t01 剧本
  12 步复跑全绿;014 几何随动冒烟通过。验证:日志。
- **AC-05** 文档:根因结论入 DEBTS(关案或转正为已知并发契约);
  engine spec 契约面明示。验证:文件在库。
- **AC-06** 管线修复(rev2):VM 指针事件链 press/drag 派发修复,
  020 双区探针复跑计数非 0;wheel 子路径结果如实记录(死法差异
  入取证结论)。验证:探针日志 + 用户实点。
- **AC-07** scroll 组件(rev2):各 panel 滚动改用 AutoUI 官方
  scroll 组件,thumb 拖动与滚轮在 VM 形态可用;组件自身缺陷修复
  在案,对照实验结论入案。验证:实机操作 + 截图/日志。
- **AC-08** 命中区+split 闭环(rev2):命中区归属规则定稿(SD-02)
  并实现;三联合场景(带滚动条面板旁拖分隔条、拖 thumb、接缝
  点击归属符合规则)通过;020 AC-04 携带项关案。验证:实机三
  场景 + 规则落档。

## 8. 执行步骤

- **T-01 [D1] 复现固化**:无头复现器;基线 3/3 崩溃。前置:无。
  关联 AC-01。
- **T-02 [D2] 仪器取证**:线程标记 + 首现序列;产出根因结论
  (更新本表 SD-01 的 spec 目标路径)。前置 T-01。关联 AC-02。
- **T-03 [D3] 根修**:按定位实施;复现器 3/3 通过;套件绿。
  前置 T-02。关联 AC-03。
- **T-04 [D4] 回归+实机**:全量门 + 020 t01 复跑 + vue 浸泡;
  DEBTS/spec 文档。前置 T-03。关联 AC-04/05。
- **T-05 [D5] 管线取证+根修**:press/drag/wheel 一次仪器化;双区
  探针复跑;断点修复(020 移交项落位)。前置:无(与线 A 并行)。
  关联 AC-06。
- **T-06 [D6] 官方 scroll 组件**:各 panel 统一接入+按需修组件;
  对照实验结论在案。前置 T-05。关联 AC-07。
- **T-07 [D7] 命中区+split 复验**:归属规则定稿实现(SD-02);三
  联合场景;020 AC-04 关案。前置 T-06。关联 AC-08。
- **T-08 锚定与收口**:双仓 SHA 回填(线 B 必动 auto-lang)+
  status execution_done。前置 T-04 + T-07。

## 9. 复审记录

- 2026-09-17 stage:new · PLAN-021 · rev1 · outcome:**pass** ·
  授权:用户 2026-09-17 裁定(020 复审 F1 三选之选项 1)·
  设计依据:020 复审隔离实验矩阵 + 崩溃实录 ×2 + DEBTS #21/#24
  家族事实 · 核心风险:复现不稳定(已 2/2,基线可固化)/根因在
  DLL 深处(备选:胶水侧收敛也可达成稳定)· next:work。
- 2026-09-17 rev2:范围扩充二(用户指令,见 §0)——线 B 入案
  (D5-D7/T-05..07/AC-06..08/SD-02,收口改 T-08);rev1 复审结论
  仅覆盖线 A,扩充部分待复审后再进 work。

## 10. 待澄清事项

1. 20GB 内存实录(020 T-01 联调)与本案是否同根因——T-02 取证时
   一并观察,同源则并案关闭。
2. auto-lang 是否入役:a2r 生成代码全局两段锁至多丢更新,低先验;
   仅证据指认时才动(届时按 worktree 流程)。
3. "各个 panel"的滚动清单(终端外还有哪些面板需要 scroll)与官方
   scroll 组件 API 接入面,及终端自绘滚动条是否由官方组件替换
   ——T-06 开工前勘定回填。
