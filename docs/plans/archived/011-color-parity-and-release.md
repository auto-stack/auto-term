---
plan_id: PLAN-011
status: archived
feature_name: 产品化收口——色彩对拍去 skip + 引擎/复刻应用 release 打包
author: [衍星居士]
created_at: 2026-09-07
updated_at: 2026-09-07 (archived)

# /auto-plan:review 时填写
supersedes_spec_components:
  - "auto-term crates/autoterm-parity/tests/parity_gate.rs: parity_color_attestation 去 skip 重写——ash 分支删除,ANSI 注入(输出路径)六场景全实跑,批次 7 行标量色面逐格等价+非空转哨兵"
  - "auto-term at-gen/src/engine.rs: DLL 解析顺序增「exe 同目录」候选(003 §5 dist 同目录分发所需);新增 engine_row_style glue(签名对齐 autoterm_engine_row_style)"
  - "auto-term docs/designs/003-ash-compatibility.md §5: 第 6 条追补 release 打包口径(scripts/package-at.cmd 一键 dist 三件套,--release 后缀随构建目标)"
  - "auto-term at-gen/README.md: 增「打包」节(脚本用法/dist 清单/DLL 解析顺序/冒烟口径)"
  - "auto-term DEBTS.md: 009 观察 #8 补账——色彩对拍 ash 依赖销账(「ash 缺席 skip」→「ANSI 注入实跑」);复审遗留 #4/#5 维持核对在案"
new_spec_components:
  - "auto-term scripts/package-at.cmd: 一键打包(release 构建 core cdylib+helper+at-gen → dist 三件套 → 布局冒烟;纯 ASCII 批处理/findstr /c: 实测口径内注)"
  - "auto-term at-gen scenario color + STYLE 协议: ROW 全网格文本 + STYLE <row> <col> <fg> <bg> 逐格样式标量 dump(glue engine_row_style 消费面)"
  - "auto-term COLOR_PS_CMD 输出路径注入机制: powershell 单行命令经 cmd 会话向 stdout 输出 SGR 批次(命名 0-15/索引 16-255 抽样/RGB,2J+H 锚点相对行布局);输入管注入实测证伪在案(计划待澄清4)"
touched_goals:
  - "DEBTS 对拍门禁产品化收口(PLAN-011): 六场景全实跑零 skip + release 打包一键化;两仓零新依赖保持;无环铁律保持(at-gen→auto-lang 单边)"

current_step: 4
total_steps: 5
---

# [PLAN-011] 产品化收口(色彩对拍去 skip + release 打包)

## 变更摘要

PLAN-010 完成后,009/010 的成果还剩两类"开发态尾巴",本计划将其
产品化收口(单仓作业,auto-term):

1. **色彩对拍去 skip**:对拍门禁最后一个 skip(`parity_color_attestation`,
   ash 缺席即跳)重构为**直接 ANSI 注入对拍**——oracle 与 a2r 两侧引擎
   喂同一 SGR 字节流(前景/背景/256 色/RGB 混批),比较 `row_style`
   标量色面(kind<<24|value 编码)逐格等价。不再依赖 ash:色彩语义由
   VT 字节流本身定义,注入即可复现(006 S2 口径:语义等价)。组件侧
   色渲染已有 headless 断言(009 styled feed cells),本计划收口的是
   **引擎样式面端到端**;桌面 VM 桥的样式接入仍归 OS-013 后续
   (边界,见待澄清)。
2. **release 打包方案**:at-gen 目前只有 debug 态 + 兄弟仓路径依赖。
   建立打包脚本:release 构建 `autoterm-core`(cdylib)+ `at-gen`,
   组装 `dist/`(autoterm-at.exe + autoterm_core.dll +
   autoterm-ctrlc.exe 清单),dist 布局冒烟(exe 与 DLL 同目录可跑);
   契约口径(003 §5 release 后缀)核对入文档。

顺带:DEBTS「009 新增观察 #4 色彩自证 ash 依赖」销账口径更新;
色彩场景的语言表述从「ash 缺席 skip」改为「ANSI 注入实跑」。

## 目标

1. 对拍门禁**零 skip**:6 场景全实跑(色彩场景以 ANSI 注入实跑,
   ash 依赖移除);
2. `scripts/package-at` 一键产出 `dist/` 且布局冒烟通过
   (exe+DLL+helper 同目录、release 态);
3. at-gen release 构建绿(autoterm-at.exe release 态可跑 smoke);
4. 文档入账:003 §5 release 口径、at-gen README 打包节、DEBTS
   #4 销账更新;
5. 回归底线:auto-term workspace 全绿、auto-lang tt 档不回归
   (本计划不动 auto-lang)。

