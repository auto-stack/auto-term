---
plan_id: PLAN-025
status: executing
feature_name: 滚动闪屏根修(绝对行键行缓存+滚动位移渲染) + 默认 shell 配置文件与 profiles
author: [agent]
created_at: 2026-09-20T00:00:00Z
updated_at: 2026-09-20T00:00:00Z
plan_revision: 1
current_step: 8
total_steps: 10
supersedes_spec_components: []
new_spec_components:
  - docs/specs/terminal-scroll-render.md(add,SD-01:滚动行缓存/预取窗/位移渲染契约)
  - docs/specs/app-unified-entry.md(modify,SD-02:配置文件与 profiles 契约)
  - docs/specs/terminal-mux-model.md(modify,SD-03:mux_new_tab profile 参数语义)
touched_goals: []
---

# PLAN-025 · 滚动闪屏根修 + 默认 shell 配置文件与 profiles

## 0. 变更摘要

用户动议(2026-09-20 会话):①AutoTerm 滚动闪屏根修——前期会话已把
机理锚定到呈现层(核心层 alacritty_terminal 0.26 无责),本计划按
"判决先行"落修复;②默认启动 shell 进配置文件,并升级为 Windows
Terminal 风格 profiles(多 profile + NewTab 菜单选择)。两件事同属
auto-term 改进,独立可并行,共一计划分 A/B 两部。

## 1. 目标

### A 部:滚动闪屏根修(rust 轨载体)

- G1 滚轮连续回滚/前滚**无闪屏**(实机肉眼门,用户验收),机理上
  = 滚动每帧行 Paragraph 重建数收敛到"新暴露行数"量级,消除整窗
  重排与陈旧窗位移带来的可见闪动。
- G2 PLAN-022 滚动语义零回归:thumb 比例/全程可滚/滚轮跟随/右缘
  对齐;引擎 display_offset 单源契约不变。
- G3 vm/vue 轨与 022 全链既有行为零回归。

### B 部:配置文件 + profiles

- G4 `config.toml` 定义默认 shell 与 profile 集;无 CLI 参数时
  rust/vue 轨首 Tab 按配置 spawn(配方:默认 ash 直接裸跑,不经 cmd)。
- G5 NewTab 的 `+` 建 default profile Tab;`+` 处 profile 菜单可选
  任意 profile 建_TAB(WT 风格);profile 的 program/argv/cwd 经
  既有 SpawnSpec(`engine_spawn_ex`)生效。
- G6 配置缺席/损坏 → 降级既有 COMSPEC/cmd 行为,不崩;引擎层
  (autoterm-core FFI)零改动;CLI 增加 `--profile`(与既有 `--shell`
  共存,优先级:显式参数 > 配置文件 > COMSPEC)。

### 非目标

- VM/desktop 轨的 profile 消费面(与执行中的 PLAN-024 桌面轨合流后
  另立小计划;api.at 契约按本计划定稿,shim 臂后补);
- profile 的 `ctrl_c_mode` 字段(app 轨 Ctrl+C 策略面与 CLI 不同源,
  V1 不进 schema,记 DEBT);
- vue 轨交互能力扩展(019 只读边界维持);
- vm/desktop 轨滚动渲染(如 aura 臂有同题,凭本计划判决工件另立);
- iced 载具退换/渲染栈替换(在现 iced 立即模式内修);
- 自绘滚动条回归(022 已退役裁定维持)。

## 2. 架构方案

### 现状链(调查实证,行号为主检出/auto-lang HEAD 3df7b21a2)

- **核心层无责**:alacritty_terminal 0.26 数据模型 = 不可变 scrollback
  + live grid,滚动仅 `display_offset` 整数变化
  (`grid/mod.rs:134/163`),滚动时核心 `mark_fully_damaged()`
  (`term/mod.rs:389`)——"WT 式固化路线"的数据面已内建,核心零改动。
