# T-01 L0 读码审计——ASYNC_RESULTS/HTTP_RESPONSES 结果通道回收完整性

基线:auto-lang master a3032dcc4(stdlib.rs/engine.rs 行号以此为淮)
日期:2026-09-21 | 审计人:zcode 会话(PLAN-027)

## 通道拓扑(api.* 实际路径,AutoTerm Tick 每拍 4 请求)

```
VM CALL (http.get_json/post_json)
  → shim_http_get_json (stdlib.rs:7149)
      首轮: pop url → alloc_async_id()
            → spawn_async_http(:6865)  [线程①]
                 → simple_http_json(:6746)  [内部再 std::thread::spawn = 线程②, join]
                     → shared_api_http_client().clone()  ✓ 共享 client(Plan 446)
            → ASYNC_RESULTS.insert(req_id, None)   ← 竞态点 B
      task.waiting_http_request_id = Some(req_id); yield
  → engine call_fn_by_name Yield drain (engine.rs:2199)
      while !async_http_result_ready(req_id) { 5ms sleep; 30s deadline }
        超时 → task.waiting_http_request_id = None; return Err   ← 泄漏点 A
      就绪 → shim 重入 → check_async_http_result(:6818) map.remove ✓
```

## 确证缺陷

### A. 超时放弃路径条目泄漏(两处同构)
- engine.rs:2199-2207(call_fn_by_name drain)与 engine.rs:7027-7043
  (request-builder send 同步完成臂):30s 超时只清 `task.waiting_http_request_id`,
  **ASYNC_RESULTS[req_id] 不删**。worker 完成后写入 `Some(Ok(Body(resp)))`
  完整响应体,永驻。每次超时泄漏 ≈ 一个响应体(2-8KB 级)。
- 触发频率:低(本地往返 ms 级,30s 超时罕见)。**不足以解释稳态 1.4KB/拍**。
- 修法:超时臂补 `drop_async_result(req_id)`(或 check_async_http_result
  无视就绪直接 remove)。

### B. pending 标记覆盖竞态(get/post/put_json 系列)
- stdlib.rs:7170-7173 一带:`spawn_async_http(...)` 先启动,主线程随后
  `map.insert(req_id, None)`。worker 完成先到则被 None 覆盖 →
  `async_http_result_ready` 恒 false → 必然走 A 的 30s 超时 → 泄漏 + 挂起 30s。
- 窗口:本地 ms 级往返 vs 主线程 µs 级 insert——窗口极小,偶发。
- 修法:先 insert None 再 spawn。

### C. Err 条目无限等待(handle 型路径)
- `check_async_http_result`(:6818)对 `Err` 变体 `.ok()` → None,但条目
  已 remove → shim 判"仍在等"→ 永远 Waiting(挂起,非泄漏)。
  json 路径 spawn 恒插 Ok(Body)不触发;handle 型路径(send_with_retry
  失败插 Err)可触发。AutoTerm 不用 handle 型 natives——降级为连带修。

## 排除项

- **每请求 Client::new()**:仅 upload/download/request 等 natives
  (stdlib.rs:5856/5898/5924/6163/6188...),api.* 的 json 路径走
  `shared_api_http_client()` 共享(Plan 446 E4)✓ 排除。
- **VM 池**:AUTO_VM_MEM 实证稳定(026;本计划 C 组将再采集旁证)✓。
- **行段落缓存**:digest 门控 + ROW_CACHE_CAP(widget.rs:172);空闲期
  内容不变不重建 ✓(与"空闲期仍爬"矛盾)。
- **HTTP_RESPONSES thread_local**(stdlib.rs:3889):仅 handle 型 natives
  使用;thread_local 随线程消亡;AutoTerm 不用 ✓ 排除(通道完整性另档)。

## 稳态泄漏头号候选(L0 未定罪,交 L1/L2)

**每请求双层线程 churn**:spawn_async_http 开线程①,simple_http_json
内部**再开线程②并 join**(stdlib.rs:6759)——每请求 2 线程创建/销毁,
空闲期 4 请求/拍 × 20 拍/s ≈ **160 线程/s**。Windows 线程栈/CRT/tokio
EnterGuard 每线程分配,若堆碎片/arena 不完全归还 → 私有字节缓涨。
量级换算:1.4KB/拍 ÷ 4 请求 × 2 线程 ≈ 175B/线程/请求——线程 churn
碎片假说量级吻合。L1-B(断流:失败路径仍发射请求→线程仍 churn)与
L2(垫片按栈归因)可判。

## 结论

| 嫌疑 | 判定 | 依据 |
|---|---|---|
| A 超时泄漏 | 确证 bug,修 | engine.rs 两处 drain 超时臂 |
| B 覆盖竞态 | 确证 bug,修 | insert None 时序 |
| C Err 挂起 | 确证 bug,连带修 | check 的 .ok() 吞 Err |
| 稳态 1.4KB/拍 | 未定罪→L1/L2 | 候选=线程 churn/超低频 A+B 叠加 |
