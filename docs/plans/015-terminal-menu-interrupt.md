---
plan_id: PLAN-015
status: reviewed
feature_name: terminal 右键菜单 Interrupt 项——恢复引擎中断通路(014 回归修复)
author: [zcode-session]
created_at: 2026-09-13T12:05:00Z
updated_at: 2026-09-15T09:30:00Z
plan_revision: 1
current_step: 6
total_steps: 6
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
---

# PLAN-015 · terminal 右键菜单 Interrupt 项——恢复引擎中断通路

## 0. 变更摘要

014 直键入收口删除了 013 的 Ctrl+C 按钮,`term_interrupt()`(引擎事件
通路,autoterm-ctrlc 双投递)在前端失去调用者,Ctrl+C 仅剩 0x03 字节
路径——ConPTY 下无法中断任何运行中命令(007 F2 实证)。本计划通过
**terminal 组件右键菜单增加 "Interrupt" 项**恢复显式中断入口:用户
显式触发,不误伤 idle ash(003 §4.1:ash 无 ctrl handler,广播事件会
整体终止 idle 会话——故不能自动双投递)。

## 1. 目标

- G1 恢复中断通路入口:菜单项 → `term_interrupt()` → 引擎双投递
  (Break→C,008 语义)。
- G2 保持 Ctrl+C 键入路径不变(纯字节),idle ash 不会被事件误杀。
- G3 timeout 类命令在菜单 Interrupt 后 ≤5s 内停止(复用 008 门禁口径)。

## 非目标

- ping 类在 26200 build 不可中断(Break 被特殊处理、C 被吞,DEBTS #12
  残留)——维持文档化,非本计划可修。
