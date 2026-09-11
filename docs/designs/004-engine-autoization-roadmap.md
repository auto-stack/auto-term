# 004 — 引擎 Auto 化路线:口径修正、能力缺口账与 C 通道

> 性质:**讨论沉淀型设计记录**(2026-09-07 用户裁定,非计划产物)。
> 关联:002(前提调查,历史原文不改,勘读见 §7)、DEBTS #8、
> PLAN-006/009;ledger 条目 D004-2(goals)/D004-4(designs)。

## 0. 口径修正(用户裁定,2026-09-07)

- **Auto 语言设计目标之一:a2r 可复刻任意 Rust 代码**。遇到不可
  转译的构造,定性为 **Auto/a2r 的能力缺口(待补)**,而不是
  "Auto 不支持所以不做"。Auto 是新语言,待完善项即路线图。
- 由此,002 Q2/PLAN-009 的"引擎保持 Rust adapter"从**终局裁定**
  修正为**分期工程裁定**:先让产品形态落地,不被语言成熟度阻塞
  ("adapter 本来就是合理架构"是工程判断,不改变语言目标);
  引擎四模块的手写 Rust(`pty.rs`/`term.rs`/`ffi.rs`/
  `autoterm-ctrlc.rs`)**保留未来切换为 Auto 源的选项**。
- 002 缺口表 1.5 的定性"🔴(架构既定,非新发现)"按本口径改读:
  它记录的是**当时形态的裁定**,不是语言终局(原文不改,勘读见 §7)。

## 1. 边界辨析:两条曾被混淆的边界

002 里存在两条性质不同的边界,讨论中曾混为一条:

| 边界 | 是什么 | 证据 |
| --- | --- | --- |
| **Q2 白名单** | `use.rust`/dep **运行期 FFI 自动绑定管线**的封送限制:固有 impl + 标量/字符串 + ≤3 arity,闭包/泛型/trait 落界外。"alacritty_terminal 出局"说的是这条路 | shim-metadata/classify.rs;002 §2 |
| **Q3 表达力** | **a2r 转译**的真实能力面:trait 定义/trait impl/泛型/模式匹配/闭包/actor 全部可 emit,产物为原生 Rust;`use.rust` 透传可**原生链接外来 crate** | trans/rust.rs(23472 行);rust.rs:21977-22027;002 §3.1 |

引擎 Auto 化走的是 **Q3 这条路**——002 Q2 结论原文即列有替代形态
"或 a2r 转译为原生 Rust 后直接链接(见 Q3)"。另:普通构造**零运行时
依赖**(仅 stdlib/actor 路径链 a2r-std),引擎 a2r 产物可不链运行时,
无环铁律不受威胁。

## 2. 能力缺口账(引擎四模块 → Auto/a2r 待补能力)

**现阶段的问题不是"Auto 不支持",而是"暂无足够能力支持"**——
每一项都有明确承接路径:

| 引擎构造 | 所需能力 | 现状 |
| --- | --- | --- |
| `term: Term<ChannelListener>` 字段 | type 块声明**外来泛型类型字段** | 002 缺口 1.1 已登记 🟡(暂以侧车签名/`.rs.at` 承接) |
| `impl EventListener for …`/`impl Dimensions` | 为**外来 trait** 写 impl | 自家 trait 双向已支持;外来 trait 未显式验证(未知类型名静默透传 F6 是"碰巧"通道,需正式支持+告警面) |
| `Box<dyn MasterPty + Send>` 等 | **trait 对象**表达与解包 | 正是 a2r 遗留 37 例编译债的"智能指针自动解包族"(010 T2 起在销账轨道) |
| reader 线程(move 闭包 + 阻塞流式 read) | **裸线程/阻塞 io**;闭包与 actor→tokio 已可 emit,阻塞 read 需通道或侧车 | "ownership-闭包耦合族"同在遗留 37 例 |
| `CommandExt::creation_flags` 等平台 API | `use.rust` 透传 | 通道已在 |
| `extern "C"`/裸指针/unsafe/kernel32 | **不走语言 unsafe 面,走 C 通道** | 见 §3 |

