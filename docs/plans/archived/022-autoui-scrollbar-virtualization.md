---
plan_id: PLAN-022
status: executing
feature_name: AutoUI 官方滚动条(虚拟滚动)整备——自绘滚动条退役 + SD-02 命中区 + 020 AC-04 关案 + rust 轨分屏渲染修复
author: [zhaopuming/zcode-session]
created_at: 2026-09-18T00:00:00Z
updated_at: 2026-09-18T00:00:00Z
plan_revision: 1
current_step: 5
total_steps: 8
supersedes_spec_components: []
new_spec_components:
  - docs/specs/terminal-widget-chrome.md(命中区归属规则,SD-01,T-05 定稿路径)
  - docs/specs/terminal-mux-model.md(虚拟滚动消费契约,SD-02,视 T-00 定稿)
touched_goals: []
---

# PLAN-022 · AutoUI 官方滚动条(虚拟滚动)整备

## 0. 变更摘要

来源 = PLAN-021 两轮用户裁定移交(021 归档件 §9/§10 +
evidence/021/t07-field-notes.md):

1. T-06/T-07 裁定(2026-09-17/18):"自制滚动条问题多多,先就此
   打住,等实现了完整的 AutoUI 官方滚动条(且支持虚拟滚动)再回来
   优化它。"
2. T-07 实测缺陷簇(用户实点/实滚,rust 轨载体):thumb 太小且未
   对齐右缘;拖拽比例不对无法到最早内容;滚轮只能回滚约半屏;
   thumb 不随滚轮跟随;rust 轨分屏右面板蓝屏未渲染、分隔条未出。
3. 携带项:SD-02 命中区归属规则(021 改判移交)、020 AC-04 分隔条
   拖拽携带项(021 改判移交)。

本计划 = ①AutoUI 官方 scrollable 组件**虚拟滚动**能力;②终端与
面板换装;③自绘滚动条退役;④rust 轨分屏渲染修复;⑤SD-02 命中区
归属规则定稿 + 020 AC-04 实机关案。

## 1. 目标

- G1 AutoUI 官方 scrollable 组件具备**虚拟滚动**能力:仅物化可见
  行窗,语义与终端 019 契约兼容(引擎 display_offset 单源,widget
  为视口快照缓存——scrollbar 视觉/交互不得反向成为滚动状态源)。
- G2 终端 pane 换装官方滚动条;自绘滚动条(SCROLLBAR_HIT_W=14、
  p053 press 链、thumb 绘制)退役。
- G3 滚动行为达标:thumb 比例正确(可到达最早内容)、滚轮全程可滚
  (无半屏钳位)、thumb 随滚轮/回滚跟随、右缘对齐。
- G4 SD-02 命中区归属规则定稿并实现(自绘条时代的 14px×8px 竞争
  问题随换装重定义;若官方滚动条命中区归组件,规则落到组件边界)。
- G5 020 AC-04 分隔条拖拽实机关案(含 rust 轨分屏渲染修复:右面板
  终端渲染、分隔条出现)。

### 非目标

- vue 只读视口能力扩展(019 边界维持);
- 非 auto-term 消费面的官方滚动条推广(其它 app 换装另立);
- 自绘滚动条的继续打磨(裁定:打住,退役)。

## 2. 架构方案

调查先行(T-00 bounded investigation,决策工件):
1. 官方 scrollable 现状(auto-lang examples/ui_scroll.rs,Component/
   View 抽象双后端)与虚拟滚动语义缺口;
2. 终端 widget 喂入面(019:props-feed 形态甲 + display_offset 单源
   契约,widget.rs)与 scrollable 的接口设计——虚拟行窗由谁持有
   (组件持 offset+行高量化,引擎仍为内容真源);
3. rust 轨分屏渲染缺陷定界(auto-term codegen vs auto-lang IntoIced
   轨;021 排障已证 rust 轨从未从零可编译,T-09 后首验)。

实现:虚拟滚动核心落 auto-lang ui 层(scrollable 组件);终端换装
落 auto-lang ui/terminal/iced/widget.rs + auto-term 前端;分屏渲染
修复视 T-00 定界定仓。