- **闪屏根因候选(呈现层,auto-lang rust 轨 terminal widget)**:
  1. **行缓存按视口行号键**:`widget.rs:901-905`
     `row_caches()[key][y]`,`y` 是视口槽位,digest=内容+palette。滚动
     → 快照窗整体位移 → 每个视口槽内容全变 → 全部 digest 失效 →
     **一帧内 `build_row_paragraph`(`widget.rs:1433`)重排全部可见行**
     (文本 shaping 是帧预算大头);
  2. **滚动数据往返滞后**:滚轮 → iced scrollable 即时位移,draw 期
     `observe_view_scroll`(`widget.rs:369`)→ `terminal_queue_scroll_delta`
     回灌引擎 → **等 50ms Tick 泵**取回新快照窗,期间以 `window_shift()`
     (`widget.rs:336`)位移旧窗补偿——窗回中瞬间与新行到达的时序差 =
     空白带/内容跳变候选;
  3. 全幅底色 quad + 选中层每帧重画(成本小,次要)。
- **profile 现状链**:spawn 参数化已就位——sidecar
  `app/term.rs:255 engine_spawn`(program=null → FFI
  `crates/autoterm-core/src/ffi.rs:164 default_shell()` = COMSPEC 兜底)
  与 `engine_spawn_ex(program/argv/cwd)`(PLAN-018 D3);back
  `app/src/back/db.at:1182 mux_new_tab()` → `spawn_pane_into_tab`;
  front `app/src/front/app.at` NewTab 消息。缺的只是"从文件读预设并
  喂给 spawn_ex"+ 菜单面。
- **四装载面分工**:rust 轨(app/rust-workspace/app,iced 窗口)与
  vue 轨(app-back,axum)共享 back 实现 → 配置读取落 app-back 即
  覆盖两轨;VM/desktop 轨走 api.at 委托体(auto-lang
  `vm/ffi/term_engine.rs`),本计划只定契约不定 shim 臂(非目标)。

### 修复设计(A 部,细则由 T-01 判决工件裁剪)

1. **绝对行号键行缓存**:快照窗携带绝对行锚(引擎侧 display_offset
   单源不变,泵链把锚随行窗带出),行缓存键从"视口槽位"改为
   "绝对行号+digest"——滚动时未变行复用已排 Paragraph,仅新暴露行
   重建;
2. **预取窗**:行窗 = 可见区 ± N 行(N 滚动 notch 量级起步,判决后
   定值),新暴露行大概率已排好;
3. **滚动即时泵**:scroll delta 回灌后的新窗获取不等 50ms Tick,
   事件驱动补一次泵(视判决:若候选 2 非主因则降级为可选)。

### 配置设计(B 部)

- 文件:exe 同目录 `config.toml`(贴合三件套同目录分发契约),
  `AUTOTERM_CONFIG` env 覆盖路径;解析失败/缺席 = 空 profile 集 +
  COMSPEC 默认(行为等价现状)。
- Schema V1(TOML):

```toml
default_profile = "ash"

[[profiles]]
name = "ash"
commandline = 'D:\autostack\auto-shell\ash\target\release\ash.exe'
starting_directory = 'D:\autostack'

[[profiles]]
name = "cmd"
commandline = 'cmd.exe'
```

- 解析模块落新 crate `crates/autoterm-config`(auto-term 仓;仅
  autoterm-ui CLI 与 app-back 消费,autoterm-core 零依赖纪律不动);
- back 持 profile registry(db.at),`mux_new_tab(profile_name)` 带
  参重载(缺省 = default_profile);api.at 新增 `/api/term/profiles`
  (列表:name/commandline/cwd)+ new-tab 链路带 profile 参数;
- 前端 `+` = default profile,`+` 右键(或长按/下拉,实现取
  app.at 菜单既有形态)出 profile 子菜单。

## 3. 技术栈

- **auto-lang**(A 部主战场):`crates/auto-lang/src/ui/terminal/`
  (widget.rs 行缓存/预取窗/位移渲染,TerminalCore 快照锚)——
  worktree 流程(`plan-025-dev`,沿 022 惯例);
- **auto-term**(B 部全量 + A 部泵链):app/(term.rs/api.at/db.at/
  app.at/rust-workspace)、crates/autoterm-ui(CLI)、
  crates/autoterm-config(新)——主检出直落;