### 非目标(明确排除)

- VM 桥样式接入(row_style 经 VM ffi 进桌面)——OS-013 后续条目;
- 会话层 tabs/splits(下一个大相位,单独立项);
- ash 真实集成场景(003 兼容矩阵的桌面复验归桌面侧);
- Unix 基座(DEBTS #6 维持)。

## 架构方案

无新架构,两条收口原则:

1. **色彩等价 = 标量色面逐格等价**:注入的 SGR 批次覆盖
   Named(0-15)/Indexed(16-255)/RGB 三类;比较维度 = `row_style`
   的 fg|bg 标量序列(编码 `kind<<24|value`,009 T6 定稿)。
   oracle 读 `PtySession.term.visible_styled_lines`,a2r 读 glue
   `engine_row_style`(新增,对齐 ffi.rs 签名);
2. **打包 = 布局契约的机械化**:003 §5 三件套(exe+DLL+helper)
   同目录 + smoke 断言(从 dist 目录跑 `scenario echo` 即证布局)。

## 技术栈

零新依赖。ANSI 注入用既有 `write_input`/`engine_write_line`;
打包脚本用仓库现有脚本形态(cmd/ps1)。

## 需求分析与背景调查

### spec 依据

- P009-5(测试设计)/P010 沉淀:对拍门禁的 skip 语义与实编门
  「显式 skip 不得静默」惯例;
- DEBTS「009 新增观察 #4」:色彩自证 ash 依赖——本计划以注入法
  移除该依赖后销账口径更新;
- 003 §5(010 T5 已补 cdylib 条):打包清单的契约依据。

### 现状证据(file:line)

| 事实 | 证据 |
|---|---|
| 色彩场景 skip 位 | `crates/autoterm-parity/tests/parity_gate.rs` `parity_color_attestation`(AUTOTERM_ASH_BIN 缺席即 return) |
| row_style 面已就绪 | `crates/autoterm-core/src/ffi.rs` `autoterm_engine_row_style`(fg|bg 交错 u32,cap 校验) |
| oracle 样式读面 | `crates/autoterm-core/src/term.rs` `visible_styled_lines`(StyledChar fg/bg) |
| a2r glue 缺 row_style | `at-gen/src/engine.rs`(仅 7 函数;注:`engine_feed_snapshot` 已内联进 engine_rows) |
| 打包件已产 | `target/debug/autoterm_core.dll`、`target/debug/autoterm-ctrlc.exe`、`at-gen/target/debug/autoterm-at.exe`;release 态未构建 |
| 003 §5 契约 | `docs/designs/003-ash-compatibility.md:198-205`(exe+helper+cdylib 随包) |

## 详细设计

### T1 [auto-term] 色彩对拍场景重构(ANSI 注入)

- `at-gen/src/engine.rs` 增 `engine_row_style(handle, row, out, cap)`
  (对齐 ffi.rs 签名)与 `engine_write_raw`(不补 \r\n 的原始写);
- at-gen scenario 增 `color` 模式:注入 SGR 批次(如
  `ESC[31mR ESC[42mG ESC[38;5;208mI ESC[48;2;10;20;30mB ESC[0m` …,
  覆盖 0-15/16-255/RGB 抽样)→ tick → 按行输出样式标量
  (`STYLE <row> <col> <fg> <bg>` 协议行)→ `SCENARIO_OK`;
- `parity_gate.rs`:`parity_color_attestation` 重写——oracle
  `write_input(同字节流)` → drain → `visible_styled_lines` 提取
  标量色;a2r `A2r::spawn("color")` 解析 STYLE 行;两侧逐格
  assert 相等(删除 ash 分支与 skip);
- 注入批次的行布局确定性:单行内多 run,行号固定,规避时序噪声
  (无需 wait_prompt 以外的同步点)。

验证:`cargo t -p autoterm-parity` 全绿(6 场景零 skip)。

### T2 [auto-term] release 打包脚本

- `scripts/package-at.cmd`(或 ps1,沿仓库脚本惯例):
  ① `cargo build -p autoterm-core --release`(cdylib+helper)
  ② `cd at-gen && cargo build --release`
  ③ 组装 `dist/`:autoterm-at.exe + autoterm_core.dll +
     autoterm-ctrlc.exe(+ LICENSE)
  ④ 布局冒烟:dist 目录内跑 `autoterm-at.exe scenario echo`,
     断言 SCENARIO_OK;
- `at-gen/Cargo.toml` 确认 release profile 缺省即可(不引自定
  profile);exe 目录向上 4 级找 DLL 的既有解析(009)对 dist
  布局天然成立。

验证:脚本跑通,`dist/` 三件在案,冒烟 SCENARIO_OK;`autoterm-at
smoke` release 态复跑绿。

### T3 [auto-term] 文档入账

- 003 §5 部署条补 release 口径一句(`--release` 后缀随构建目标);
- `at-gen/README.md` 增「打包」节(脚本用法 + dist 清单);
- DEBTS 009 新增观察 #4:色彩自证 ash 依赖移除,销账口径更新。

验证:grep 三处命中。

### T4 [auto-term] 回归 + 收账

- `cargo test --workspace`(61+ 基线不回归);
- DEBTS 复审遗留两项状态核对(011 不涉,维持记录)。

验证:workspace 全绿;grep 在案。

## 测试设计

- 自动:parity 6 场景零 skip(含新色彩逐格等价)+ dist 布局冒烟
  + release smoke;
- 回归:workspace 全量、auto-lang tt 档(不动上游,跑一次确认)。

## 验收标准

1. parity 门禁 6 场景全实跑,**零 skip**,含色彩逐格等价;
2. 打包脚本一键产出 dist/ 三件套,布局冒烟 + release smoke 绿;
3. 003 §5 / at-gen README / DEBTS 三处文档更新命中;
4. workspace 回归全绿;两仓零新依赖。

## 执行步骤

> 单仓作业(auto-term),worktree `.wt/auto-011/auto-term`
> (分支 `plan-011-dev`),无依赖仓改动。

- **T1** [auto-term] 色彩对拍重构:glue 增 row_style/write_raw +
  scenario `color` + 门禁重写去 skip。
  验证:parity 6 场景全绿零 skip。
  [✅ 已完成] commit 2f46980;注入机制按用户裁定改**输出路径**
  (powershell 单行命令经 cmd 会话向 stdout 输出 SGR 批次,2J+H 固定
  行布局)——实测输入管注入 SGR 仅被 cmd 字面回显、网格零样式格
  (PROBE_STYLED_CELLS=0),字面方案只会得到空转等价;write_raw 随之
  不需要(见待澄清4)。glue 增 engine_row_style;门禁重写为批次 7 行
  标量色面逐格等价 + 非空转哨兵(≥24 种非默认标量);
  `cargo t -p autoterm-parity` 5 测试 6 场景全绿零 skip。
- **T2** [auto-term] 打包脚本 `scripts/package-at` + dist 冒烟。
  验证:dist 三件套 + scenario echo 在 dist 布局内 SCENARIO_OK。
  [✅ 已完成] commit 815c33d;scripts/package-at.cmd(release 构建
  core+at-gen → dist 三件套 → 布局冒烟 exit 0);dist 内 release
  smoke UI_SMOKE_OK。计划偏离两处随事实入账:① glue DLL 解析增
  「exe 同目录」候选(既有解析只扫 target/debug 祖先,「对 dist
  天然成立」前提不实,dist 冒烟实证需要);② 实测两坑记录:cmd
  批处理按 OEM 代码页解析(含中文注释即炸,脚本纯 ASCII)+
  findstr /x 对 LF-only 流匹配失败(改 /c: 子串)。
- **T3** [auto-term] 文档三处入账(003 §5 / at-gen README / DEBTS)。
  验证:grep 命中。
  [✅ 已完成] commit c890ff2;grep 三处命中(003 §5 release 口径 /
  README「打包」节 / DEBTS #8)。DEBTS 实际无「#4 色彩自证 ash」
  条目(原引为误指,#4 实为 Vue/web 留白),销账以新增观察 #8 补账
  并在案注记误引。
- **T4** [auto-term] 回归 + 收账。
  验证:workspace 全绿;auto-lang tt 一次确认不回归。
  [✅ 已完成] `cargo test --workspace` 61 passed / 0 failed
  (61+ 基线吻合);auto-lang tt 2989 passed / 1 failed
  (`test_charts_gallery_compiles` = master 既有基线红,010 复审
  已记录「charts+7 book=master 既有基线」,011 未动 auto-lang,
  零新增红)/ 340 skipped;DEBTS 复审遗留两项(#4 Vue/web 留白、
  #5 性能余量)grep 在案,维持记录。
  → 收尾 fold(merge 技能流程)。

## 复审记录

**reviewer**: ZCode(/auto-plan:review) · **时间**: 2026-09-07 ·
**worktree**: `.wt/auto-011/auto-term`(plan-011-dev,3 commits
2f46980/815c33d/c890ff2,树干净;auto-lang 兄弟 worktree 零改动于
master 13f4d5253)

### 验收标准逐条复核

1. **parity 门禁 6 场景全实跑,零 skip,含色彩逐格等价** —
   **pass**。复审全量门禁内 parity_gate 5 测试(6 场景,startup+echo
   共函数)`5 passed; 0 failed; 0 ignored`(0 ignored = 零 skip 的
   机械证据);旧 ash 分支已删(diff 在案),色彩测试无条件断言;
   色彩面实测 33 种非默认标量(命名 0-15/索引抽样/RGB 全透传,
   `2<<24|0x0A141E` 类 RGB 精度逐位在案)。
2. **打包脚本一键产出 dist/ 三件套,布局冒烟 + release smoke 绿** —
   **pass**。复审现场重跑 `scripts/package-at.cmd` exit 0(四步
   trace 全过),dist 三件在案;dist 内 `scenario echo` 布局冒烟
   SCENARIO_OK + `smoke` UI_SMOKE_OK + `scenario color` SCENARIO_OK
   (dist 布局下 glue 走「exe 同目录」DLL 候选,实证)。
3. **003 §5 / at-gen README / DEBTS 三处文档更新命中** — **pass**。
   grep 三处命中(003 §5 release 口径 / README「打包」节 /
   DEBTS #8)。
4. **workspace 回归全绿;两仓零新依赖** — **pass**。复审全量门禁
   `cargo test --workspace` **61 passed / 0 failed / 22 suites /
   exit 0**(61+ 基线吻合);`git diff 39c3053..HEAD` 对两仓
   Cargo.lock/Cargo.toml 均为空;auto-lang 仓零 commit 零改动。

### 遗漏 / 延后 / workaround 猎查

- **遗漏**: 无。T1 计划原文的 `engine_write_raw` 未实现属**用户
  裁定的机制偏离**(输出路径注入使 input 管写函数失去用途,待澄清
  4 + T1 标记 + commit 三处在案),非静默丢弃。
- **延后**: 无。四步全毕;非目标四项(VM 桥样式接入/tabs-splits/
  ash 真实集成/Unix 基座)均为计划明示排除,非执行期缩减。
- **workaround**: 打包脚本纯 ASCII + `findstr /c:`(替代 /x)为
  实测平台事实的适配(OEM 代码页解析/LF-only 流匹配),已内注入
  注,非遮盖式 hack;COLOR_PS_CMD 门禁/at-gen 双处镜像为既有锚点
  复用惯例(同 ECHO_ANCHOR),镜像注释「改一处必改另一处」在案。

### 债候选(均不阻塞)

1. COLOR_PS_CMD 镜像常量漂移风险——两侧失同步即测试红(语义防线
   自然暴露),评审改临时无碍;若后续场景增多可考虑共享常量位。
2. 打包未 CI 化(计划明示非目标,维持)。
3. 色彩场景依赖 powershell.exe 在场(Windows 全系自带;缺席时
   fail-loud 非静默,符合零 skip 惯例)。

### 结论

四项验收全 pass,无阻塞债 → **status: reviewed**,就绪
`/auto-plan:merge`(spec-impact 元数据已填,merge 按单仓 auto-term
收口;auto-lang 兄弟 worktree 无物可折,随 merge 清理)。

## 待澄清事项

1. **色彩等价的 RGB 精度**:alacritty 对 256 色与 RGB 的内部表示
   经 `kind_color` 标量化后应逐位一致;若遇 `Spec` 与 `Indexed` 表示
   漂移(同色异码),归一规则按 kind+value 严格比,暴露即修。
2. **打包形态**:dist 是否纳 `.pdb`/符号——缺省不含,需要时脚本
   加开关(非验收项)。
3. **release 构建时长**:at-gen release 全量首建较慢(iced 栈),
   可接受;CI 化不在本计划。
4. **注入机制偏离计划字面(已裁定,2026-09-07 执行时)**:T1 原文
   「oracle write_input(同字节流)」经探针证伪——ConPTY 输入管收到
   的 SGR 只是键入字节,cmd 按字面回显(^[[31mR…),网格零样式格;
   按字面执行会得到两侧全默认色的空转等价,色彩语义零实证。用户
   裁定改**输出路径注入**:同一 powershell 单行命令经 cmd 会话向
   stdout 输出 SGR 批次(2J+H 固定行布局),两侧引擎同串同管道;
   `engine_write_raw` 随之不需要、未引入。注意:因 ConPTY 在路径中
   会重编码色彩(实测 26200 索引/RGB 均原样透传),对拍语义为
   「同一输出流两侧逐格等价」,而非「注入常量 vs 网格标量」直接
   断言;计划待澄清 1 的「Spec 与 Indexed 漂移」在此口径下两侧同流
   不可能出现,不适用。