## 3. 技术栈

- auto-lang(主):ui scrollable 组件(虚拟滚动)、terminal widget
  接口、ui_gen 回归面 —— worktree 流程;
- auto-term(主):前端换装(app.at)、分屏消费面 —— 主检出直落;
- 载体:rust 轨 auto-term.exe(T-11 产物线,PLAN-021 target-p2 配方)。

## 4. 需求分析与背景调查

### 4.1 授权记录

- 用户裁定 2026-09-17(T-06):等官方滚动条+虚拟滚动实现后换装;
- 用户裁定 2026-09-18(T-07):自绘滚动条问题就此打住,移交本计划;
  021 §9/§10 移交清单在案;
- 预算/自动续跑:未指定。

### 4.2 已有证据(全部在库)

- 021 t07-field-notes.md:六条实测缺陷(§0 清单);
- 021 vm-delegation-break.log:rust 轨分屏"右面板蓝屏/无分隔条"
  记录(2026-09-18 用户截图+实测);
- 019 契约:display_offset 单源、泵三档、键入贴底
  (docs/specs/terminal-mux-model.md V1);
- 020:分隔条 8px 命中/磁吸 ±40/钳位 [100,900] 全在 back 落定
  (curl 剧本 t01_model.sh 12 步仍绿——PLAN-021 复审轮复跑);
- auto-lang 4579d59e3 裁定:虚拟滚动容器过渡形态问题清单(拇指比例/
  拖拽位置漂移/拖拽表现/hover 加宽/pointer 联动/theme)暂缓,待官方
  组件统一——本计划即收敛面。

### 4.3 代码事实(读码 2026-09-18)

- 自绘滚动条:auto-lang ui/terminal/iced/widget.rs(SCROLLBAR_HIT_W
  =14.0:1029、press 链 :513-590、wheel :615、scrollbar_metrics);
- 官方 scrollable:auto-lang crates/auto-lang/examples/ui_scroll.rs
  (View::scrollable,iced/gpui 双后端);
- 命中区:020 分隔条 = 定尺寸 div + Fill mouse-area(8px),与自绘条
  14px 命中带相邻/重叠(021 t07 实测分隔条未出,rust 轨);
- rust 轨 app 生成:app/rust-workspace(PLAN-021 T-09 后从零构建
  0 错,进程内 db 吸收)。

## 5. 详细设计

| # | 改动 | 文件:符号(仓) | 说明 |
|---|---|---|---|
| D1 | 虚拟滚动调查+语义定稿 | auto-lang ui scrollable + 决策工件 | 行窗量化/offset 归属/与 display_offset 契约对齐;产出决策记录 |
| D2 | 官方 scrollable 虚拟滚动实现 | auto-lang ui scrollable 组件 | 按定稿语义 |
| D3 | 终端换装 | auto-lang terminal widget + auto-term app.at | 退役自绘条;滚轮/thumb/跟随三达标 |
| D4 | rust 轨分屏渲染修复 | 视 T-00 定界(auto-term codegen / auto-lang IntoIced 轨) | 右面板终端渲染 + 分隔条出现 |
| D5 | SD-02 命中区归属 | 组件边界 + 规范落档 | 定稿于官方组件命中面确定后 |

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add(或 modify) | docs/specs/terminal-widget-chrome.md | 前:滚动条命中/归属无规则。后:官方滚动条命中区与分隔条 8px 命中的归属规则(宽度/优先级/z-order) | 换装后防"边缘互吞"回归 | AC-05 |
| SD-02 | add | docs/specs/terminal-widget-chrome.md(或 terminal-mux-model.md,视 T-00) | 前:虚拟滚动语义无契约。后:滚动状态单源(display_offset)与组件行窗量化契约 | 虚拟滚动不得反向成为状态源(019 契约延续) | AC-02 |

(T-00 定稿后回填精确路径与条目;此表先立语义。)

## 6. 测试设计

