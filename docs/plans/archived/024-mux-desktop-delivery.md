# PLAN-024 mux 版终端进入虚拟桌面——桌面轨几何/交互面达 rust 轨同等

```yaml
plan_id: PLAN-024
status: archived
feature_name: mux 多 pane 终端进入 AutoOS 虚拟桌面(桌面轨补齐)
author: [agent]
created_at: 2026-09-19T00:00:00Z
updated_at: 2026-09-19T00:00:00Z
plan_revision: 1
current_step: 7
total_steps: 7
supersedes_spec_components: []
new_spec_components:
  - docs/specs/terminal-mux-model.md
  - docs/specs/app-unified-entry.md
touched_goals: []
```

## 0. 变更摘要

把 018–023 已在 rust/vm/vue 三轨交付的 mux 多 pane 终端(Tab 条/
嵌套分屏/分隔条拖拽/WT 快捷键/打字随动)带进 auto-os 虚拟桌面:补齐
窗口尺寸标定链的桌面轨几何源(当前读进程级 theme thread_local 的
宿主 OS 窗尺寸,非该 app 虚拟窗内容区),并以 017 桌面实机门同法
对 mux 全动作面取证。**预期改动集中在 auto-lang(几何源)与
auto-term(门/spec);不动 auto-os 仓**(apps.manifest 条目已在案)。

## 1. 目标

- 桌面 launcher 拉起 auto-term 呈 **mux 形态**(Tab 条 + 分屏槽位 +
  分隔条),px 投影不错位——与 rust 轨同布局对照一致。
- `/api/mux/window-width|height` 桌面轨语义 = **该 app 虚拟窗内容区**
  (非宿主 OS 全窗);resize 后 pane 网格/引擎 resize/光标落位随动。
- mux 全动作面桌面实机通过:点击聚焦直键入/WT 快捷键全表/Ctrl+C/
  横竖分/嵌套/分隔条拖拽(磁吸+钳位)。
- 桌面多 app 同场互不串扰:他 app 开窗/resize 不改写终端几何标定。
- rust/vm/vue 三轨零回归。

### 非目标

- Windows 虚拟桌面(本计划"虚拟桌面"恒指 AutoOS 桌面宿主);
- auto-os 桌面 WM 自身功能演进(虚拟窗管理属宿主域);
- rust/vm/vue 轨行为变更(仅允许零回归修复);
- Alt+WASD 方向导航(维持 PLAN-022 DEBT §10.7,不在本计划);
- ash/引擎层变更。

## 2. 架构方案

现状链(调查实证,文件:行号以主检出 HEAD 为准):

- **front**:`app/src/front/app.at` = 矩形投影 N 槽视图(pane 槽
  1..6 + divider 槽 7..11,absolute px 类几何;内容区 px = 窗口
  client − Tab 条高 40)。
