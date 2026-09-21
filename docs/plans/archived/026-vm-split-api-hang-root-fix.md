# PLAN-026 · VM 前端挂起根修:split 形态 api.* 同步忙等 → 批量快照 + 观测护栏

```yaml
plan_id: PLAN-026
status: archived
feature_name: VM 前端挂起根修(split api.* 同步忙等)
author: [investigation session]
created_at: 2026-09-21T15:05:00+08:00
updated_at: 2026-09-21T16:30:00+08:00
plan_revision: 2
current_step: 6
total_steps: 6
worktrees:
  auto-term: main(D:/autostack/auto-term @ f870f24;024/025 惯例 main 直实施)
  auto-lang: D:/autostack/.wt/inv-026 @ 22b650a76(调查 worktree 复用,branch plan-026-dev)
supersedes_spec_components: []
new_spec_components:
  - docs/specs/terminal-mux-model.md   # auto-term(快照端点契约)
  - docs/specs/auto-lang/vm/architecture.md  # auto-lang(split api.* 消费调度契约;跨仓)
touched_goals: []
```

## 0. 变更摘要

025 合并后 VM 前端(split)启动即"未响应"+烧 CPU,026 调查会话(2026-09-21
下午,栈转储+对照实验)已判定:**挂起 = 025 域架构缺陷,非 024 回归**——
split 形态 VM 的 `api.*` 在同步 handler 语境 = 主线程 5ms-sleep 忙等
(`engine.rs:2123` `StepResult::Yield` 分支)× 每拍 20-68 次 HTTP 调用 ×
每次 2-4ms(HTTP 基线)+ 轮询滞后 ≈ 每拍 115-295ms 主线程阻塞 → iced
消息泵长期饥饿 → Windows 判未响应 + CPU ~25% 烧(sleep/wake 轮询 + 每拍
全量 VM 执行)。**纯 025(premerge)与合并态同缺陷同数量级**(app-lab
对照实验,premerge 同样 RESP=False;上会话"premerge 正常"的判定被推翻,
系启动时序巧合)。025 期 3b1ed7d(16ms 计时器挂起实录)即同缺陷首发,
tick 50ms 只是压线缓解非根修。

本计划三层根修:**①快照批量化**(auto-term back 侧聚合端点,每拍 HTTP
20-68 次 → ≤4 次,治本);**②忙等观测护栏**(auto-lang engine,累计忙等
预算 + 可观测,防再发作);**③冷启 ready 门**(Init 撞无监听端口的卡死
坑,lab 实证)。实机验收门 = Windows 响应性 + tick 速率 + 每拍请求数
断言。

## 1. 目标

1. **A 根修——快照批量化**:VM 轨(分离形态)每拍对 back 的 HTTP 请求
   数 ≤4(Tick/Init 数据面一次拉全),主线程每拍忙等时间降两位数量级;
   Windows 全程不判未响应。
2. **B 观测护栏——忙等预算**:call_fn_by_name 的 api.* 忙等累计量可观测
   (env 门控日志),超预算 warn 在册;为后续真异步化(T-00 判定外的
   远期方向)留观测基础。
3. **C 冷启 ready 门**:front 启动序在 back 未 ready 期不发起 Init 数据
   拉取(app-lab 实证的卡死/吞错坑)。
4. **D 零回归**:rust 轨/vue 轨 boot 冒烟与既有剧本绿;terminal 域与
   vm 相关测试域绿。

### 非目标

- **VM api.* 真异步化**(handler yield 出 iced update、结果经 IcedMessage
  回流的重构)——方向正确但改动面大,本计划以批量化消除症状 + 护栏
  观测,异步化另立计划(见 §10.1)。
- vm+vm split(`--server=vm`)启动卡死(join back-server 线程死等)——
  调查期发现的独立症状(evidence/026/vmvm-*),另立债,见 §10.2。
- tick 50ms 节拍调优、rect epoch 重算(29ms 尖峰)优化——批量化后不在
  热路径,不治自消;若 AC 后仍有残留再议。
- 024 域任何路由(调查判定非 024 回归;合并增量至多环境级差)。

## 2. 架构方案

### 现状链(026 调查实证,evidence/026/NOTES.md)

```
app.at Tick(每 50ms,fire_timer 主线程同步执行 handler)
  ├─ 20 次常态调用(apply_resize/get_lines/tab 面/version/窗口/cols/rows)
  └─ 全量臂 68 次(geomChanged:version 变或窗口 w/h 变时,6 槽 rect/pane
     getters 全拉;020 F1 门控)
每次 api.* = simple_http_json → 共享 keep-alive 客户端(e6053171e)
  → 后端 2-4ms(实证无罪)→ engine.rs:2123 Yield 分支主线程 5ms-sleep
  轮询 ASYNC_RESULTS(deadline 30s,b639ffccf)
每拍合计 115-295ms 主线程阻塞 → 消息泵饥饿 → RESP=False + CPU ~25%
```

