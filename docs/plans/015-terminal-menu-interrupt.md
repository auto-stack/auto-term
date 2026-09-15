---
plan_id: PLAN-015
status: executing
feature_name: terminal 右键菜单 Interrupt 项——恢复引擎中断通路(014 回归修复)
author: [zcode-session]
created_at: 2026-09-13T12:05:00Z
updated_at: 2026-09-15T09:30:00Z
plan_revision: 1
current_step: 3
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
  `menu_item_any_takes_across_terminals`;terminal 套件 31/31 绿。
- T-03 D4+D5 侧车/db/app.at 接线(手工或 auto build 后 codegen 就绪)
- T-04 T-03 完成后集成验证(008 口径三场景)
- T-05 文档收尾(SD-01/02)

## 9. 复审记录

- 2026-09-13 stage:new · outcome:pass(可进入 work;T-01 有前置依赖)
  · next:/auto-plan:work(T-01 前置确认后)。

## 10. 待澄清事项

- ~~T-01 依赖现场 agent 修完 rust codegen 回归~~ **已解除(2026-09-15
  T-00 勘定)**:移交落地于 auto-lang ee2fafa76 并经 017/018 两轮复验,
  当日活体探针 terminal 29/29 绿(证据见 §8 T-00)。
- VM 轨生效依赖 014 VM shim 补全——014 交付时已随 ee2fafa76 落地
  (backlog shim 四件三表登记),018 复审 VM 轨全绿,本计划 VM 臂
  可直接依赖。
