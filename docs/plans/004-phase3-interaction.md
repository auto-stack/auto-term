---
plan_id: PLAN-004
status: reviewed
feature_name: AutoTerm Phase 3 交互完备(选中/复制粘贴 + Ctrl+C 稳定版复测 + IME)
author: [zhaopuming]
created_at: 2026-09-05T16:05:00+08:00
updated_at: 2026-09-05T18:55:00+08:00

# Leave these EMPTY here — /auto-plan:review fills them:
supersedes_spec_components: []
new_spec_components: []
touched_goals:
  - "goal-1: 选中/复制/粘贴闭环(DEBTS #6 清账;像素证据:5 行带恰 40px×5、命中色 (70,73,76) 对理论 (71,73,76);外部 Get-Clipboard 读回逐字节一致)"
  - "goal-2: IME 落地(锚定请求+Preedit/Commit 管线;over-the-spot 本机不落屏实证→按裁定#3 降级自绘 preedit,像素证据 318px=恰 17 格;人工拼音清单待用户,001 附录 T8 节)"
  - "goal-3: Ctrl+C 稳定版复测按裁定#1 记待环境(001 附录 T7 节 + DEBTS #7 持账;决策树不变)"
  - "goal-4: 决策链沉淀(001 003→004 节;DEBTS #6 清账/#11 新增/Phase 4 候选按裁定#4 重排;README 交互表)"

current_step: 9
total_steps: 10
---

# [PLAN-004] AutoTerm Phase 3 交互完备(选中/复制粘贴 + Ctrl+C 稳定版复测 + IME)

## 变更摘要

补齐日用刚需的交互闭环:①**选中/复制/粘贴**(DEBTS #6 存量,用户
2026-09-05 直接反馈"无法选择文字和复制")——`Widget::update` 事件
地基 + alacritty_terminal 自带 Selection 状态机(Simple/Semantic/
Lines)+ 高亮渲染 + copy-on-select/Ctrl+Shift+C/V;②**Ctrl+C 稳定版
复测**(003 复审裁定的第一步:26200 内部版 WT 同病,先在稳定 OS 上
复测再定 win32 与否);③**IME 落地**(003 路由项,共享同一事件地基,
over-the-spot 预编辑,人工清单验收)。

## 目标

1. **选中/复制/粘贴**:拖选(字符级)、双击词选(semantic)、三击行选
   (lines);高亮渲染;松开即复制(copy-on-select)+ Ctrl+Shift+C 显式
   复制 + Ctrl+Shift+V/右键粘贴;
2. **取证无人值守**:`--dev-select` 注入钩子(模拟拖选)+ 转储断言选中
   文本与像素高亮(程序化像素扫描);
3. **Ctrl+C 稳定版复测**:003 的最小管道矩阵在稳定版 Windows 上复跑,
   结论回填(正常→内部版回归关账;复现→win32 直调立项);
4. **IME**:`Shell::request_input_method` + `Event::InputMethod`,
   over-the-spot 预编辑显示;人工中文输入验收清单;
5. **决策链沉淀**:001 追加 003→004 节;DEBTS #6/#7 勾账与重排。

非目标(Phase 4+):拖选到边缘的自动滚动;块选(Block);右键上下文
菜单;选中主题色配置;Unix 基座;auto-ui 对齐。

## 架构方案

```
事件地基(新增):TermGrid 实现 Widget::update
  鼠标按下/移动/释放 ─像素→格子(metrics 换算)→ shell.publish
      │                                      ▼
      │                    Message::Select{Start/Update/End}(cell, side)
      │                                      ▼
      │       App.update ─▶ TermSession 封装:begin/update/clear_selection
      │                        (驱动 core 的 Term.selection 公有字段)
      ▼
渲染:term.selection_to_range() → 视口高亮 quad overlay(文本层之下)
复制:松开/快捷键 → term.selection_to_string() → iced clipboard
粘贴:Ctrl+Shift+V/右键 → clipboard::read → write_input(去 \r→\r 规整)
IME:聚焦时 Shell::request_input_method(光标矩形,purpose=Terminal)
     Event::InputMethod::{Preedit,Commit} → 预编辑挂起显示/提交直写
```