- Ctrl+C 自动双投递/前台 shell 检测(引擎无法感知 cmd 内手敲 ash)。
- vue 轨交互(009 #4 留白维持)。

## 2. 架构方案

```
widget 右键菜单(第4项 Interrupt=载荷3)
  → terminal_set_menu_item(core,3) + on_menu 消息
  → app.at .Menu 处理器 → db.term_menu_take() 取载荷
  → 载荷3 → term_interrupt() → engine_interrupt → autoterm-ctrlc 双投递
```

需三层接线:① widget 菜单层扩项+registry 级取载荷 accessor(现
`terminal_take_menu_item(&core)` 需 core 引用,宿主泵模式需要
`terminal_take_menu_item_any()`,镜像 `terminal_drain_all_inputs`);
② a2r codegen terminal 臂支持 `onmenu` 事件(现硬编码
`on_menu: None`,rust.rs:2508);③ at-app app.at/db.at/侧车接线。

## 3. 技术栈

Auto(.at 前端/后端)+ auto-lang(terminal 组件/registry/a2r codegen)
+ autoterm-core(FFI 既有,零改)。rust/vm 双轨同源,vm 轨 shim 依赖
现场 agent 014 VM 布线补全后生效。

## 4. 需求分析与背景调查

- 授权:用户于 2026-09-13 会话明确指定"按推荐修法(菜单式)独立立项"。
- 既有地基(已核实):widget MENU_ITEMS 三项+载荷通道(widget.rs:95,
  terminal_set/take_menu_item);db.term_interrupt/api 路由存活但无调用者;
  008 中断语义与门禁(tests/ctrl_event.rs);DEBTS #12/#13 责任矩阵。
- 缺口(已核实):codegen terminal 臂无 onmenu 事件提取;registry 无
  by-key 菜单载荷 accessor;app.at 无菜单消息。

## 5. 详细设计

| # | 改动 | 文件:符号 | 说明 |
|---|---|---|---|
| D1 | 菜单第4项 | auto-lang `ui/terminal/iced/widget.rs` MENU_ITEMS+menu 绘制/命中 | "Interrupt"(载荷3);菜单宽度/高度随项数自适应已由 len() 驱动 |
| D2 | registry accessor | auto-lang `ui/terminal/mod.rs` terminal_take_menu_item_any() | BTreeMap 首个 Some 即返(镜像 take_any_resize) |
| D3 | codegen onmenu | auto-lang `ui_gen/rust.rs` terminal 臂 | events.get("onmenu"/"on_menu") → on_menu 表达式(对齐 oninput 模式) |
| D4 | 侧车/db | at-app `term.rs` engine_menu_take + db.at term_menu_take() | 返回载荷(0-3),-1/无=0 |
| D5 | 前端接线 | at-app `app.at` | msg+.Menu;菜单载荷3→term_interrupt();其余载荷暂忽略(copy/paste 已有组件内建) |

### 规范增量

| delta_id | add/modify/retire | target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify | DEBTS.md #12 附记 | at-app 无中断入口 → 菜单式入口恢复(014 回归修复记录) | 014 回归收口 | AC-01/03 |
| SD-02 | modify | at-app/README.md 014 边界 | 无中断说明 → Interrupt 菜单项语义与限制(ping/26200) | 用户可见行为 | AC-01 |

## 6. 测试设计

- 单测(mod.rs):take_menu_item_any 多终端取走语义。
- codegen 金样(a2r 期望文件):terminal 带 onmenu 事件产出
  `on_menu: Some(...)`。
- 集成(复用 008 口径):at-app 内跑 `cmd /c timeout 10`,菜单
  Interrupt 后 ≤5s 回提示符;ash idle 按键 Ctrl+C 会话不死(字节路径
  不变);ping 在本机维持不可中断(记录,不断言)。
- 回归:几何随动/最小化行为不受影响(PLAN-014 配方抽查)。

## 7. 验收标准

- AC-01 菜单出现 "Interrupt" 且点击可中断 timeout 类命令(≤5s 回
  提示符,截图/日志双证)。
- AC-02 Ctrl+C 键入路径保持纯字节:ash idle 下按 Ctrl+C 会话存活
  (003 §4.1 不回归)。
- AC-03 db/api 面可观测:term_menu_take 载荷路由有测试或日志留痕。
- AC-04 a2r 金样与单测绿;双仓 cargo check/test 0 error。
- AC-05 文档:DEBTS #12 附记 + README 更新在库。

## 8. 执行步骤

- T-00 开工门勘定(2026-09-15):
  ①**前置确认=T-01 阻塞解除**——014 §10 所记"rust codegen 回归
  (terminal 臂属性退化/i64 不匹配)"已随现场 agent 移交落地
  (auto-lang ee2fafa76,2026-09-13)并为后续 017/018 两轮全链复验
  覆盖(017 G3:`auto build -r rust` Finished 零错;018 复审:rust/vm
  GUI 全新启动+ui_gen scheme 臂复审绿);当日活体探针:auto-lang
  master(d74f34e50)`cargo test -p auto-lang --features ui-iced,
  iced-layout-tests --lib terminal` = **29/29 绿 0 failed**;DEBTS 无
  在账条目。ui_gen/rust.rs terminal 臂现状健康(属性全带 as u16/
  as i32 casts),`on_menu: None` 硬编码缺口与 §4 描述一致=待办非
  回归。②**worktree 勘定**:沿 014/016/017/018 同款"无 worktree
  主检出交付"——auto-term 主检出直接实现;依赖仓 auto-lang master
  主检出直接变更(018 master 直落惯例);其工作区他人遗留项
  (renderer.rs/session.rs 桌面图标拖拽 WIP、fs.at、570 文档)不属
  本计划,保留不提交,仅提交本计划文件(与 WIP 文件集零重叠)。
  ③**路径基准**:017 已统一入口——§2/§5 所记"at-app"即现行
  `app/`(src/front/app.at、src/back/{db.at,api.at}、term.rs 四触面,
  018 T-00 同款裁定),语义修订记录在此,范围不变。
- T-01 ✅ 调查+D3 codegen onmenu(含金样)——前置依赖已解除(见 T-00);
  auto-lang 0ce22c27d:terminal 臂 events(onmenu/on_menu/oncontextmenu/
  contextmenu)→ `on_menu: Some(DirectMsg)`(键集与 VM 臂同源);
  金样 `terminal_onmenu_emits_on_menu_direct_msg` 有/无 onmenu 双臂绿;
  ui_gen 回归 779 绿 0 failed。附注:VM 臂 convert_terminal 原已支持
  onmenu(oncontextmenu/contextmenu/onmenu 链),本任务只补 rust 臂。
- T-02 ✅ D1+D2 widget 菜单项与 accessor(含单测)——auto-lang
  b0e603501:MENU_ITEMS 3→4("Interrupt" 载荷 3,命中臂零改动,宽/高
  len() 自适应)+ `terminal_take_menu_item_any()`(BTreeMap 键序首个
  Some 即返,镜像 take_any_resize);单测
  `menu_item_any_takes_across_terminals`。
  **R015-F1 修复轮重收口(2026-09-15,auto-lang 11a2b9bb6)**:标签
  不可见根因 = iced wgpu fill_paragraph 排队 WeakParagraph,draw 局部
  段落析构后 flush upgrade 失败静默丢弃(行文本因静态缓存存活而正常
  ——全部 forensic 由此自洽);修正 = MENU_PARAS 静态缓存强引用;
  实机四项标签全部可见(含 "Interrupt"),打断链同跑复验;badge/
  preedit 同族记 DEBTS #17 待修。
- T-03 D4+D5 侧车/db/app.at 接线(手工或 auto build 后 codegen 就绪)
- T-04 ✅ 集成验证(008 口径三场景,evidence/015 实机):
  ①字节路径基线——`timeout /t 30 /nobreak` 倒计时中裸 Ctrl+C → 倒计时
  继续(t3 截图,007 F2 基线成立);②菜单事件路径——右键浮层(四项)
  → 点 "Interrupt" → `menu item=3` trace 留痕 + 倒计时 24s 被 `^C`
  打断、≤5s 回提示符(t4/t5 截图);③AC-02——cmd 内启动 ash(idle)
  → Ctrl+C → `echo alive2` 存活(t8 截图);④会话存活 `echo alive`
  (t6);AC-03 日志留痕 = `[term-trace] menu item=3`(AUTO_TERM_TRACE)。
  **边界外顺手项(用户当场指令)**:IME 聚焦默认中文修复(auto-lang
  c9cc31ab3→029d80557,TSF 权威 Shift 切换 + 重试制;用户实测确认
  "不默认中文了"),该修复同时解自动化取证键入被拼音劫持的死结。
  开发布局发现:app exe 住 auto-lang target 时 helper 三级解析扑空,
  脚本设 `AUTOTERM_CTRLC_BIN` 解决(dist 同目录免设,已记 SD-01 附记)。
- T-05 ✅ 文档收尾:DEBTS #12 前端入口附记(SD-01)+ 新增 #17(菜单
  标签 rust 轨 200% DPI 不可见——四轮实验+像素扫描取证,面板 2× 尺寸
  偏移渲染,013 起既有,归 auto-lang 像素台专项)+ #18(a2r 块尾值
  调用缺分号 E0308,app.at 已 () 型收尾规避);app/README 边界节更新
  (SD-02);本计划 §9 work 记录与 frontmatter 终态。