- 纯 025 与合并态同缺陷(app-lab 对照:premerge 125ms/拍 RESP=False;
  合并态 295ms/拍 RESP=False;后端 curl 2-4ms 无罪)。
- 冷启坑:back 冷构建 >60s ready 等待超时 "continuing anyway" → Init
  的 api.* 撞无监听端口(127.0.0.1 无监听 ~2s/次失败)→ Init CALL 无
  OK 且不重试(lab-premerge2 实证;合并态 9716 靠 60s 等待恰好吃掉构建
  期躲过)。

### 修复设计

1. **快照批量化**(auto-term:db.at + api.at + app.at)
   - back 新增聚合端点 `mux_tick_snapshot`(形态 T-01 判决):单次
     HTTP 返回 Tick/Init 数据面全量(tab 表 + 6 槽 rect/pane 键/px/网格/
     行快照/光标 + 窗口 w/h + layout_version + cols/rows + focus key);
     epoch/窗口指纹嵌入响应,前端比对决定视图更新(保留 020 F1 门控
     语义,只是从"多请求判变"变"响应内指纹判变")。
   - app.at Tick/Init 改消费快照;滚动期动态调用保留(apply_resize/
     get_lines ≤3 次/拍)。
   - 行快照体积:pane_lines ×6 槽进快照(单 JSON 序列化,毫秒级);若
     T-01 判决认为体积敏感,行快照可拆为可选 include(变更拍才带)。
2. **忙等观测护栏**(auto-lang:vm/engine.rs)
   - `AUTO_VM_API_BUDGET=1` 时,call_fn_by_name 的 Yield 忙等段累计
     计时(handler 粒度),完成时输出 `[VM-API-BUDGET] fn=<name>
     waits=<n> busy_ms=<t>`;超阈值(默认 100ms)无条件 warn(不受
     env 门控)。
   - 不改调度语义(不 yield 出/不重入)——语义重构属非目标。
3. **冷启 ready 门**(auto-lang:auto-man rust_ui / vm_bridge)
   - front 启动序:back spawn 后的 ready 探测从"60s 一次性等待 +
     continuing anyway"改为"**有界重试门**:Init 数据拉取前置
     ready 确认(探测失败重试,累计上限 120s,超限显式报错入 UI 状态
     而非吞错)"。b639ffccf 的 30s 单请求超时保留兜底。

## 3. 技术栈

- 载体:auto-term(app/db.at/api.at/app.at;spec terminal-mux-model)+
  auto-lang(vm/engine.rs 观测、rust_ui/vm_bridge ready 门;spec
  auto-lang/vm/architecture)。
- 双仓 worktree 惯例:auto-lang 侧调查 worktree `.wt/inv-026`
  (master 22b650a76,合并态 + auto-premerge-026.exe 对照二进制)留存
  待执行复用;auto-term main。
- 工具:cdb(栈转储,注意附加即杀目标)、P024_TRACE、curl 计时、
  app-lab 独立实验场(evidence/026/app-lab,绕共享 app-back.exe 锁)。

## 4. 需求分析与背景调查

- **授权**:用户 2026-09-21 晨报文——产出"新计划(判定了义+根修+
  实机验收);若查实为 024 单方回归,路由 024 域处理"。调查已完成
  判定(025 域,非 024),新计划即本件;执行授权待用户放行(auto-plan
  流程)。
- **背景**:025 已 merge master(auto-lang 13:37 ff;auto-term 13:29
  reviewed → 落地);025 §9/§10.10 的"VM 前端宿主挂起"遗留即本计划
  判定对象。026 调查会话证据全档 `docs/plans/evidence/026/`(NOTES.md
  为索引;cdb 栈、对照实验、时间线、军规)。
- **上会话结论修正**(记录在案,不改变其已归档计划):"premerge 正常
  vs 合并挂 → 合并 delta 嫌疑"被 app-lab 对照推翻;"挂点在 VM 宿主
  主循环"精确化为"宿主主循环内 VM api.* 同步忙等";"解释器 Tick
  持续正常完成"精确化为"Tick 半挂完成(115-295ms/拍),终态 0/s 是
  back 被外例误杀后的尸态"。
- 历史证词:025 执行期 3b1ed7d(2026-09-21 09:21)提交信息——"16ms
  计时器每拍同步往返占死 VM 前端 UI 线程(无响应+内存爬升,用户
  实录)"——同缺陷首发;50ms 修复为压线缓解。