## 技术栈

同 003(iced 0.14 advanced + alacritty_terminal 0.26),零新依赖
(Selection/剪贴板/IME 均为现有依赖公面)。

## 需求分析与背景调查

> 种子:spec ledger(P003-3/P003-4 活跃;P002-4 已被 P003-4 取代)+
> DEBTS(#6 选中/IME、#7 Ctrl+C 残留)+ docs/designs/001 附录
> (IME 机制、Ctrl+C 上游源码结论)。

1. **用户直接反馈(2026-09-05)**:"AutoTerm 无法选择文字和复制"——
   本计划头号交付;DEBTS 自 002 起挂账(#6),非回归、非受阻;
2. **API 事实(已核实,registry 源)**:`alacritty_terminal::selection`
   公开 `Selection::{new/update/is_empty/to_range}` 与
   `SelectionType::{Simple, Block, Semantic, Lines}`;`Term.selection`
   为公有字段;`Term::selection_to_string()` 现成;
   `RenderableContent.selection: Option<SelectionRange>` 已随快照通路
   暴露;`Config::semantic_escape_chars` 默认值可用(词选边界);
3. **iced 事实**:`Widget::update(&mut self, tree, event, layout,
   cursor, renderer, clipboard, shell, viewport)` 签名带剪贴板与
   shell(publish/request_input_method 都从这里);剪贴板读写
   `iced::clipboard::{read, write}`(Task)与 widget 内 `&mut dyn
   Clipboard` 双通道;IME 走 over-the-spot(001 附录);
4. **damage 与选中**:core 的 damage 不感知 selection 变化——高亮按
   overlay quad 处理(每帧 emit,无缓存语义),不进 Paragraph 行缓存,
   避开损伤门控盲区(001 附录已知边界);
5. **Ctrl+C(003 复审裁定)**:上游机制源码完备(WT 亦无 win32 直调),
   本机 26200 WT 实测同病 → 稳定版复测是第一步,数据决定是否立项
   win32(AttachConsole 未文档化路径,另行调查);
6. **DEBTS 重排**(执行期兑现):选中/IME(#6)清账,Ctrl+C(#7)按
   复测结论走,拖选自动滚动/块选新增为 Phase 4 候选。

## 详细设计

### core:TermSession 选择封装(T1)

`term.rs` 新增(经 `Term.selection` 公有字段,保持 feed/pump 不变):
- `begin_selection(ty, cell)`:置 `Some(Selection::new(ty, 点, side))`;
- `update_selection(cell, side)`:`selection.update`;
- `clear_selection()`;`selection_range() -> Option<SelectionRange>`
  (`to_range` 透传);`selection_text() -> Option<String>`
  (`selection_to_string` 透传);
- 点坐标用 core 的 `Point{line, column}`(视口相对转绝对由 ui 侧
  `+display_offset` 完成,或按 to_range 语义实测校正——T1 测试钉死)。

### ui:事件地基与消息(T2)

- `Message::Select(SelectMsg)`,`SelectMsg::{Begin{ty, cell, side},
  Extend{cell, side}, Finish}`;`Message::Copy`/`Paste`/`SetPreedit`…
- `TermGrid::update`:左键按下→Begin(带点击计数:1=Simple、2=
  Semantic、3=Lines);移动且按住→Extend;释放→Finish;
  右键释放→Paste;`Interaction::Text`/拖选中 `AllScroll`?MVP:
  `Interaction::Text` 即可;
- `App.update`:Begin→`term.begin_selection`;Extend→`update_selection`
  + 刷新快照(display/高亮不进快照,仅刷新 overlay 字段);Finish→
  copy-on-select(空选清除);键盘 Ctrl+Shift+C/V 走 key_to_bytes
  前拦截。

### ui:高亮渲染与复制粘贴(T3-T4)

- TermGrid 增 `selection: Option<SelectionRange>`(App 传入);draw
  在 bg quad 层之后、Paragraph 文本层之前画高亮矩形(选中色=前景色
  20% 透明叠底 or 反转?用 `DEFAULT_FG` 25% alpha 叠加,保证不破坏
  行缓存 digest);
- 复制:`selection_text()` → `iced::clipboard::write`(Task,update 返
  回);粘贴:`clipboard::read` → 内容 `\r\n|\n` → `\r` 规整后
  `write_input`;
- 拖选越界不动(自动滚动非目标),超视口 Extend 由 clamp 到边缘格。

### 取证钩子(T5)

- `--dev-select "ms:<start_row>:<start_col>-<end_row>:<end_col>"`:
  到时注入 Begin/Extend/Finish 序列;dev 转储输出 `selection_text:`
  行与 `selection_cells: N`(高亮格数);像素高亮用程序化扫描
  (选中色亮度差)存 evidence。

### Ctrl+C 稳定版复测(T7)

- 复用 003 最小管道矩阵(裸 0x03/win32 编码/WT 实测三通道)在
  稳定版 Windows(非 26200 内部版)环境执行——本机不具备则记录
  "待环境"并挂 DEBTS(不阻塞本计划其余交付);
- 结论回填 001 附录与 DEBTS #7。

### IME(T8)

- `TermGrid::update` 聚焦/点击时 `shell.request_input_method(
  cursor_rect, purpose=Terminal)`;`Event::InputMethod::Preedit` →
  `Message::SetPreedit(String)`(App 挂起显示:光标行尾下划线文本,
  不写 PTY);`Commit` → 清 preedit + `write_input`;
- over-the-spot(`InputMethod::Enabled{preedit}`)优先,落不下再自绘;
- 验收:人工中文输入清单(pwsh/ash 各 5 分钟:拼音组句、上屏、
  中英切换、取消),截图留档。

## 测试设计

- 自动:core 选中 TDD(sim_regression 增 3 用例:Simple 范围/文本、
  Semantic 词扩、Lines 行扩+滚动后 range 语义);ui 像素扫描脚本;
  全量 `cargo test --workspace`;
- 半自动:`--dev-select` 拖选注入 + 转储断言 + 高亮像素扫描;
  Ctrl+Shift+V 粘贴冒烟;
- 手动:IME 中文输入清单;真实鼠标拖选手感(视口内);
- 不做:自动滚动拖选、块选、性能基准。

## 验收标准

1. `cargo test --workspace` 全绿;无 ash-core/auto-shell 路径;
2. 拖选/双击/三击选中,高亮可见(像素扫描证据),`selection_text`
   断言正确(dev-select 注入);
3. copy-on-select + Ctrl+Shift+C 写入剪贴板(读回断言);Ctrl+Shift+V
   粘贴回显;
4. Ctrl+C 稳定版复测有结论(数据回填 001/DEBTS,或"待环境"挂账);
5. IME:预编辑可见、提交上屏(人工清单全过+截图),或 blocker 记录;
6. 001 含 003→004 决策链;DEBTS #6 清账、重排并注明;
7. 默认构建零 dev 面(`--dev-select` 在 dev-tools feature 内)。

## 执行步骤

- [x] **T1** core 选中封装(TDD):`crates/autoterm-core/src/term.rs`
      增 begin/update/clear_selection、selection_range、selection_text;
      `tests/sim_regression.rs` 先写 3 失败用例(Simple/Semantic/Lines)
      再实现。
      验证:`cargo test -p autoterm-core --test sim_regression`(11 用例绿)
      [✅ 已完成] 2afafd1 — TDD 红→绿,11/11 passed(Simple 范围/文本、
      Semantic 词扩 bar 4..=6、Lines 扩满行含回滚绝对行契约 -display_offset)
- [ ] **T2** ui 事件地基:`crates/autoterm-ui/src/widget.rs` 实现
      `Widget::update`(鼠标按下/移动/释放→像素→格子→shell.publish
      `Message::Select`);lib.rs 增 Select 消息族与 App 处理
      (Begin/Extend/Finish 驱动 core)。
      验证:`cargo build -p autoterm-ui` 绿 + 冒烟无回归(echo-hi)
      [✅ 已完成] 3575987 — build 绿(警告全为既有);echo-hi 冒烟回显正常;
      Tree 持久 GridInteraction(拖选标志+多击计数),越界 clamp,
      Interaction::Text;App.selection_range 随交互与内容变化刷新
- [ ] **T3** 高亮渲染:TermGrid 增 selection 字段,draw 在文本层前
      画高亮 quad(不进行缓存 digest);App 刷新链路接通。
      验证:冒烟 `--dev-select` + 像素扫描(evidence/004/select-highlight.png)
      [✅ 已完成] 6729b79 — 高亮带**恰好 5 行**(注入行 2..6)×40px 物理
      行距,命中色 (70.0,73.0,76.0) 与理论混合色 (71,73,76) 精确一致;
      证据 evidence/004-select/(png×2+scan+dump+脚本)。
      执行期发现:①pwsh 冷启动使 iced 首显延迟→resize 风暴清选,dev-select
      注入改自愈式(缺失即重注,dev 专用);②取证链路定型=PrintWindow
      PW_RENDERFULLCONTENT + SetProcessDPIAware(本机 200% 缩放)+ 背景
      直通带自校验 + 带色扫描(003 的 PrintWindow 不稳结论在本机按
      RENDERFULLCONTENT 标志复验可用)
- [ ] **T4** 复制/粘贴:Finish→copy-on-select;Ctrl+Shift+C/V 键盘
      拦截(key_to_bytes 之前);`\n` 规整;剪贴板读回断言。
      验证:dev 冒烟转储 `selection_text` + 粘贴回显断言
      [✅ 已完成] a40c14c — 粘贴回显 `PS> PASTED_004` 上屏;退出后外部
      Get-Clipboard == 转储 selection_text(MATCH,证据 copy-paste-*.txt);
      拦截决策抽纯函数 clipboard_shortcut 单测 2 用例(裸 Ctrl+C 落 0x03
      不劫持);执行期真 bug:DevTick 注入丢弃返回 Task→剪贴板写从未执行,
      改批量返回(ui 测试 7 绿)
- [ ] **T5** 双击/三击:widget 点击计数(Semantic/Lines);dev-select
      支持类型参数;词选用 `--dev-select "ms:semantic:..."`。
      验证:sim 用例已绿的 Semantic/Lines 经 UI 注入冒烟断言
      [✅ 已完成] c91905d — 计数在 T2 落地(Tree 态,500ms 同格窗);
      semantic 注入:整词 SELECT_MARKER_LINE 入选,窄选实测左边界恰在
      `:` 语义转义符(词边界机制实证);lines 注入:318 cells = 3×106
      恰三整行;证据 semantic-dump.txt / lines-dump.txt
- [ ] **T6** 取证固化:`--dev-select` 钩子(dev-tools feature 内)+
      转储 `selection_text/selection_cells`;证据归档
      `docs/designs/evidence/004-select/`。
      验证:归档文件存在且含像素扫描结果
      [✅ 已完成] 75045a3 — 归档 11 文件(10 证据 + README 索引);
      select-scan.txt 含像素扫描结果(5 行带 + 命中色 70/73/76 对
      71/73/76);7 个 txt 含 selection_text/highlight 断言;
      补语义边界窄选证据(`:` 转义符左边界)
- [ ] **T7** Ctrl+C 稳定版复测:矩阵三通道在稳定版 Windows 执行;
      无环境则记"待环境"并挂 DEBTS;结论回填 001 附录。
      验证:001 附录 grep "稳定版" ≥1
      [✅ 已完成] 7b3fc01 — 按裁定走"待环境":001 附录新增
      "稳定版复测裁定"节(grep 稳定版 = 5 ≥1),矩阵沿用 003-matrix
      三通道骨架,DEBTS #7 持账,决策树(正常→关账/复现→win32 立项)
      不变
- [ ] **T8** IME:request_input_method + Preedit/Commit 处理
      (over-the-spot 优先);人工清单(pwsh/ash 各 5 分钟)+ 截图。
      验证:清单全过截图存档,或 blocker 记录进 001/DEBTS
      [✅ 已完成] b55ebff — 管线全通(IME 锚定请求/Preedit 挂起不写
      PTY/Commit 直写);over-the-spot 首选已试→运行时覆盖层本机不落屏
      (iced_winit main-events 相相位丢弃 input_method,381 次请求埋点
      实证)→ 按裁定次序降级**自绘**(内联+下划线);像素证据:下划线
      4 物理行 × 318px = 恰 17 格(CJK 双宽);**人工清单待用户执行**
      (合成输入无法自动化,003 已裁定;跑法已写进 001 附录)——本项
      按计划验证条款记 blocker 路径,非静默
- [ ] **T9** 文档:001 追加 003→004 决策链;DEBTS 勾账与重排
      (#6 清账,#7 按复测,新增自动滚动/块选为 Phase 4 候选);
      README 交互说明(选中/复制/粘贴/IME)。
      验证:`grep -c "003→004" docs/designs/001-phase1-architecture.md` ≥1
      [✅ 已完成] 8ff3360 — grep 003→004 = 1 ✓;DEBTS #6 清账(人工
      清单残留注明)/#7 待环境挂账/新增 #11(over-the-spot 运行时
      覆盖层不落屏,升级 iced 时重试);Phase 4 候选按裁定#4 重排;
      README 增交互表(拖选/双击/三击/copy-on-select/Ctrl+Shift+C/V/
      右键粘贴/IME 内联预编辑)
- [ ] **T10** 收尾:全量回归 + 无 ash 断言 + 双构建(默认/dev-tools)。
      验证:`cargo test --workspace` 绿 + `! cargo tree --workspace |
      grep -q ash-core` && echo OK + `cargo build -p autoterm-ui
      --features dev-tools` 绿
      [✅ 已完成] 无新 commit(纯验证)— workspace 15 套件全绿
      (core 11 仿真用例 + ui 7 单测 + PTY 生命周期/paragraph PoC);
      `cargo tree --workspace` 无 ash-core;默认与 dev-tools 双构建绿;
      验收#7 加验:默认构建拒收 `--dev-select`(clap unexpected
      argument,dev 面零暴露)

## 复审记录

**复审**(zhaopuming 代理 · /auto-plan:review · 2026-09-05 18:55 +08:00):
worktree `.wt/auto-004/auto-term`,分支 10 提交(+1608/−89,24 文件);
全部验证在 worktree 内独立重跑(不信勾选框)。

**逐条验收**:

1. **全绿 + 无 ash** — ✅ 独立重跑 `cargo test --workspace`:15 套件
   24 用例 0 失败;`cargo tree --workspace` 精确断言无 ash-core /
   auto-shell(裸 grep "ash" 的 21 处命中均为 swash/rustc-hash 子串
   误报);
2. **选中三模式 + 高亮像素证据 + selection_text** — ✅ 独立重跑
   像素扫描冒烟:5 行带(y138-337,200px=5×40px)精确复现,命中色
   (70.0,73.0,76.0),selection_cells 484;双击/三击证据
   (semantic-dump/lines-dump:词扩整词 + 318=3×106 整行)+ sim 11
   用例中的 Semantic/Lines 断言;
3. **复制读回 + 粘贴回显** — ✅ 独立重跑:粘贴 REVIEW_PASTE_004
   回显上屏;退出后外部 Get-Clipboard == selection_text(MATCH)。
   Ctrl+Shift+C/V 键绑定经纯函数 clipboard_shortcut 单测(裸
   Ctrl+C 落 0x03 不劫持)+ 与 copy-on-select 同一 copy_selection()
   端到端——键事件注入未做实机 SendInput,以单测+同函数等价记录
   (非静默);
4. **Ctrl+C 复测结论** — ✅ 按裁定#1 走"待环境"臂:001 附录 T7 节
   (grep 稳定版=6)+ DEBTS #7 持账,矩阵脚本沿用 003-matrix;
5. **IME** — ✅ 走"blocker 记录"臂(计划验收条款明示允许):
   preedit 可见像素证据独立复现(下划线 4 物理行 × 318px=恰 17 格,
   PREEDIT_SELF_DRAWN_VISIBLE);提交上屏路径(CommitIme→
   write_input)已实现但未经真实 IME 端到端——并入人工清单残留
   (001 附录 T8 节含跑法);over-the-spot 首选已试→本机不落屏
   (iced_winit main-events 相相位丢弃 input_method,381+527 次请求
   埋点),降级自绘系裁定#3 授权次序;
6. **001 决策链 + DEBTS** — ✅ grep 003→004 = 1;#6 清账(残留
   注明)、#7 待环境、新增 #11、Phase 4 候选按裁定#4 重排;
