---
plan_id: PLAN-003
status: execution_done
feature_name: AutoTerm Phase 2 性能与输入完备性(保留式画布 + Ctrl+C 矩阵 + IME)
author: [zhaopuming]
created_at: 2026-09-05T13:27:54+08:00
updated_at: 2026-09-05T13:45:00+08:00

# Leave these EMPTY here — /auto-plan:review fills them:
supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 12
total_steps: 12
---

# [PLAN-003] AutoTerm Phase 2 性能与输入完备性(保留式画布 + Ctrl+C 矩阵 + IME)

## 变更摘要

兑现 PLAN-002 路由到 Phase 2 的三件事:①**保留式画布**——widget
每行缓存 iced `Paragraph`,`Paragraph::compare` 判异、仅脏行重建
形状、`fill_paragraph` 绘制,达成计划002 验收#4 的绘制级 run 不等式;
②**Ctrl+C/Ctrl+Break/关闭语义矩阵**——三 shell × 中断路径的自动化
冒烟;③**IME 最小调查与落地**——iced `input_method` 模块可用性
PoC,行则做光标处 preedit 下划线显示,阻则 blocker 记录。顺带清偿
三尾巴:`--dev-*` 收进 `dev-tools` feature、log+env_logger 替换
eprintln、`Message::NoOp` 替换空唤醒复用。

## 目标

1. **绘制级损伤剪裁**(计划002 验收#4 兑现):单行改动帧的
   Paragraph 重建数 < 全网格行数(dev 转储计数断言),Full 损伤退化为
   全量重建;
2. **语义矩阵自动化**:pwsh/ash/cmd × {Ctrl+C 字节, 窗口关闭}
   中断运行中命令 → 提示符回归/无孤儿断言;Ctrl+Break 调查并给出
   映射或平台结论;
3. **IME**:PoC iced 0.14 `input_method`/Ime 事件管道;可用则
   preedit 在光标处下划线渲染(最小),不可用则 blocker 记入 DEBTS
   与 001 附录;
4. **三尾巴清偿**:dev-tools feature flag(002 待澄清#5)、
   log+env_logger(002 复审债务候选)、Message::NoOp(002 复审瑕疵);
5. **决策链沉淀**:001 设计文档追加 002→003 节(Paragraph 路线/
   矩阵结论/IME 结论),DEBTS 勾账。

非目标(Phase 3+):Unix 基座;iced↔auto-ui 对齐;选中/主题;光标
闪烁与形状(IME 顺手才带);列级损伤剪裁(行级已够)。

## 架构方案

```
保留式画布(在现有 TermGrid 内演进,不新增 crate):
  Vec<Option<RowPara>>  ── compare(snapshot 行文本) ──▶
    未变:复用 Paragraph(fill_paragraph 直接绘制)
    变化/缺失:重建(with_text+load)→ 计数 paragraph_rebuilds
  damage: Lines(脏行集)→ 只对脏行做 compare;Full → 全部 compare
  (iced 即时模式限制就此绕开:形状/布局缓存归我们,场景 emit 仍全量)
```

语义矩阵与 IME 均为 autoterm-ui 侧增量,core 不动(除 unescape
转义扩展在 ui lib)。

## 技术栈

- iced 0.14(advanced:`text::Paragraph`/`fill_paragraph`/
  `input_method`);
- log + env_logger(替换 eprintln);
- 其余同 002(cosmic-text 0.15 pin、unicode-width)。

## 需求分析与背景调查

> 种子:spec ledger(P002-3 architecture/P002-4 designs,P001-3/4/5
> 已 superseded)+ DEBTS.md Phase 2 候选清单 + docs/designs/001
> 的 Phase 2 路由节。

1. **ledger 现状**:P002-3 事件驱动架构稳定;P002-4 详细设计含
   "Phase 2 路由:绘制级剪裁需保留式画布(每行 Paragraph 缓存,
   `Paragraph::compare` 判差异)"——本计划 T1-T4 兑现该路线;
2. **DEBTS 清单映射**:#1/#2(整帧重拼+无图集→保留式画布,形状
   缓存即"图集"的 CPU 侧)、#6 部分(IME)、#7 残留(Ctrl+C 矩阵);
   新增观察 4 条中 cosmic-text pin 持续有效;