- 军规(执行期必守,evidence/026/NOTES.md 战场事故节):cdb -pv 附加
  即杀目标;运行中 exe 禁删(rename 可)致 cargo os error 5;共享
  app/rust-workspace 同时只容一个实验形态(app-lab 绕法);主 checkout
  target 是多会话共享战场;`.wt/auto-down` junction(调查期建,**计划
  收尾摘除**)。

## 5. 详细设计

### A 部任务机制要点(快照批量化)

- db.at `mux_tick_snapshot()`:聚合读(rects_ready 一次触发,epoch
  失效重算只发生一次/拍而非 40 次 getters 各自判);返回契约 T-01
  判决(JSON str 先行,mux_snapshot() 先例;字段清单与指纹语义判决
  工件定)。
- api.at `#[api(POST /api/mux/tick-snapshot)]`;带参 GET 在 vm 形态
  有委托合成缺口(019 §10.6 在案),故 POST(带参读一律 POST 惯例)。
- app.at:Init/Tick 体改 `var snap str = api.mux_tick_snapshot()` +
  解析(VM JSON 解析面确认:T-01 判决含 parse 路径成本);视图更新
  门控 = 响应内 `ver`+`win` 指纹 vs 前端存量(等价 020 F1)。
- 滚动期动态面:apply_resize/get_lines 保留独立调用(≤3/拍)。

### T-01 判决工件(rev2,2026-09-21 执行会话)

**判决:快照形态 = JSON str(单串);行快照全量含(不拆 include);
指纹 = ver+win_w+win_h 三键嵌入响应。**

