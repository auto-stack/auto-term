# PLAN-026 调查笔记 · VM 前端合并态挂起(栈转储定位)

2026-09-21 下午 · 调查会话 · 接 025 §10.10 战况

## 已确证事实链

1. **复现**(vm-front-out2/err2,前端 PID 9716):合并态(master 13:37 ff 含 025)
   VM 前端 split 启动 → back ready(17401)→ 数据回流后 **Responding=False**、
   CPU 烧(~19s/100s;另一实例 7896 十余分钟 162s)、tick 完成速率 **0.7/s**
   (每拍 ~1.4s;目标节拍 50ms)。后端缺席时前端正常(上会话事实 #3 复证)。
2. **栈定罪**(cdb-stacks.txt,非侵入附加):主线程 =
   `iced about_to_wait → DynamicComponent::fire_timer → on_with_input_for
   → VmBridge::call_handler_for → AutoVM::call_fn_by_name → std::thread::sleep`
   ——engine.rs:2123-2137 `StepResult::Yield` 分支:**VM api.*(HTTP)在同步
   handler 语境 = 5ms-sleep 忙等轮询 ASYNC_RESULTS,deadline 30s(b639ffccf)**。
   烧 CPU 主谋也是主线程(runaway user time 第一/第二均在 VM 执行链);
   reqwest-internal-sync-runtime 线程为共享客户端 runtime(正常)。
3. **每拍调用面**(app.at 静态):Tick 体 68 次 api.*(含 6 槽 rect/pane 全量
   拉取臂);门控 = layout_version 或窗口 w/h 变化才进全量臂(020 F1)。
   挂起终态日志:CALL/EXEC 连发无 OK(Tick 堆积,817 调用:409 完成)。
4. **后端无罪**:curl 直测(合并态 back,17402)——简单端点 2.5-3ms、
   pane-lines(POST)4.3ms、rect-kind 5/11/29ms(29ms=epoch 失效重算尖峰,
   稳态命中 ~5ms);tick 2.9ms(上会话)。**慢在前端进程内,不在网络/back**。
5. **025 期同症状首发实录**(auto-term 3b1ed7d 09:21 提交信息):
   "16ms 计时器每拍同步往返占死 VM 前端 UI 线程(无响应+内存爬升,用户
   实录)"——当时修法 = tick 50ms(**缓解非根修**,架构缺陷仍在)。
6. **app.at 侧零合并增量**:3b1ed7d(纯 025 期)与当前 HEAD 的 Tick 体 api
   计数完全一致(149/149,Init 38/38)——024 状态栏未给 tick 加调用。
   合并增量(若为放大器)锁定在 auto-lang(前端二进制/back 生成物)侧。
7. 上会话对照(在案,本会话未重验):premerge e6053171e(=master 0020accb6,
   纯 025,无 024 delta)正常;纯 024 master 正常;仅合并态挂。
   024 几何注入两提交(f592fa466/0e9a14340)均在桌面轨路径,VM 轨"零变化"
   (shim 读序 override→theme 回落,VM 轨回落 theme=WindowResized 事件写,稳定)。

## 根因定性(判定了义)

- **第一因(架构,025 域)**:split 形态 VM 的 api.* = 主线程同步 HTTP 忙等
  (call_fn_by_name Yield 分支 5ms 轮询)× 每拍 20-68 次调用。每拍主线程
  阻塞数百 ms-1.4s → iced 消息泵饿死 → Windows 判未响应;CPU 烧 =
  sleep/wake 轮询 + 每拍 VM 全量执行 + tick 堆积。025 的 3b1ed7d(16ms 挂)
  即同缺陷首发;50ms 只是压到临界下。
- **第二因(触发器,024×025 合并增量)**:合并 delta(auto-lang 侧)把每拍
  成本推过"50ms 节拍下的可泵消息线"。premerge 与合并态的 back 生成物/
  运行时库差异待 C 实验闭环(premerge back 的 rect/端点延迟矩阵未测)。

## 战场事故与军规(本会话新增)

- **cdb -pv 非侵入附加会杀死目标进程**(9716、7896 两次实录;qd 分离后进程
  随即死亡)——一次附加 = 一次实验终结,采样要在附加前完成。
- **Windows 允许 rename 运行中 exe 但禁止 delete**:cargo 链接替换 auto.exe
  报 os error 5 = 有进程在跑该 exe(本例:merge 会话 14:04 起的验证实例
  27848/13916 锁住主 checkout auto.exe,不可杀非本会话进程)。
- **主 checkout target 是共享战场**(merge 会话 cargo test / 验证实例),
  调查构建一律走独立 worktree。
- **auto-lang workspace 的 autodown-core 依赖相对路径 ../../../auto-down**:
  worktree 构建需 sibling——junction `D:\autostack\.wt\auto-down →
  D:\autostack\auto-down` 已建(**调查收尾须摘除**)。
- 复现实验一律独立端口(-B 17402/17403),不碰 17401(他人实验域);
  孤儿 back 链(app-back.exe + cargo 父链)按 PID 清杀,勿按名杀
  (场上常驻他人 auto.exe:lang-672/lang-039/musk-080 各线)。