1. 组件级:虚拟滚动单测(行窗物化正确性/offset 往返/边界);
2. 终端级:滚轮全程可滚(含最早内容)、thumb 比例正确、跟随滚轮
   (t07-field-notes 四缺陷逐条反转);
3. 实机:三联合场景(拖分隔条/拖 thumb/接缝点击)用户实点通过;
4. 回归:020 t01 剧本 12 步;auto-term workspace 全绿;rust 轨
   从零构建 0 错;014 几何随动冒烟。

## 7. 验收标准

- **AC-01** 虚拟滚动语义定稿:决策工件在案(行窗/offset/契约对齐),
  组件实现与其一致。验证:决策工件 + 单测。
- **AC-02** 虚拟滚动契约入规范(SD-02)。验证:规范落档。
- **AC-03** 终端换装:官方滚动条承载终端滚动,自绘条退役。验证:
  实机 + 代码(自绘路径删除或停用)。
- **AC-04** 滚动行为达标:滚轮全程可滚、thumb 比例正确、跟随滚轮、
  右缘对齐(t07 四缺陷逐条反转)。验证:实机 + 截图。
- **AC-05** 命中区归属规则定稿并实现(SD-01),接缝点击归属符合
  规则。验证:实机三联合场景 + 规则落档。
- **AC-06** rust 轨分屏渲染修复:右面板终端渲染、分隔条出现可拖。
  验证:rust 轨载体实机。
- **AC-07** 020 AC-04 关案:分隔条拖拽磁吸/钳位/比例实机全过。
  验证:实机 + 020 t01 剧本仍绿。
- **AC-08** 回归面:020 t01 12 步、workspace 全绿、rust 轨从零构建
  0 错、014 几何随动冒烟。验证:日志。

## 8. 执行步骤

- **T-00 [D1] 调查与语义定稿**(bounded investigation,决策工件):
  scrollable 现状/虚拟滚动设计/分屏缺陷定界。前置:无。关联 AC-01。
  [x] ✅ 已完成 2026-09-18:决策工件 evidence/022/t00-decision.md——
  虚拟滚动语义定稿(引擎 display_offset 单源、scrollable offset=视图
  投影、CELL_H 量化无损、读写双臂映射);分屏定界=分隔条缺陷代码级
  闭合(rust codegen 丢 mouse-area/事件 → 空层跳过),slot2 蓝屏留
  T-03 实机探针(两候选在案);§10 澄清项 1/2/3 裁定。
- **T-01 [D2] 虚拟滚动实现**(auto-lang worktree)。前置 T-00。
  关联 AC-01/02。
  [x] ✅ 已完成 2026-09-18:lang-022 提交 50a2d4ad4——TerminalCore 同步
  态三字段 + view_y⇄offset 行量化换算 + observe/bind 双臂状态机
  (scroll_to 回声抑制);virtual_scroll_tests 4/4 绿。
- **T-02 [D3] 终端换装 + 自绘条退役**(auto-lang worktree +
  auto-term 主检出)。前置 T-01。关联 AC-03/04。
  [x] ✅ 已完成 2026-09-18:同提交 50a2d4ad4——renderer View::Terminal
  臂包官方 scrollable(id=terminal_scroll_<key>,scrollbar_style 现款),
  virtual_scroll=true 虚拟画布((rows+history)×CELL_H);自绘条全集退役
  (SCROLLBAR_*、ScrollbarDrag、metrics、press/wheel/drag 臂、draw thumb、
  Grab 形态);014 几何随动改 VIEWPORT_H 记账(scrollable 内子件滚动轴
  无穷约束);scroll_offset prop 非零哨兵。实机:单面板满幅渲染,蓝带
  消失(对比 021 t07 载体)。scoped 门:--lib terminal 40 过(1 失败=
  preedit 像素顺序 flaky,基线同败)。