- 载体:rust 轨 `auto run -r rust`(A/B 实机门)、vue 轨
  `auto run -r vue`(B 部回归)、独立 `autoterm.exe`(CLI 门);
- 与 PLAN-024 协调:025 不动 auto-lang theme 几何/native_catalog
  (024 域),024 不动 ui/terminal;合并顺序互不阻塞,worktree 分名。

## 4. 需求分析与背景调查

- 授权:用户 2026-09-20 会话明示——"按照你的分析,整理出修理方案,
  搞一个计划吧。默认启动 shell 的配置(加 profile)也可以放到同一
  个计划里。都是 auto-term 的改进计划。" 授权范围 = 本计划 A+B 两部;
  仓库 = auto-term(主检出)+ auto-lang(worktree);未设预算/自动
  续行限制,沿 auto-plan 惯例逐阶段放行。
- 背景结论(会话取证,已核码):
  - ash 直跑可行性/契约:PLAN-007 与 designs/003(7 用例门禁 + UI
    配方全绿);中断语义 008(auto 模式 ash 豁免仅字节,profile V1
    不含 ctrl_c_mode 的原因);
  - 渲染路线:designs/000(iced 一等应用)+ PLAN-022(官方滚动条
    虚拟滚动,引擎 display_offset 单源,widget 视口快照缓存)+ 
    PLAN-023(浮层交互投递死亡已根修,实车交互恢复);
  - alacritty_terminal 0.26 源码核验(cargo 缓存
    `alacritty_terminal-0.26.0/src`):display_offset/scroll_display/
    行级损伤/mark_fully_damaged 语义如 §2 引;
  - specs 现状:docs/specs/ 四件(app-unified-entry/terminal-mux-
    model/terminal-widget-chrome/engine-ffi-color-encoding)均现行,
    无滚动渲染契约(本计划 SD-01 补)、无配置契约(SD-02 补)。
- 相关在飞计划:PLAN-024(executing,桌面轨几何)——§3 协调条款。

## 5. 详细设计

### A 部任务机制要点

- T-01 判决工件须回答:主因配比(候选 1 整窗重排 vs 候选 2 往返
  滞后)、重排行数/帧与滚动 notch 的实测曲线、headless 复现器
  (iced_test 或 draw 期埋点)能否稳定断言。修复三件套(绝对键/
  预取/即时泵)按配比取舍,判决进 evidence/025/。
- 绝对行锚的泵链:display_offset 语义已单源(022);锚 = 行窗首行
  的绝对行号(自 bottom 锚定),经 `/api/term/*` 泵载荷或新 getter
  带出——具体挂载点以 T-01 对 snapshot()/rows_for 链的定界为准
  (`app/term.rs:469 engine_rows_for`、widget `self.core.snapshot()`)。
- 行缓存并发/生命周期沿现状(`row_caches()` 全局 Mutex
  HashMap,keyed by terminal key;019 多 pane 契约不变)。

### B 部任务机制要点

- `crates/autoterm-config`:`load(path) -> Config`、
  `resolve(cmdline) -> (program, argv)`、默认路径解析(exe 目录);
  单测覆盖:合法/缺席/损坏/字段缺失/相对路径 commandline。
- db.at registry:Init 期读一次(文件变更热重载非目标);profile
  名冲突 = 后者覆盖 + 启动告警(stderr);
- api.at 契约(VM/desktop 臂后补,契约先定):`/api/term/profiles`
  返回 `[name, commandline, starting_directory]` 列表;
  `mux_new_tab(profile)` 的 api 形态随 024 定稿的 new-tab 路由对齐。
- CLI:`--profile <name>`(配置缺席时给错并退出 1,信息含配置
  搜索路径);`--shell` 保留且优先于 `--profile`(显式程序覆盖预设)。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/terminal-scroll-render.md | before:无滚动渲染契约(仅 022 滚动条交互契约在 chrome spec);after:行缓存以绝对行号+digest 键、预取窗 N 行、滚动位移语义(引擎 display_offset 单源不变)、重排预算断言纪律 | 闪屏根修的行为契约与防复发回归锚 | AC-01/AC-02 |
