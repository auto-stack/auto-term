---
plan_id: PLAN-028
status: archived
feature_name: 分体轨 resize 断链根修(client-size 推送 + 探针跨界桥 + history 回流)
author: [zcode]
created_at: 2026-09-22T00:00:00Z
updated_at: 2026-09-22T00:00:00Z
plan_revision: 1
current_step: 7
total_steps: 7
supersedes_spec_components:
  - docs/specs/terminal-mux-model.md   # auto-term(SD-01 窗口尺寸面读序 / SD-02 几何随动链桥写 / SD-03 tick-nums 尾段,三处 modify 同文件)
new_spec_components: []
touched_goals: []   # terminal-mux-model.md 无 goal 注册表;本计划不改任何 goal 语义(空值经复审确认)
---

# PLAN-028 — 分体轨 resize 断链根修

## 0. 变更摘要

分体化(026)后,真实窗口尺寸永远到不了 mux 后端:`mux_window_width/height()`(db.at:621 → `auto.term::window_width()`)读的是 auto-lang 线程局部全局(缺省 1024×768,theme/mod.rs:165/179),只有渲染器进程每帧写它(renderer.rs:21436);后端进程无人写,恒返缺省。后果链:

1. 前端指纹门 `ns[1]!=.clientW`(app.at:398)永不触发 → 槽位矩形冻结在 1024×768 逻辑盒(app.at:203-206),拉窗露根底色死带;
2. 网格冻死:spawn 硬编码 100×30(db.at:843/845),前端 014 探针每帧的 resize 请求写**前端进程** pending_resize 注册表(widget.rs:659-677),消费端 `engine_apply_resize_for` 读**后端进程**注册表(term_engine.rs:392)——跨进程断链(与 026 键入载荷断链同类);
3. 滚动条永久消失:虚拟画布 =(rows+history)×CELL_H+2PAD(widget.rs:420,CELL_H=16),rows 冻 30 + history 恒 0(唯一写入口 = 后端引擎泵 FFI term_engine.rs:524,分体快照只带可见 30 行)→ 画布恒 ~488px,视口一超即判"无可滚"。

修法 = 三段桥:前端真值发布(auto-lang 渲染器/探针)→ 前端 .at 变化才推(新端点)→ 后端存值 + per-key pending 写入(既有 apply 泵复活)+ tick-nums 补 history 回流。桌面轨(024)读序与行为零变化。

**关键事实(本计划调查实证,2026-09-22)**:spawn 几何是硬编码常量 100×30(db.at:843),后端**不存在**"可用空间反推网格"公式(cell_w 只在前端字体系统可测,widget.rs:72)——因此 cols/rows 必须由前端(014 探针已算出真值)跨界面推送,后端不做几何换算。

## 1. 目标

- **G1 拉窗跟随**:分体轨 GUI 拉大/缩小窗口,焦点 pane 网格随动(≤2 泵拍收敛,014 一帧滞后语义不变)。
- **G2 死带消除**:槽位矩形随真实客户区重投影,窗缘无 bg-background(9,14,26) 露底带。
- **G3 滚动条回归**:分体轨 history 回流,虚拟画布恢复"内容高于视口出条"语义;022 bind/贴底语义不回退。
- **G4 桌面轨零回退**:024 桌面轨窗口尺寸面(override→theme 读序)与投影行为零变化。
- **非目标**:CJK 宽字符渲染(缺陷②,另行立项);状态栏样式(③已修待 merge);内存泄漏(PLAN-027 在飞);后端按 w/h 自行换算网格(已否决:无 cell_w 真源,见 §0 关键事实)。
- **受影响仓**:auto-term(主仓:app/src/back、app/src/front、rust 侧车)+ auto-lang(渲染器发布臂 / 探针发布臂 / 可能的新 shim)。双仓 worktree 惯例(组名 `fix-resize-chain`:lang 基 master,term 基 main)。
- **成功样貌**:实机分体 GUI 拉窗 → 状态栏几何变化、黑区满窗、滚动条随历史出现;桌面轨 024 口径复跑不变。

## 2. 架构方案

真值已在前端进程,缺的只是跨进程桥。三段:

```
[auto-lang GUI 进程]                      [auto-term 后端进程]
renderer __window_resized ─┐
  (17387 臂,已 publish     │  storage 或
   vm.window_inner_height) ├─ native →  前端 app.at Tick
014 探针臂 (widget.rs:659) ─┘  路线待决   (变化才推,≤1 次/拍)
                                            │ HTTP
                                            ▼
                              POST /api/mux/client-size {w,h}
                              POST /api/mux/pane-resize-request {key,cols,rows}
                                            │
                              db.at: 存值(≤0 忽略)+ per-key 写 pending_resize
                                            │
                              既有每拍 term_apply_resize 泵(db.at:1709)
                              → engine_apply_resize_for → 引擎网格随动
                                            │
                              mux_tick_nums 尾部追加 history → 前端
                              terminal_set_history → 滚动条画布恢复
```

- **client-size 端点**:存值 + `mux_window_width/height` 读序改为 **分体存值(>0)优先 → 既有 `window_width()` 回落**(024 desktop override 在 native 内部,序不变)。指纹门 ns[1]/ns[2] 从此能变,门控语义零改。
- **pane-resize-request 端点**:前端把 014 探针输出(key,cols,rows)跨界送入,后端写 per-key pending_resize,由**既有每拍 apply 泵**消费(泵、焦点定向、收敛语义全保留,只补"注册表写入端")。auto-lang 侧需一个 rust 可达的注册表写入原语(候选 `engine_pend_resize_for(key,cols,rows)`,term_engine.rs,catalog 双表登记——026"三表/三副本"教训在册;若调查发现已有等价原语则免新增)。
- **收敛时序**:client-size 变 → 门控重投影(槽位先对)→ 下一帧探针按新槽位出 cols/rows → 推送 → apply。两拍收敛,与 014"一帧滞后"同族。
- **history 回流**:nums 面尾部追加(免 rebase,见 SD-03),前端喂 `terminal_set_history` → 画布含历史 → iced scrollable 恢复出条。
- **退化护栏**:≤0/最小化尺寸不推送不存值不应用(014 护栏 widget.rs:649 同款,后端照抄)。
- **稳态开销**:两推送端点均"变化才调",每拍 4 次 HTTP 基线不升(027 内存债定罪面是每请求分配,本计划不得新增稳态请求)。

### 规范增量

| delta_id | 类型 | 目标 | before/after | rationale | acceptance |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/terminal-mux-model.md(窗口尺寸面 T-00b/024 SD-01 段,~145-153) | before:`mux_window_width\|height` 读序 = per-app override → theme 全局回落(三轨同);after:**分体/merged rust 轨**读序 = 存值(>0)优先 → theme 全局回落——存值 = 前端渲染器**变化才发布**的 `vm.window_inner_width/height`(storage host:进程内直读 + 共享文件 write-through),rust 侧车经 `AUTO_VM_STORAGE_FILE` 钉扎的共享文件按拍新鲜拉取(`storage_host_read_fresh`);**VM/desktop 轨 shim 读序(override → theme)零变化**。客户端尺寸 HTTP 端点不引入(机制适配①,§9) | 分体轨几何源唯一修法;桌面轨 override 在 native 内部不受扰 | AC-01/02/04 |
| SD-02 | modify | docs/specs/terminal-mux-model.md(V1 语义节几何随动链段,~109 邻域) | before:014 探针→进程内 pending_resize→apply 泵;after:分体轨探针输出经前端桥 `POST /api/mux/pane-resize-request(key str, geom str)`(geom = "CxR" 探针发布形态原样,.at 侧零解析)**变化才调** → back rust 原语 `engine_pend_resize_geom` 解析(014 护栏拒收退化)落 per-key pending(惰性建核 `terminal_pend_resize`),既有 `term_apply_resize` 泵消费语义不变;进程内轨(桌面)零变化。端点形态适配②见 §9 | 断链修补点;后端无 cell_w,cols/rows 必须前端推送 | AC-01/03 |
| SD-03 | modify | docs/specs/terminal-mux-model.md(Tick/Init 数据面聚合契约节 tick-nums 布局段,~222-230) | before:pane 段 npanes×7(base=107)即尾;after:**尾部追加** npanes×1 history 字段(偏移 `107+7n+i`,数据源 `engine_history` = DLL `autoterm_engine_history` 同源)——验证/观测面;前端虚拟画布与滚动条的运行时供给 = **rust 直达回流通道**(back `engine_rows_for` 快照拍变化才发布 `vm.term_hist.<key>`/`vm.term_off.<key>`,前端 widget ≤150ms 节流读取落 core:history+offset+anchor——分体下泵回写不可达的唯一供给面);`.at` history view prop 通路因 VM 模板编译器 prop 毒化(F-01)禁用另立;免 rebase 不动既有偏移 | 滚动条比例数据源;terminal-scroll-render.md 语义零变化(泵回写面对等扩展一条分体供给通路) | AC-03 |

