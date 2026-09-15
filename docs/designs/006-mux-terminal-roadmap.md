# 006 — 多终端路线蓝图:从 Mux Core 到类 WezTerm/Kitty 完整产品

> 性质:**讨论沉淀型设计记录**(2026-09-15 用户裁定,非计划产物)。
> 关联:PLAN-018(Mux Core,已交付归档)、
> docs/specs/terminal-mux-model.md(SD-01 契约)、005(三形态契约)、
> 外部八相分析(原文锚定于 PLAN-018 §0/§4.4);ledger 条目
> D006-2(goals)/D006-4(designs)。

## 0. 裁定记录(用户)

- **2026-09-14**:外部分析(ChatGPT 八相方案)采纳为多终端架构路线,
  PLAN-018 切片其 Phase 1(Mux Core)落地——已交付归档(2026-09-15,
  merge 收据在案)。
- **2026-09-15 包含关系裁定**:单纯单终端形态**永久保留**(交付为
  `auto-term-uno.exe`),多终端壳作为**主 app** 交付、占用
  `auto-term.exe` 名——两个 exe,包含而非替换。
- **2026-09-15 路线确立**:本蓝图七计划入档;**下一个计划 = ①
  可见多终端 UI**。

## 1. 已落地现状(Phase 1 = PLAN-018)

五层对象模型(Workspace/Tab/LayoutTree/Pane/TerminalRuntime)、
Action 单入口、引擎 spawn_ex(cwd/argv)、泵面 per-key/per-handle
隔离、`/api/mux/*` 控制面、配色 scheme 面(classic-dark/light)。

**关键缺口**:GUI 仍渲染单焦点 Pane 全屏——多终端**在模型与 API
层是真的,在界面上还看不见**;控制面无 socket/CLI;会话无持久化。
这正是本蓝图其余计划的动工面。

## 2. 路线图(七计划)

| 序 | 计划 | 外部 Phase | 交付面 | 状态 |
| --- | --- | --- | --- | --- |
| ① | **可见多终端 UI** | 6(渲染半边)+ 018 遗留 UI 债 | Tab 条渲染、分屏渲染(View 消费 LayoutTree 做真实布局)、末 Pane 关闭产品语义、scheme 选择器进 settings、tmux 式快捷键(分屏/切 Tab/zoom) | **下一计划(已立项排头)** |
| ② | **双 exe 打包/交付形态** | 新增范围(本蓝图,用户裁定) | `auto-term-uno.exe`(单终端保留)+ 主 **`auto-term.exe`**(多终端壳);部署收口:引擎 DLL/`autoterm-ctrlc.exe` 同目录、launcher `exe_name` 对齐(pac.at 挂点现成) | 待立项 |
| ③ | Shell Integration / OSC 7/133 | 2 | 分屏动态 cwd 继承、CommandBlock 语义层 | 待立项(排序约束见 §4) |
| ④ | Workspace 持久化 + Mux Daemon | 3+7(合并) | session 文件(定义/运行分离,kitty session.py 先例)+ detach/attach | 待立项 |
| ⑤ | Control API + CLI | 4 | IPC socket + CLI + 权限分级(`/api/mux` 为雏形但无 socket/CLI) | 待立项 |
| ⑥ | Domain 抽象 | 5 | WSL / SSH / Container(SpawnSpec.domain) | 待立项 |
| ⑦ | Agent / Extension API | 8 | 结构化 execute/事件流 | 远期可选 |

节奏口径:**①+② 到"能当主终端用的产品";③–⑥ 到"完整类
WezTerm/Kitty"**;⑦ 远期。

## 3. 双 exe 打包裁定(② 详述)

- **命名**:单终端 = `auto-term-uno.exe`;多终端主 app = `auto-term.exe`。
  主 app 概念上即现 `app/` 统一工程(017 入口统一,桌面 launcher 已以
  "auto-term" 装载);② 将其正式化为可分发 exe 交付面。
- **待澄清(归计划 ② 裁定)**:uno 的源码基座二选一——
  (a) `crates/autoterm-ui` 重打包更名(功能完备但受 PLAN-009 冻结
  约束:只修对拍阻断项,不加产品功能);(b) `app/` 工程单 Pane 形态
  出第二 exe(与主 app 同源演进,零冻结债)。两案对 Oracle 纯净性、
  维护成本各有得失,立项时提请用户裁定。
- **部署面**:016 教训——`autoterm-ctrlc.exe` 与主 exe 同目录,缺失
  降级不崩;引擎 DLL 解析序(env → exe 同目录 → 祖先 target)。
  ② 的交付物须自带自洽目录布局。

## 4. 排序约束与节奏

1. **① → ② 建议顺序**:功能闭环先行、形态收口随后;② 工作量小,
   若用户裁定"名字先落位"可先行,二者无代码触面冲突。
2. **OSC(③)先于 SSH/Domain(⑥)**:018 §4.4 已采纳外部排序理由
   (Shell Integration 先于远程 Domain),维持为路线约束。
3. **④ 合并 Phase 3+7**:session 定义/运行分离与 detach/attach 同属
   会话生命周期,合一个计划立项(kitty 先例:session.py 定义态 /
   运行态分离)。
4. 每计划独立走 auto-plan 流(new→work→review→merge);依赖冲突
   触面(auto-lang widget 侧为主)按 017/018 先例串行、先 merge 者为基。
5. 遗留债顺带面:DEBTS #17 同族(badge/preedit 标签渲染,015 根修
   MENU_PARAS 后注记待修)随 ① 的 widget 工作面顺带清偿;R015-F2
   像素金样环境漂移维持独立跟踪,不阻任何计划。

## 5. 勘读注记

- 外部八相分析原文**不入本仓**(会话输入),其完整映射以 PLAN-018
  §4.4 表为准;本蓝图 = 该映射 + 用户裁定(双 exe 包含关系)的合并
  路线,后续以本文件为准。
- PLAN-018 §0"分屏/Tab 条渲染 UI 是紧随其后的下一计划候选"由本
  蓝图 ① 正式承接;① 立项时该表述视为已消化。