| SD-02 | modify | docs/specs/app-unified-entry.md | before:无配置面;after:新增"配置文件与 profiles"节——位置(exe 旁 + env 覆盖)、schema V1、解析优先级(参数>env>文件>COMSPEC)、降级语义 | 配置是四装载面共享的入口行为 | AC-04/AC-05/AC-06 |
| SD-03 | modify | docs/specs/terminal-mux-model.md | before:`mux_new_tab()` 无参;after:带 `profile` 参数(缺省 default_profile),spawn 经 SpawnSpec,profile 集合为 back registry 单源 | mux 建_TAB 语义扩展的契约面 | AC-05 |

## 6. 测试设计

- **headless(A 部门禁)**:auto-lang 新增 p025 系测试——滚动 K notch
  后行 Paragraph 重建数 ≤ K·每 notch 行数 + 预取增量(数字断言);
  现有 p022 系(滚动条/交互)与 ui_gen 回归全绿;
- **auto-term 回归**:`cargo test -p autoterm-core`(引擎零改动应
  全绿基线不变);app-back/http 剧本扩 profiles 路由断言(沿 020
  t01_model.sh curl 剧本惯例);autoterm-config 单测;
- **实机门(A/B 分别)**:A = rust 轨滚轮长回滚/前滚录屏 + 用户肉眼
  验收(闪屏消失);B = 配方三则——默认 ash 首_TAB(prompt 证据)、
  profile 菜单建 cmd/pwsh_TAB(echo marker 证据)、删除 config.toml
  后降级 cmd;vue 轨 B 部同验(HTTP 面);
- **oracle 红线**:`at/autoterm.at` 与 at-gen 冻结对拍面零改动
  (git diff 门)。

## 7. 验收标准

- **AC-01** rust 轨滚轮连续回滚/前滚无可见闪屏:实机录屏
  (evidence/025/) + 用户验收;headless 断言滚动重排行数收敛
  (≤ 新暴露行 + 预取增量)。验证:实机配方 + p025 测试绿。
- **AC-02** 022 滚动语义零回归:thumb 比例/全程可滚/滚轮跟随/右缘
  对齐;`p022_*` 系测试全绿。验证:auto-lang cargo test + 实机点检。
- **AC-03** vm/vue 轨零回归:VM 直跑冒烟(Tab/分屏/键入)、vue HTTP
  剧本绿。验证:020 t01_model.sh + vue 轨 `auto run -r vue` 冒烟。
- **AC-04** 配置默认 shell 生效:config.toml 指 ash → 无 CLI 参数时
  rust 轨首_TAB 裸跑 ash(prompt/`color info` 证据);配置缺席/损坏
  → COMSPEC cmd 降级不崩。验证:实机配方三则。
- **AC-05** profile 菜单与参数生效:`+` = default profile;菜单选
  cmd/pwsh 建_TAB,program/argv/cwd 经 spawn_ex 生效(cwd 用
  `/api/mux` 既有 cwd 回显配方)。验证:实机 + curl 剧本。
- **AC-06** CLI:`--profile`/`--shell` 优先级正确;`--profile` 命中
  不存在的名字 = 明确报错退出 1。验证:autoterm.exe 冒烟 + oracle
  零 diff 门。
- **AC-07** 引擎层零改动:`git diff crates/autoterm-core` 仅允许
  零改动(ffi default_shell 兜底保持)。验证:git diff 门。

## 8. 执行步骤

- [x] **T-01** 有界调查:闪屏机理判决工件(auto-lang)。headless 滚动
  复现器 + 重排计数/帧时序埋点,回答主因配比与修复取舍。产出
  `docs/plans/evidence/025/t01-verdict.md`。依赖:无。关联:AC-01。
  验证:复现器测试可重复跑出计数曲线。
  [✅ 已完成 2026-09-20 · lang-025 bb6dd5d22]判决:候选 1(槽位键
  整窗重排)主因——每 notch 30/30 行重排、打印 1 行亦 30/30;候选 2
  (泵滞后)次因——常规滚轮 3 行×9 帧空白带、快速 9 行×4 帧。取舍:
  绝对行号键+预取 N=8+视图锚定位进 T-03;T-04 即时泵豁免转 DEBT
  (残余仅极端连滚 1-2 行薄带,50ms 泵兜底)。复现器
  p025_scroll_render_tests 4 绿,`AUTO_MA_DBG=1` 实机埋点 `[P25-ROWS]`
  在册。