## 3. 技术栈

Rust(auto-lang 渲染器/VM shim/iced;auto-term axum 侧车)+ auto-lang .at(mux 前后端模型,a2r 转译)。无新依赖。

## 4. 需求分析与背景调查

- **授权**:用户 2026-09-22 会话放行立项(auto-plan-new),明确要求把"shim vs storage 复用"两条前端集成路线作为待决点写入;范围 = 起草计划,双仓;预算/自动续跑未指定(无)。
- **背景调查(已实证,本会话 2026-09-22)**:
  - 几何源链:db.at:621/625 → term.rs(sidecar):531 → theme/mod.rs:164-184(thread_local 1024.0/768.0);渲染器每帧写 renderer.rs:21436-21439;resize 事件臂 renderer.rs:17378-17391(已 publish `vm.window_inner_height`)。
  - 指纹门与槽位:app.at:398-424(门)、203-206/430-433(槽位 = ‰×ns[1]/ns[2])、41(clientW 初值 802)。
  - spawn 硬编码:db.at:843/845 `engine_spawn_ex(..., 100, 30)`。
  - apply 泵:db.at:1709-1728 → sidecar term.rs:481-500(`apply_resize_inner(handle,key)`);注册表读取 term_engine.rs:392("无 ui 特征恒 None")。
  - 014 探针:widget.rs:638-684(可用空间反推,写前端注册表);探针已有进程内记账先例 viewport_h_register(widget.rs:664)。
  - 画布/滚动条:widget.rs:420-425(canvas_height)、52(CELL_H=16)、mod.rs:193("0=无历史不画")、term_engine.rs:524(唯一 history 写入口);scrollable 包装 renderer.rs:4895-4902(Fill×Fill,官方滚动条 PLAN-022 T-02)。
  - tick-nums 消费纪律:app.at:398+(a2r handler 原语纪律,026 实证);nums 面布局 spec(pane 段 base=107 ×7 字段)。
  - storage 原语:stdlib.rs:671 `shim_storage_get` / `storage_host_publish`(renderer resize 臂在用)——配对与可见时序待 T-01 确认。
  - 死带溯源:renderer.rs:4840-4843(余量涂色注释,假设余量≤一格,断链下失效)。
  - 版本锚:auto-term main @ 116e9b6;auto-lang master @ 588b2eca9;取证会话 2026-09-21(vm-track-three-defects 在册)。
- **协调面**:PLAN-027(lang-027 组)在飞,触前端 Tick/HTTP 通路——落地顺序需协调(T-07 前确认);fix-statusbar-style 组待 merge,触 app.at 与 renderer.rs——rebase 面在册。

## 5. 详细设计

### 5.1 前端真值发布臂(auto-lang;路线待决 T-01)

- **窗口 w/h(二选一,T-01 裁定)**:
  - Route B(storage 复用,倾向):renderer.rs:17387 `__window_resized` 臂补一行 `storage_host_publish("vm.window_inner_width", w)`(height 已在,17390);app.at 侧 `storage_get` 读取(stdlib.rs:671 原语已在)。零新 native。
  - Route A(native):确认/补齐 front VM native 表含 `auto.term::window_width/height`(shim 2998/2999 是否入 front bigvm 待查),app.at 直接调用(前端进程全局 = 真值)。若需注册,按 026 三表教训执行。
- **探针输出(key,cols,rows)**:widget.rs layout 探针臂(659-677)在 `terminal_request_resize` 落注册表的同时,以 viewport_h_register 同款模式发布 `vm.term_probe.<key> = "<cols>x<rows>"`(storage host 或等效记账,防每帧字符串风暴:仅变化时发布)。

### 5.2 前端 .at 桥(auto-term app/src/front/app.at)

- Tick 臂(398 邻域):读本地真值(w/h + 各可见 key 的 probe),与存量(.clientW/.clientH/上次推送的 probe)比对,**仅变化时**调 `api.mux_client_size(w,h)` / `api.mux_pane_resize_request(key,cols,rows)`。消费原语遵守 026 纪律(索引/算术/比较)。