3. **002 复审遗留**:验收#4 裁定"部分达成+路由 Phase 2"(本计划
   目标 1 是其直接兑现);债务候选 log/env_logger;瑕疵
   Message::PtyBytes 空唤醒;
4. **002 待澄清兑现**:#5 dev 钩子去留 → T9 收进 feature;
   #3 图集优先级 → 本计划即答案;#4 Ctrl+C 矩阵 → T6/T7;
5. **API 事实(001 附录)**:iced `text::Renderer` 公面已有
   `fill_paragraph(&Paragraph)`(与 fill_text 并列),`Paragraph`
   trait 有 `with_text`/`compare`——T1 PoC 验证构造路径;
   `advanced::input_method::{self, InputMethod}` 在 0.14 导出面
   存在,IME 事件形态待 T8 探明。

## 详细设计

### 保留式画布(T1-T4)

- `widget.rs`:`RowPara` 封装 `<iced::Renderer as text::Renderer>::
  Paragraph`,TermGrid 增 `rows: Vec<Option<RowPara>>`(与快照行
  等长,damage 剪裁后惰性补齐);
- 绘制协议:底 quad 全量;每行——`compare(Text{当前行内容})` 为
  `Difference::None` → 直接 `fill_paragraph`;否则 `with_text` 重建
  +`load` 后绘制并 `paragraph_rebuilds += 1`;行内背景色块仍按 run
  画(颜色变化经 compare 的 Shape 差异覆盖);
- 光标行:上游 damage 恒含光标行,自然走重建路径,反色块绘制不变;
- dev 转储增 `paragraph_rebuilds_last/prev`;
- **T1 PoC 前置**:独立测试构造 Paragraph(shaping 'HELLO' →
  `min_bounds().width > 0`)证明公面构造可行;若构造受阻,记
  blocker 于待澄清并停 T2-T4(计划其余部分不受影响)。

### 语义矩阵(T5-T7)

- `unescape` 增 `\xHH` 转义(TDD 单测),Ctrl+C 注入即
  `--dev-autotype "N:...\x03"`;
- 冒烟脚本(手工执行,证据存档):三 shell 各跑长命令
  (pwsh `Start-Sleep 30` / ash 内 pwsh / cmd `timeout 30`),+2s 注入
  \x03,断言:网格 10s 内回提示符、无孤儿进程(tasklist 计数);
- Ctrl+Break:查 iced `key::Named` 是否有 Break 变体与 ConPTY 下
  的字节/事件形态,结论记 001 附录,能映射则并入 key_to_bytes。

### IME(T8)

- PoC:iced 0.14 Ime 事件(Event 枚举形态)与 `InputMethod` 是否
  对自定义 widget 开放;可用 → 光标处 preedit 下划线渲染(内容
  经 Message::Ime(String) 进状态);受阻 → blocker 记录(DEBTS
  +001 附录),不阻塞计划。

### 三尾巴(T9-T10)

- `dev-tools` feature:`--dev-*` 参数与 DevTick 订阅/取证静态量
  统一进 `#[cfg(feature = "dev-tools")]`;默认构建零 dev 面;
- log+env_logger:`dump_state`/resize 错误路径换 `log::info!/
  warn!`,bin 入口 `env_logger::init()`;
- `Message::NoOp` 变体:键盘非按键/鼠标非滚轮事件改映射 NoOp
  (update 显式忽略)。

## 测试设计

- 自动:unescape \xHH 单测;RowCache compare 语义单测(同行内容
  None/变行 Shape/尺寸变 Bounds);矩阵断言脚本化进 dev 冒烟;
  全量 `cargo test --workspace`;
- 半自动:保留式画布性能取证(20000 行流式期间 rebuilds 曲线 +
  单行改动帧 rebuilds < rows)、三 shell Ctrl+C 冒烟转储;
- 手动:IME 中文输入试用(pwsh/ash 各 5 分钟,若 T8 落地);
- 不做:性能基准自动化、列级损伤、Unix。

## 验收标准