## 3. C 通道:FFI/win32 的归宿(a2c/c_ffi/auto-bindgen)

**事实前提**:kernel32.dll 与 cdylib 同为 **C ABI 动态库**——FFI 与
win32 是同一性质的问题。Auto 双后端(a2r=Rust、a2c=C)现阶段路径
独立,但 **C ABI 是两条通道的天然会合点**。

**现状盘点**(2026-09-07 查证,auto-lang 侧):

| 部件 | 现状 |
| --- | --- |
| a2c(`trans/c.rs`,4534 行) | 实体转译器:闭包→函数指针(Plan 060)、spec→vtable、`use c <header.h>` 接 manifest 直调(Plan 216 Phase 3)、快照测试设施在库 |
| VM c_ffi(`vm/ffi/c_ffi.rs`) | libloading 加载 C 库 + manifest 签名封送在库(Plan 216 Phase 2);**回调桥 = 267 定性 "Impossible" 级**——VM 路径不承载回调 |
| auto-bindgen | manifest 提取器**硬编码 5 标准头**(string/math/stdio/stdlib/time);缺 windows.h 子集(数据活,非管线活) |

**unsafe 消解论**(语言设计姿态):C ABI 互操作路由给 C 通道后,
"Auto 要不要 unsafe 面"的问题不是被解决而是被**消解**——裸指针/
extern/回调在 C 里是母语。**Auto 可能永远不需要发明 Rust 式 unsafe,
全部 C ABI 互操作外包 C 通道。**

**三种角色对账**(归途不同,勿混):

| 角色 | 模块 | 归宿 |
| --- | --- | --- |
| win32 调用 | `autoterm-ctrlc.rs` | **a2c 纯产物,最佳首靶**:独立 exe(argv 收 pid、exit code 回话),零链接 junction;仅 5 API(FreeConsole/AttachConsole/SetConsoleCtrlHandler/GenerateConsoleCtrlEvent/Sleep)。SetConsoleCtrlHandler 的回调恰是 a2c 强项(闭包→函数指针),VM 路径则 Impossible |
| FFI 消费侧 | at-gen `engine.rs`(libloading 胶水) | C 通道可行但有 junction:**c_ffi 仅适用 VM 应用**,at-gen 是 a2r 原生产物,走 a2c 须把 C 对象链入 Rust 宿主(C ABI junction);近路 = a2r+侧车签名既有模式(A2R_EXTERN_SIGS)。12 符号面按"opaque handle+标量/字符串"设计,天然 C-marshalling 友好 |
| FFI 生产侧 | `ffi.rs` 12 个导出 | **不走 C 通道,归 a2r 发射特性**(从 Auto 声明 emit `pub extern "C" fn` + no_mangle)——这是"Auto 产物被 C 调用",转译器特性而非语言 unsafe 问题 |

## 3.5 C 通道的 a2r 后端:`use.c` 的三后端下降(2026-09-07 补)

**缺口实证**:`trans/rust.rs`(23472 行)对 `use.c`/CHeaderManifest 的
处理为**零**——C 通道今天只有两个下降目标(VM 运行期 c_ffi、a2c 直发)。
`use.c` 的 Auto 源进 a2r 产物即断线:其余模块都转成了 Rust,却无法调用
C 库。**必须补 a2r 后端**(对既有 Rust FFI 的复刻方向同理闭合:
`extern "C"` 块 ↔ `use.c` 声明互映)。

**原则:一套语义,三个 lowering,manifest = 共享 IR**——auto-bindgen
从 manifest 提取器升格为多后端生成器:同一份 CHeaderManifest(签名+
封送规则)喂三个后端:

| 后端 | 下降形态 | 现状 |
| --- | --- | --- |
| VM | c_ffi.rs 运行期(libloading + manifest 封送) | 已在 |
| a2c | 直发 C 调用(闭包→函数指针,Plan 216 P3) | 已在 |
| **a2r** | **生成 Rust FFI 模块** | **缺(本节工作项⑥)** |

a2r 下降**双形态按判据生成**(非只 libloading 一种):