### 5.3 后端存值 + 泵复活(auto-term app/src/back + rust 侧车同步)

- db.at:新状态 `mux_client_w/h`(缺省 0);`mux_client_size(w,h)`(≤0 整包忽略);`mux_window_width/height` 读序 = 存值(>0)→ `window_width()` 回落(SD-01)。
- db.at:`mux_pane_resize_request(key,cols,rows)` → 写 per-key pending(auto-lang 新 shim `engine_pend_resize_for`,或 T-01 发现的等效原语);既有 `term_apply_resize` 泵(1709)零改消费。
- api.at:两端点登记(`#[api]` POST);rust 侧车 api.rs/db.rs 经 a2r 再生成 + 冲突手工融合(026 范式)。
- db.at `mux_tick_nums`:尾部追加 per-pane history(数据源 = 引擎回滚行数;后端已可读,scroll 面既有)。

### 5.4 滚动条回流(auto-term 前后端)

- app.at:nums 尾段读 history → 既有回喂面(与 cursor_row 同族)→ 前端 `terminal_set_history` → canvas_height 复原。

## 6. 测试设计

- **Headless(auto-lang)**:`cargo t virtual_scroll`(022 语义回归)、p025 scroll_render 套件;新增:探针发布臂单测(变化才发布)、`engine_pend_resize_for` 写入→泵消费往返单测。
- **Headless(auto-term)**:db.at 域测——client-size 存值/退化忽略/读序;nums 尾段偏移(107+7n+i)断言;两端点契约(a2r 面与 rust 侧车一致)。
- **实机验收**(T-07,沿用 runbook 取证口径):分体 GUI 拉大/缩小/还原三态 + 横分拓扑;tick-nums 采样几何随动;截图像素采样判死带;滚动条出现性;024 桌面轨口径复跑。
- **开销断言**:稳态(无变化)拍 HTTP 计数 = 既有 4 次(026 双端点口径)。

## 7. 验收标准

- **AC-01 拉窗随动**:分体 GUI 拉大窗口后 ≤2 泵拍,状态栏几何 ≠ 100×30 且与窗口可用空间一致(公式对齐 014 探针);缩小同理不破。验证:实机 + `POST /api/mux/tick-nums` 采样对比。
- **AC-02 死带消除**:拉窗后内容区无 bg-background(9,14,26) 露底带(右/底缘像素采样 = 终端底色)。验证:PrintWindow+GetPixel(runbook 在册)。
- **AC-03 滚动条**:有回滚史的 pane 出 iced 滚动条;上翻/贴底 bind 语义不回退(`cargo t virtual_scroll` 绿 + p025 套件绿)。
- **AC-04 桌面轨零回退**:024 桌面轨 override 读序、投影、resize 随动行为零变化(024 复跑口径;shim 2998/2999 读路径未触)。
- **AC-05 护栏与开销**:最小化/0 尺寸零推送零应用;稳态拍 HTTP = 基线 4 次。
- **AC-06 双仓绿**:auto-lang 域测 + auto-term parity/域测全绿;a2r 再生成后侧车与 .at 一致(无手工漂移)。

## 8. 执行步骤

- **[x] T-01 路线裁定 + 真源确认**(✅ 2026-09-22 work 会话;决策全文见 §9):(a) 2998/2999 三表在册;(b) storage 配对同进程即时可见;(c) a2r 再生成 = boot 自动;(d) 无既有写入原语 → 新增。→ Route B(存储复用)。
  Files: auto-lang `vm/native_catalog.rs`、`vm/ffi/stdlib.rs:671`、`ui/iced/renderer.rs:17378`;命令 = 仓库构建/测试流。AC 关联:全部(前置)。