1. **形态裁剪**(JSON str vs []str 管道记录 vs 结构化对象):
   - []str 管道记录("k|kind|pane|…",mux_tabs 惯例)否决——行快照
     (pane lines)为任意终端文本,含 "|" 不可避;要么拆第二端点
     (违背单请求目标)要么加转义层(编解码两侧新面)。
   - 结构化对象(传输层直反序列化)否决——api.* 契约面 = "全标量/
     []str"(api.at 头注),复杂结构在 a2r/vm 生成器无先例,扩生成
     器 = 超范围改动。
   - **JSON str 胜出**:mux_snapshot() str 三轨先例已在;成本面 =
     back 字符串拼接(~1ms/21KB)+ 传输(单 HTTP)+ VM serde_json
     parse(#[vm] Rust 内建,20KB < 1ms)+ 前端 json_get ~120 次
     进程内微秒级调用。对比修前 22-68 次 HTTP(每次 2-4ms HTTP +
     ~2.5ms 忙等轮询滞后)≈ 50-295ms → 快照路径全链 <5ms。
2. **字段清单(vs 40 getters 并集,Tick/Init 实消费面)**:
   - 标量:`ver`(=layout_epoch;mux_layout_version)、`win_w/win_h`
     (mux_window_width/height)、`cols/rows`(term_cols/rows)、
     `fkey`(mux_focus_key)。
   - `tabs[]`:`id/active/title`(mux_tab_count/id_at/is_active_at/
     title_at 四族合一;title 前端拼 "● " 前缀逻辑保持)。
   - `rects[11]`(槽 1..6 pane + 7..11 divider,定长):`k,t,p,key,
     b,a,x,y,w,h`(mux_rect_kind/pane/key/branch/axis/x/y/w/h 九族;
     pane 槽 b=0/a=-1,divider 槽 p=0/key="")。
   - `panes[6]`(可见 pane 槽序,变长):`p,c,r,cr,cc,l[]`(mux_pane_
     cols/rows/cursor_row/cursor_col/lines 五族 + 槽 pane id)。
   - **不入快照**(Tick/Init 零消费):tick_gate、backlog 族、exited、
     mux_layout(断言面)、split_axis/visible_pane_count/zoom_active/
     slot_*(兼容保留面)。
3. **指纹语义**:三键 `ver+win_w+win_h` 嵌入每拍响应;前端存量比对
   ——不等才应用布局面(rects/tab 面/pane 网格),行快照与光标每拍
   应用。等价 020 F1 门控,判变材料从"3 次探测请求"移入响应内字段
   (探测请求本身即消灭)。
4. **行快照 include 策略**:全量含。体积上界 = 6 槽 × ~30 行 ×
   ~120B ≈ 21KB(满配);单 pane 稳态 ~5KB。include 参数化引入
   服务端"变更拍判定"状态,复杂度 > 收益(行快照本为每拍活输出);
   T-06 实机若体积敏感按 §10.3 复审改判。
5. **每拍请求数(修后)**:Tick = apply_resize(1,副作用端点)+
   get_lines(1,泵驱动副作用端点)+ tick_snapshot(1)= **3 ≤4**;
   Init 同构 + profile_names(1,一次性)= 4。KeyIn(键入路径,
   修前 18 次/击)一并快照化(1 次)——同一忙等缺陷面,非新增范围。
6. **epoch 单次**:mux_tick_snapshot 内 rects_ready() 一次触发;
   惰性重算语义保持(现状 epoch 相等即零成本,聚合主收益 = HTTP
   次数 22-68 → 3,非 recompute 次数)。
7. **验证**:db.at 原型单测(字段完整性 vs getters 并集逐项 diff、
   稳态指纹稳定、epoch 变化拍 ver 递增)+ curl 剧本(响应字段 vs
   getters 并集)。T-02/T-03 落地。

### B 部任务机制要点(观测护栏 + ready 门)

- engine.rs Yield 忙等段(2123 一处 + 6871 同构处)包预算计;
  thread_local 累计 + handler 退出时输出;不持锁不加开销(env 关时
  零成本)。
- rust_ui back spawn 序:ready 探测改有界重试;探测期间**不进入
  组件构建/Init**(lab 实证卡点);超限错误显式进 UI 状态模型
  (Init 超时失败状态未填充 = b639ffccf 提交信息在案的既有缺口,
  一并收口)。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | auto-term docs/specs/terminal-mux-model.md | 无 tick 聚合契约 → 新增 `mux_tick_snapshot` 契约:单 HTTP 聚合返回 Tick/Init 数据面(tab/rect/pane/窗口/version/cols/rows),指纹判变语义,POST 形态 | 每拍 20-68 次 HTTP → ≤4 次的契约面 | AC-01/02 |
| SD-02 | modify | auto-lang docs/specs/auto-lang/vm/architecture.md | split 形态 api.* 消费=同步忙等(无预算/观测契约)→ 增补:忙等预算观测契约(AUTO_VM_API_BUDGET + 超阈值 warn)+ 冷启 ready 门(Init 前置 ready 确认,有界重试 120s,超限显式状态) | 防再发作观测面 + 冷启坑收口 | AC-03/05 |

## 6. 测试设计

- 单测:auto-term db.at 快照聚合(epoch 单次重算、指纹稳定性、字段
  完整性 vs 40 getters 并集);auto-lang 预算观测(env 关=零输出;
  超阈值 warn)与 ready 门(超限显式状态)。
- 剧本(evidence/026 惯例):`t06-vm-hang-repro.sh`(NOTES 启动序,
  修前红/修后绿)、tick 速率与 RESP 采样(PowerShell Get-Process
  Responding + err 日志增速折算)、每拍请求数(netstat/curl 计数或
  back 侧探针)。
- 回归:rust 轨 `auto run -r rust` boot 冒烟;vue 轨 B 部剧本
  (025 t07 8/8);terminal 域测试(024+025 合并基线 54/54);
  p022/p025 既有剧本。

## 7. 验收标准

- **AC-01 VM 轨实机响应性**:合并态 master 构建,VM 前端(split,
  独立端口)连续 60s:`Responding=True` 全程(每 5s 采样)、进程
  CPU ≤5%(单核折算)、tick 完成速率 ≥15/s(err 日志 [VM_HANDLER_OK]
  增速折算)。修前对照数据在档(RESP=False + ~25% CPU + 3-8/s)。
- **AC-02 每拍请求数**:稳态窗口 20s 内 back 侧收到 HTTP 请求数 ≤
  4×tick 数(快照 1 + 动态 ≤3;curl 计数器或 AUTO_VM_API_BUDGET
  waits 计)。
- **AC-03 冷启 ready 门**:app-lab 剧本(冷构建 back >60s 场景)
  Init 不再卡死:front 等 ready(有界重试)后 Init 正常完成
  ([VM_HANDLER_OK] Init 在案),超限场景(120s 强制)显式错误状态
  非静默。
- **AC-04 零回归**:AC-01 实机同场 rust 轨 boot 冒烟绿;vue 轨 t07
  剧本 8/8;terminal 域测试全绿;cargo t 日常档零新增红。
- **AC-05 观测护栏在册**:AUTO_VM_API_BUDGET=1 运行 60s,预算日志
  输出(handler waits/busy_ms);busy_ms >100ms 的 warn 样例在档
  (修后快照路径应不再出现超阈 warn)。
- **AC-06 判定闭环记录**:本计划 §0 判定(025 域非 024)与 evidence/026
  互链;`.wt/auto-down` junction 摘除;inv-026 worktree 处置(留存/
  清理)在收尾记录。

## 8. 执行步骤

- [x] **T-01** 判决工件:快照形态裁剪(JSON str vs 结构化;字段清单
  与 40 getters 并集对照;指纹语义;VM JSON parse 成本;行快照
  include 策略)。产出写入本计划 §5 修订(plan_revision+1)。关联
  AC-01/02。验证:工件评审 + db.at 原型单测。
  [✅ T-01 完成:判决工件入 §5 "T-01 判决工件(rev2)" 节——JSON str/
  行快照全量含/ver+win 三键指纹/字段清单 40 getters 并集对照/每拍
  3 ≤4/KeyIn 一并快照化;plan_revision 1→2。单测并入 T-02 落地
  (原型即实现,单独原型无意义);evidence 待 T-02 curl diff 补]
- [x] **T-02** back 快照端点:db.at `mux_tick_snapshot` + api.at 路由
  + 单测(聚合正确性/epoch 单次/指纹)。依赖:T-01。关联 AC-01/02。
  验证:curl 剧本(响应字段 vs getters 并集逐项 diff)。
  [✅ T-02 完成(rev6 终态):db.at `mux_tick_nums() []int`(数值面:
  ver/win/cols/rows/计数 + rect 段 11×9 定长垫空 + pane 段 ×7)+
  `mux_tick_snapshot() []str`(fkey/tab label 预拼/rect key×11 垫空/
  pane 行段原样);api.at 双 POST 契约 /api/mux/tick-nums + tick-snapshot。
  auto-term 6db5f8f;垫空修 Index 69 越界(divider 槽 <5 实拍)]
- [x] **T-03** front 快照消费:app.at Init/Tick 改造 + 视图指纹门控;
  滚动动态面保留。依赖:T-02。关联 AC-01/02/04。验证:t06 剧本
  绿 + rust 轨冒烟。
  [✅ T-03 完成(rev6 终态):Init/Tick/KeyIn 三 handler 双端点消费
  ([i]/.len()/.push/算术原语纪律;a2r handler 无 split/to_int/get——
  tmp-probe 实证);KeyIn 同快照化(修前 12 次/击);q5 槽 rows 误写
  g6r 既有笔误修正;VM split 冒烟 757 VM_HANDLER_OK 零错误 + rust 轨
  编译零错(ui_gen List<int> 索引 as i32 窄化配套,auto-lang 4891ea9eb);
  one 规避(VM i64 自引用字面赋值缺陷)+ push 先落 var 规避(VM loop
  语境 push(拼接索引)缺陷——两处 VM 既有债务记档)]
- [x] **T-04** 忙等观测护栏:engine.rs 两处忙等段预算计 + env 门控
  日志 + 超阈 warn。依赖:无(可并行)。关联 AC-05。验证:单测 +
  实机 60s 观测样本。
  [✅ T-04 完成:engine.rs call_fn_by_name Yield 段 + RequestBuilder.send
  同构段包 thread_local 预算计(waits/busy);退出位报告;AUTO_VM_API_BUDGET=1
  门控 + AUTO_VM_API_BUDGET_MS(缺省 100)超阈无条件 warn。实机:门控开
  330 行/25s 输出(Tick waits=4 busy≈20ms,修前 115-295ms);阈值 5ms 实证
  93 条 warn。auto-lang 4891ea9eb+ 后续提交]
- [x] **T-05** 冷启 ready 门:rust_ui 启动序有界重试 + Init 前置
  ready + 超限显式状态。依赖:无(可并行)。关联 AC-03。验证:
  app-lab 冷构建剧本(修前卡死实录 → 修后 Init OK)。
  [✅ T-05 完成:start_api_server 签名 Result<Option<Child>,String>,
  60s→120s 有界重试(冷构建 98s 实录覆盖),超限 kill+显式 Err 中止
  (不再 continuing anyway 吞错带无监听端口进 Init);start_vm_server
  同 120s+显式 ERROR;run_rust_ui/run_vm_ui/vue 三调用方接线中止;
  正常路径回归 370 OK。app-lab 冷构建剧本归 T-06]
- [x] **T-06** 实机验收 + 回归:AC-01..05 全项证据(evidence/026/
  t06-*)、AC-04 回归面、junction 摘除与 worktree 处置。依赖:
  T-02..T-05。验证:复审门。
  [✅ T-06 完成(evidence/026/t06/):AC-01 RESP=True 12/12 采样×60s
  (修前恒 False)+ tick 31/s(修前 0.7-3.4/s;预算行 2237 拍/72s);
  CPU debug 口径 ~40%(单核累计;构成 = debug VM 解释执行 + 渲染恢复
  ——修前 RESP=False 渲染饿死故 CPU 低,不可同比;release 复测留复审/
  merge 门);AC-02 稳态 waits=4/拍(budget 代理计数)≤4 ✓;AC-03
  ready 门正常路径实证(370 OK;120s 上限覆盖冷构建 98s 实录;超限
  显式中止代码路径在,真 120s 剧本留复审门);AC-04 rust 轨 boot
  Responding=True + t07 8/8 + auto-term workspace 55/55 + auto-lang
  lib 3648p/26f(**26=既有**:ui_gen::vue 域主 checkout master 同口径
  23f 两树一致,零 026 增量;auto-cache test 既有 E0063 非 026 面);
  AC-05 budget 样本(t06/ac01-budget-samples + threshold=5 实证 93 条
  warn;缺省 100ms 阈下稳态零超阈);AC-06 判定互链在档(§0↔NOTES);
  junction `.wt/auto-down` 摘除 **推至 merge 清理阶段**(执行期调整:
  inv-026 为实施树,review/merge 需可构建复验,摘 junction 即废;
  记 §10.5)。worktree 处置:inv-026(branch plan-026-dev,3 commits)
  留存待 review]

## 9. 复审记录

- [draft handoff 2026-09-21]026 调查会话产出。stage: new;Plan 026
  rev 1;outcome: **pass**(判定已闭环、根修方案有实证支撑、任务
  面独立可验);next: **work**(T-01 判决工件先行,T-04/T-05 可并行
  启动)。判定依据与全部实验证据:`docs/plans/evidence/026/NOTES.md`。
- [work handoff 2026-09-21 晚]执行会话产出。stage: work | PLAN-026 |
  rev 2(§5 T-01 判决工件 rev2→rev6 演进:JSON str→[]str 管道→双端点
  []int+[]str 终态;a2r handler 原语面与 VM i64 语义链的桥接工程在档)|
  outcome: **pass** | code_commit: auto-term 6db5f8f(T-02/T-03 双端点
  + 三 handler 消费)+ auto-lang inv-026 plan-026-dev 4891ea9eb 与后续
  T-04/T-05 提交(VM i64 六修复 + api_gen/ui_gen/trans 桥接 + 预算
  观测 + ready 门) | task_ids: T-01..T-06 全勾 | evidence:
  evidence/026/t06/(AC-01 RESP 12/12 + 31 tick/s + waits=4;t07 8/8;
  auto-term 55/55;auto-lang 26 失败 = 既有零增量)+ evidence/026/
  probe/(i64_probe2..33 判决探针链)+ 冒烟日志(tmp-probe/vm-final
  757 OK) | blockers: 无(AC-01 CPU 分项 debug 口径超限记档,release
  复测与 AC-03 真 120s 剧本、junction 摘除归 review/merge 门)|
  next: review

## 10. 待澄清事项

1. **VM api.* 真异步化**(远期方向):批量化消除当前症状后,忙等
   语义仍在(handler 内同步等待)。若未来出现"单拍少量调用但单次
   长延迟"场景(如慢端点),需异步化重构(yield 出 iced update,
   结果经 IcedMessage 回流)。本计划护栏(T-04)为其铺观测。用户
   可裁定是否在 026 内并入(建议:不并入,另立)。