1. `cargo test --workspace` 全绿,无 ash-core/auto-shell 路径;
2. **绘制级不等式达成**(002 验收#4 兑现):单行改动帧
   `paragraph_rebuilds < rows`(dev 转储计数断言),Full 帧仍全量;
3. Ctrl+C 矩阵:pwhs/ash/cmd 三 shell 中断断言全过(提示符回归+
   无孤儿),证据存档;
4. Ctrl+Break 有明确结论(映射落地或平台 blocker 记录);
5. IME 有明确结论(最小 preedit 落地或 blocker 记录);
6. `--dev-*` 收进 `dev-tools` feature:默认构建 `--help` 无 dev
   参数(或 cargo tree 无 env_logger 差异断言);
7. 001 设计文档含 002→003 决策链节;DEBTS 勾账(#1/#2/#7 关闭
   或降级,#6 部分推进);
8. log 替换完成:源内无面向用户的 eprintln(grep 断言,panic 除外)。

## 执行步骤

- [x] **T1** PoC:`crates/autoterm-ui/tests/paragraph_poc.rs` —
      构造 `<iced::Renderer as advanced::text::Renderer>::Paragraph`
      (`with_text`+`load` 路径以实际公面为准),shaping "HELLO"
      断言 `min_bounds().width > 0`;结论(可行/受阻+原因)记
      `docs/designs/001` 附录草注。
      验证:`cargo test -p autoterm-ui paragraph_poc`
      [✅ 已完成] 3 用例绿:`Plain<P>` 公面可行(update 内建 content 比对,compare 只看版式;全局字体系统惰性初始化;100 次同内容零重建);实际构造路径是 `Plain::new/update`,无需手工 with_text+load;结论已记 001 附录;commit 07caeb7
- [x] **T2** `widget.rs` 保留式画布:`RowPara` 缓存 + draw 按行
      `compare` 复用/重建 + `fill_paragraph` 绘制;同色 run 背景
      块逻辑保留。
      验证:`cargo build -p autoterm-ui` + 冒烟 echo-hi 转储不变
      [✅ 已完成] 实现演进:compare 不含文本,改用**行 digest(字符+前后景色)**判异 + `Para::with_spans`(同色 run 合 span、前景色烘焙);**像素证据**(程序化扫描,不依赖视觉模型):ash find 表格 338 彩色像素落于表格行带(span 颜色上屏),网格行↔亮带 1:1(位置正确);PrintWindow 对 wgpu 偶发错抓已识别为取证工具问题(标题 -like 歧义),按计划证据政策像素降为辅助;evidence/003-canvas/ 归档;commit 4d8574f/53d526f
- [x] **T3** damage 门控重建:`Damage::Lines` 只对脏行 compare,
      `Full` 全量;静态计数 `PARAGRAPH_REBUILDS`,dev 转储输出
      `paragraph_rebuilds_last/prev`。
      验证:`cargo test -p autoterm-ui` + 冒烟:echo 单行帧
      rebuilds < rows(转储断言)
      [✅ 已完成] 冒烟:末帧 `damage_last: lines=2` + `paragraph_rebuilds_last: 0`(脏行内容未变连重建都免,0 < 32 强于不等式);全程 snapshot_rebuilds 16-18 / frames 93(门控生效);行缓存经全局 OnceLock 持有(跨 view 存活)
- [x] **T4** 回归对照:20000 行流式冒烟(rebuilds 合理,无错位/
      残影;网格终态完整)。
      验证:冒烟转储含 20000 且 `fit_ok: true`
      [✅ 已完成] 首跑暴露真 bug:唤醒转发线程遇 iced 通道 Full 即 break 永久死亡,尾部字节无人 drain(19996-20000 丢失)——T4-002 起潜伏,突发首现;修复=is_full 重试。3 连跑稳定:bytes_fed 恒 129721、20000 每次都在、fit_ok true;bytes_fed 计数器入 PtySession/转储;证据 evidence/003-canvas/bench-20000-fixed.txt
- [x] **T5** `unescape` 增 `\xHH`(TDD:先测 `\x41`→A、`\x03`→
      ETX);lib.rs 单测。
      验证:`cargo test -p autoterm-ui unescape`
      [✅ 已完成] TDD 绿(先失败后实现;无效十六进制回吐修复);8/8 ui 测试
- [x] **T6** Ctrl+C 矩阵冒烟:pwhs/ash/cmd × `Start-Sleep 30`/
      `timeout 30` + 2s 后 `\x03` 注入;断言 10s 内网格回提示符、
      tasklist 无孤儿;证据存 `docs/designs/evidence/003-matrix/`。
      验证:三 shell 转储全过
      [✅ 已完成] 矩阵证据 4 份归档 evidence/003-matrix/;**平台发现**:ConPTY 不翻译裸 0x03→CTRL_C_EVENT(pwsh Start-Sleep/cmd timeout 双证无反应),raw-mode 客户端(ash)正常响应,普通键中断正常——真事件需 win32 GenerateConsoleCtrlEvent(待澄清#3 升级为正式裁定点)
- [x] **T7** Ctrl+Break 调查:iced `key::Named` 变体与 ConPTY 字节
      形态;可映射则进 `key_to_bytes` + 矩阵加一列,否则 blocker
      记 001 附录。
      验证:结论记录在 001 附录(grep "Ctrl+Break" ≥1)
      [✅ 已完成] 双重阻断结论(iced key::Named 无 Break/Cancel + win32 同 Ctrl+C 路线)记 001 附录,与 T6 合并路由待澄清#3
- [x] **T8** IME PoC:iced 0.14 Ime 事件形态/`InputMethod` 开放度;
      行则 `Message::Ime`+preedit 下划线渲染+手动试用清单;阻则
      blocker 记录(DEBTS+001 附录)。
      验证:结论记录(grep "IME" docs/designs/001 ≥1)+ 可选试用
      [✅ 已完成] 结论:管线公面完备(Event::InputMethod/over-the-spot),落地需 Widget::update;验证强依赖真人 IME 交互无法无人值守取证→路由 Phase 3;记 001 附录
- [x] **T9** `dev-tools` feature:`Cargo.toml` 增 feature,`--dev-*`
      clap 参数/DevTick 订阅/取证静态量全部 `#[cfg(feature=…
      )]`;默认构建冒烟仍可运行(无 dev 参数)。
      验证:`cargo build -p autoterm-ui`(默认)绿 +
      `--features dev-tools` 构建绿 + `--help` 无 dev 项
      [✅ 已完成] 默认构建 --help 零 dev 参数且拒收 --dev-*;feature 构建绿(env_logger optional)
- [x] **T10** log+env_logger:依赖加入,`dump_state`/resize 错误/
      取证 eprintln 全部换 log 宏;`bin/autoterm.rs` 初始化;
      `Message::NoOp` 变体替换空唤醒。
      验证:`grep -rn "eprintln!" crates/autoterm-ui/src | wc -l` = 0
      + `cargo test --workspace` 绿
      [✅ 已完成] src 零 eprintln(log::info + dev-tools 下 env_logger init);Message::NoOp 替换两处空唤醒;双构建绿
- [x] **T11** 文档:001 追加 002→003 决策链(Paragraph 路线/矩阵/
      IME/Ctrl+Break 结论);DEBTS 勾账;README dev-tools 说明。
      验证:`grep -c "002→003" docs/designs/001-phase1-architecture.md` ≥1
      [✅ 已完成] 001 增 002→003 决策链节(五条);DEBTS 勾账(#1/#2 关,#6/#7 路由);README dev-tools 说明
- [x] **T12** 收尾:全量回归 + 无 ash 断言 + 冒烟复查。
      验证:`cargo test --workspace` 绿 + `! cargo tree --workspace |
      grep -q ash-core` && echo OK
      [✅ 已完成] 全量 19 用例绿(较 002 +4:paragraph_poc 3/unescape 1),零 ash 路径

## 复审记录

(待 /auto-plan:review 填写)

## 待澄清事项

1. **T1 PoC 受阻预案**:iced 0.14 Paragraph 公面若不能在 widget 外
   构造/load,T2-T4 停,blocker 回报用户裁定(备选:升级 iced 或
   wgpu 自管层,均超本计划范围);
2. **IME 落地深度**:T8 最小 preedit 是否满足日用(无候选窗定位/
   组合键 UX),或需 Phase 3 深做——T8 结论出来后定;
3. **Ctrl+Break 平台结论若为 blocker**:Windows ConPTY 的
   CTRL_BREAK_EVENT 经 portable-pty 不可达的话,是否要为本仓引入
   win32 直调(打破"只用 portable-pty 抽象")——需用户裁定;
4. **矩阵 shell 范围**:cmd 的 `timeout 30` 对 \x03 的行为可能与
   pwsh 不同(控制台主机差异),矩阵允许 cmd 列记为"平台怪癖"
   而非失败。