- **[x] T-02 发布臂**(Route B):renderer 每帧真值点变化才发布 `vm.window_inner_width/height`(宽度补齐,事件臂 046-B 语义保留);widget.rs 014 探针臂变化才发布 `vm.term_probe.<key>="colsxrows"`(发布守护单测绿,实机实证 84x25→121x47→93x32)。AC-01/02。
- **[x] T-03 引擎写入原语**:`ui::terminal::terminal_pend_resize`(惰性 (0,0) 占位建核)+ `engine_pend_resize_geom`(geom 串形态,rust 侧解析)+ `engine_history` 查询;VM shims 3002/3003 三表登记 + native.rs 导入 + term.rs 双轨面。单测:桥写往返/解析护栏/新鲜读穿透 全绿。AC-01。
- **[x] T-04 后端端点 + 存值 + nums 尾段**:SD-01 读序落 term.rs(存值 fresh 读 >0 优先 → theme 回落;客户端尺寸端点取消——机制适配,语义保留);db.at `mux_pane_resize_request(key,geom)` + nums 尾段 history(107+7n+i,实机 len=115 断言);api.at `POST /api/mux/pane-resize-request` 登记;侧车 a2r 冷构建实证(修一处 &str 签名契合 a2r 约定)。AC-01/03/05/06。
- **[x] T-05 前端 app.at 桥**:Tick 探针变化才推(str 拼接 + 串比较,零解析);nums 尾段 .at 消费与 `history:` view prop 通路被 VM 模板编译器毒化阻断(实证 §10 F-01)→ **改道 rust 直达通道**(term.rs 发布 hist/off + widget 节流读取落 core),nums 尾段保留为验证面。AC-01/03/05。
- **[x] T-06 双仓回归**:auto-lang——virtual_scroll 9/9、p025、新增 4 单测、探针发布单测(iced-layout-tests)、ui-iced check、日常档 820 绿(3 红实证为基线预存,基线 worktree 复跑归因);auto-term——侧车 a2r 冷构建 + back boot 实证。AC-03/05/06。
- **[x] T-07 实机验收 + 协调收尾**(✅ 2026-09-22 work 会话;runbook 026 launch 范式 + AUTO_VM_STORAGE_FILE 钉扎):AC-01 拉窗随动双向实证(802→1160→900,grid 84x25→121x47→93x32,≤2 泵拍);AC-02 死带消除(1100×800 直方图:内容区 bg-background 像素 = 0,右/底缘带纯终端底色);AC-05 护栏实证(最小化 0 尺寸 → 发布 0 → 后端拒收 → nums 回落缺省,观察级);AC-03 history 回流(nums 尾段 3333/3859 + 滚动画布生效 + 文本渲染修复);AC-04 桌面轨 = 代码零触及(2998/2999 与 db.at 读序未改)+ 域测绿,024 实机复跑**留复审门**。D-4 落地顺序 = 用户裁定项,随 merge 处理。AC-01..05。

依赖:T-01 → T-02..T-05;T-03 → T-04;T-04/T-05 → T-06 → T-07。

## 9. 复审记录