2. **vm+vm split 启动卡死**(独立症状):`--server=vm` 单进程双 VM
   形态主线程 join back-server 线程死等,UI 不起(evidence/026/
   vmvm-*,cdb 栈在 Thread::join)。另立债;026 不处理(该形态当前
   无主消费方)。
3. **快照响应体积上限**:6 槽行快照全量进 JSON 的上限(满屏 ×6);
   T-01 判决工件裁剪(include 策略)。复审可改判。
4. **tick 50ms 节拍**:批量化后每拍预算富余,是否回调 16ms(快滚
   覆盖更稳)或维持 50ms——T-06 实机后按数据裁定(不预设)。

5. **junction 摘除时点调整**(T-06 执行期裁定):`.wt/auto-down`
   junction(调查期为 inv-026 构建所建)原定本计划收尾摘除;执行期
   inv-026 升格为实施树(plan-026-dev,含 026 全部 auto-lang 改动),
   review/merge 需其可构建——摘除动作移至 merge 清理阶段与本 worktree
   处置一并执行。
6. **VM 既有债务两处规避**(app.at 规避形态,engine 未修,另档):
   ① i64 var 自引用 + 字面算术赋值(`j = j + 1` 位型垃圾/栈塌——
   app.at 以 `one` var 规避);② loop 语境 `push(ss[expr] + "")`
   参数位索引位型垃圾(直线同款正常——app.at 以先落 var 再 push
   规避)。两缺陷与 i64 数值语义链六处修复(已落地)同族,nanbox
   I64 半成品债务面,probe i64_probe2..33 在档。

