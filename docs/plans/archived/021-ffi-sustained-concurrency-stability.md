---
plan_id: PLAN-021
status: archived
feature_name: VM 视图/事件管线稳定性专项(FFI sustained 堆破坏 + 指针事件派发/scroll/split 交互修复 + auto-lang 跑法缺陷簇 Phase 2)
author: [zhaopuming/zcode-session]
created_at: 2026-09-17T00:00:00Z
updated_at: 2026-09-18T00:00:00Z
plan_revision: 3
current_step: 8
total_steps: 11
supersedes_spec_components: []
new_spec_components:
  - docs/specs/engine-ffi-color-encoding.md#并发契约(SD-01,merge 步发布)
completion_kind: delivered
touched_goals: []
---

# PLAN-021 · VM 视图/事件管线稳定性专项(FFI sustained 堆破坏 + 指针事件/scroll/split)

## 0. 变更摘要

**范围扩充三(2026-09-18,用户裁定,rev3 · Phase 2)**:T-07 排障
发现的 auto-lang 跑法缺陷簇纳入本计划为 Phase 2(用户指令:"直接
在当前计划文件里记录即可(新的 phase),但是工作可以开新的
auto-lang 的 worktree 去工作"):
- **T-09** rust 轨 codegen 漂移:以当前运行时从零编译生成的
  app 前台 crate 252 错(serde_json::Value vs i32/Vec<String>,
  mux_* 面签名不一致)——master 的 rust 轨 codegen 与运行时接口
  漂移,被增量编译缓存长期掩盖;根修后才能重建含 021 修复的
  rust 轨 app(T-07 载体)。
- **T-10** dev 跑法 VM api 委托断供:`auto run -r vm` 前台 tick
  照跑但 api.* 零到达(AUTO_FFI_TRACE 环境敏感 + 轮询/渲染多断
  点,证据链 evidence/021/vm-delegation-break.log)。
- **T-11** 部署态重建:rust 轨 auto-term.exe 链回含 021 修复的
  运行时 + 修复版 DLL,作为 T-07 实点载体。
bisect 终案与逐变量排除记录见 evidence/021/vm-delegation-break.log
(结论:非提交回归;019/020 实机会话实际使用部署态 auto-term.exe
跑法,见 evidence/019/probe-steps.ps1)。

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
| SD-01 | add | docs/specs/engine-ffi-color-encoding.md(T-02 定稿:引擎契约面即本 spec,增"并发契约"节) | 前:引擎 FFI 线程安全承诺未明示。后:明示**每柄串行化契约**——FFI 边界按引擎指针串行化(每柄锁),同柄全部导出(含读类)互斥;调用方(VM shim/rust 侧车/vue back)无需外锁;导出不重入(不得持柄锁再调其它导出)。回归 = tests/ffi_concurrency_serialization.rs 金样 | 崩溃根因落档为契约;调用方(三形态)按契约使用 | AC-02/03 |

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
  **(rev3 裁定改判)**:三联合场景实测暴露自绘滚动条度量/绘制与
  rust 轨分屏渲染缺陷簇,用户裁定连同 SD-02/020 AC-04 一并移交
  官方滚动条(虚拟滚动)计划;021 交付事件链修复(thumb 可拖 +
  滚轮可滚,实机在案)。

## 8. 执行步骤

- **T-01 [D1] 复现固化**:无头复现器;基线 3/3 崩溃。前置:无。
  关联 AC-01。
  [✅ 已完成 2026-09-17] scripts/repro/vue_crash_repro.ps1(无头
  Edge + 页面轮询驱动,WER Event-1000 判崩 + 内存看门狗);基线
  3/3 = 0xc0000374(r1 65s / r2 65s / r3 140s,页面打开→崩均 ≤3min,
  WER 时间戳 evidence/021/repro-r{1,2,3}-wer.txt);全程 WS ~13MB
  无膨胀(20GB 实录未复现,待澄清#1 初证:非同相)。
- **T-02 [D2] 仪器取证**:线程标记 + 首现序列;产出根因结论
  (更新本表 SD-01 的 spec 目标路径)。前置 T-01。关联 AC-02。
  [✅ 已完成 2026-09-17] 仪器 = glue `FfiGuard`(spawn/free/resize/
  pump/feed 五包点,AUTO_FFI_TRACE=1,t/seq/tid/in-flight+OVERLAP)+
  DLL `DllGuard`(变异类导出整行标记;读类由 glue feed 链窗口覆盖)。
  **根因结论(AC-02 三选一:DLL 内部)**:引擎 FFI 导出面全部是
  `ptr_or_null(h)` 裸 `&mut` 别名,零线程安全承诺——vue back 的
  axum 多 worker 对同柄 sustained 并发时,腐蚀对 = feed_ready∥
  feed_ready(drain→term.feed 双 &mut 推进 vte 网格 → Vec 元数据
  堆腐坏)、take_dirty_rows∥row_text(snapshot Vec 整体替换 vs 读
  侧悬垂 UAF)、resize∥*(网格重分配)。证据:①无仪器复现 3/3
  (65/65/140s);②仪器轮 OVERLAP 1062-1213 次/300s,**含同 ptr
  跨线程 feed_ready∥feed_ready**(r4 撕裂日志/r6 整行日志均在案,
  evidence/021/repro-r{4,6}-run.err.log);③静态读码 ffi.rs
  (feed_ready→drain→term.feed 需 &mut;take_dirty_rows 无条件
  `engine.snapshot = visible_styled_lines()`);④金样去锁红相 =
  6 线程同柄混合流 STATUS_ACCESS_VIOLATION(0xc0000005)。
  **海森堡附记(如实)**:带全量仪器的轮次 r4/r5/r6 计 ~26 分钟
  sustained 轮询不再崩(日志开销收窄腐蚀窗口)——"崩溃瞬间最后
  序列"不可得,以"同 ptr 跨线程相邻序列"(r6 seq 12611-12613,
  ThreadId(4)/ThreadId(1) 交替 take_dirty_rows/feed_ready)+ 上列
  四联证定案。a2r 生成码排除(仅调用方);胶水侧属触发面
  (多 worker 无收敛),根修落 DLL 内(见 T-03)。20GB 内存实录
  未复现(全程 WS 13-15MB,待澄清#1 关闭:非同相)。
- **T-03 [D3] 根修**:按定位实施;复现器 3/3 通过;套件绿。
  前置 T-02。关联 AC-03。
  [✅ 已完成 2026-09-17] DLL 内每柄串行化(ffi.rs ENGINE_LOCKS
  按引擎指针取锁,全部导出含读类;Box::leak 一次性锁,表项地址
  复用,锁序 = 柄锁→内部 ring 锁单向无环)。验证:①金样
  `cargo test -p autoterm-core --test
  ffi_concurrency_serialization`(6 线程同柄 feed/take/读/resize/
  键入混合流 6s):去锁红相 = 进程 STATUS_ACCESS_VIOLATION
  (0xc0000005),带锁绿;②复现器修复后 3×≥10min 浸泡零崩
  (r7 629s/r8 626s/r9 623s,WER 无新事件);③014 面零回归
  (resize_guard 等全量门内绿)。commit auto-term 1cded12。
- **T-04 [D4] 回归+实机**:全量门 + 020 t01 复跑 + vue 浸泡;
  DEBTS/spec 文档。前置 T-03。关联 AC-04/05。
  [✅ 已完成 2026-09-17,用户目验件待补] `cargo test --workspace`
  78 过 0 挂(两次全跑一致);020 t01 十二步剧本对修复后 back
  复跑全绿(magnet/clamp/键盘/MAX_PANES/关 Pane 重算逐断言,
  evidence/021/t01-model-r21.log);014 冒烟 = resize_guard(014
  T-06)随门内绿 + t01 resize 路径;vue 浸泡 = r7-r9 即浸泡本体,
  headless 截图 vue-soak-visual.png(Tab 条五钮渲染正常;用户目验
  tab 条/布局交互留待用户,与 T-07 实点同场)。DEBTS #25 立案
  (根修+红绿+浸泡在案);SD-01 契约文本已精确化随案,merge 步
  落 docs/specs/engine-ffi-color-encoding.md。commit auto-term
  1cded12。
- **T-05 [D5] 管线取证+根修**:press/drag/wheel 一次仪器化;双区
  探针复跑;断点修复(020 移交项落位)。排查入口配方(020 f5dab16
  收据移交,rev2 复审补记):renderer.rs 21162 VM 动态臂入口 +
  aura_view_builder convert_mouse_area 入口,AUTO_MA_DBG=1 门控
  eprintln 插桩。前置:无(与线 A 并行)。关联 AC-06。
  [✅ 修复在案 2026-09-17,实点复验待用户] 根因 = iced 0.14
  mouse_area layout 直通子件 + 事件面 `!cursor.is_over(自身
  bounds)` 早退:尺寸类挂外层 build_container、空内容 → 自身
  0×0 bounds → press/hover 全死而渲染正常。证据链:①iced
  0.14.2 mouse_area.rs 源码读码;②[MA_BUILD] 探针 press=true
  w=Fixed(300/10)(接线本无恙);③iced_test 无头回归钉×2 红绿
  (修复前块中心点击 FAILED,修复后 PASS+条外静默,layout_tests
  ma_press_*)。修复 = auto-lang 双臂(renderer.rs VM 动态臂 +
  IntoIcedElement 臂)内容侧镜像宽高透明容器,命中区=可视区,
  外层树形不动;commit plan-021-dev 6c6950f75。实窗合成输入三路
  (SetCursorPos+mouse_event/PostMessage/SetForegroundWindow 被
  拒均取证在案)不可达 winit——020"Button 正常"系用户实点,实点
  复验(计数 0→非 0)留待用户,与 T-07 三联合场景同场。
  [✅ 用户实点复验通过 2026-09-17,AC-06 关案] probe021 实点:
  hitsBig=1/hitsStrip=1(双区 0→非 0),仪器日志 HitBig×4/
  HitStrip×3 到达 update 派发层(probe-user-run.log)。hitsBtn=0
  = 对照件笔误:button 事件词汇仅 onclick/click(aura_view_
  builder convert_button :106-109),不认 onmousedown——按压实达
  (8 条 widget="" event="click" 通用消息),绑定未成立;Button
  链路活着(020 生产窗证据不变)。wheel 子路径:TERM_WHEEL 仪器
  在库,实点读数随 T-07 滚动场景。
- **T-06 [D6] 官方 scroll 组件**:各 panel 统一接入+按需修组件;
  对照实验结论在案。前置 T-05。关联 AC-07。
  [✅ 范围裁定结案 2026-09-17,用户指令] 裁定:**维持现状半成品**
  (终端自绘滚动条与现有 panel 形态不动);"接入官方 scroll 组件"
  延后,顺序改为 ①先强化官方滚动条(支持虚拟滚动;auto-lang 独立
  工作项,顺带收敛 4579d59e3 暂缓的虚拟滚动容器问题清单)②之后
  再回来换装。AC-07 原文(各 panel 接入官方组件)随裁定改判为
  "维持现状 + 链修通对照证据在案"(无头回归钉×2 + 双区实点非 0
  即链修通判据);换装+虚拟滚动强化 = 新计划项移交(§10 #3)。
  勘定事实随案关闭。
- **T-07 [D7] 命中区+split 复验**:归属规则定稿实现(SD-02);三
  联合场景;020 AC-04 关案。前置 T-06(已裁定结案,解除)。关联 AC-08。
  [✅ 实测完成 + 裁定收口 2026-09-18,见 t07-field-notes.md] 载体
  = T-11 rust 轨 app。实测:①thumb 拖拽基本可拖(021 事件链修复
  实机证实);②滚轮文字可滚(链活);③自绘滚动条度量/绘制缺陷
  (thumb 太小/未对齐右缘/比例不对无法到最早/不随滚轮跟随);④
  rust 轨分屏:右面板未渲染+分隔条未出(分屏渲染缺陷)。用户裁定:
  自绘滚动条与分屏打磨**就此打住**,移交官方滚动条(虚拟滚动)计划
  (与 T-06 裁定同向);SD-02 命中区归属规则随该计划定稿;020 AC-04
  携带项改判移交同一目标。021 线 B 交付物维持"事件链修复"(已证)。
- **T-08 锚定与收口**:双仓 SHA 回填(线 B 必动 auto-lang)+
  status execution_done。前置 T-04 + T-07。
  [✅ 完成 2026-09-18] 锚定:auto-term main c7f40d5..(工作线)+ 
  lang-021 plan-021-dev 6c6950f75(线B 管线修复)+ lang-021-p2
  plan-021-p2-dev(T-09 codegen 根修,SHA 见该 worktree HEAD)+
  组内 auto-down 兄弟检出(detached)。明细见 §9 交棒记录。

**Phase 2(auto-lang 跑法缺陷簇,rev3 扩充;工作在 lang-021-p2
worktree,branch plan-021-p2-dev,基线 master 75fb01808)**

- **T-09 [Phase2] rust 轨 codegen 整备(范围升级,排障实证) [✅ 根修完成 2026-09-18]**:
  从零生成对任一时代运行时均 253 错——非回归而是**从未可编译**:
  14:48 的 exe 依赖"化石 main.rs"(生成器 skip-if-exists 保留古老
  产物,021 排障全清后暴露)。整备面:①merged 生成器是 CRUD 原型
  (GET 全件/find-by-id),与 auto-term 声明式 api 契约(int/[]str)
  语义不符;②split 生成器强 Value 化返回(Plan 388 起),调用点按
  .at 声明类型消费;③正确形态 = rust 轨 wrapper 直调进程内吸收的
  db 逻辑(017 merged 承诺)。规模 = codegen 子系统整备,非点修。
  前置:无。关联 T-11。证据:bisect-codegen.log + 排障记录。
  根修(plan-021-p2-dev,rust_ui.rs):①generate_api_client 优先
  db.at 进程内吸收(merged_db_impl 类型化直调,017 承诺),吸收
  不可用才回落 split-HTTP;②split 返回类型按 api.at 声明类型化
  (auto_type_to_rust),未知类型回落占位;③POST 标量返回 local_
  result 占位 → 阻塞反序列化;④GET Option 声明类型直反序列化。
  验证:rust-workspace 全清从零生成 → cargo build = 0 错;实机
  shell 横幅/提示符/●shell 1 全活(t07-rust-vehicle.png)。
- **T-10 [Phase2] dev 跑法 VM api 委托断供定因+根修**:
  `auto run -r vm` api.* 静默失效(证据 vm-delegation-break.log
  #5)。定因 VM api 派发断点;根修 + app 实机验证。
  前置:无(与 T-09 并行)。
  **(rev3 移交)**:独立 auto-lang 缺陷,不阻塞任何 021 交付物
  (019/020 实机会话所用部署态跑法不受影响)——移交 auto-lang
  侧后续缺陷计划,与 codegen 整备同批。
- **T-11 [Phase2] 部署态重建**:rust 轨 auto-term.exe 以含
  021 修复的运行时构建 + 修复版 DLL 同布;shell/Tab/内容三查。
  前置 T-09。关联 T-07 载体。
  [✅ 完成 2026-09-18] auto-term.exe(lang-021-p2 运行时)+
  修复版 autoterm_core.dll 同目录;三查全过(t07-rust-vehicle.png)。
  T-07 实点载体就绪。

## 9. 复审记录

- 2026-09-18 stage:merge · PLAN-021:r3 · checkpoint:**ledger_refreshed** ·
  .autoos/specs.json 六节 P021-1..6 入账(reports/goals/designs/tests/
  reviews → 归档路径;architecture → docs/specs/engine-ffi-color-encoding.md
  SD-01 权威源);round-trit 回读校验 OK(6 items 计数+file 链);
  投影脚本 scripts/repro/ledger_p021_projection.py 入库 · next:archive。
- 2026-09-18 stage:work(阶段交棒)· PLAN-021 · rev2 ·- 2026-09-18 stage:merge · PLAN-021:r3 · checkpoint:**landed** ·
  auto-lang master 折回:plan-021-dev → fc8c9561b(renderer.rs
  +59/widget.rs +15/layout_tests)、plan-021-p2-dev → 7132bb136
  (rust_ui.rs +59/-14);零冲突,另一会话 WIP(643)未受扰 ·
  merge 门:tf 全量在基线即编译失败(E0433,019 已在案谱系,
  plan024 文件折叠区间零改动)→ 改跑 cargo t --no-fail-fast
  5073 测:折叠后 41 红 vs 折叠前 2de64ba08 同 41 红(红名逐一
  相同,含两个谱系自带新红 conditional_style/palette_has_no_drift
  在 2de64ba08 同败)——**零新增红,门过**;auto-term 侧
  workspace 78/0 复跑 ✓ · next:ledger→archive→cleaned。
- 2026-09-18 stage:work(阶段交棒)· PLAN-021 · rev2 ·- 2026-09-18 stage:merge · PLAN-021:r3 · checkpoint:**prepared** ·
  复审基线 auto-term 526ff51(rev3 pass)+ lang-021 6c6950f75 +
  lang-021-p2 ef5e0538c · canonical diff:SD-01 →
  docs/specs/engine-ffi-color-encoding.md §并发契约(已写入,提交
  见 git log;SD-02 随官方滚动条计划移出)· 投影目标:specs.json
  P021 项(landed 后刷新)· 交付 commit:auto-term main(spec 提交
  即 landed 前置)+ auto-lang 两分支折回 master · next:fold→tf 门
  →landed→ledger→archive→cleaned。
- 2026-09-18 stage:review · PLAN-021 · rev3 · outcome:**pass** ·- 2026-09-18 stage:review · PLAN-021 · rev3 · outcome:**pass** ·
  reviewed_commit:auto-term main 526ff51(+ 复审轮测试修正
  lang-021-p2 ef5e0538c)· base_commit:auto-term e9f813d ·
  dependency_revisions:lang-021 plan-021-dev 6c6950f75(base
  75fb01808)/ lang-021-p2 plan-021-p2-dev ca1d1260e+ef5e0538c
  (base 776fe8f6c)/ auto-down 兄弟检出 detached · spec_inputs:
  docs/specs/engine-ffi-color-encoding.md(SD-01 目标,存在)、
  docs/specs/terminal-mux-model.md、docs/specs/terminal-widget-chrome.md ·
  声明:实现会话内复审,结论自工件重建(测试实跑 + 日志/截图在案
  复核),未采信执行者摘要。
  - **acceptance_results**(AC→T→证据):
    - AC-01 pass:复现器 3/3 崩(65/65/140s,WER repro-r{1,2,3}-wer.txt;
      复用理由=修复后运行时不可能复现修复前崩溃,基线证据不可变)。
    - AC-02 pass:仪器定因链(OVERLAP 1062-1213 次/300s、同 ptr 跨线程
      feed_ready、静态 UB 读码)——vm-delegation-break.log + repro-r4/
      r6 日志(已入库)复核。
    - AC-03 pass:**本轮实跑** 金样 ffi_concurrency_serialization 过
      (workspace 78/0 内含)+ 浸泡 r7-r9 日志复用(修复后无崩溃不可
      重演,同 AC-01 理由)。
    - AC-04 pass:**本轮实跑** cargo test --workspace 78/0;020 t01
      十二步剧本日志(t01-model-r21.log)复用(t01 后 back 仅加 spawn
      失败留痕,成功路径零行为变化);014 冒烟 = resize_guard 门内绿。
    - AC-05 pass:DEBTS #25 在 DEBTS.md 复核 ✓;SD-01 契约文本随案
      (§5)与实现一致(每柄串行化);canonical spec 未发布(merge 步)
      合规。
    - AC-06 pass:**本轮实跑** ma_press 无头回归 2/2 + 用户实点
      (probe-user-run.log HitBig×4/HitStrip×3)+ wheel 实测如实记录
      (t07-field-notes.md #4)。
    - AC-07 pass-per-ruling:裁定链在案(§8 T-06),交付 = 链修复实证。
    - AC-08 pass-per-ruling:实测 + 缺陷清单移交(t07-field-notes.md),
      SD-02/020 AC-04 随官方滚动条计划。
    - T-09 pass:**复审轮从零构建 0 错** + 契约锁定测试新增(标量 POST
      阻塞反序列化);T-10 移交在案;T-11 pass(载体实测)。
  - **findings**:
    - **F1(已修,ef5e0538c)**:过期断言 test_w1_get_body(断言旧
      Option→Value 形状)——随 T-09 契约更新(类型化 + flatten 断言)
      + 新增标量 POST 契约锁定测试。
    - **F2(环境基线,无行动)**:vue.rs 三测试轮转失败(plan609/
      desktop_extra_app_roots/incremental_parse——本机缺 auto-os
      mirror 文件 + fs 时序 flaky;无修复代码时同败;与 rust_ui.rs
      改动无涉)。归 auto-lang 环境整备,非 021 项。
    - **F3(已修)**:计划引用的 bisect-codegen.log 当时未持久化——
      已补判据输出摘录入 vm-delegation-break.log(证据持久化要求)。
  - **SD-01 delta 复核**:语义 = FFI 边界每柄串行化(与实现逐条对
    应:ENGINE_LOCKS 含读类、Box::leak 单次锁、锁序柄锁→ring 单向);
    目标路径存在;描述当前行为非执行日记 ✓;frontmatter
    new_spec_components 已定稿。SD-02 随官方滚动条计划移出本计划
    (§8 T-07 裁定在案)。冻结件 = 本计划 §5 SD-01 行(计划文件
    已提交,哈希随仓)。
  - evidence:evidence/021/(repro-r*/wer/mem、probe-user-run.log、
    t01-model-r21.log、t07-field-notes.md、t07-rust-vehicle.png、
    vm-delegation-break.log)、workspace 78/0 本轮实跑记录(§上)、
    ma_press 2/2 本轮实跑。· **next**:merge(auto-plan-merge;
    merge 门含 auto-lang 侧全量 tf——本轮以受影响 crate 测试 + 载体
    实测代替,full tf 留 merge 前置门)。
- 2026-09-18 stage:work(阶段交棒)· PLAN-021 · rev2 ·- 2026-09-18 stage:work(终态交棒)· PLAN-021 · rev3 ·
  outcome:**pass**(裁定改判后全范围交付;移交项显式在案)·
  code_commit:auto-term main(T-01..T-04 工作线 1cded12 起 + T-07/
  T-08 记录线)/ auto-lang plan-021-dev 6c6950f75(线B 管线修复)/
  auto-lang plan-021-p2-dev(T-09 codegen 根修,见该分支)·
  worktree:lang-021 + lang-021-p2(均待 merge)·
  task_ids:T-01..T-06、T-08、T-09、T-11 完成(证据见 §8 逐条),
  T-07 实测+裁定收口,T-10 移交 ·
  **交付面**:①线 A(FFI sustained 堆破坏):复现器 3/3→仪器定因
  (DLL 导出面裸 &mut 别名)→每柄串行化根修(金样红 0xc0000005/
  绿 + 3×≥10min 浸泡零崩)→全量门 78/0 + 020 t01 十二步绿 +
  DEBTS #25;AC-01..05 达成。②线 B(指针事件链):iced mouse_area
  0×0 bounds 根因 + 双臂修复 + 无头回归钉红绿 + 用户实点关案
  (probe021 双区 0→非 0);AC-06 达成。③线 B 消费面(AC-07/
  AC-08):两轮用户裁定改判移交——官方滚动条(虚拟滚动)计划承接
  自绘滚动条整备 + SD-02 + 020 AC-04;021 交付链修复实证。④
  Phase 2(T-09/T-11):rust 轨 codegen 根修(进程内吸收优先 +
  类型化返回 + 阻塞标量 POST),从零构建 0 错,部署态载体实测
  shell/Tab/内容全活;T-10 移交 auto-lang 缺陷计划。·
  **移交清单(后续计划输入)**:官方滚动条(虚拟滚动)计划 = 自绘
  滚动条缺陷簇(thumb 尺寸/对齐/比例/跟随)+ wheel 半屏钳位 +
  rust 轨分屏渲染 + SD-02 + 020 AC-04 + T-06 换装;auto-lang 缺陷
  计划 = T-10 dev 跑法 api 委托断供 + codegen 整备未尽面(merged
  CRUD 原型退役)。· blockers:无 · next:review(auto-plan-review)。
- 2026-09-18 stage:work(阶段交棒)· PLAN-021 · rev2 ·- 2026-09-17 stage:work(阶段交棒)· PLAN-021 · rev2 ·
  outcome:**pass(线A 全闭环)+ blocked(线B 消费面待用户输入)** ·
  code_commit: auto-term main 1cded12+c7f40d5 / auto-lang
  plan-021-dev 6c6950f75 · base: auto-term e9f813d / auto-lang
  master 75fb01808 · worktree: lang-021(wt-guard clean)+ 组内
  auto-down(detached 3a05255)· task_ids: T-01..T-04 ✅(证据见
  §8 逐条),T-05 修复+无头回归钉在案(AC-06 余"用户实点"一件),
  T-06/T-07 未开工 ·
  **线 A(FFI 堆破坏)AC-01..05 全达成**:复现器 3/3 崩→根因定案
  (DLL 导出面裸 &mut 别名,腐蚀对仪器在案)→每柄串行化根修(金样
  红 0xc0000005/绿 + 3×≥10min 浸泡零崩)→全量门 78/0 + 020 t01
  十二步复跑绿 + DEBTS #25 + SD-01 精确化文本随案。SD-01 spec
  目标路径定稿 = docs/specs/engine-ffi-color-encoding.md(merge 步
  入库)。待澄清#1(20GB 实录)关闭:全程 WS 13-15MB 非同相。
  **线 B(管线)AC-06 差一件**:根因证明(iced mouse_area 0×0
  bounds)+ 双臂修复 + iced_test 无头回归钉红绿均已入库;实窗
  合成输入三路不可达 winit(SetForegroundWindow 被拒等,取证在
  案),"双区探针计数 0→非 0"须用户实点复验。
  **blockers**:
  1. T-05 收口/T-07 三联合场景:需用户实点(合成输入不可达,
     scripts/repro/probe_ma.ps1 + spikes/021-ma-probe 已备好,开
     探针后用户点三区读屏上计数即可);
  2. T-06 范围勘定(待澄清#3):app.at 前端现无任何 scroll 消费面
     (778 行全量 grep 零命中),官方组件在库(View::scrollable,
     examples/ui_scroll.rs)——"哪些面板要滚动/终端自绘滚动条是否
     换官方组件"是用户范围决策,勘定事实已录 §10;
  3. SD-02(命中区归属规则)依赖 T-06 勘定后联合设计,同批。
  next:用户指令(实点复验 + T-06 范围裁定后进 review 前收口)。
- 2026-09-17 stage:new · PLAN-021 · rev1 · outcome:**pass** ·
  授权:用户 2026-09-17 裁定(020 复审 F1 三选之选项 1)·
  设计依据:020 复审隔离实验矩阵 + 崩溃实录 ×2 + DEBTS #21/#24
  家族事实 · 核心风险:复现不稳定(已 2/2,基线可固化)/根因在
  DLL 深处(备选:胶水侧收敛也可达成稳定)· next:work。
- 2026-09-17 rev2:范围扩充二(用户指令,见 §0)——线 B 入案
  (D5-D7/T-05..07/AC-06..08/SD-02,收口改 T-08);rev1 复审结论
  仅覆盖线 A,扩充部分待复审后再进 work。
- 2026-09-17 stage:review(rev2 设计稿复审)· PLAN-021 · rev2 ·
  outcome:**pass**(带 F1-F3 补记/在案,均 P3 不阻 work)·
  reviewed_commit c3b8d50 · base_commit c422796(rev1 内容源:
  914fdb5 立项 + 2a6e148 020 移交扩充)· dependency_revisions:
  auto-lang master 4579d59e3 · spec_inputs:
  docs/specs/terminal-mux-model.md(020 a55b454 落库版)、
  docs/specs/engine-ffi-color-encoding.md(SD-01 语义目标,已核实
  存在)、docs/specs/terminal-widget-chrome.md(无滚动条命中面
  记载)· 声明:实现会话内设计稿复审(线 B 无实现可验,验计划面
  +代码事实),结论由读码与工件独立重建。
  - **复审面与结果**:①代码事实核真——widget.rs:513-590(press 起
    ScrollbarDrag→Moved→Released)/:606(WheelScrolled)/:1029
    (SCROLLBAR_HIT_W=14.0)✓;AutoUI 官方 scroll 组件在库
    (examples/ui_scroll.rs,Component/View 抽象 iced/gpui 双后端)
    ✓;020 探针事实(press 2/2 计数 0/渲染臂正常/Button 对照正常)
    与归档计划一致 ✓。②AC→T→D 映射完备无环(T-05→06→07;
    T-08=T-04+T-07;AC-06..08 验证手段:探针复跑/实机操作/三联合
    场景,均可在案复现)✓。③授权链:§4.1 两条用户记录 ✓。
  - **佐证**:auto-lang 4579d59e3 用户已裁定虚拟滚动容器过渡形态
    问题清单(拇指比例/拖拽位置漂移/拖拽表现/hover 加宽/pointer
    联动/theme)暂缓、待官方组件统一解决——与线 B T-06 方向互为
    印证,官方组件接入即收敛该批已知问题。
  - **findings**:
    - **F1(P3,记账一致性,已补记)**:020 f5dab16 cleaned 收据
      声称"重加配方已录 PLAN-021 排查清单",但 021 在案文本原本
      无此配方——本轮补记进 T-05(renderer.rs 21162 VM 动态臂 +
      aura_view_builder convert_mouse_area 入口,AUTO_MA_DBG=1
      门控)。(020 F3 同款处置。)
    - **F2(P3,frontmatter,书面说明在案)**:supersedes/new_spec_
      components 空而 SD-01/SD-02 在案——两 delta 精确 spec 路径
      分别待 T-02/T-07 定稿回填,届时同步补记。
    - **F3(P3,spec 漂移→SD-02 定稿输入)**:terminal-mux-model.md
      §5 分隔条契约记 mouse-area **6px** 厚,020 rev2 实现
      (2a6e148)已加粗 **8px**——spec 与代码漂移;T-07 定稿命中区
      归属规则时以代码为准对齐,并定 SD-02 主落点
      (terminal-mux-model.md 或 terminal-widget-chrome.md 择一,
      另侧引用);terminal-widget-chrome.md 现无滚动条命中面记载。
  - **evidence**:本计划读码记录(§0/§4.3)、020 归档计划与收据
    (docs/plans/archived/020-nested-split-divider-drag.md)、
    spikes/020-split-probe(探针)、auto-lang examples/ui_scroll.rs
    · **next**:work(待用户指令;T-05 无前置可与线 A 并行即开,
    status 维持 drafting 至 work 指令)。

## 10. 待澄清事项

1. ~~20GB 内存实录(020 T-01 联调)与本案是否同根因~~ **已关闭
   (T-02)**:本案全程 WS 13-15MB 无膨胀,非同相;20GB 归 #23
   投影膨胀家族(020 已根修)。
2. ~~auto-lang 是否入役~~ **已关闭(T-02 定案)**:根因在 DLL 导出
   面,非 a2r 生成码——auto-lang 线 B 因 mouse_area 缺陷独立入役
   (worktree lang-021),与线 A 无涉。
3. ~~T-06 滚动清单~~ **已由用户裁定关闭(2026-09-17)**:维持现状
   半成品;官方滚动条先做虚拟滚动强化(auto-lang 独立工作项,移交
   后续计划;顺带收敛 4579d59e3 暂缓的虚拟滚动容器问题清单:拇指
   比例/拖拽位置漂移/拖拽表现/hover 加宽/pointer 联动/theme);
   强化完成后再回来换装官方滚动条并重开命中区联合设计。
4. **VM 轨 api 委托断供(2026-09-18 新缺陷,阻塞 T-07 全场景)**:
   app VM 形态 UI 渲染正常但 api.* 委托零到达引擎(DLL 仅 boot
   spawn_ex 一次即永寂;面板空、Tab/分屏全失效)。A/B 双 exe
   (worktree 6c6950f75 / 主检出 75fb01808)同症 → 非 021 回归;
   020 时点(4579d59e3)VM 实机正常 → 断点窗口 = auto-lang
   4579d59e3..75fb01808 共 37 commits(637 fold 一带)。vue 轨不受
   影响。证据:evidence/021/vm-delegation-break.log +
   app-{user,main}-run.log。处置待用户裁定:(a) 021 会话内 bisect
   定因 + fix worktree 根修;(b) 移交 637/639 归属方;(c) T-07 挂起
   等修。
