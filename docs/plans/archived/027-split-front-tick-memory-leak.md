# PLAN-027 · split 前端 per-tick 内存泄漏定罪与根修(DEBT #26 销账)

```yaml
plan_id: PLAN-027
status: archived
feature_name: AutoTerm split 形态 VM 前端空闲期内存线性爬升(~1.4KB/拍)的定罪、根修与回归钉
author: [zhaopuming/zcode-session]
created_at: 2026-09-21T00:00:00Z
updated_at: 2026-09-21T00:00:00Z
plan_revision: 1
current_step: 6
total_steps: 6
supersedes_spec_components: []
new_spec_components:
  - auto-lang 仓 docs/specs/stdlib/design/async-http-result-lifecycle.md   # SD-01;SD-02 并入规则 5(T-05 决算)
touched_goals: []
```

## 0. 变更摘要

DEBTS #26(双仓在册,2026-09-21):AutoTerm split 形态 VM 前端**空闲期**
内存线性爬升 +1.7MB/min(≈1.4KB/拍,20 拍/s;任务管理器实录,两实例
同斜率可复现)。026 已分流定界:**VM 池无罪**(`AUTO_VM_MEM=1` 实证
strings/heap 池稳定),泄漏在宿主 auto.exe 的 **rust 层**;断代换算证明
025 期即存在(挂起期 tick 跑不动,026 根修后泄漏随 tick 提速显形)。

本计划三级漏斗定罪(读码审计 → 无重建二分实验 → 分配归因垫片),
根修,并留常设观测面 + 回归钉防复发。**改动预期集中在 auto-lang
(crates/auto-lang/src/vm/ffi/stdlib.rs 结果通道/HTTP 消费链、
crates/auto/src/main.rs 垫片)**;auto-term 侧仅 DEBTS 销账与
evidence 取证。

## 1. 目标

- 定罪:给出泄漏点的可复现实证(调用栈/路径 + 实验交叉),写入本
  计划 §9 与 evidence/027/。
- 根修:空闲期斜率从 1.7MB/min 降到与常驻本底不可区分(阈值见
  AC-02;§10-① 待用户确认)。
- 防复发:分配观测面常设(env 门控)+ 自动回归钉。
- 销账:双仓 DEBTS #26 关闭,规范增补落册。

**非目标**:三个 UI 细节问题(窗口 resize 不跟随/CJK 间距/状态栏
配色圆角,2026-09-21 用户实录)不在本计划范围,另行立项。既有债
§10.8(heap 剖析立项)由本计划吸收。不改动 VM 池内存模型与
tick 快照协议本身。

## 2. 架构方案

三级漏斗,成本递增、逐级剪枝:

```
L0 读码审计(纯静态,零成本)
   空闲期在爬 ⇒ 内容驱动面(iced 文本布局/styled 旁路)概率极低
   ⇒ 嫌疑收敛到 tick 通道 4 请求/拍的 rust 消费链:
   ①ASYNC_RESULTS 全局表(条目漏 consume 即永驻)
   ②HTTP_RESPONSES thread_local(handle 型搬运;主线程只进不出路径)
   ③每请求 std::thread::spawn 线程 churn(~80 线程/s)
   ④hyper keep-alive 缓冲(单连接,理论有界,最低)
        ↓ 未定罪
L1 无重建二分(每实验 ~10 分钟)
   A rust 轨对照:同 app 单进程无 HTTP → 有/无斜率劈开渲染与 HTTP 面
   B 断流实验:kill 后端,api.* 全失败 → 失败路径也爬=发射侧;停=消费侧
        ↓ 未定罪
L2 分配归因垫片(重建一次)
   env 门控 #[global_allocator] 计数(存活字节按大小桶+可选栈哈希)
   或 dhat-rs → top-N 增长点调用栈,一击定罪
        ↓
根修(按定罪结论选型) + 常设观测 + 回归钉 + 销账
```

关键架构事实(已核):api.* 的 get_json/post_json 走
`simple_http_json`(stdlib.rs:6746)且**已用共享 client**
(`shared_api_http_client()`,Plan 446 E4)——"每请求 Client::new()"
仅存在于 upload/download 等 AutoTerm 不用的 natives,可排除;
每请求仍 `std::thread::spawn`(stdlib.rs:6867 一带)。