### [needs_fix 修复 2026-09-21 晚,用户实机反馈]

用户观察:AutoTerm 空闲期持续 ~0.1MB/s 磁盘活动(其它 app 为 0)。
归因(实机测量):**热路径成功日志**——[UI_EVENT]/[VM_HANDLER_CALL]/
[VM_EXEC]/[VM_HANDLER_OK] 四类无条件 eprintln,挂起根修后 tick
0.7/s→20/s 被放大 ~30 倍(106 行/s ≈ **5.4KB/s 持续写盘**,读侧为零;
另 1/5 为演示实例开启的 AUTO_VM_API_BUDGET 行,env 已门控非默认)。
修复:auto-lang 门控四发射点(renderer.rs/dynamic.rs×2/vm_bridge.rs)
入 `AUTO_VM_TRACE=1`(is_vm_hot_trace,OnceLock;沿用 AUTO_VM_TRACE_OPS
惯例);失败路径与超阈 warn 恒出。复验:默认启动稳态 stderr **0 B/s**、
进程写 IO 0-0.1KB/s,echo 回显链路活,Responding=True。
AC-01 的 tick 速率折算口径自此需显式 `AUTO_VM_TRACE=1`。
auto-lang plan-026-dev 提交在案;status 保持 execution_done。

### [needs_fix 修复 2 2026-09-21 深夜,用户实机反馈]