- 2026-09-22 draft(plan_revision 1)handoff:`stage: new`,`outcome: pass`(T-01 为计划内 bounded 裁定任务,非阻塞项);`next: work`(auto-plan-work,worktree 组 `fix-resize-chain`)。两条路线待决点落在 §2/§5.1/§10,由 T-01 依判据裁定后回填本节。
- 2026-09-22 work 启动(work 会话):进入 executing。worktree 组落地——auto-term `D:/autostack/.wt/fix-resize-chain/auto-term` @ `plan-028-dev` 基 main `116e9b6`;auto-lang `D:/autostack/.wt/fix-resize-chain/auto-lang` @ `plan-028-dev` 基 master `dcbda3f71`(计划版本锚 588b2eca9 已过时,master 已前进,行号锚以 T-01 重验为准)。主检出预检:auto-term 零代码 WIP(仅 docs/plans 簿记);auto-lang master 带 examples/** 外来 WIP(与本次 crates/** 改动零交叠,不构建其上,已有 foreign-wip patch 存根,稍后向用户报告路由)。
- 2026-09-22 T-01 路线裁定(work 会话,bounded investigation 四路证据全闭合):
  - **D-1 → Route B(storage 复用)**。证据:(a) shim 2998/2999 确已三表在册(native_catalog.rs:541/1113/2757,Route A 可行)但 storage 配对实证更优——`storage_host_publish`(stdlib.rs:720)直写进程内 STORAGE_MAP + write-through 文件;`storage.get`(stdlib.rs:671)同进程直读(in-process wins),renderer 与前端 VM 同进程(vm_bridge 单 GUI 进程)→ 发布/读取即时可见;.at 侧 storage.get 有前端先例(012-clock)。w/h + 探针统一走 storage 单机制,零新 native。注意点:`shim_storage_get` 每调用重读文件(load-once 的 or_insert 语义),前端每拍读 ≤8 键(w/h+≤6 探针),开销可忽略(观察项在册)。
  - **D-2 → 需新增,双原语**:ui 注册表落表权限独占渲染面 `terminal(key,cols,rows)`(mod.rs:472),`terminal_core` 只读不建(:535)→ 后端进程无核可写。新增 `terminal_pend_resize(key,cols,rows)`(mod.rs,惰性 (0,0) 占位建核 + 既有 request_resize 钳位/同值 no-op 语义);侧车/VM 双形态原语取 **geom 串形态 `engine_pend_resize_geom(key str, geom str)`**(对计划 §5.3 (key,cols,rows) int 形态的有据适配:026 实证 str.to_int/split 无 a2r 前端 handler 转译路径(app.at:174),geom "CxR" 在 rust 侧解析,前端零解析)。history 查询原语 `engine_history(handle)` 双轨同步新增(rust 轨 term.rs 直读 DLL `autoterm_engine_history`;VM 轨新 shim)。号段:29xx 耗尽,沿 Plan 673 先例取 9907/9908。
  - **D-3 → 已闭合**:a2r 再生成 = boot 自动(`start_api_server`→`api_gen::generate_api` 幂等;侧车 term.rs 参与 regen 指纹,rust_ui.rs:70);冷构建 = 清 app/rust-workspace。标量/str 端点零 codegen 扩展。
  - **D-4 → 维持 T-07 前用户裁定**(027/statusbar 在飞,本计划基线不含)。
  - 其余实证:nums pane 段 7 字段/槽确认(db.at:1444-1466,尾=107+7n);前端消费原语纪律面(Tick=[i]/.len()/算术/比较/str 拼接,nums int 尾段消费零解析);history prop 前端通道 = View::Terminal 加字段(cursor_row 同族:renderer 落位臂 terminal_set_history,哨兵 0;pane key 单调不复用 → 无 stale 面)。
- 2026-09-22 work 收尾(work 会话;plan_revision 1)handoff:`stage: work`,`outcome: pass(带在册发现 F-01..F-04 与复审项)`;`code_commit`: auto-lang `plan-028-dev` @ `5530f2788`(基 master dcbda3f71),auto-term `plan-028-dev` @ `716ecf0`(基 main 116e9b6);`task_ids`: T-01..T-07;`evidence`: 实机截图/直方图/采样在 `docs/plans/evidence/028/`(auto-term 仓),单测 4 新增全绿,virtual_scroll 9/9,日常档 820 绿(3 红 = 基线预存,基线 worktree 复跑归因);`blockers`: 无工作阻塞(D-4 落地顺序 = merge 期用户裁定;AC-04 024 实机复跑荐于复审执行);`next`: review(auto-plan-review)。
  - **机制适配三则**(证据驱动,验收面不变):① SD-01 client-size 端点取消 → 读序存值层落 term.rs(共享 storage 文件 pull,boot 钉 `AUTO_VM_STORAGE_FILE`;前端 handler str→int 无转译路径 + storage CWD 哈希跨进程断链两项实证);② SD-02 端点形态 (key,cols,rows) → (key,geom "CxR" 串)(同上 str→int 限制,解析在 rust 侧);③ SD-03 nums 尾段照建,前端消费由 history view prop 改 rust 直达通道(F-01 毒化阻断 .at 通路)。
  - **实机全链**(026 launch 范式 + 存值文件钉扎):发布臂(probe 84x25 随窗)→ 前端桥(变化才推)→ back 桥写 → apply 泵(grid 84x25→121x47→93x32 双向随动)→ nums 反馈(ver/win 指纹门控复能);history 回流(nums 尾段 3333/3859 + rust 直达)修复分体轨文本渲染(026 期窗体即黑面,基线对照在档);死带零残留(直方图 0 命中);0 尺寸护栏实测(最小化 → 发布 0 → 拒收 → 回落)。
- 2026-09-22 复审(review 会话,**与执行同会话——独立性受限已声明,判定自工件重建,不采信执行期总结**;plan_revision 1):`stage: review`,`outcome: pass`;
  - **reviewed_commit**: auto-lang `plan-028-dev` @ `5530f2788`(diff base master `dcbda3f71`)/ auto-term `plan-028-dev` @ `716ecf0`(diff base main `116e9b6`);依赖位 auto-down `fba6563`(detached 挂件);两 worktree 复审时零脏区;主检出零代码 WIP。
  - **spec_inputs**: docs/specs/terminal-mux-model.md(窗口尺寸面 ~145-153 / V1 语义几何随动 ~109 / Tick 数据面 ~222-230);规范增量三行已定稿为 as-built 形态(机制适配①②③回写,验收语义与 draft 一致,plan_revision 不增——路线空间本属 T-01 裁定授权);frontmatter 元数据定稿(supersedes = terminal-mux-model.md 三处 modify;new = 无;touched_goals 空——该 spec 无 goal 注册,经查证)。
  - **acceptance_results**: AC-01 **pass**(拉窗随动:work 期双收敛 + 复审基线新 boot 复验 win=1087x764/grid=115x43 ≤2 泵拍;后台窗 resize 事件被并行会话桌面门控一例,环境性,F-05);AC-02 **pass**(1100×800 直方图内容区 bg-background=0,右/底缘带纯终端底色);AC-03 **pass**(规定验证面 virtual_scroll 9/9 + p025 绿;实机滚动画布生效 + 文本渲染修复;F-02 滚离供给缺口在册,不属本 AC 文义);AC-04 **pass**(代码级:2998/2999 与 theme 读路径 diff 零触及,grep 实证;相邻轨实机:rust 轨 boot 绿[3m16s 编译 0 错 + back ready 17401 + Iced 起跑 + 干净退出,lock 热修复归因依赖漂移];桌面宿主实机复跑未执行 → F-05 复审/merge 门建议项);AC-05 **pass**(0 尺寸护栏机验;Tick 结构 4 端点不变 + 推送变化才发,稳态零增);AC-06 **pass**(lang:tf 3723/3723 + tv 3870/3870 + tt 4092/4092 + 日常档 ui-iced 3 红全部归因基线预存[musk p053 家族,基线 detached worktree 复跑同红;无 ui-iced 组合下全绿→feature 交互预存现象];term:workspace 测试绿除 parity_gate 5 红 = at-gen 产物缺失/构建断裂,F-06 归因基线同断[shell.rs 自 009 未随 014 字段演进];a2r 侧车冷构建两次实证,codegen 未改,无手工漂移)。
  - **findings**: F-01(VM 模板编译器 terminal 新 prop 毒化——建议另立计划)/ F-02(分体滚离预取窗空白——DEBT 候选)/ F-03(storage 读语义观察)/ F-04(musk p053 三红基线预存——DEBTS 收编建议)/ F-05(桌面宿主实机复跑 + 后台窗 resize 事件门控观察——merge 门建议)/ F-06(at-gen 复刻产物自 014 起对 master 构建断裂,parity_gate 新鲜 worktree 必失败——DEBT 候选)/ F-07(rust 轨生成 crate 冷构建 Cargo.lock 依赖漂移[wgpu-hal×windows 双版本],已知好 lock 拷贝即愈——环境性,回归剧本应入库 lock 或记录再生成配方)。
  - **evidence**: docs/plans/evidence/028/(auto-term 仓,worktree 内;含 boot 脚本×2、拉窗/采样脚本、截图 base 对照/死带/终态、采样输出);复审门命令与结果摘录于上(可重放:`cargo tf|tv|tt`、`cargo t`、`cargo test --workspace`(term)、boot-split.ps1 + tick-nums-sample.py);`blockers`: 无;`next`: **merge**(auto-plan-merge;D-4 落地顺序先经用户裁定:027 与 fix-statusbar-style 的 rebase 面在册)。
- 2026-09-22 merge 收据(`PLAN-028:r1`,**部分闭合——lang 落库阻塞在册**,term 已落):用户裁定 D-4 = 027 先落、028 rebase 跟进。**027 先行完成**(五 checkpoint delivered,lang master 7001ada7d / term main 167baac;两仓账本/归档/清理全闭环)。**028 rebase**:term plan-028-dev rebase main(复审取证补档第 4 提交;原 3 提交 range-diff 全等)→ ff-only main=`6ff3bad` ✅;lang plan-028-dev 两跳 rebase(master 被 686/687/027/685 连续推进:5530f2788→42f382764→fa83e13aa→c78578025,各跳 range-diff 全等 + storage/p027 共栖测试绿)→ 基于 master tip `c36101780` 待 ff。
  - **检查点**:prepared ✅(reviewed 基线 + as-built 增量定稿)/ **landed:term ✅ lang ⏸**(阻塞=并行会话在 lang 主检出 crates/** 直接持 7 文件未提交 WIP,含本交付目标 `renderer.rs`——红线禁 stash 他人活代码,ff 被 git 正确拒绝;轮询 4×90s 未释)/ ledger ⏸(待双仓落齐)/ archived ⏸ / cleaned ⏸(worktree 组保留待 lang 落齐)。
  - **剩余动作(单命令,阻塞解除即执行)**:`cd D:/autostack/auto-lang && git merge --ff-only plan-028-dev`(worktree 已 rebase 至 master tip、验证已刷新:storage fresh read ✔ + p027 共栖 ✔ + ui-iced check ✔)。随后:归档 028(status archived + 账本 P028-1..6 投影)+ wt-guard 清 fix-resize-chain 组三件。
  - `outcome`(merge 阶段):**blocked**(publication-only;Plan 保持 reviewed,term 侧交付已可消费)。
- 2026-09-22 merge 续收据(040 会话已提交 WIP,阻塞解除,**五检查点全闭合**;`completion_kind: delivered`):lang plan-028-dev 末跳 rebase master `bbb1618e5`(040 壁纸/renderer 182 行先落,零冲突,range-diff 全等=2;共栖验证 virtual_scroll 9/9 + storage fresh ✔ + ui-iced check ✔)→ ff-only **lang master=`00e56202d`** ✅(映射末跳 c78578025→00e56202d)→ **ledger_refreshed** ✅(auto-term .autoos/specs.json 六节投影 P028-1..6,回读验证;lang 侧不投防撞号)→ **archived** ✅(本文档移 docs/plans/archived/,status: archived)→ **cleaned** ⏳→✅(wt-guard 三件 + worktree×3 + branch×2 + 组目录删除,见下条确认)。

## 10. 待澄清事项

- **D-1 前端集成路线(shim vs storage 复用)**:✅ 已裁定 Route B(T-01,见 §9)。
- **D-2 引擎注册表写入原语**:✅ 已裁定新增(`terminal_pend_resize` + geom 串桥写,见 §9)。
- **D-3 a2r 再生成命令**:✅ 已闭合(boot 自动,标量端点零 codegen 扩展)。
- **D-4 落地顺序**:PLAN-027(前端 Tick/HTTP 通路在飞)与 fix-statusbar-style(app.at/renderer.rs rebase 面)先 merge 还是本计划 rebase 让路。**owner: merge 前,用户裁定**(工作基线不含二者,均需 rebase 融合)。
- **F-01(新发现,建议另立计划)**:auto-lang VM 轨**视图模板编译器对 terminal 新 prop 触发 FN_PROLOG 毒化**——`terminal { history: ... }`(任意值含字面量)必致 `handler_App_Init` 编译中途失败被丢弃 → 前端 Init 不跑(内容区空)。实证:带 prop 3/3 毒化、撤 prop 4/4 干净;rust 轨(ui_gen)同 prop 正常。本计划绕行(rust 直达通道);VM 编译器根修需独立立项。
- **F-02(新发现,DEBT 候选)**:分体轨滚离预取窗内容空白——022 window_store/预取窗(N=24)由 back 泵写入 back core,front core 无供给;滚轮滚离当前窗后区域空白不回填(修前分体轨根本无滚动,非回归)。若要分体轨完整回滚浏览,需预取窗行数据跨进程(nums 尾段扩样或独立面)。
- **F-03(观察在册)**:`shim_storage_get` 每调用重读 backing 文件(or_insert 首读冻结语义);本计划前端每拍读 ≤6 键、后端 2 键,开销可忽略;若未来 storage 读面扩容,建议 load-once 语义加失效通道。
- **F-04(归因在册)**:auto-lang 日常档 3 红(`musk_vm_track_tests::p053_1_widget_computed`×2 + `p053_4_merged_api_warning`)——基线 master dcbda3f71 detached worktree 复跑同红,预存与本计划无关(建议 DEBTS 台账收编)。
- **F-05(复审项)**:AC-04 桌面轨 024 口径实机复跑未在 work 阶段执行(代码零触及 + 域测绿已证),建议复审/merge 门补一次桌面轨 boot 冒烟;另实机 resize 事件偶发滞后收敛一例(nums 短窗保持旧值,机制在多点实证下有效)归因环境(并行会话桌面),复审如遇可先排查窗口事件链。