- **T-03 [D4] rust 轨分屏渲染修复**。前置 T-00(定界)。关联 AC-06。
  [x] ✅ 已完成 2026-09-18:三层根因全修——①rust codegen 补
  `mouse-area` 发射(0bfa1f4af);②窗口尺寸面双漏斗+种子+退化护栏
  (62b3d6251);③into_iced Column/Row 臂镜像 PLAN-530 absolute 分区
  + run_app_devtools 单次构造(Init 双跑根除,9df7c5a5e)。实机实锤
  (screen-split.png,屏幕捕获+校验):50/50 双终端各带活 shell、分隔
  条清晰;模型面 [P22] s1/s2/dv 全对;进程面 5 子进程=双面板(原 3
  cmd=Init 双跑)。PrintWindow 在 wgpu 上服务旧缓冲——探针配方改
  屏幕区域捕获+灰条校验。AC-06 功能闭合。
- **T-04 [D5] SD-02 命中区归属定稿实现**。前置 T-02。关联 AC-05。
  [x] ✅ 已完成 2026-09-18:SD-01 落 docs/specs/terminal-widget-chrome.md
  (滚动条命中归 scrollable 组件/分隔条 8px 命中归 app/z-20>z-10 结构
  操作优先/视觉=scrollbar_style 现款);SD-02 落 docs/specs/
  terminal-mux-model.md §V1.10 修订(官方滚动条虚拟滚动契约:单源不变/
  视图投影缓存/坐标映射/读出写臂/回声抑制/键入贴底)。