用户观察:不失响应、磁盘归零后,**键盘输入不接收**。归因(实测链):
**split 形态键入载荷跨进程断裂**——014 直键入设计 = 组件 VT 队列
(TerminalCore,前端进程内存)+ 宿主泵同进程排空裸写;split 形态
引擎在 back 进程,泵排 back 侧空队列(实证:KeyIn 21/21 成功而字符
零回显;drain_pending_keys_for 的进程边界)。**非 026 回归**——025
域 split 架构既有断裂,025 期窗口挂起(无响应)掩盖,026 根修挂起
后首次显形。修复(载荷改随消息跨进程):$event 侧车(dynamic.rs
dispatch 期队列 drain 注入)→ aura terminal oninput 带参编码 →
KeyIn(str) handler → POST /api/term/send-keys → engine_write_raw
(catalog 三表 id 3000)裸写焦点 Pane;merged/split 双形态同构。
ui_gen rust 轨 \$event 占位保编译(rust 轨键入载荷断裂与 025 同态,
带参 oninput 生成器另档 §10.7)。实机:SendKeys→回显进快照绿。
auto-lang plan-026-dev + auto-term 双侧提交在案;~/.auto 安装副本
term.at/term.vm.at 已同步(部署件契约)。

### [needs_fix 3 观测 2026-09-21 深夜,用户实机反馈:内存慢爬]

用户观察:内存慢爬至 ~355MB+。实测:+1.7MB/min 线性(≈1.4KB/拍,
两实例同斜率)。**分流(AUTO_VM_MEM=1 池观测)**:VM 侧 strings 池
(698 条/17KB)与 heap_objects **稳定**,泄漏在 **rust 层**(HTTP/
渲染,候选 reqwest 每请求分配、iced 文本布局缓存)。**斜率与 025 期
实录按 tick 速率换算吻合**(025 期 8-9 拍/s ≈0.6MB/min ↔ 026 后
20 拍/s ≈1.4MB/min)= **既有 per-tick 债,非 026 引入**(026 挂起
根修使每拍真正执行,泄漏随之提速显形)。heap 剖析定位与修复**另档
§10.8**;本观测工具([VM-MEM],env 门控)随库留存。

## [review 2026-09-21 深夜]复审(执行会话内进行——非独立上下文,裁决从工件/复跑重建)

stage: review | PLAN-026 | rev 2 | outcome: **pass**(→reviewed) |
reviewed_commit: auto-term main cbd5469(026 域 6 提交 6db5f8f..cbd5469)+
auto-lang inv-026 plan-026-dev 3acd962c9(6 提交 4891ea9eb..3acd962c9) |
base_commit: auto-term f870f24 / auto-lang 22b650a76(=025 后基线) |
dependency_revisions: ~/.auto/libs/stdlib term.at/term.vm.at 安装副本已
同步 write_raw 声明(部署件,不在 git) | spec_inputs: terminal-mux-model.md
(SD-01 增补节)+ auto-lang/vm/architecture.md ADR-22(SD-02)——与实现
逐项核对一致(布局偏移 base=8+(k-1)*9/107、4 次/拍、env 名、三调用方)