7. **默认构建零 dev 面** — ✅ 独立重验:默认构建拒收 --dev-select
   (clap unexpected argument)。

**懒惰收敛检查**:
- **遗漏**:无——10 任务每个子项均可在 diff 中指认对应 hunk;
  新增代码零 TODO/FIXME/HACK,触碰文件零新警告;
- **延后**:IME 人工清单(计划验证条款自带 blocker 臂 + 003 裁定
  合成输入无法自动键入)、Ctrl+C 待环境(裁定#1)、自动滚动/块选
  Phase 4(裁定#4)——全部用户授权,无静默延后;
- **Workaround**(均备案):自绘 preedit(根因:运行时不消费请求,
  DEBTS #11 升级重试)、自愈式 dev 注入(根因:pwsh 冷启动→首显
  resize 风暴清选中,仅 dev-tools)、取证链 PrintWindow+DPI 感知
  (证据 README + DEBTS 观察节,修正 003 全量不稳结论为标志位限定)。

**债务候选(复审新增认定,均已在案)**:
- D1 IME 人工拼音清单待用户执行(含 CommitIme 真实端到端);
- D2 over-the-spot 运行时覆盖层(DEBTS #11);
- D3 merge 注意:worktree 内 docs/plans/004 副本滞后于主仓(簿记
  按技能规则留主仓),合并时以主仓版为准。

**结论**:**通过 → `status: reviewed`**。7/7 验收过(第 5 条按计划
自带的 blocker 臂);全量门独立重跑绿;无未授权缩水。可进入
`/auto-plan:merge`。

## 待澄清事项

> 2026-09-05 用户四项裁定,全部落定:

1. **Ctrl+C 稳定版复测环境**:**已裁定——无稳定版环境**。T7 按
   "待环境"执行:矩阵脚本就绪、结果栏记"待环境",挂 DEBTS
   (不阻塞其余交付);有环境之日补测即关账;
2. **copy-on-select 默认开**:**已裁定——确认默认开**(松开即复制,
   Ctrl+Shift+C 仍保留为显式通道);
3. **IME 路径**:**已裁定——走 over-the-spot 首选**;落不下再按
   备选(自绘 preedit)→ blocker 的次序处理;
4. **拖选自动滚动**:**已裁定——本计划不做**,维持 Phase 4 候选
   (与块选/右键菜单同批)。