- **S 静态链接**:`#[link(name="…")] extern "system" { … }` + unsafe
  调用——系统库/链接期必有符号,零运行期机器;
- **D 动态加载**:libloading 运行期 load→resolve→安全包装——第三方
  DLL、路径运行期解析、**优雅降级**(引擎先例:ctrlc 缺失降级 0x03
  字节路径 = try-load-fallback,静态做不到)。

**unsafe 与数据边缘**:全部驻留生成模块内部,对外只暴露安全签名
(与 at-gen 手写 engine.rs 包装 12 符号面同构,机器生成版);Auto 源
零 unsafe。封送规则沿用 manifest CTypeDesc(标量直过/str↔CString/
数组↔指针+长度)。**回调 = 最硬一族**:捕获闭包不能直接转
`extern "C" fn`,需 trampoline 生成模式(`Box::into_raw`+静态注册表+
`extern "C" fn` 壳),SetConsoleCtrlHandler 即试金石,单独排期。
win32 注记:ABI = `extern "system"`(manifest 带 ABI 注记);DLL 解析
顺序沿用引擎先例(env 覆盖 → exe 同目录 → 祖先 target)。

## 4. 分期迁移路径

- **P0(已落地,现状)**:引擎 Rust adapter + 手写胶水;Rust 参考
  实现冻结为对拍 oracle(PLAN-009 形态)。
- **P-C(C 通道先行,独立于语言能力补齐)**:ctrlc → a2c 产物;
  engine 胶水 Auto 化(a2c 链接或 a2r+侧车)。
- **P-L(语言能力补齐,auto-lang 侧)**:§2 表逐项清偿;引擎四模块
  即 a2r 最苛刻的进阶语料(多线程+trait 对象+FFI+平台 API 全占,
  远狠于既有 24 组快照)。
- **P-S(源码切换)**:`pty.rs`/`term.rs` 语义部分源码归 Auto,a2r
  产物即今日之 Rust;对拍门禁六场景升格为语言能力验收器(oracle
  独立性重设计:冻结 Rust 参考作历史 oracle);`ffi.rs` 导出面走 a2r
  cdylib 发射;interop 胶水即便终局驻留手写,是"**选择不表达**"
  而非"不能表达"。

## 5. 工作项账单(全部 auto-lang 侧,本仓零依赖)

1. auto-bindgen 补 **kernel32 console 子集 manifest**(上表 5 API)——
   **前置:manifest 类型层扩展**(2026-09-07 查证:`CTypeDesc` 仅原语+
   不透明指针,**无 FnPtr 回调变体、无 ABI 注记**,SetConsoleCtrlHandler
   的函数指针参数现模型表达不了;serde enum 加变体+三后端映射,小改)
   ——**✅ 已落地(PLAN-595,2026-09-09)**:FnPtr 变体 + abi 注记 +
   windows.h manifest 6 函数(GetCommandLineA 追加,argv 零运行时通路)
   + windows.json 导出;VM c_ffi 对 FnPtr 注册期明确报错(267 定性);
   验收 `cargo test -p auto-bindgen` 6 测绿;
2. **a2c 复刻 autoterm-ctrlc** + 与 Rust 版行为对拍(验收复用本仓
   `ctrl_event.rs`/`parity_interrupt` 门禁与 26200 exit 0xC000013A
   语义)——**✅ 已落地(PLAN-595,2026-09-09)**:复刻源
   `auto-lang crates/auto-lang/test/a2c/18_c_interop/003_autoterm_ctrlc/`
   (闭包回调/双发 Break→Sleep(50)→C/exit 0/2/3);执行期抓出并修复
   a2c 真缺陷(闭包定义在 main 后无原型 → 真 MSVC 编译 C2065,快照
   盲区);MSVC 构建脚本产出 ctrlc-a2c.exe;**静态对拍 3,3/2,2 + 本仓
   ctrl_event 门禁产物置换跑 a2c helper:3 passed 1 ignored(26200 ping
   已知边界),Rust 版复跑同绿**——①②一组清账;
