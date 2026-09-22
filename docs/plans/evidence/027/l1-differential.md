# T-02 L1 无重建二分实验——A/C 判别 + 斜率基线

日期:2026-09-21/22 | 执行:zcode 会话(PLAN-027)
采样:Get-Process WorkingSet64/PrivateMemorySize64,每 30s,线性拟合。
环境:同机同后端(127.0.0.1:17401,app-back)、同 tick 节拍(50ms×4 请求)、
同 Iced 渲染栈;auto CLI = master a3032dcc4 重建(修前基线件)。

## 结果

| 组 | 前端 | 进程 | 样本数 | 均值 priv | 斜率(priv) |
|---|---|---|---|---|---|
| C | VM 轨(auto run -r vm) | auto.exe | 22 | 378MB | **+2.23 MB/min** |
| A | rust 轨(auto run -r rust,a2r 编译前端,Iced backend) | auto-term.exe | 20 | ~225MB | **+0.09 MB/min** |

原始数据:l1-c-baseline.csv / l1-a-rust.csv(同目录)。

## 判读

1. **DEBT #26 复现成立**:C 组 +2.23MB/min,与 DEBTS 记录 1.7MB/min
   同量级(偏高的解释:双前端共享后端、用户偶发交互)。
2. **A 组平**(0.09MB/min=噪声水位):同 HTTP、同后端、同渲染栈下
   编译型前端不漏 ⇒ **排除**:iced 文本布局/渲染缓存、后端、共享
   reqwest client 层(a2r 前端自己的 HTTP 消费也不漏,进一步排除
   hyper/keep-alive 通用面)。
3. **定罪面收缩到 VM 前端特有层**:stdlib shim 链(spawn_async_http
   双层线程 160/s churn + ASYNC_RESULTS)与 engine drain 机械。
   换算:2.23MB/min ÷(4 请求/拍×20 拍/s)≈ 465B/请求 ≈ 232B/线程。
4. 架构注记(实验设计修正):rust 轨现在同为 split over HTTP
   ("backend: rust-ui"),故 A/C 差分语义 = "编译前端 vs VM 前端",
   不是 "无 HTTP vs HTTP";判别力不受损(两前端各自消费 HTTP)。
5. B 组(断流)未跑:修复验证轮若斜率归零则无须;若有余量再补
   (失败路径恒返回错误 JSON body,worker 恒完成,实验有效性预核过)。

## 下一步

修复验证轮:worktree CLI(plan-027-dev:A 超时回收/B 竞态 or_insert/
C Err 映射/双层线程坍缩)重跑 C 组同法采样,判据 = 斜率回落到
A 组噪声水位(≤0.1MB/min 量级)。

## 修复验证轮(终局)

| 轮 | 固件 | 线程/请求 | 斜率(priv) | 判读 |
|---|---|---|---|---|
| 基线(C 组) | master a3032dcc4 | 2(双层) | +2.23 MB/min | DEBT 复现 |
| 坍缩轮 | plan-027-dev ①②③+内层坍缩 | 1 | +1.15 MB/min | **恰半减 ⇒ 斜率∝线程创建数** |
| 池化轮(终局) | plan-027-dev 全量(0392499d1) | 0(常驻 2 worker) | 尾段 **+0.078 MB/min**(全段 +0.418 含一次用户交互尖峰) | **= A 组噪声水位(0.09),泄漏归零** |

数据:l1-fix-verified.csv(坍缩轮 11 样本)/l1-pool-final.csv(池化轮 21 样本,
其中样本 10-12 为用户交互尖峰 +3.2MB——内容型合法增长,尾段 12 拍逐字节
平稳)。AC-05 顺带:池化实例全程 0 条 VM-API-BUDGET 告警(修前 100-143ms 连刷;
线程创建开销原占 drain 延迟大头)。B 组(断流)/T-03(dhat)未启用——定罪链
(L0 三缺陷 + L1 判别 + 坍缩半减 + 池化归零)已闭合,无须补充。