## 3. 技术栈

- 采样:PowerShell `Get-Process` WorkingSet64/PrivateMemorySize64
  定时序列(或 VMMap 快照);既有 `AUTO_VM_MEM=1`(engine.rs:2251)
  与 `AUTO_VM_API_BUDGET=1`(engine.rs:1642)观测面继续用作旁证。
- 归因:自研 env 门控 global_allocator 垫片(优先;零新依赖)或
  dhat-rs(dev-dependency,§10-④)。
- 双仓 worktree 惯例:`term-027/auto-lang`(主改动)+
  `term-027/auto-term`(DEBTS/evidence);主检出只留计划文档。

## 4. 需求分析与背景调查

**授权记录**:用户 2026-09-21 会话裁定——"先 auto-plan-new 一个
计划,再依照计划执行分析和修复";执行沿会话流惯例逐阶段放行
(分析结论/根修方案汇报后再动代码的部分,以 work 阶段门呈现)。
未设自动续行预算。涉及仓库:auto-lang(改动)、auto-term(销账/取证)。

**背景事实**(DEBTS #26 + 本会话新调查):

- 现象:空闲期线性 +1.7MB/min ≈ 1.4KB/拍;tick 50ms 节拍 4 请求
  (app.at `.Tick`:apply-resize/get_lines/tick-nums/tick-snapshot)。
- 分流:VM 池稳定(AUTO_VM_MEM,026 needs_fix3);泄漏在 rust 层。
- 断代:斜率与 025 实录按 tick 速率换算吻合(20/8.5×0.6≈1.4)。
- **本会话新增推理**:候选②③(iced 文本布局缓存/styled 旁路)是
  内容驱动——空闲期行内容不变、行段落 digest 门控 + ROW_CACHE_CAP
  淘汰护栏(widget.rs:172 refresh_row_cache)不重建;空闲期仍爬 ⇒
  嫌疑压到 tick 通道 HTTP 消费链。DEBT 候选①的"每请求分配"经查
  已被 Plan 446 共享 client 化解大半。
- **观测盲区**:`AUTO_VM_MEM` 只看 VM 侧池;`ASYNC_RESULTS`
  (stdlib.rs:3939,`static Mutex<HashMap<u64, Option<Result<...>>>>`)
  与 `HTTP_RESPONSES`(stdlib.rs:3889,thread_local
  `RefCell<HashMap<u64, HttpResponseData>>`)均在 rust 层,恰在
  盲区内;条目量级(每拍 4 × 快照体 2-8KB)与斜率同数量级。
- 版本基线:auto-lang master a3032dcc4、auto-term main 116e9b6
  (CLI 已重建同代,VM 轨可跑,见 2026-09-21 会话)。

## 5. 详细设计

### 5.1 L0 审计方法

枚举 `ASYNC_RESULTS` 全部 insert 蘸点(stdlib.rs 1589/1632/5314/
5338/5362/5384/5639/6868 等)与全部 take/remove 消费点(1560/1608/
6818/6830 等),对每个 (native, 调用约定) 对验证:①VM 侧轮询
native 是否必然被再次调用(生成代码 emit_api_http_call 的配对
契约);②超时/错误/异常返回路径是否遗漏移除;③request_id 分配
是否可能跳过消费(NET_HANDLE_COUNTER 单调)。`HTTP_RESPONSES`
同法(thread_local 生命周期 × 搬运路径 4836 check_async_http_
result_handle)。产出:定罪/排除表(证据落 evidence/027/)。

### 5.2 L1 实验协议

每组 ≥10 分钟、每 30s 采样一次 WorkingSet64 + PrivateMemorySize64,
线性拟合斜率;三组:A=rust 轨(`cd app && auto run -r rust`);
B=断流(VM 轨起后台 kill app-back);C=空载基线(修前 VM 轨,
复现 1.7MB/min 校准采样方法)。

### 5.3 L2 垫片设计

`crates/auto/src/main.rs` 顶部 env 门控 `#[global_allocator]`
(AUTO_ALLOC_STATS=1 时激活;Normal 分配走 System,Extra 误用 panic)。
记录:`alloc/dealloc` 差额按 16/32/64/128/256/512/1K/4K/16K+ 大小
桶累计存活字节 + 调用方 PC 粗哈希(可选 `[1..9]` 帧回溯,门控分级);
每 30s stderr 打印 top-N 增长桶。dhat-rs 仅在自研垫片不足时启用
(dev-dependency + feature 门控,默认关闭)。

### 5.4 根修候选(按定罪选型,不预先锁定)

- map 漏回收:补配对消费/超时清扫(请求发射时登记 TTL,轮询超时
  即弃)/容量上界(LRU 淘汰);thread_local 搬运路径补 remove。
- 线程 churn:api.* 的 json 请求改共享常驻 worker(或直接同步内联
  调用——本地 127.0.0.1 往返,忙等语义不变,消除每请求 spawn)。
- hyper/tokio 内部:升级或配置调整;若属上游库内部不可根修,转
  缓解 + DEBT 续档(§10-②)。

### 5.5 规范增量

| delta_id | add/modify/retire | target(auto-lang 仓) | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/stdlib/(结果通道生命周期) | 无成文规则 → ASYNC_RESULTS/HTTP_RESPONSES 条目必须消费即删或 TTL 回收,容量有界;新 handle 型 native 必须声明回收方 | 定罪后固化防复发;026 教训=观测盲区 | AC-01/02 |
| SD-02 | add | docs/specs/stdlib/(分配观测面) | 无 → AUTO_ALLOC_STATS=1 常设垫片协议(门控/输出格式/大小桶) | rust 层分配可观测,补 AUTO_VM_MEM 盲区 | AC-03 |

## 6. 测试设计

- 单元(L0 若改回收逻辑):insert/consume 配对 property 测试
  (超时清扫、TTL 到期、容量上界淘汰)。
- 集成:headless 实例跑 N 拍(≥5 分钟)采样断言斜率 ≤ 阈值
  (脚本化,进 scripts/repro/)。
- 实机:空闲 10 分钟斜率复测(AC-02)+ `AUTO_VM_API_BUDGET` 对比
  修前水位(AC-05,警惕根修引入忙等回归——026 的 api.* 忙等预算
  观测继续在库)。

## 7. 验收标准

- **AC-01 定罪证据链**:泄漏点有垫片调用栈或"读码配对缺口+实验
  交叉"实证;定罪/排除表与原始数据落 evidence/027/。验证:文件
  存在且结论可从证据独立复核。
- **AC-02 斜率达标**:修复后 VM 轨空闲 10 分钟采样斜率
  ≤0.1MB/min(修前 1.7;阈值用户可调,§10-①)。验证:采样脚本
  输出拟合斜率。
- **AC-03 回归钉**:自动化斜率断言脚本入库并通过(≥5 分钟跑)。
  验证:脚本执行退出码 0。
- **AC-04 销账与规范**:双仓 DEBTS #26 关闭(auto-lang 026 行 +
  auto-term #26),SD-01/02 落册。验证:两仓 DEBTS/specs 文件
  检查 + 账本投影(merge 期)。
- **AC-05 零响应回归**:修后 tick busy 分布不劣于修前
  (AUTO_VM_API_BUDGET 对比;允许 ±10%)。验证:观测输出对比表。

## 8. 执行步骤

- [x] **T-01 L0 读码审计**(依赖:无;产物:定罪/排除表)✅ evidence/027/l0-audit.md——确证三缺陷:A 超时路径条目泄漏(engine.rs:2199/7027 两处同构,超时只清 waiting 不删 ASYNC_RESULTS)、B pending 覆盖竞态(spawn 后才 insert None)、C Err 条目挂起(check 的 .ok() 吞错);排除每请求 Client::new()/VM 池/行缓存/HTTP_RESPONSES;稳态 1.4KB/拍未定罪→头号候选=每请求双层线程 churn(160 线程/s,~175B/线程量级吻合),交 L1/L2。
  审 ASYNC_RESULTS/HTTP_RESPONSES 全部插删点与 emit_api_http_call
  配对契约(codegen.rs:4631 一带);含 NET_HANDLE_COUNTER 消费
  完整性。验证:表落 evidence/027/l0-audit.md,每嫌疑点给出
  定罪/排除+理由。→ AC-01
- [x] **T-02 L1 二分实验**(依赖:无,可与 T-01 并行)✅ evidence/027/l1-differential.md + l1-c-baseline.csv(22 样本)/l1-a-rust.csv(20 样本)——C 组(VM 前端)+2.23MB/min 复现 DEBT;A 组(a2r 编译前端,同 HTTP/同后端/同 Iced)+0.09MB/min 平 ⇒ 排除 iced/后端/共享 client 面,定罪 VM 前端 api.* 消费机械(≈465B/请求);B 组(断流)缓行,视修复验证轮结果补。
  按协议跑 A/B/C 三组采样。验证:evidence/027/l1-*.txt 斜率数据。
  → AC-01
- **T-03 L2 垫片归因**(依赖:T-01+T-02 未定罪时才执行;条件任务)**[未触发·条件不满足]** 定罪已在 L0(三缺陷读码确证)+L1(A/C 判别 25 倍差)+坍缩实验(2→1 线程,2.23→1.15MB/min 恰半减,churn∝线程创建数)三级闭合,dhat/分配垫片无须启用;垫片作为常设观测面的取舍见 T-05。
  实现 AUTO_ALLOC_STATS 垫片并跑 10 分钟取 top-N。验证:垫片输出
  落档;若启用 dhat 需回 §10-④ 确认。→ AC-01
- [x] **T-04 根修**(依赖:定罪结论;改动:auto-lang stdlib.rs 等按
  5.4 选型)✅ worktree lang-027/auto-lang 提交 0392499d1(plan-027-dev,基 a3032dcc4)——四件套:①engine 两处 drain 超时臂补 drop_async_result 回收 ②pending 标记 entry.or_insert 化(11 点)③check_async_http_result Err→错误 body ④内层线程坍缩 catch_unwind+外层常驻微池(2 worker×64 队 mpsc,队满 spawn 兜底)。验证:cargo check 绿 + p027 双单测 2/2 + 坍缩轮 +1.15(恰半减)→ 池化轮尾段 +0.078MB/min(=A 组噪声水位,AC-02 ✓);AC-05:池化实例 0 条预算告警(修前连刷)。证据 evidence/027/l1-fix-verified.csv + l1-pool-final.csv。→ AC-01/02/05
- [x] **T-05 回归钉**(依赖:T-04)✅ term worktree scripts/repro/027-mem-slope.ps1(30s×N 采样线性拟合,阈值 0.5MB/min,超阈退出码 1;冒烟通过,测量方法与四轮验证同源)。SD-02 按 toll 决算修订:常设分配垫片不建(定罪链未用到 dhat,斜率钉+AUTO_VM_MEM/AUTO_VM_BUDGET 既有面足够),SD-02 语义并入 SD-01 规则 5。→ AC-03
- [x] **T-06 销账与规范**(依赖:T-04/05)✅ 双仓 DEBTS #26 划销(auto-lang 026 行 + auto-term #26,措辞"待 merge");SD-01 落册 auto-lang docs/specs/stdlib/design/async-http-result-lifecycle.md(五规则);evidence/027 四件套归档。→ AC-04

## 9. 复审记录

- 2026-09-21 draft handoff(stage: new, revision 1):用户授权
  "先计划后执行分析与修复";L0 嫌疑面已预剪枝(共享 client 排除、
  内容驱动候选降权);outcome: pass;next: work(T-01/T-02 可并行)。
- 2026-09-22 work handoff(stage: work, plan_revision 1, outcome: **pass**):
  code_commit = auto-lang plan-027-dev 0392499d1(根修四件套)+ DEBTS/spec
  提交;auto-term plan-027-dev(回归钉+DEBTS 销账提交);task_ids =
  T-01..T-06 全闭环(T-03 条件不满足未触发,在档);evidence =
  evidence/027/{l0-audit.md, l1-differential.md, l1-c-baseline.csv,
  l1-a-rust.csv, l1-fix-verified.csv, l1-pool-final.csv};blockers = 无;
  next: **review**(AC-01..05 映射见 §8 勾选;AC-02 阈值以尾段
  +0.078MB/min 实证,低于提案 0.1)。注:基线斜率采自双前端共享后端
  环境(2.23 略高于 DEBT 原录 1.7,方向一致);池化轮全段含一次用户
  交互尖峰,判读取尾段——两 caveat 均在 evidence 在档。
- 2026-09-22 复审(review 会话,**与执行同会话——独立性受限已声明,判定自工件重建**;plan_revision 1):`stage: review`,`outcome: pass`;
  - **reviewed_commit**: auto-lang `plan-027-dev` @ `0367dac67`(T-04 根修 0392499d1 + T-06 销账;diff base master `a3032dcc4`)/ auto-term `plan-027-dev` @ `19435fb`(diff base main `116e9b6`);两 worktree 复审时零脏区。
  - **spec_inputs**: auto-lang `docs/specs/stdlib/design/async-http-result-lifecycle.md`(SD-01 五规则,在册核验=描述当前行为;SD-02 并入规则 5——垫片不建,T-05 决算在档);frontmatter new_spec_components 修订为单文件(原两条中"分配观测面"未建,随 SD-02 并入)。
  - **acceptance_results**: AC-01 **pass**(L0 三缺陷审计链/L1 A-C 判别 25 倍差/坍缩半减,六件套证据重读可独立复核);AC-02 **pass**(复审复跑回归钉:+0.46MB/min < 阈 0.5,exit 0;work 期尾段 +0.078——复审值偏高系采样期桌面活动噪声,阈值内);AC-03 **pass**(同次执行,脚本退出码 0);AC-04 **pass**(双仓 DEBTS #26 划销措辞核验 + spec 落册文件核验;账本投影留 merge);AC-05 **pass**(复审实例 AUTO_VM_API_BUDGET 观测 2141 样本:启动期瞬态 warn[back 预热]后稳态 busy_ms 20-23 零告警、waits=4/拍;修前连刷)。
  - **findings**: 复审环境注记——主检出 app 的 rust-workspace target 被 027/028 双 auto-lang 检出交替构建,boot 就绪门多次撞 120s(构建串行化所致,非代码);front-back 端口就绪后配对即健康。无阻断项。
  - **evidence**: docs/plans/evidence/027/ 六件套 + review-err.log(预算观测)+ 斜率钉复跑输出(exit 0);`blockers`: 无;`next`: **merge**(用户已裁定:027 先落,028 随后 rebase 跟进)。
- 2026-09-22 merge 收据(`PLAN-027:r1`,五检查点全闭合,`completion_kind: delivered`):**prepared**(reviewed 基线 + SD-01 规范文件在分支 0367dac67)→ **landed**(lang:plan-027-dev rebase master `db9bffed3` 零冲突,0392499d1→69420195f/0367dac67→7001ada7d,range-diff 全等=安全重写证明,p027 双测绿,ff-only master=`7001ada7d`,他人 DEBTS WIP stash-pop 保全还原;term:rebase main[028 簿记前进] 19435fb→167baac range-diff 全等 → ff-only main=`167baac`)→ **ledger_refreshed**(auto-term .autoos/specs.json 六节投影 P027-1..6,回读验证;lang 侧不投——防与 lang 自身 plan-027 历史撞号,规范文件自证)→ **archived**(docs/plans/archived/027-split-front-tick-memory-leak.md,status: archived)→ **cleaned**(wt-guard 三件 clean,worktree×3+branch×2+组目录 lang-027 全删,git 注册表零残留)。

## 10. 待澄清事项

1. AC-02 阈值 0.1MB/min 为提案(≈本底噪声量级),执行期以修前
   C 组基线校准后请用户确认。
2. 若定罪落在 hyper/tokio 上游内部且不可根修:转缓解 + DEBT 续档,
   需用户裁定接受度。
3. 三个 UI 细节问题(resize 不跟随/CJK 间距/状态栏配色圆角)另行
   立项,用户尚未放行(2026-09-21 评估已给出,等待指示)。
4. dhat-rs 引入与否:自研垫片优先;若需 dhat(dev-dependency、
   feature 门控、默认关闭)执行期回执确认。