- **T-05** [x] 020 AC-04 实机关案(三联合场景用户实点)。前置
  T-02+T-03+T-04。关联 AC-05/07。
  [✅ 已完成 2026-09-19] 用户实点多轮验证(023 交互投递修复后解阻):
  ①分隔条拖拽磁吸钳位 ✓ ②滚动条 thumb 全程/跟随 ✓("滚动条和分隔
  条操作都正常")③接缝命中归属 ✓。SD-01 落册
  docs/specs/terminal-widget-chrome.md(8b0ec0fed)。
- **T-06** [x] 回归门(§6.4 全项)。前置 T-02..T-04。关联 AC-08。
  [✅ 已完成 2026-09-19 收口] 2026-09-18 部分完成基础上:交互投递
  死亡阻塞由 PLAN-023 解除(用户实点三动作过);滚轮锚定回声判别
  根修(118289cbf/287154c37)+ div 发射修正(adf3c724e)+ Alt+WASD
  骨架(d865571e9,功能面入 DEBT 见 §10.7)后,载具从零构建 0 错
  (多轮实测),用户复测"功能恢复 OK,滚轮 OK,滚动条/分隔条正常"。
  ①workspace 全绿 ✓(09-18,27/27)②rust 轨从零构建 0 错 ✓(今日
  多轮)③020 t01 剧本 vue 轨=外部归属(642/643 路由 + 653 待并,
  09-18 用户裁定在案;rust 轨全项 ✓)④014 几何随动 ✓(09-18)。
  原文:①auto-term workspace 全绿 ✓(27/27,含
  engine_ffi 7);②rust 轨从零构建 0 错 ✓(master 路径与 022 运行时
  路径双验);③**020 t01 剧本受阻****:vue back(17401)投影
  单幅化(visible=5 但 rects 单 pane 满幅)——归属 master 侧 642/643
  合并(vue.rs +243 行;022 diff 的 auto-man 仅 rust_ui.rs 11 行=master
  修复并入),非本计划回归,路由 642 归属方/用户裁定;④014 几何随动
  冒烟 ✓(实测 pane 缩槽后引擎 resize;app.at 的 g 值刷新缺口
  [仅 geomChanged 门] 已修:app.at 每拍刷新槽位 cols/rows,实测
  g1=53x45 随槽收敛)。
- 2026-09-19 stage:work 补二(用户实点 fed back):横分/分隔条渲染+拖拽 ✓
  (路径重置修复后);**残留阻塞 = 交互投递死亡**:首层交互后右侧面板
  无法聚焦输入、滚动条/滚轮失效——headless 确定性复现已入册
  (lang-022 `p022_stack_click_tests.rs` #[ignore]:交替点击两浮层第一轮
  后计数冻结;已排除 Opaque 满幅捕获与终端 mouse_interaction 抬升,
  移除 opaque 后轨迹不变),需要 iced Stack/Opaque/运行时交互投递层的
  内部分析,超出本计划范围 → 随 T-06 移交 review 路由(独立调查计划)。
  另:**PLAN-653 已被并行会话执行完毕**(execution_done,全 5 任务,
  AC-01..06 实测证据;653 计划文件随其会话重组移位)——vue 轨修复在
  plan-653-dev 待 review/merge;本计划 rust 载具的剩余症状与 653 无关。
  载具构建陷阱固化:每次 `auto build -r rust` 会重置 Cargo.toml 路径
  覆盖(见 §10.6),022/653 合并前重生成后必须重新指向。
- 2026-09-18 stage:work 补记(用户问询"vue 版打开无 cmd 输出"诊断):
  vue 轨空白根因 = **back API 生成器把带参 GET 路由(tab-id-at/
  tab-title-at/tab-is-active-at 族)生成为 GET + `Json` 提取器**——
  axum 对无 body GET 一律 400("Expected request with Content-Type:
  application/json",实测 curl 复现);前端每拍轮询到 tab-id-at?i=0
  即抛错,循环中断 → tab_labels/槽位/终端永不渲染(受控浏览器实测:
  静态按钮渲染、fetch 28 次/3s、bad=[tab-id-at 400×6])。019 规范
  明文"带参读一律 POST"(rect-* 面即如此),tab-* GET 面生成违反。
  归属 master 侧 back/api 生成(642/643 合并族;022 diff 未触及),
  路由归属方;修复方向 = GET+Json 改 Query 提取或按规范改 POST+Json。
  叠加层:dev 跑法 back 引擎 DLL 解析失败(需 AUTOTERM_ENGINE_DLL,
  否则 tick panic app-back/src/term.rs:225)+ 投影单幅化(见 T-06③)。
- **T-07** [x] 锚定收口(双仓 SHA + status execution_done)。前置 T-05+T-06。
  [✅ 已完成 2026-09-19] auto-lang plan-022-dev {a4e4b291f 同步,
  287154c37 滚轮回声判别,236364f93 div 发射修正,d865571e9 捷径
  捕获,8b0ec0fed SD-01/02 spec 沉淀};auto-term master
  {app.at 六槽 alt.wasd+分派 10..13, db.at mux_focus_dir,
  api.at focus-dir 路由};execution_done 转审。

## 9. 复审记录

- 2026-09-18 draft handoff:`stage: new`,PLAN-022 rev1。`outcome:
  pass`(起草授权 = 021 两轮用户裁定移交;T-00 调查任务为首批可执行
  项)。`next: work`(work 前按惯例 `/auto-plan:review`;D1/D2 深设计
  在 T-00 决策工件后细化属计划内演进)。
- 2026-09-18 stage:work 进入:status drafting→executing(起草授权
  = 021 移交裁定,§4.1 在案)。worktree 布局:auto-lang 走
  `D:/autostack/.wt/lang-022/auto-lang`(branch plan-022-dev,基线
  master 1f4e3e32c);auto-term 主检出直落(020/021 惯例,主检
  tracked 代码零 WIP,untracked 遗留件 = 020/021 证据/spike/repro
  脚本,不构成代码 WIP)。并行会话占用面勘定:lang-026(p026
  native display)/lang-642/lang-647 在途,与本计划冲突面待 T-00
  改动前复查。
- 2026-09-18 stage:work · plan_id PLAN-022 · rev1 · outcome:
  **pass(部分)**——T-00/T-01/T-02 完成,T-03 auto-lang 侧两提交
  落库(0bfa1f4af + 62b3d6251,已并 master 1fc4772d9)。code_commit:
  auto-lang plan-022-dev 62b3d6251;auto-term 主检出 tracked 零改动
  (main.rs 生成物为 untracked,DBG 门控探针随重生成消失)。task_ids:
  T-00/T-01/T-02 完成,T-03 进行中(残留第三层:槽位绝对定位渲染
  不生效 + Init 双跑疑点),T-04..T-07 未动。evidence:
  evidence/022/t00-decision.md + 探针截图/日志/脚本 7 件。blockers:
  用户裁定 2026-09-18——split 链路测试暂停(新面板空置观感,spawn
  已证活着,渲染/合成链待根因),T-05 顺延;AC-06 未关。next:
  work 续 T-03 残留(§10.6 vehicle 复建配方)→ T-04 → T-05..T-07。
- 2026-09-19 stage:work 恢复(023 交付后,用户指令"开 022 T-06
  复验,以滚轮锚定为首项"):worktree 同步新 master(023/657 已并,
  a4e4b291f)作用域绿。**滚轮锚定缺陷根修**(118289cbf,plan-022-dev):
  用户实点三动作已过但滚轮上翻一格即卡/下翻阶梯卡顿——机理 = 一次性
  bind_suppress 吞掉的第一拍观察可能是用户下一格滚轮而非 scroll_to
  回声(读出臂先到先吞,两者不可区分),增量被吞引擎不动、下一格跳两格。
  修复 = 判别式回声:core 记 bind 期望回声位+滚轮代数,observe 时代数
  未进且落点贴合(±4px)才吞为回声,代数已进或错位一律真实回灌;
  WheelScrolled 臂 bump 代数。headless 回归 `echo_does_not_eat_user_wheel`
  (三段:回声对齐零回灌/用户增量不吞/错位差值回灌)+ 虚拟滚动全套
  5/5 绿。载具已重指 lang-022 重建交付用户复测(滚轮上下翻 + T-05
  实点清单 ①②③ 同场)。
- 2026-09-19 stage:work 补三(同日第二轮,用户指令:pane 切换快捷键
  改 Alt+WASD):**三方落地**——①auto-lang 终端捷径命中即
  capture_event(commit 内含,分屏多终端同表防多重触发,
  shortcut_hit_captures_single_fire_across_stack 回归);②back
  mux_focus_dir 几何导航动词(db.at 槽位矩形中心,方向半平面最近邻
  主轴×1000+正交评分;api /api/mux/focus-dir);③app.at 六槽
  onkeydown.alt.wasd→Shortcut(10..13)+分派。**顺带修 master 潜在
  回归**(236364f93,归 027 面备案):PLAN-027 T-04 div→container
  映射的发射形态与 view.rs container(child) 一参签名错配,div 大户
  rust 重生成编译不过且多子链式 .child() 静默丢子;div/container 臂
  镜像 center(0/1/N 子分发射,N 子装 col),ui_gen 面零新增失败
  (~25 项存量红为 027 时代在案)。载具重指 lang-022 重建交付复测。**首版 div→container 臂实车
  回归**(用户实点:内容区半窗空白/横分表象变竖分)→ 复盘修正
  (adf3c724e):div 回退 col 发射(027 映射对多子流式 div 语义错:
  ViewContainerBuilder 无弹性类映射 + child 替换语义静默丢子),
  container 标签保留一参包装;重生成结构回归好形态(col+flex-1),
  Alt+WASD ×6 槽与 mux_focus_dir 分派保留。修正版载具交付复测
  (Alt+WASD 四向 + 滚轮锚定 + T-05 实点清单 ①②③ 同场)。
- 2026-09-19 **PLAN-023 交接备案**(023 T-06):交互投递死亡独立
  立项已交付——判决 = into_iced opaque 捕获边界×spacer 几何错位 +
  on_click 构建期闭包测量伪影双案(iced 上游无罪;022 §0"已排除 1"
  由 023 判决反转,opaque 即元凶,原实验被伪影致盲),根修
  auto-lang plan-023-dev {fc6ccc810, 6c8a4f3d0, 7d070317b};实车
  三动作 + 双播复测用户实点通过(023 AC-03 关)。**本计划 T-06/
  T-07 解锁条件达成**,可恢复执行;复现测试 p022_stack_click_tests
  归属移交 023(已去 ignore 转正,本计划在档记录保留为历史);
  023 分支基于本计划 plan-022-dev,合流序 = 023 先并,本计划
  fold 时按 merge 面归位。详见 023 计划 + evidence/023/。

## 10. 待澄清事项

7. **DEBT·Alt+WASD pane 切换未生效**(2026-09-19 用户裁定挂账,
   "锦上添花,不影响使用"):骨架已落(app.at 六槽
   onkeydown.alt.wasd→Shortcut(10..13)→api.mux_focus_dir 几何
   导航;捷径命中即捕获防多发)但用户实点无切换效果。根因候选:
   **mux 模型焦点 ≠ iced 组件键盘焦点**——mux_focus 切的是 back
   模型(引擎输入路由/zoom/close 目标),而键入跟随 iced 组件
   state.focused(点击决定),模型切焦不迁移组件焦点,用户感知
   "没切换"。解除方向:KeyIn 路由改跟随模型焦点,或模型切焦时
   程序化迁移组件焦点(iced focus API 面)。归后续计划。

1. 虚拟滚动行高/量化语义(等宽字符网格假设是否成立,长行折行?);
   **已裁定 2026-09-18 T-00**:等高网格 CELL_H=16 成立,量化无损;
2. 换装清单:终端之外哪些面板需要 scroll(021 §10 #3 勘定遗留);
3. 滚动条视觉规格:已裁定沿用 iced 官方 scrollbar_style() 现款;
4. rust 轨分屏渲染缺陷归属:**T-00/T-03 已定界两层**——①rust codegen
   丢 mouse-area(已修,0bfa1f4af);②窗口尺寸面断供(已修,
   62b3d6251);**残留第三层**:槽位绝对定位渲染不生效(模型矩形正确、
   样式解析正确、视觉仍单幅满窗)+ Init 双跑疑点,根因待查(用户裁定
   2026-09-18:此未明前暂停 split 链路测试,T-05 顺延);
5. 与并行会话(642/646/647+)的 auto-lang 冲突面:本计划 surface =
   terminal/iced/widget.rs、ui_gen/rust.rs(mouse-area 臂)、renderer.rs
   (Terminal 臂/TickWrap/devtools 窗口尺寸臂)——已并 master
   1fc4772d9,继续开发前需重新对 master;
6. **vehicle 复建配方(下一会话用)**:①worktree lang-022 构建
   codegen `cargo build -p auto`;②`app/` 下
   `<worktree>/target/debug/auto.exe build -r rust` 重新生成 main.rs;
   ③`app/rust-workspace/Cargo.toml` 的 auto-lang path 临时指向
   `../../../.wt/lang-022/auto-lang/crates/auto-lang` 后
   `cargo build -p auto-term`(产物在 auto-term 根 target/debug,经
   AUTOTERM_ENGINE_DLL 挂 autoterm_core.dll);④探针脚本
   evidence/022/t03-probe-*.ps1,AUTO_MA_DBG=1 开 slot 数值日志
   (main.rs 探针补丁为 DBG 门控,重新生成即消失)。
- 2026-09-19 stage: work | plan_id PLAN-022 | rev1 | outcome:
  **pass(全任务闭合,execution_done)** | code_commit:
  auto-lang plan-022-dev {a4e4b291f 同步, 287154c37 滚轮回声判别,
  236364f93 div 发射修正, d865571e9 捷径捕获, 2733c0879 测试门控,
  8b0ec0fed SD-01/02 spec};auto-term master 999cb1b(六槽
  alt.wasd+分派+mux_focus_dir 动词/路由)| task_ids T-01..T-07 |
  evidence: 用户实点多轮(滚轮 OK/滚动条分隔条正常/三动作过/
  Alt+WASD 未生效挂 DEBT §10.7)+ 虚拟滚动套件 5/5 +
  echo_does_not_eat_user_wheel + shortcut 单发回归 | blockers:
  无 | next: review。
- 2026-09-19 stage: review | plan_id PLAN-022 | rev1 | outcome:
  **pass** | reviewed_commit: plan-022-dev 8b0ec0fed(工作树零
  脏改)| base_commit: e5ddf41ba(经 a4e4b291f 同步含 023/657)|
  dependency_revisions: auto-down fae21d9(detached 只读)|
  spec_inputs: terminal-mux-model.md + terminal-widget-chrome.md
  (冻结于 8b0ec0fed)| acceptance_results: AC-01 pass(决策工件
  +状态机单测)/AC-02 pass(SD-02 落册)/AC-03 pass(官方滚动条
  换装,用户实点)/AC-04 pass(T-06 回声判别修后"滚轮已经OK")/
  AC-05 pass(SD-01 落册+接缝归属实点)/AC-06 pass(右面板渲染
  +分隔条可拖,用户实点)/AC-07 pass(磁吸钳位实点)/AC-08
  pass(rust 轨全项;020 t01 vue 轨=外部归属 642/643+653 备案,
  用户 09-18 裁定路由)| findings: F1 info=ui::iced ~8 项 +
  ui_gen ~25 项失败存量于各自主计划基线(027 时代在案);F2
  info=iced flex 零宽 Space 怪异(KD 023-R8 互引);F3 info=
  Alt+WASD 骨架未生效挂 §10.7 DEBT(骨架保留,非回归);F4
  info=master tip lib-test 红归 656 eee04e787(Scrollable 增
  字段未同步 layout_tests,fix-p656-vue-probe 在修)——本计划
  同内容树全绿在案,非本计划回归 | 门禁:仓规 Category B
  (cargo check + 作用域模块测试)+ 用户实点门,均过 | next:
  merge。独立性限制:实现会话内 review,verdict 自工件重构
  (被审树复跑+探针+用户实点记录)。
- 2026-09-19 stage: merge | PLAN-022:r1 | outcome: **blocked
  (publication-only,主体已落)** | 收据 checkpoints——
  `prepared`: reviewed 8b0ec0fed + 冻结 delta(terminal-mux-model.md
  + terminal-widget-chrome.md)+ 账本投影 P022-1/2/3 |
  `landed`: **主体已落** auto-lang master 5f5f1b6a6(no-ff fold,
  含滚轮回声判别/捷径捕获/div 修正/SD-01+02 spec;用户实点全过)|
  `ledger_refreshed`: 尾段排队——P022-1/2/3 重放 + 捷径测试门控
  卫生在 plan-022-dev {2733c0879, b6b69aca9} 待并(master 上
  656 会话同日亦动账本,冲突已按"master 底+重放"解于分支侧)|
  `archived`: 待 landing 尾段落定后执行 | `cleaned`: 待清理
  (lang-022 双 worktree + 分支 + 组目录 + 载具指向归位)|
  **blocked 原因**: auto-lang 主检出存在并行会话未决合并
  (MERGE_HEAD 在途,UU .autoos/specs.json + 659/KNOWN-DEBT 改动,
  实测 2026-09-19),不可争写 | **解除动作**: 主检出合并清结后
  `cd /d/autostack/auto-lang && git merge --no-ff plan-022-dev`
  (若仍冲突:.autoos/specs.json 取 master 底重放 P022-1/2/3,
  脚本同本轮)→ 冒烟 → 归档 → 清理。 Plan 保持 reviewed。

- 2026-09-19 stage: merge(续) | PLAN-022:r1 | outcome: **pass** |
  尾段落定:主检出并行合并清结后重试,账本 P022-1/2/3 校验幂等
  (660 会话合并已带入,重放 0 条),合并落定 9e9da8b39;master
  冒烟 p022_stack 5/5 绿(656 的 E0063 已由其 fix 分支修复,
  layout_tests 含新字段)——`ledger_refreshed` ✓ | `archived`:
  本文件 git mv 至 docs/plans/archived/,status: archived |
  双仓 SHA: auto-lang fold 5f5f1b6a6 + 尾 9e9da8b39;
  auto-term 999cb1b(Alt+WASD 骨架+动词)| `cleaned`: ✓ wt-guard
  双 clean 复检;worktree lang-022/auto-lang + lang-022/auto-down
  (down 仓侧)均移除,分支 plan-022-dev 删(was b6b69aca9,已并入
  master 9e9da8b39),组目录 lang-023 同法先前已清;载具 Cargo.toml
  指向归位主检出(untracked 运行态)。收据五 checkpoint 全闭合。