- [x] **T-02** 快照窗绝对行锚(auto-term 泵链 + auto-lang core glue):
  display_offset 单源下把行窗绝对锚带出引擎,widget `snapshot()`
  消费。依赖:T-01。关联:AC-01/AC-02。验证:auto-lang 单测(锚
  随滚动正确递变)。
  [✅ 已完成 2026-09-20 · lang-025 fd380a64f + auto-term 231ddac]
  TerminalCore `window_anchor`(= h − o,泵回写,几何替换承载,
  `WINDOW_ANCHOR_UNSET` 哨兵 = 未锚回退);term.rs 泵 Key 侧同拍
  回写;p025_anchor_and_store_roundtrip 锚往返/哨兵/几何承载绿。
- [x] **T-03** 行缓存绝对行号键 + 预取窗(auto-lang widget.rs:901 一带
  改造)。依赖:T-02。关联:AC-01/AC-02。验证:p025 重排数断言绿。
  [✅ 已完成 2026-09-20 · lang-025 fd380a64f]三件套:①ROW_CACHES
  槽位键→行 id 键(digest 门控,容量 512 护栏);②预取窗 N=8
  (window_store 绝对 id→cells/digest;泵瞬态 scroll 采样上/下各
  8 行,终态 offset 恢复,引擎零改动);③视图锚定内容层(可见行
  区间由视口矩形推导,行位 id×CELL_H;槽位/cursor/selection 面保持
  引擎锚位移)。p025 断言翻转绿:每 notch 30→3、打印 1 行 30→1、
  预取覆盖先行视图零空白、未锚回退槽位语义;terminal 系回归 47 绿;
  rust 轨实车 boot 冒烟 `[P25-ROWS]` 增量重建生效(首帧 30→内容帧
  3→稳态 0),无 panic。
- [x] **T-04** 滚动即时泵(视 T-01 判决取舍;若主因纯属候选 1 可记
  DEBT 豁免)。依赖:T-01。关联:AC-01。验证:headless 时序断言或
  判决豁免记录。
  [✅ 已豁免转 DEBT 2026-09-20]判决(evidence/025/t01-verdict.md):
  预取 N=8 后残余仅极端连滚(≥5 notch/帧持续)1-2 行瞬态薄带,
  50ms 泵周期兜底;事件驱动补泵的接线收益不足以抵消 022 回声抑制
  状态机扰动风险。启用条件 = 实机验收仍见可见薄带。见 §10.5。
- **T-05** A 部门禁:022/023 回归 + 三轨零回归 + 实机滚动录屏与
  用户验收。依赖:T-03(及 T-04 若执行)。关联:AC-01/02/03。
  [进行中 2026-09-20]已毕:auto-lang terminal 系回归 47 绿(p022/
  p023/虚拟滚动/terminal::);rust 轨 boot 冒烟绿(P25 增量重建
  生效无 panic);全量套件基线对照毕(失败集差分 ≤4 项且单测重跑
  全绿 = 抖动/环境项,非 025 回归,详见 §10.7)。待办:vm 直跑冒烟、
  020 t01_model.sh 回归、**实机滚动录屏 + 用户肉眼验收(闪屏消失)
  ——用户门**。
- [x] **T-06** `crates/autoterm-config` 新 crate(schema/解析/路径解析
  + 单测)。依赖:无(B 部可并行启动)。关联:AC-04/06。
  [✅ 已完成 2026-09-20 · auto-term 87d94c9]schema V1 解析全绿
  (10 单测:合法/缺席/损坏/字段缺失/重名覆盖/default 未命中/引号与
  裸名拆分/相对路径透传/env 覆盖路径/文件往返);缺席与损坏回落空集
  不崩,坏条目跳过 + stderr 告警;SpawnSpec(program/argv/cwd)映射
  与 engine_spawn_ex 一一对应。