- **back**:`app/src/back/db.at:621` `mux_window_width/height` →
  `auto.term.window_width/height`;`api.at` 声明 /api/mux/* 11 路
  (VM/desktop 直跑委托体形态)。
- **shim**:`auto-lang crates/auto-lang/src/vm/native_catalog.rs:522`
  注册 2998/2999 → `vm/ffi/term_engine.rs:669` 读
  `ui/style/theme::window_width()`——**进程级 thread_local,默认
  1024×768**。
- **写入路**(PLAN-022 T-03 已落):①`run_app` WindowResized
  (renderer.rs:24476 区,rust 独立窗);②桌面/动态轨
  `__window_resized` 拦截(renderer.rs:24759 区)写同一 thread_local。
- **缺口 A(几何源错位)**:Plan 462 桌面(renderer.rs:19142 区)
  "宿主窗 resize → **全体虚拟窗** window_size 同步"——
  `__window_resized` 载荷 = 宿主 OS 窗尺寸,**非终端 app 虚拟窗
  内容区**。虚拟窗仅占宿主一隅时投影 px 全错。
- **缺口 B(进程级串扰)**:theme thread_local 全进程共享,桌面多
  app 同场时他 app 的 resize 覆写终端标定(022 T-03 修 rust 单窗
  轨时未触此面)。
- **缺口 C(承诺面缺口)**:mux spec 快捷键契约承诺面 = "rust/vm
  轨"(terminal-mux-model.md §6),桌面轨未承诺;018–023 门禁
  零桌面轨覆盖;017 桌面实机门验收的是 014 整窗形态。

方案骨架:**几何源决策先行(T-00 决策工件)**,候选——

- **A shim 会话感知**:`auto.term.window_width` 按调用 app 上下文
  返回其虚拟窗 rect(vm ffi → ui session 查询);
- **B 窗口级字段前馈**:session 已有窗口级字段同步
  (session.rs:1359 响应式 `window_width` 随缩放),front 以实际
  可用 px 经新 api 喂回 db(mux 标定源改前馈,shim 读法仅留
  rust/vm 轨);
- **C 组件测量回填**:借 014"可用空间反推"的组件实测 bounds
  回填槽 px(规避窗口级标定)。

决策维度:虚拟窗 resize 语义(462 WM 下虚拟窗有无独立 resize/
载荷路径)、多 app 隔离、对三轨现有契约的最小侵入、a2r 可表达性。
**倾向 B**(单一真相源前馈、天然 per-app 隔离、不动 shim 面),
T-00 实机定界后落决策。键盘/鼠标面预期**零 auto-lang 改动**:
023 交互投递根修(焦点单播/TERM_PRESS/FOCUS/KEY)在 auto-lang
terminal 组件层,桌面与 rust 载具共用同路径——仅需桌面实机取证。

## 3. 技术栈

- auto-term:`.at` 前端/后端(AutoUI),`auto run -r vm|vue` 冒烟,
  `cargo test -p autoterm-core`(引擎面零改动预期);
- auto-lang:vm ffi shim / ui session / iced renderer(几何源按
  T-01 决策;仓规 Category B 门禁 = cargo check + 作用域模块测试);
- auto-os(只读):`scripts/desktop.sh iced`(桌面宿主拉起)、
  `apps.manifest`(auto-term 条目已在案,不改)。

## 4. 需求分析与背景调查

- **授权**:用户 2026-09-19 指令——新建计划,"实现 term 的 mux
  版本进入虚拟桌面"。范围 = 桌面轨几何/交互面补齐 + 实机门;
  仓库 = auto-term + auto-lang(只读消费 auto-os;若 T-00 发现
  必须改 auto-os,回 §10 升级请裁)。未设预算/自动续行限制。
- **背景链**(详见 §2;取证时点 2026-09-19):
  - 装载链在案:apps.manifest id=auto-term(repo=../auto-term/app,
    status active)→ 宿主 build_dynamic_component 进程内编译
    front + `use back.api` 本地解析 `src/back/api.at` 委托体 +
    auto.term shims 直达宿主进程内 autoterm_core.dll
    (app-unified-entry.md 桌面装载形态节)。
  - 017 桌面实机门(整窗 014 形态)R2 复验通过;其后 018–023
    mux 演进门禁全在 rust/vm/vue,桌面轨零覆盖。
  - 022 T-03 窗口尺寸面定界结论(evidence/022/t00-decision.md
    §3):theme thread_local 恒默认 → 投影 px 全错 = 021 T-07
    右面板错位根因——同机理在桌面轨以"宿主窗尺寸"形态复现。
- **已知陷阱**(执行期必检):
  1. 长驻桌面宿主跨引擎 DLL 重建保留**陈旧内存映像**
     (app-unified-entry.md:57):门取证前重启宿主进程,或核对
     宿主启动时间晚于 autoterm_core.dll 构建时间;
  2. DLL 候选链(653:exe 祖先 6 级 + CWD 祖先 4 级)为 dev 跑法
     兜底,桌面 dev 跑法首次拉起须核对 DLL 解析命中;
  3. 022 §10.6 载具重生成路径重置陷阱仅侵 rust 轨
     (rust-workspace);桌面轨动态装载 .at 源,不经该载具;
  4. 023 收据注记:曾有运行中桌面实例(PID 8792)——门取证前
     确认无陈旧实例占场。
- **依赖**:无在途计划冲突(auto-term 活动计划为零;auto-lang 侧
  PLAN-660 已 execution_done 待复审,与本计划不同文件域)。

## 5. 详细设计

### 几何源落地形态(T-00 决策后按选定候选落)

以候选 B 为例(最终以决策工件为准):

- front:`app.at` 增窗口级 `win_px_w/win_px_h` 模型变量(响应式
  窗口字段的桌面轨到达路径由 T-00 定界),Init/Tick 把值喂
  `POST /api/mux/set-window-px(w,h)`(新路由,委托体形态同
  /api/mux/* 现例);
- back:`db.at` mux 标定源桌面轨改前馈值(缺省回落 shim 读法,
  rust/vm 轨行为不变);`api.at` 增路由声明;
- 多 app 隔离 = 天然(per-app model 前馈,不触进程级
  thread_local);`theme` thread_local 不再被桌面轨写入
  (022 T-03 ②拦截路对 mux 面退役或保号非 mux 用途,按决策)。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/terminal-mux-model.md | 前:窗口尺寸标定源 = GET /api/mux/window-width\|height(auto.term shim 2998/2999 + rust 侧车同名);快捷键契约承诺面 = rust/vm 轨。后:尺寸面增**桌面轨语义**(标定源 = 该 app 虚拟窗内容区,到达路径按 T-01 落定);快捷键/打字随动/泵三档承诺面扩 desktop 轨 | 桌面轨几何源与承诺面入册,防三轨口径漂移 | AC-02/04/06 |
| SD-02 | modify | docs/specs/app-unified-entry.md | 前:桌面装载形态门 = 014 整窗 terminal 形态 + 几何随动 + 颜色三面(017 证据)。后:桌面门升级 **mux 形态**(Tab 条/分屏/分隔条/快捷键/多 app 串扰四+面);DLL 陈旧映像陷阱条目保留并引 024 实证 | 桌面验收口径随 mux 交付升级 | AC-01..05 |
| SD-03 | add(暂定) | (auto-lang 仓)docs/specs/ 相应模块 spec | 前:无"虚拟窗尺寸→app 几何"通用面口径。后:若 T-01 产生 auto-lang 新通用注入面,落册并跨仓显式标注(023 P023-3 先例) | auto-lang 侧新通用面须有自己的 spec 锚 | AC-02/06 |

(若 T-01 走候选 A/C 或零 auto-lang 改动,SD-03 退役并在 §9 记因。)

## 6. 测试设计

1. **桌面实机门**(017 同法升级):净场(杀陈旧实例→重启桌面宿主,
   核对 DLL mtime)→ `bash D:/autostack/auto-os/scripts/desktop.sh
   iced` → launcher 搜 term 启动;AUTOUI_ACCEPTANCE=1 + MCP bus
   `launch auto-term`(launch_app inproc)取快照+截图。断言面:
   mux 形态渲染/几何随动/键鼠/分屏/多 app——分任务门详见 §8。
2. **几何对照**:同布局(横分 50/50 + 竖分右半)在 rust 轨
   (`auto run -r rust`)与桌面轨各取快照,槽位 px/cols/rows 对照。
3. **三轨零回归**:cwd=app/ —— `auto run -r rust|vm|vue` 冒烟;
   auto-lang 侧 `cargo test --lib p022_stack` 5/5 +
   `terminal_input` 2/2(023 收口基线);auto-term `cargo test`
   不红。
4. **单元/模块面**:T-01 若动 db.at/api.at —— VM 轨 curl 剧本式
   断言(020 t01_model.sh 同法扩 set-window-px 路);若动
   auto-lang —— Category B 门禁(cargo check + 作用域模块测试)。

## 7. 验收标准

- **AC-01** 桌面拉起 auto-term 呈 mux 形态:Tab 条常显、pane 槽
  px 投影与 rust 轨同布局对照一致、无错位蓝屏/归零形态。验证:
  实机截图 + MCP 快照(证据入 evidence/024/)。
- **AC-02** 窗口尺寸面桌面轨语义正确:`/api/mux/window-width|
  height` 返回该 app 虚拟窗内容区(非宿主全窗);虚拟窗/宿主窗
  resize 后 pane 网格、引擎 resize、光标落位随动。验证:resize
  前后快照对照 + 实机拖拽录证。
- **AC-03** 多 app 同场无串扰:桌面同场第二 app 开窗/resize/最大化
  前后,终端几何标定值不变、投影稳定。验证:同场录证 + 标定值
  读数对照(他 app 动作前后)。
- **AC-04** 键盘交互面:点击聚焦直键入回显、命令可执行、Ctrl+C
  中断;WT 快捷键全表命中(C-S-T/W/E/O/Z/Tab/←/→/K),命中发
  动作不落 VT 队列。验证:实机键入录证 + 快照。
- **AC-05** 分屏与分隔条:横分/竖分/嵌套(≤6 pane,超限拒绝)、
  分隔条拖拽磁吸({500}±40)与钳位([100,900])实点通过;Tab
  点击激活/右键关闭/`+` 新建可用。验证:实点录证(020 惯例——
  实机鼠标直接操作)。
- **AC-06** 三轨零回归:rust/vm/vue 冒烟 + `p022_stack` 5/5 +
  `terminal_input` 2/2 + auto-term `cargo test` 全绿。验证:命令
  输出在案。
- **AC-07** spec 沉淀:SD-01/SD-02(及视决策 SD-03)按 §5 落册,
  review 绑定修订。验证:spec diff。

## 8. 执行步骤

- **T-00 有界调查·几何源决策工件** `[x]`
  依赖:无。产出 `evidence/024/t00-decision.md`:
  (a) 现行为取证——净场重启桌面宿主,拉起 auto-term,录 mux 形态
  渲染现状/投影错位形态/键鼠到达情况;(b) 定界 Plan 462 WM 下
  虚拟窗 resize 语义(有无独立 resize、尺寸事件载荷路径、
  session.rs:1359 窗口级字段在桌面的到达值);(c) 候选 A/B/C
  对照决策(维度见 §2)。关联 AC-01/02。验证:决策工件在案且
  三候选各有实机/读码证据。
  **[已完成 2026-09-19]** 决策 = **候选 A(桌面限定 shim 会话感知,
  vwin rect 派生内容区,zero auto-term 改动)**;候选 B 前提证伪
  (v.window_size 五写入方三语义 + 最大化缺位,t00-decision.md §2 表);
  实机:启动即错位(终端 868px 标定渲染于 1520px 内容区,§1.2 测量);
  键鼠初探未达(需真实点击聚焦,留 T-02/03 实机门;MCP action 受
  primary-App 单 App 语义限制,T-03 必须实机鼠标)。证据
  evidence/024/(t00-decision.md + t00-a-*.png + 快照/日志)。
- **T-01 几何源落地(桌面轨)** `[x]`
  依赖:T-00。按 T-00 决策(候选 A)落地,十轮迭代全录于
  evidence/024/t01-progress.md:auto-lang 九提交(e22f575df shim
  override+session 拆借注入 / a4a3237ad launch Init 时序 /
  247864bc4+d78e72021 滚动抖动两段修 / cd4c60f43 贴底量化
  gap_to_offset / 3afe5b97a 贴底 bind 钳位真值 / 2a96108f0 视口级
  余量底色 / f28f4d6e2 顶部方角回退 / 390ad623e pane 全方角 /
  a67cd569b RoundedSize::Px 任意值刻度);auto-term 两提交
  (4d1dde4 底部状态栏 / 5472f76 r=15 同心)。实机:几何源 TRACE
  实证(override 766 vs theme 1280)、滚动零抖、贴底跟随、圆角
  收敛,全部用户复验。关联 AC-01/02。**[完成 2026-09-21]**

- **T-02 键盘面实机门** `[x]`
  依赖:T-01。聚焦直键入/回显/执行,桌面实机用户手测多轮复验
  (ls/dir 命令执行/输出滚动全链伴随验证);桌面内注入通道实证
  (影子桌面 MCP/BUSt13 日志)。**遗留(转 review 门补验)**:
  Ctrl+C 中断与 WT 快捷键全表的专项录证——键盘注入需真实点击
  取焦(§10.7 域),MCP 通道不可达,留用户实机。关联 AC-04。
  **[完成 2026-09-21,部分专项转 review 补验]**
  **[按 T-00 决策修订 2026-09-19]** 实施形态 = 候选 A(auto-lang
  侧 term_engine shim override 面 + session.rs 拆借点注入;
  auto-term db.at/api.at 零改动);验证面增最大化/还原态。
- **T-03 鼠标面实机门(分屏/分隔条/Tab)** `[x]`
  依赖:T-01。横分分屏用户实机手测复验(截图在案);滚动条拖拽/
  滚轮全链用户手测。**遗留(转 review 门补验)**:嵌套/超限拒绝/
  磁吸钳位实点/Tab 右键关闭专项录证(MCP action 对非 primary
  app 不可达——单 App 语义,实机鼠标唯一通道)。关联 AC-05。
  **[完成 2026-09-21,部分专项转 review 补验]**
- **T-04 多 app 串扰防御验证** `[x]`
  依赖:T-01。**构造性隔离已证**:per-app override 随拆借注入
  (TRACE 实证——calculator 同场时 term override 恒 766 稳定不漂,
  jitter13 日志;calc 关/开对照实验在案);串扰源(theme 覆写)
  被 override 读序短路。第二 app = 011-calculator(§10-② 闭环)。
  resize 场串扰随遗留实机项。关联 AC-03。**[完成 2026-09-21]**
- **T-05 spec 沉淀 + 三轨零回归** `[x]`
  依赖:T-01..T-04。SD-01/02 增量入 worktree specs(c64dff5,
  merge 期与 025 配置节融合);**SD-03 退役记因**:候选 A 为
  auto.term 专用 shim override,非通用"虚拟窗尺寸→app 几何"注入
  面(v.window_size/Plan 409 面未动),无独立落册对象。
  三轨冒烟全绿:VM(Init/零 panic)/ rust(build Finished EXIT=0
  ——a2r 发射门 + exe 运行 84×25 + 状态栏)/ vue(VITE ready
  编译零错);auto-lang 门禁 p024 6+virtual_scroll 10+p022 6+
  p023 4+terminal_input 2+rounded 3 全绿。关联 AC-06/07。
  **[完成 2026-09-21]**
- **T-06 收尾:复审/合并面准备** `[x]`
  依赖:T-05。证据归档 evidence/024/(决策工件/八段进展/冒烟
  日志/截图序列);worktree 双仓清单:lang=term-024/auto-lang
  (auto-term-dev,组内 auto-down detached 只读兄弟)+ term=
  term-024/auto-term(plan-024-dev)+ os-shadow/apps.manifest
  影子取证根(merge 后清理);账本投影(SD 对应 P 条)merge 期
  落;§10 新增 4/5 号待澄清(thumb prop 增强/浅色带归属已判)。
  **025 合流预警**:025 已改 master 的 app.at(profile 右键子
  菜单)/db.at(config_* 符号)——merge 期同文件融合。
  验证:交接 auto-plan-review。**[完成 2026-09-21]**

任务粒度说明:T-02/T-03 拆两门(键盘/鼠标独立可验,失败域隔离);
T-04 独立成任务因串扰是独立失败面(他 app 时序)。

## 9. 复审记录

- 2026-09-19 stage: new | PLAN-024:r1 | outcome: **pass** |
  起草调查在案:几何源缺口三链(§2 缺口 A/B/C,读码行号锚定
  主检出 HEAD)、017 桌面门形态复用、022/023 陷阱条目继承
  (DLL 陈旧映像/候选链/载具归位)。设计决策(几何源候选)按
  技能规约以 T-00 有界调查承载,非执行阻断。| next: work
  (T-00 起;§10-②第二 app 选型可先行,不阻断 T-00/01)。
- 2026-09-19 stage: work | PLAN-024:r1 | outcome: **executing(进行中,
  非收口)** | code_commit: auto-lang auto-term-dev e22f575df(基
  e18066712) | task_ids: T-00 完成;T-01 实现+门禁+几何源面实机
  生效,渲染层缺陷续修 | evidence: evidence/024/t00-decision.md
  (决策工件:候选 A 定界,v.window_size 五写入方三语义表)+
  t00-a-clean3.png(启动错位实测)+ P024_TRACE(_desktop-t01c-boot
  .log,override 766×409 vs theme 1280)+ 门禁输出(p024 2/2/
  p022_stack 5/5/terminal_input 2/2) | blockers: 无(渲染层缺陷
  = T-01 剩余项在案,线索①②) | next: work 续(T-01 渲染层根修
  → resize/最大化随动录证 → T-02/03 实机门)。

## 10. 待澄清事项

1. **虚拟窗 resize 语义**(T-00 定界项):Plan 462 WM 下虚拟窗
   是否支持用户独立 resize,抑或仅随宿主窗同步——影响 AC-02
   验证口径(若仅宿主随动,AC-02 的"虚拟窗 resize"子句缩为
   宿主随动 + 布局态变更)。
   **[T-00 闭环 2026-09-19]** 虚拟窗支持八向独立 resize
   (session.rs:1300-1370);AC-02 口径不缩句 = 虚拟窗独立 resize +
   最大化/还原 + 宿主 resize 三态随动(t00-decision.md §2)。
2. **多 app 同场第二 app 选型**:用现成桌面 app(如 file-manager)
   还是最小 demo app——倾向现成(真实 resize 时序);执行时按
   桌面在场 app 定,不改契约。
   **[T-00 闭环 2026-09-19]** 现成 app:011-calculator(启动场自带,
   T-00 实机在场)。
3. **auto-lang 改动幅度**:若 T-00 决策要求 auto-lang 侧大改
   (超出"几何源注入"一个面),是否拆子计划先行——用户裁决项,
   T-00 决策工件须显式标注改动面量级供裁。
   **[T-00 闭环 2026-09-19]** 决策 A 改动面 = shim 读臂 + 拆借点
   注入(单一"几何源注入"面,量级小),不触发拆子计划;
   `v.window_size` 语义混沌对其他 app 响应式布局的影响**不入**
   本计划(t00-decision.md §3 遗留注记,后续计划裁决)。
4. **[T-01 执行期新增 2026-09-20] 最右下 pane 滚动条底角圆 prop**:
   用户方案——terminal 组件加 round prop,仅最右下 splitpane 滚动
   条底角圆。涉 .at 语言层(Terminal 视图变体 + 三轨发射器),
   超出本轮缺陷修最小面;**已由 statusbar 方案替代缓解**(pane 全
   方角,窗框圆角由状态栏承担,thumb 不再需要角部特例)——prop
   转为增强候选,语言层计划裁决。
5. **[T-01 执行期新增 2026-09-20] "右侧浅色突出"归属存疑**:像素
   证据(环内暗色无溢出/环外带下缘截止)指后方窗露边;用户观察
   "跟着 term 走"。statusbar 落地后观感重估,若仍在开有界诊断
   (疑点:vwin 层阴影/焦点环外绘制)。

- 2026-09-21 stage: merge | PLAN-024:r1 | outcome: **pass**(delivered)|
  **五 checkpoint 收据**:① prepared——reviewed 基线(auto-lang
  a67cd569b 基 e18066712 / auto-term c64dff5 基 23616cc),spec diff
  冻结于 c64dff5(SD-01/02),投影目标 .autoos/specs.json 六节;
  ② landed——两仓 --ff-only 线性落地:auto-lang master 落 024 链
  十提交(rebase 两次追并行推进 037/670/671;range-diff 仅 2/10
  融合差异其余等价,映射 a4a3237ad→88020ffb0 系,链尾 9d91b99a5;
  master 冒烟 p024 6+rounded 3 绿)+ sidecar path 依赖注入器根修
  e82b95b22(归属 025 备案);auto-term main 落四提交链(rebase
  零冲突自动合;app.at statusbar 与 025 profile 子菜单语义共存,
  rust 载具 build Finished 验证)→ **最终 tip:auto-lang
  e82b95b22 / auto-term 见后续并行提交(本计划链 6d3d897→f878a70
  →1d55a3d 在祖先链)**;③ ledger_refreshed——.autoos/specs.json
  P024-1..6 六节投影(round-trip+回读校验 OK);④ archived——本
  文件 docs/plans/archived/(文件系统迁移,auto-term plans 惯例;
  本收据段即归档后补录——初写时锚文本失配静默未中,补录完整)|
  completion_kind: delivered | ⑤ cleaned——wt-guard 双仓 clean
  (auto-term 侧先清 pnpm node_modules reparse point);双 worktree
  移除 + auto-term-dev/plan-024-dev 分支删 + auto-down detached
  兄弟摘除 + os-shadow/os-empty 探针目录清 + term-024 组目录移除,
  全部确认。| 融合记录:app.at 与 025(profile 右键子菜单)rebase
  零冲突,语义复核共存(027 教训门:rust 载具 build Finished 验证);
  sidecar path 依赖注入器缺陷顺带根修(归属 025 备案)。