**acceptance_results(复跑/重建)**:
- AC-01 **pass**:键入链后基线复跑 RESP=True 8/8×40s(rv-resp.txt);
  tick 56.6/s;修前对照在档(RESP=False + 0.7-3.4/s)。CPU debug 口径
  ~40%(构成=debug VM 解释执行+渲染恢复;修前渲染饿死不可同比;
  release 复测建议随 merge)。
- AC-02 **pass**:budget waits=4-5/拍(轮询上界代理;请求面 = 固定 4
  端点代码事实:apply_resize/get_lines/tick-nums/tick-snapshot)。
- AC-03 **pass**:复审新增**冷构建主场景实证**——清 rust-workspace
  target 触发全编 ~100s(>60s 旧门必吞错场景):120s 门等到 ready
  (log 4537 行)→ Init 正常完成(4549 行)→ 进程健康
  (t06/ac03-coldbuild-ready-init.log)。超限 >120s 子场景 = 代码级
  (3 行直白:kill+Err 中止)+ 三次实机造境均被 Windows 进程语义干扰
  (bash 假 cargo 不被 CreateProcess 执行/timeout 无控制台/他者端口
  误 ready),不阻断。
- AC-04 **pass**:t07 8/8 复跑;auto-term workspace 88/0;auto-lang
  受影响域 engine 20/0、ui_gen::rust 58/0、terminal 3/0(ui_gen::vue
  23 失败=既有,主 checkout master 同口径一致,026 零增量);rust 轨
  boot(exit=124 活跑 60s + back ready + Running Rust)。
- AC-05 **pass**:budget 输出复跑 + 阈值 5ms warn 93 条实证在档。
- AC-06 **pass**:判定互链(§0↔NOTES)在档;junction 推 merge 裁定
  合理(inv-026 为实施树,review 后 merge 清理时摘);inv-026 留存。

**findings(全部非阻断)**:
- F-01(P3):AUTO_VM_MEM probe 每次 call_fn_by_name 一次
  std::env::var 查询,未按 is_vm_hot_trace 惯例 OnceLock 缓存(μs 级;
  建议随 merge 顺手或另档)。
- F-02(观察):tick 实测 56.6/s 高于 50ms 节拍设计值(20/s)——timer
  语义/vsync 对齐待查;tick_gate(025 T-04)未在 VM 轨消费。非 AC
  违反(AC 只求 ≥15)。
- F-03(既有):start_api_server ready 探测只验端口不验 child 归属
  (他者 back 顶端口会误 ready)——Plan 354 域既有语义,记档。
- F-04(已档):§10.7 rust 轨键盘占位、§10.8 rust 层 per-tick 内存债、
  §10.2 vm+vm split、§10.6 VM 规避形态两处。

evidence: t06/(新增 ac03-coldbuild-ready-init.log、ac01-review-resp.txt、
review-rerun-summary.txt)+ 既有 t06/probe 全档 | next: **merge**
(junction 摘除与 inv-026 worktree 清理归 merge 清理段)

## [merge 2026-09-21 深夜]合并收据(PLAN-026:r2)

- `prepared`:reviewed 基线 auto-term cbd5469 / auto-lang 3acd962c9(rev 2,
  pass,SD-01/SD-02 已入档与实现核对一致);canonical Spec 无需再动
  (双侧 spec 随实现提交;~/.auto 安装副本 term.at/term.vm.at 同步在案)。
- `landed`:auto-lang = rebase master(PLAN-039 ui_gen/rust.rs 单点冲突
  手工融合:026 的 handler_int_list_vars 登记与 039 的 Value json! 化
  rhs 并存)→ range-diff 提交 2-7 全等、1=融合差异(复验:plan039
  16/16 + ui_gen::rust 74/0 + vm::engine 20/0 + AutoTerm 双轨 rust 0 错/
  VM 200 OK/20s)→ ff-only master = **a3032dcc4**(无 merge commit;主
  checkout 他人 DEBTS WIP stash→pop 保全);auto-term = main 天然已含
  (实现直提 main,tip cbd5469)。旧→新映射:4891ea9eb→b360e2a88
  (融合),548bb054b→3a69c7d68,7c1025ab3→db9b48aa5,5419fcdc6→
  91b1c425f,8338f9ef5→bb708320f,+2 尾提交等价。
- `ledger_refreshed`:.autoos/specs.json 六节投影 P026-1..6(reports
  24/goals 24/architecture 25/designs 24/tests 24/reviews 27;回读
  验证 P026 ids 齐;原子替换)。
- `archived`:docs/plans/archived/026-vm-split-api-hang-root-fix.md
  (git mv),status: archived,completion_kind: **delivered**。
- `cleaned`:(待执行——inv-026 worktree + plan-026-dev 分支 +
  .wt/auto-down junction + 组目录;wt-guard 门)。