- [x] **T-07** back 接线:db.at registry + `mux_new_tab(profile)` +
  api.at `/api/term/profiles` + term.rs spawn 链消费 profile。依赖:
  T-06。关联:AC-04/05。验证:curl 剧本。
  [✅ 已完成 2026-09-20 · auto-term 231ddac]term.rs 配置 sidecar
  (config_profiles/_full/default/spawn 三元组,Init 一次装载);
  db.at spawn_pane_into_tab(tab,axis,profile)/mux_new_tab(profile)
  /动作槽位编码/profile_name_of_arg;api.at /api/term/profiles +
  /api/term/profile-names + new-tab(profile)。剧本
  `evidence/025/t07_profiles.sh` 干净跑 **8/8 PASS**
  (t07_profiles_run.log:names/records/default-spawn+cwd/marker
  argv/ghost 回落/tabs 计数)。
- [x] **T-08** app.at 前端:`+` 语义与 profile 子菜单(沿既有菜单形态)。
  依赖:T-07。关联:AC-05。验证:实机点检。
  [✅ 已完成 2026-09-20 · auto-term 231ddac]`+` 左键 = default
  (NewTab→arg 0,既有语义零变);`+` 右键 = ProfileMenu 开/收
  (与 Tab 条右键关闭同形态,§10.4 取形落地);菜单 = 行内按钮组
  (Init 一次装载 api.term_profile_names;空配置不渲染,右键仍可用);
  选中 NewTabProfile(i)→mux_enqueue(1, i+1) 建后收菜单。编译绿 +
  剧本面等价断言;实机点检归 T-10 用户门。
- [x] **T-09** CLI `--profile`(crates/autoterm-ui)+ 配置默认值接入。
  依赖:T-06。关联:AC-06。验证:autoterm.exe 冒烟。
  [✅ 已完成 2026-09-20 · auto-term 231ddac]`--shell` 改可选(显式
  覆盖),新增 `--profile`;解析链 显式 shell > profile > 配置
  default > pwsh 兼容缺省(COMSPEC 兜底属 app 轨 spawn 面——与计划
  G6 字面差异已记 §10.6);AppConfig 增 argv/cwd,PtySession::
  spawn_in 投递(引擎零改动)。冒烟:`--profile ghost` → 明确报错
  含搜索路径 **exit 1**;`--profile cmd --dev-exit-after 3 --dev-dump`
  → 转储含 `D:\autostack>` prompt(cwd 经 spawn_in 生效)。
- **T-10** B 部门禁:实机配方三则 + vue 轨 + oracle 零 diff 门 +
  specs 落盘(SD-01/02/03)。依赖:T-07/T-08/T-09。关联:
  AC-04/05/06/07。
  [进行中 2026-09-20]已毕:specs 落盘(docs/specs/terminal-scroll-
  render.md 新增;app-unified-entry.md/terminal-mux-model.md 增补);
  oracle 与 autoterm-core `git diff` 零文件(AC-07);vue back HTTP
  剧本 T-07 8/8;CLI 冒烟(exit 1 + cwd prompt)。待办:rust 轨
  实机配方三则(default ash 首_TAB/profile 菜单建_TAB/删配置降级
  cmd)+ vue 轨 `auto run -r vue` 冒烟——**用户门**;vue 前端
  prod 构建 TS 报错为 646 遗留(§10.8)。

新路径:`crates/autoterm-config/`(新 crate)、
`docs/specs/terminal-scroll-render.md`(新 spec)、
`docs/plans/evidence/025/`(证据)。

## 9. 复审记录

- 2026-09-20 · stage: new · PLAN-025 rev1 起草 handoff。
  取证完成(闪屏根因候选锚到 widget.rs 行号;profile 接入点锚到
  spawn_ex/db.at/api.at/CLI);A/B 两部独立可并行;待用户放行 work。