3. c_ffi/a2c **驱动 autoterm_core.dll 12 符号面**验证(顺带探明
   engine 胶水 Auto 化的最终路径)——**✅ 已落地(auto-lang PLAN-597,
   2026-09-09)**:a2c 全量驱动真机通过(12 符号 fn.c + MSVC 链
   .dll.lib import lib——直接 DLL 输入 LNK1107 不可行;echo 锚点经
   take_dirty_rows+row_text 见证 CFACE_OK/exit 0);VM 标量子集打通
   (use_scanner 潜伏缺口修复——此前任何 VM 模式 use.c 均失败 +
   JSON manifest 加载面 + spawn/write_input/resize 三分派臂,
   VFACE_OK 冒烟;缓冲出参 4 符号实证不可达归 004⑥);路径裁定书:
   独立工具→a2c 链入/宿主内胶水→a2r+侧车/VM 应用→标量子集;
   附带:VM 轨两枚存量缺陷实测定位(if 条件位内联 C-FFI 调用+循环
   挂起;循环计数器嵌套 if 赋值静默断流);
4. a2r 语言能力四项:外来泛型类型字段、外来 trait impl、trait 对象、
   裸线程+阻塞 io(002 缺口 1.1 扩展)——**✅ 已落地(auto-lang PLAN-599,
   2026-09-10)**:语料 `test/a2r/25_foreign_types/` 五件 + rustc 实编
   运行门(witness 断言);capstone=term.rs 子集 Auto 版与手写 Rust
   oracle 共享 fake_core stub **黑盒 stdout 全等**(四能力同语料命中);
   执行期两修:①dyn 字段派生门控语义修订(384 A5 的 Clone,Debug 对
   无约束外来 trait 仍 E0277 → 默认不派生,显式 #[derive] 透传可覆盖);
   ②spawn 自动 move 与显式 move 闭包双发缺陷。发现:①③(泛型字段/
   外来 trait impl)在既有计划中已原生可用,本计划补语料+实编+文档;
   F6 告警面=env 门控 AUTO_WARN_UNRESOLVED_TYPES(默认静默,全量噪音
   实测);与 Q2 轨 PLAN-596(430 dep 管线)边界显式无交;
5. a2r **cdylib 导出面发射**——**已落地清账(auto-lang PLAN-610,
   2026-09-11,与⑥合并一项)**:#[export]/#[export(system)] 兄弟包装
   模块发射(no_mangle extern,符号=fn 名;int i64↔i32 边界 cast/
   cstr CString 边界/句柄缓冲双形参规则,unsafe 全居生成码);引擎
   12 符号面 Auto 版 capstone(005 语料)被 597 a2c 驱动器链接跑出
   CFACE_OK/exit 0(a2r_cabi_engine_face_gate);
6. **C 通道 a2r 后端**(§3.5)——**已落地清账(auto-lang PLAN-610,
   2026-09-11,与⑤合并一项)**:S 静态 #[link] extern/D 动态 libloading
   双形态落地(manifest 共享 IR 增 link 字段,三后端收口);终局闭环
   实证=⑥ 驱动同产物改链 ⑤ 产物 CFACE_OK(**Auto↔Auto 零手写胶水**,
   a2r_cabi_use_c_gate 三腿:AC-04 真 DLL/AC-05 ⑤产物/AC-06 D 运行期);
   trampoline 选型=A' 具名 #[export(system)] fn 按名传值(闭包直转
   E0308 证伪,捕获案 B 兜底,实作 defer;PLAN-610 §5 附录)。

## 6. 验收与证据设施复用

本仓既有门禁即语言能力验收器,无需新建:parity gate 六场景
(启动/echo/resize/中断/色彩/选中)、`ctrl_event.rs`、`live_pty`;
S2 spike(term.rs 子集 round-trip + rustc 实编 + 运行)是既成先例
(`spikes/autoize-roundtrip/`)。

## 7. 勘读注记

002 为历史调查记录,原文不改(同 PLAN-011 对 009 skip 表述的处理
惯例);凡 002 与本文件口径不一致处(尤其 Q2"出局"表述、缺口 1.5
定性),**以本文件为准**。