## 9. 复审记录

- 2026-09-13 stage:new · outcome:pass(可进入 work;T-01 有前置依赖)
  · next:/auto-plan:work(T-01 前置确认后)。

- 2026-09-15 stage:work · plan_id:PLAN-015 · plan_revision:1 ·
  outcome:**pass** ·
  code_commit:auto-lang 0ce22c27d(D3)+ b0e603501(D1/D2)+ c240fb216
  (D4-VM shim 2987)+ c9cc31ab3/f0dc16732/029d80557(IME 顺手项)+ 
  8ac159423(菜单 chrome 配色);auto-term 59cc630(D4/D5 接线+T-00) ·
  task_ids:T-00..T-05 全勾 ·
  evidence:evidence/015(t1-t8 截图 + trace.log `menu item=3` +
  t04_menu_interrupt.ps1 配方);门禁:terminal 套件 31/31(nextest)、
  ui_gen 779 绿、catalog 完整性锁 3/3、双 feature 臂 check 绿、
  `auto build -r rust` 全链 Finished、集成三场景实机过 ·
  blockers:无 ·
  findings(移交复审裁量):F-1 菜单标签 rust 轨 200% DPI 不可见
  (013 起既有,DEBTS #17,功能链不受影响);F-2 a2r 块尾值调用缺
  分号(DEBTS #18,已规避);F-3 IME 英文起步顺手项交付(用户需求,
  边界外,DEBTS 无账/提交在案) ·
  next:/auto-plan:review。

- 2026-09-15 stage:review · plan_id:PLAN-015 · plan_revision:1 ·
  outcome:**needs_fix** ·
  reviewed_commit:auto-term 3ae065a · base_commit:auto-term c2dc03d ·
  dependency_revisions:auto-lang 0326241b5(本计划 7 提交 0ce22c27d..
  8ac159423 全在祖先链;主检出有并发 agent 在途 WIP,已清点不属本
  计划) ·
  spec_inputs:DEBTS.md(#12 附记/#17/#18)、app/README.md;**无
  docs/specs 触面说明**:菜单载荷通道为 widget-registry 层,不动
  19 符号引擎 FFI face;VM shim 2987 登记待 merge 时随
  terminal-mux-model spec-sync 补记(F-4) ·
  acceptance_results:AC-01 **fail**(可见性子项——标签不可见,功能链
  通)/AC-02 pass(t3 倒计时免疫+t8 ash 存活+代码面:interrupt 唯一
  调用者=菜单载荷 3,键盘路径零触及)/AC-03 pass(trace `menu item=3`
  + D2 单测)/AC-04 **partial**(a2r 金样绿、单测绿、双臂 check 绿、
  workspace 全绿 0 fail、build Finished;像素金样 2 红=环境漂移,
  见 F-2)/AC-05 pass(DEBTS #12 附记+README 在库) ·
  findings:
  **R015-F1(high,AC-01/T-02)**菜单标签 rust 轨不可见→needs_fix;
  取证:fill 循环执行+坐标/颜色/构造四轮排除+面板 2× 尺寸偏移渲染
  (200% DPI),像素台模拟器可离线复现(修法入口);
  **R015-F2(medium,merge 门)**像素金样 selection/cursor 环境漂移
  ——二分定案:f0dc16732(晨绿)洁净 worktree 现亦红,同代码跨树
  复现、与提交零相关(simulator 系统字体栅格化对环境敏感,金样逐字
  节比对);非本计划回归,但 merge 全量门前须环境归因(字体缓存/
  远程桌面/显示设置)后复跑;
  **R015-F3(low,证据卫生)**evidence/015 的 t4/t7 为中途跑残影
  (中间跑失败打破脚本全量覆盖前提),与 t2/t5/t8 终态跑不一致;
  F-1 修复轮重跑生成一致全套后以新帧为准;
  **R015-F4(info,merge)**VM shim 2987 使 018 spec "shim 2983-2986"
  记录过期,merge spec-sync 补记 ·
  evidence:docs/plans/evidence/015/(8 帧+trace.log+脚本)、
  auto-lang 提交 0ce22c27d/b0e603501/c240fb216/c9cc31ab3/f0dc16732/
  029d80557/8ac159423、auto-term 59cc630/3ae065a、洁净 worktree
  nextest 输出(HEAD 与 f0dc16732 各 29/31+2 环境红)、auto-term
  workspace 全绿 0 fail ·
  复审限制声明:同会话复审,结论以工件重建(提交/截图/trace/测试
  输出),未采信执行者摘要 ·
  next:/auto-plan:work(R015-F1 标签可见性;修后 F-3 重取证,
  F-2 环境归因后复跑全量门)。

- 2026-09-15 stage:work(修复轮) · plan_id:PLAN-015 ·
  plan_revision:1 · outcome:**pass** ·
  code_commit:auto-lang 11a2b9bb6(R015-F1 根修)+ 8ac159423(前置
  chrome 配色+插桩,上一轮已入) · task_ids:T-02(重开部分) ·
  evidence:evidence/015 重取证全套(t2 打字零乱码/t4 四项标签可见
  含 "Interrupt"/t5 24^C 打断回提示符/菜单 item=3 trace;像素扫描
  面板内标签亮像素 3461);根因:iced wgpu WeakParagraph 静默丢弃
  (详见 DEBTS #17 修复节) ·
  blockers:F-2 像素金样环境漂移仍红(晨绿午红二分已定案非代码,
  merge 全量门前的环境归因独立完成,不阻本计划) ·
  next:/auto-plan:review(F-3 已满足:全套 8 帧一致重取证)。

- 2026-09-15 stage:review(修复轮再审) · plan_id:PLAN-015 ·
  plan_revision:1 · outcome:**pass** ·
  reviewed_commit:auto-term 0112052(修复增量 936bc90..0112052,代码
  面仅 auto-lang 11a2b9bb6) · base_commit:auto-term 936bc90(前次
  review) ·
  dependency_revisions:auto-lang 87eba67ab(11a2b9bb6 在祖先链;主
  检出已洁净,并发 WIP 已落地;另注 87eba67ab 起本仓 L0 改动改走
  worktree 为后续计划约束) ·
  spec_inputs:同前次(DEBTS #12 附记/#17 根修/#18、app/README;无
  docs/specs 触面说明维持,F-4 shim 2987 spec-sync 归 merge) ·
  acceptance_results:**AC-01 pass**(独立复现:t4 菜单四项标签全部
  可见含 "Interrupt",点击 → 25s `^C` 打断 ≤5s 回提示符,截图+trace
  `menu item=3` 双证——前次 fail 项闭合)/AC-02 pass(修复轮未触键盘
  路径,前次证据维持+本轮 trace 干净键入)/AC-03 pass(trace 同上)/
  AC-04 pass(本计划触面:menu 单测绿、a2r 金样绿、双臂 check 绿、
  auto-term workspace 0 fail——3ae065a 后 auto-term 零代码改动;
  像素金样 2 红=R015-F2 环境漂移非回归,维持 merge 门前置)/AC-05
  pass(DEBTS #17 根修节+#12 附记+README 同步在库) ·
  findings:R015-F1 **resolved**(独立复现确认)/R015-F3 resolved
  (全套一致重取证)/R015-F2 维持(merge 前置:环境归因+全量门复跑)/
  R015-F4 维持(merge spec-sync);新增无 ·
  evidence:docs/plans/evidence/015/(复审当日独立复现 8 帧+trace)、
  auto-lang 11a2b9bb6/87eba67ab、auto-term 0112052 ·
  复审限制声明:同会话复审,以独立重跑+工件重建定案 ·
  next:merge(merge 门:F-2 环境归因后全量复跑;F-4 spec-sync)。

## 10. 待澄清事项

- ~~T-01 依赖现场 agent 修完 rust codegen 回归~~ **已解除(2026-09-15
  T-00 勘定)**:移交落地于 auto-lang ee2fafa76 并经 017/018 两轮复验,
  当日活体探针 terminal 29/29 绿(证据见 §8 T-00)。
- VM 轨生效依赖 014 VM shim 补全——014 交付时已随 ee2fafa76 落地
  (backlog shim 四件三表登记),018 复审 VM 轨全绿,本计划 VM 臂
  可直接依赖。