- 2026-09-20 · stage: work · PLAN-025 · rev1 · 执行中场记录(非
  execution_done)· code: auto-lang `fd380a64f`(+`bb6dd5d22` 仪器)/
  auto-term `231ddac`(+`87d94c9`)· tasks: T-01..T-04, T-06..T-09
  完成(T-04 判决豁免转 DEBT)· evidence: evidence/025/t01-verdict.md
  (判决曲线)、t07_profiles.sh + _run.log(8/8)、CLI 冒烟 exit1 +
  cwd prompt、rust 轨 boot `[P25-ROWS]` 增量重建、terminal 系回归
  47 绿、oracle/autoterm-core 零 diff · blockers: 无 · next: T-05/
  T-10 用户实机门(滚动录屏肉眼验收 + 配方三则 + vue 轨冒烟)。

## 10. 待澄清事项

1. **config 文件位置**:推荐 exe 同目录 + `AUTOTERM_CONFIG` env
   覆盖(贴合三件套分发);备选 %APPDATA%\autoterm\(WT 习惯,
   多用户/多版本共存友好)。按推荐执行,用户复审时可改判。
2. **VM/desktop 轨 profile 臂**:非目标理由 = PLAN-024 执行中,同
   轨 shim 改动有合流冲突;建议 024 归档后另立小计划补
   `vm/ffi/term_engine.rs` 臂。可改判进本计划(任务面 +1)。
3. **profile V1 不含 `ctrl_c_mode`**:app 轨 Ctrl+C 策略(015 菜单
   interrupt/裸键纯字节)与 CLI `--ctrl-c-mode` 不同源,贸然入
   schema 易语义分叉;记 DEBT 待两轨策略统一后进 V2。
4. **`+` 菜单形态**:右键/长按/下拉三选一,倾向右键(与 Tab 条
   既有右键关闭一致),T-08 实现时按 app.at 菜单既有能力取形。
   [已定 2026-09-20:T-08 取右键 + 行内按钮组形态落地。]
5. **T-04 即时泵 DEBT**:判决豁免(见 T-01 判决工件与 spec 悬置节)。
   启用条件 = 实机验收仍见可见薄带(极端连滚下 1-2 行、≤50ms)。
6. **CLI 缺省差异**:计划 G6 字面 = "显式参数 > 配置文件 > COMSPEC";
   CLI 实现为 … > pwsh 兼容缺省(既有 `--shell` default pwsh 的行为
   保持,避免破坏现有用户;COMSPEC 兜底属 app 轨 spawn 面且已生效)。
   复审可改判统一为 COMSPEC。
7. **全量套件基线归属**:auto-lang lib 全量(ui-iced,iced-layout-tests)
   在 master 基线即有 ~270-280 红且逐跑抖动(夹具编译/文件系统/图像
   注入类)。025 工作树对照:失败集差分 ≤4 项,单测重跑全绿(抖动/
   环境项:app_registry 聚合、mcp thumbs、charts、plan536);terminal
   域(025 战场)全绿。结论:非 025 回归。022 期"~25 项存量红"为
   无 iced-layout-tests 组合下的口径,本组合基线红更多,属在案事实。
8. **vue 前端 prod 构建失败(既有)**:`auto build -r vue` 的
   `vue-tsc` 报 `import.meta.env`(PLAN-646 模板代码,vite/client
   类型缺席)——025 之前已存在,与本计划改动无关(gen 未变更);
   B 部 vue 验收走 back HTTP 剧本(020 惯例),不依赖前端 prod
   build。归属 646 遗留,待另立修复。
9. **app-back main.rs 路由表**:`-r vue` 再生成链在前端 TS 失败点中断,
   未重写 main.rs 路由表(手补 2 行过审;生成物不入库,合并期全量
   再生即消)。与 §10.8 同根。
10. **PLAN-024 合流协调**:024 分支(auto-term-dev,未并 master)有
   9 个 `fix(term)` 提交触及 ui/terminal(widget +322 行:抖动根修/
   圆角/几何源),与 025 A 部同文件不同域(025 = 行缓存/内容层,
   024 = bind 状态机/样式)。master 基线开发成立;合并时
   observe/bind 区可能文本冲突,按两计划语义调和(025 不动 bind
   语义,024 不动行缓存)。