- merge 会话与调查并行:lang-025 worktree 已被 merge 流程清理(13:37 ff,
  13:5X 目录删除);auto-term main 13:29 reviewed → 13:37 后落地。

## 工件清单(evidence/026/)

- launch-vm.ps1 / quant-run.ps1 / launch-vm3.ps1:三个启动脚本(17401/
  17402/17403 变体,P024_TRACE 版)
- vm-front-out/err(第一次,back 编译失败 E0433=master 缺门控时实录)
- vm-front-out2/err2(PID 9716 完整复现日志;5526 行,tick 409 完成)
- cdb-stacks.txt(PID 9716 全线程栈 + !runaway;300KB)
- q-merged-out/err(PID 7896;back 70s 构建实录 + "60s continuing anyway")
- cdb-7896.txt / cdb-7896-t2.txt(PID 7896 两次附加;线程 2 = reqwest runtime)
- t3-out/err(PID 20396;13:32 坏 exe 链接失败实录 term.config_spawn_program)
- app-025-only.at(3b1ed7d 期 app.at 导出,对照用)

## 待闭环(C 实验,worktree inv-026 构建中)

premerge(0020accb6)二进制 + 同法画像:tick 速率 + 端点延迟矩阵 +
(若正常)与合并态逐项差分 → 合并增量定罪到具体机制(候选:back 生成物
rect 路径、shim 链、VM 调度)。构建:P024_TRACE 实验待合并态好 exe。

## C 实验终判(2026-09-21 14:2X-14:4X,app-lab 独立实验场)

实验场:evidence/026/app-lab(app 拷贝,path 依赖绝对化,独立
rust-workspace target——绕开共享 app-back.exe 锁;AUTOTERM_ENGINE_DLL
指向 auto-term/target/debug/autoterm_core.dll;端口 17404)。二进制:
inv-026 worktree(master 22b650a76,合并态 + PDB)与同树构建后备份的
auto-premerge-026.exe(checkout 0020accb6=premerge e6053171e 落地形态)。
两版 back 同库(主 checkout master)——唯一变量 = 前端二进制。

1. **premerge 初跑卡 Init**:冷构建 back 98s > 60s ready 等待 → Init 的
   api.* 撞无监听端口(127.0.0.1 无监听 ~2s 失败/次)→ Init CALL 无 OK
   且 Tick 不进(CPU 19% 窗口响应)。**启动时序巧合,非版本差异**
   (A 的 Init 恰在 back Running 后——60s 等待吃掉构建期)。
   lab-premerge2:*;lab-run.ps1。
2. **premerge 重跑(back 预构建,lab-premerge3)**:**RESP=False(挂起!
   CPU 25%)、tick ~8-9/s(每拍 ~115-125ms,4 行/tick ≈ 175B 折算)、
   795 个 VM_HANDLER_OK、速率 4 分钟不衰减**。curl 同 back:
   rect-kind 3.6-4.1ms、layout-version 3ms(与合并态 back 同速——
   "rect 慢是合并特有"不成立,4ms 是 POST+HTTP 基线)。
3. **A 账本修正**:9716 从 back ready(~13:17:30)到 cdb 附加(13:19)
   仅 ~2 分钟,409 OK/120s ≈ **3.4/s(295ms/拍)**;13:31 后的 0/s 尸态
   = back(13020)被 musk-080 runner 清杀后 api 全部连接失败的尸体,
   非自然恶化。817:409 的 CALL:OK ≈ 2:1 尾部堆积同尸态。
4. **vm+vm 对照(--server=vm)**:合并态单进程双 VM + loopback HTTP——
   **卡启动**(主线程 run_backend join back-server 线程死等,cdb 栈
   在 Thread::join;vmvm-* 日志)。另一独立症状,不在本挂起主线,
   计划外记档。

### 终判

- **挂起 = 025 域架构缺陷,纯 025 复现**:split 形态 VM 的 api.* 在
  同步 handler 语境 = 主线程 5ms-sleep 忙等(engine.rs:2123 Yield 分支)
  × 每拍 20-68 次调用 × 每次 (HTTP 2-4ms + 轮询滞后 ~2.5ms + 偶发
  重算尖峰)→ 每拍 115-295ms 主线程阻塞 → 50ms 节拍下消息泵长期饥饿
  → **Windows 判未响应(RESP=False)**;CPU ~25% 烧在 sleep/wake 轮询
  + 每拍全量 VM 执行。premerge(125ms/拍)与合并态(295ms/拍)同缺陷
  同数量级。
- **上会话"纯 025 premerge 正常"的判定被推翻**(观察巧合:启动初期
  或时序使然;025 期 3b1ed7d 的 16ms 挂起实录即同缺陷首发,50ms 修复
  只是压线缓解)。
- **024 合并非因果**(至多每拍成本 ±2 倍内的环境差);**不路由 024 域**。
- 根修方向(计划展开):①api 批量化快照端点(每拍 1 次 HTTP 拉全量
  rect/pane/tab 面,/api/mux/snapshot 已有雏形);②VM api.* 真异步化
  (handler yield 出 iced update,结果经 IcedMessage 回流,消灭主线程
  忙等);③忙等改预算让出+背压(防御层,防再发作)。
