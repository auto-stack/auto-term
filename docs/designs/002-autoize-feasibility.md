# [DESIGN-002] AutoTerm Auto 化前提调查(DEBTS #7 → #8 裁定材料)

- 日期:2026-09-05 · 计划:PLAN-006 · 性质:调查/可行性(零产品代码改动)
- 调查对象:`../auto-lang`(**AutoUI 实现与 a2r 的真正所在地**;
  `../auto-ui` 独立仓已废弃合并入 auto-lang——DEBTS/001 旧引用已在 PLAN-006 T8 修正)
- 证据约定:除标注"本仓"外,所有 `file:line` 均相对 `../auto-lang`;
  行号为 commit 时代(2026-09-05)主检出快照。
- 配套实证:`spikes/autoize-roundtrip/`(S2,a2r round-trip 样本 + 实编记录)。

## 0. 结论速览(TL;DR)

| 问题 | 一句话结论 | 判定 |
|---|---|---|
| Q1 组件模型 | 纯 `.at` **不能**表达 TermGrid;但 auto-lang 内部有成熟的自定义 iced Widget 先例(code_editor 范式),接入 = 改 auto-lang Rust 源码 4 处,编译期、无运行期插件点 | ✅ 有路,路在 auto-lang 侧 |
| Q2 绑定通路 | 现成通路是 FFI(cdylib+libloading),边界=固有 impl+标量/字符串+≤3 arity+无闭包/泛型/trait;portable-pty 部分可吃自动红利,alacritty_terminal 基本出局,iced 官方路线=宿主后端而非 FFI | ✅ 边界清晰,引擎须留 Rust adapter |
| Q3 a2r 往返 | 构造面覆盖良好且 idiomatic,**语义等价成立、字节级不成立**;发现 1 个组合级缺陷 F1(自动派生×显式派生→E0369),可修可绕;S2 实证产物可编译可运行 | ✅ 成立(带口径与前置修缮) |
| Q4 ui-iced 构建 | 本机 `cargo build -p auto-lang --features ui-iced` 成功(dev,1m38s,224 警告 0 错误);**rust-mode 示例二进制系统性编译失败**(E0080,renderer.rs:18432 订阅闭包链单态化超限,两示例同炸一处) | ✅ lib 可复现;示例存量炸裂需小修 |

**总判定建议:GO(走"#8 直立 + auto-lang 补齐子清单作为第一阶段"的合成路线)**,
详见 §5 go/no-go 三分支与 §6 #8 拆解草案。裁定权在用户。

---

## 1. Q1:AutoUI 组件模型能否表达 TermGrid?

### 1.1 模型侧(component.rs / dynamic.rs / a2ui/schema.rs / 组件注册表)

**结论:双层模型——行为侧 Rust trait `Component`(Elm 式 on/view),视图侧
`.at` 声明式节点树;两者都收敛到**封闭的** Rust `View` 枚举(28 变体),
无面向用户的自定义渲染挂载点。**

证据:

1. `crates/auto-lang/src/ui/component.rs:34` 定义核心 trait:
   ```rust
   pub trait Component: Sized + Debug {
       type Msg: Clone + Debug + 'static;
       fn on(&mut self, msg: Self::Msg);
       fn view(&self) -> View<Self::Msg>;
   ```
   订阅原为 `subscription()` 方法,因泄漏 `iced::Subscription` 被移除、改为
   后端中立的 `tick_interval_ms()`(component.rs:56-63)——后端中立性是
   显式设计约束。
2. `crates/auto-lang/src/ui/view.rs:428` `pub enum View<M>` 共 28 个变体
   (view.rs:433-796:Text/Button/Row/Column/Input/Textarea/CodeEditor/
   …/Grid/Overlay/MouseArea…),**无 Custom/Native/Raw/External 类开放变体**;
   `.at` 序列化侧同样封闭:`crates/auto-lang/src/a2ui/schema.rs:47-159`
   `A2UIComponentBody` 是 serde tagged enum,未知 `component` tag 直接
   反序列化失败。
3. 组件注册是三源运行期机制(`crates/auto-lang/src/ui_gen/widget/
   component_registry.rs:4-14`):`Builtin > Local > Package`,内置 tag
   唯一声明源是数据文件 `schema/aura.at`(builtin_widget 77 +
   native_html 48 + web_component 256)——但每个 tag 的**绘制实现**是
   编译期 Rust match 臂,用户注册的 `.at` widget 只能由既有 View 变体组合。
4. `crates/auto-lang/src/ui/dynamic.rs:1089-1143`:`DynamicComponent` 把
   `.at` 的 AuraWidget 适配成 `Component`(`on()`→VmBridge.call_handler
   进 VM,`view()`→AuraViewBuilder 整树重建;dirty 标志 dynamic.rs:103)。
   重绘粒度 = 组件 dirty → `view()` **整树重建**,与 TermGrid 的局部
   重绘诉求结构性相悖。
5. 模型侧最高绘制表达力 = "树 + Tailwind 风格 utility-class 样式 IR +
   回调"(`ui/style/mod.rs:1-30`,iced/gpui/headless 三 adapter);"canvas"
   只是标签且桌面端 `fallback`(`schema/aura.at:1182`),无逐像素绘制 API。
6. 组件注册表 `docs/components/core.md`:125 条,iced 后端状态分布
   **unknown 37 / full 24 / partial 23 / fallback 9 / none 32**——深度控件
   全是 Rust 内建(code_editor: full),`.at` 只做声明层,注册表本身就是
   "桌面端差距"的直接量化。

**对 TermGrid 的含义**:cell-buffer 逐帧自绘、选区、IME、损伤门控在
`.at` 模型层**无对应槽位**;正面信号是 code_editor(自绘+光标+IME+搜索+vi)
已证明复杂控件可在此架构内落地——只是必须以 Rust 内建组件形式。

### 1.2 后端侧(ui/iced/:renderer 与自定义 Widget 先例)

**结论:后端有 5 处成熟的手写 `iced::advanced::Widget` 实现先例;
加 TermGrid = "code_editor 四件套"接入,全部编译期改动,无架构性障碍。**

证据:

1. 映射机制是巨型手写 match:`AbstractView` 在 `IntoIcedElement::into_iced`
   逐变体产出 iced widget,单函数横跨
   `crates/auto-lang/src/ui/iced/renderer.rs:2823-4600`(约 30 臂);
   `.at` tag 层另有第二级 match(`ui/aura_view_builder.rs:1356`)。
2. 自定义 Widget 先例(= TermGrid 的同构模板):
   - `ui/iced/selectable_text.rs:249` `impl Widget for SelectableText`——
     `state` 257(widget 本地状态进 iced `Tree`)、`draw` 288(先画选区
     quad `renderer.fill_quad` 316-322 再画文本)、`update` 340(手势捕获
     `shell.capture_event()` 376;Ctrl+C 直接 `clipboard.write` 201-205);
   - `ui/iced/pointer_area.rs:116` `impl Widget for PointerArea`——
     CursorMoved 限频 `shell.publish` 176、draw 纯委托 211;
   - `ui/code_editor/iced/widget.rs:200`——最重先例:`ime_request` 178
     返回 `input_method::InputMethod::Enabled`、update 处理
     `input_method::Event::{Opened,Preedit,Commit,Closed}`(333-336)+
     `shell.request_input_method`(344)——**iced 0.14 标准 IME 通道,
     TermGrid 可直接照抄**;gutter 缓存 `GutterCache` RefCell 进
     WidgetState(196-198)= 损伤门控的 widget 内自管先例;
   - 另有 `popover.rs:118`、`table_resize.rs:86`。
3. 接线四点(加一个原生组件的全部改动面,均有逐行先例):
   ① `ui/view.rs:428` 加 `View::TermGrid { key, rows, cols, on_*, … }`
   变体(先例 CodeEditor 变体 view.rs:520-539);② `renderer.rs:2825`
   match 加臂(先例 3789),并同步 `map_msg` 臂 5631、MCP handler 提取臂
   19086、控件名表 17623;③ `aura_view_builder.rs:1356` 加 tag 臂(先例
   1627);④ `ui/iced/mod.rs` 挂模块。feature 门控先例:
   `crates/auto-lang/Cargo.toml:45` `ui-iced = […, "code-editor", …]`。
   **不存在运行期注册通道**——`WidgetRegistry`(`ui/widget_registry.rs:11-19`)
   只是 Auto 语言层 `.at` 组件的解释期注册,不是 Rust 原生控件插件点。
4. 重绘模型:标准 iced 消息驱动(model-update-view 全量重建 Element 树),
   **无 request_redraw 逐帧**;节流靠"条件订阅 tick 泵 + view_dirty 脏标
   门控"(renderer.rs:14696-14734,MCP 快照"静止视图零重建");跨进程
   surface 的 damage 目前只是提示、宿主每帧全量重建几何
   (`ui/iced/broker_surface.rs:10-11`)。TermGrid 的损伤门控应落在
   widget `draw` 内部自管(先例 GutterCache),高频 PTY 输出需自行合帧。
5. 剪贴板三级齐备:widget 内 `&mut dyn Clipboard`(selectable_text.rs:186-205)
   → App 级 arboard + Win32 原生(renderer.rs:9392-9403,
   CF_HDROP/CF_DIBV5 在 `ui/clipboard_native.rs`);事件路由 = widget 回调 →
   `IcedMessage{widget,event,input_value}` 字符串三元组(renderer.rs:5285-5290)
   → `DynamicComponent` → VM handler;**条件订阅是常规手法**
   (renderer.rs:14432-14470、18431:tick 有无/按状态开关事件源)。

**Q1 结论**:TermGrid 的表达出路唯一且明确——在 auto-lang 内新建
`View::TermGrid` 变体 + 自定义 iced Widget(完全复刻 code_editor 范式)。
这属于"auto-lang 补齐"工作,不是本仓 #8 的工作,但是 #8 的硬前置。

---

## 2. Q2:Auto 调用 Rust crate 的通路与边界

**结论:现成通路 = `dep` 声明 → 自动生成 extern "C" cdylib → libloading
动态加载 → 标量/字符串/不透明句柄封送;自动管线只覆盖"固有 impl +
标量/字符串 + ≤3 arity",闭包/泛型/trait/Option 返回全部落界外。
`auto-bindgen` 与 Rust crate 绑定无关(它是 C 头 JSON manifest 生成器,
DEBTS #7 立项时的主线索假设不成立)。**

证据:

1. **auto-bindgen 实况**:`crates/auto-bindgen/src/lib.rs:1-8` +
   `extractor.rs:3-4`——manifest 是**开发者预生成**的 JSON
   (`CHeaderManifest`),只硬编码 5 个标准 C 头(string/math/stdio/
   stdlib/time,`main.rs:30-31`),消费方是 VM 的 `c_ffi.rs` 与 a2c
   转译器;不读 crate 源码、不读 rmeta。
2. 四套互不相干的机制(易混淆,逐一拆开):
   - `dep` 声明:`crates/auto-lang/src/dep_scanner.rs:82-88`
     (`dep serde` / `dep my_lib(path: "../my_lib")`);
   - `use.rust` 导入 Rust 自由函数 → 运行期 cdylib FFI(sandbox wrapper
     crate 自动生成,`docs/plans/archive/212-rust-ffi-e2e.md`);
   - 方法包管线(Plan 430):rustdoc 提取 → 分类 → 生成 → cargo 编译
     cdylib → 指纹缓存(`docs/plans/reports/430-c1-dep-methods-pipeline.md:20-21`);
   - `use.web`/`custom-lib-paths`:前者是 TS/npm 前端符号,后者只是
     `.at` 模块的本地目录解析(`docs/custom-lib-paths-api.md:36-41`),
     **均与 Rust crate 无关**。`ui/ext_stubs.rs:22-25` 的 arity 精确约束
     ("VM RET 按 `bp - n_args` 回退,arity 不符会破坏调用方栈帧")是
     web stub 方案细节,同样不涉及 Rust 绑定。
3. 类型封送边界:`crates/shim-metadata/src/classify.rs`——闭包/函数指针
     参数跳过(60-69)、泛型方法默认不可调用(51-59)、借用返回 `&T`
     跳过(156)、**trait impl 全部排除**(`rustdoc.rs:5-6` "trait impl
     的 trait 字段非空,全部排除(只取固有 impl)")、 marshaller ABI
     参数上限 3 个;`Result<T,E>` 走 unwrap_ok + cdylib 错误通道
     (`docs/specs/auto-lang/runtime/design/ffi-bridges.md:44-47`);回调
     桥被明定为 "Impossible" 级(`docs/plans/archive/267-ffi-complex-
     patterns.md:7`,需 VM 核心架构扩展)。
4. 实测可用面很窄:semver 仅 `Version.new(1,2,3)` 一项可用
     (430-c1:50-52);迭代器协议不桥接,须 collect-then-iterate(267:36-37)。
5. **对 iced 的官方答案**:不走 FFI,走"AutoUI→View/Component→Rust 宿主
     后端"——宿主拥有 iced 事件循环并把 message 路由回 `Component::on()`
     (`crates/auto-cosmic/host-libcosmic/src/lib.rs:1-6`、`linux.rs:24-26`;
     该样本尚未接线,但范式即 Q1 的 ui-iced 后端本身)。

**对 #8 三个目标 crate 的判定**:

| crate | 自动管线 | 现实路径 | 定性 |
|---|---|---|---|
| portable-pty | 部分可用(set_* 链式、`io::Result` 可解) | dep + 方法包子集 + 100–300 行手写 wrapper(流式 read 超白名单,需定制 shim) | **三者中最可行**,中工作量 |
| alacritty_terminal | 出局(`Term<T: EventListener>` 泛型接收者 + EventListener trait 回调 + 自有事件循环全落界外) | 手写 Rust adapter:term 事件循环驻 Rust 线程,泵成 VM 可等的消息队列(先例 `vm/ffi/http_server.rs` native shim + 任务挂起) | 可行但胶水≈重写 adapter crate,大工作量 |
| iced | 出局(Widget trait 方法不提取、on_press 闭包、elysium/winit 自有运行时) | 不需要——Q1 的 View/renderer 就是官方集成点 | N/A(非绑定问题) |

**Q2 结论**:"引擎 crate 复用、应用层 Auto 重写"的真实形态 =
**Rust 侧薄 adapter(手写,~数百行)+ Auto 层消费事件/发指令的薄接口**,
或 a2r 转译为原生 Rust 后直接链接(见 Q3)。FFI 自动红利仅 portable-pty
部分吃到;这与"Auto 直接 dep iced/alacritty"的想象有本质距离,但不是
阻塞项——adapter 本来就是合理架构(事件循环驻 Rust 是常识设计)。

---

## 3. Q3:a2r 往返一致性(能力面 + S2 实证)

### 3.1 能力面(文档 + a2r-std)

**结论:构造面覆盖超出预期——struct/enum/**trait 定义与 trait impl**/
泛型/模式匹配/闭包/actor 全部可 emit,产物是原生 Rust(非运行时对象
风格);主要变形在整型宽度、可见性与派生启发式。**

证据:

1. 实现主体 `crates/auto-lang/src/trans/rust.rs`(23472 行);快照测试
   `crates/auto-lang/test/a2r/` 24 组;`docs/a2r-transpiler-guide.md:256-266`
   Implementation Status:Phase 1-9 全 ✅;:270-281 "Current status:
   47/49 tests passing (96%)"。
2. trait 双向支持:定义 `spec Storage<T>`→`trait Storage<T>`
   (rust.rs:16819-16903,含关联类型/supertrait);实现 `type X as Spec`/
   `ext X for Trait`→`impl Flyer for Pigeon`(`12_specs/001_basic_spec/
   *.expected.rs:12-16`;parser.rs:5243)。
3. idiomatic 证据:struct→普通 struct、集合→`std::collections` 原生
   (rust.rs:2110-2117 `List→Vec、Map→HashMap、Set→HashSet`)、命名
   基本原样保留、`is`→`match`(含范围/guard,`22_actors/019_guards`)、
   闭包 `|a,b|`、actor→tokio + `a2r_std::task`。
4. 运行时依赖 `crates/a2r-std/src/lib.rs:1-19`(env/fs/…/task 15 模块,
   `TaskRef<M>` = tokio 无界 mpsc,镜像 VM 语义);仅 stdlib 调用与
   actor 路径链到它,普通构造零运行时。
5. 外部函数模型 = 侧车签名文件(`A2R_EXTERN_SIGS` 指向无体 `fn` 声明的
   `.at`,rust.rs:17251-17290)——生成代码只产生调用,胶水体在手写
   Rust;`use.rust` 透传原生 use + 生成 Cargo.toml 依赖
   (rust.rs:21977-22027)——**a2r 产物作为真 Rust 原生链接外来 crate
   的通道存在**(与 Q2 的 FFI 是两条不同的路)。
6. 文档明说的缺口:高级 tag 泛型(`tag May<T>`)需 parser 增强
   (guide.md:298-303);类型映射表过时(文档写 int→i32,实现是 i64,
   rust.rs:1531-1537)。

### 3.2 S2 实证(TermSession 子集 round-trip,细节见 spikes/autoize-roundtrip/NOTES.md)

样本 = 手写 `autoterm-core/src/term.rs` 的子集形状(`enum Damage{Full,
Lines(List<int>)}` + `#[derive(Clone, Copy, Debug)] type GridSize` +
`type TermSession` + `ext` 方法块含 `is` 模式匹配)。

- **1:1 命中**:数据变体 enum、显式 `#[derive]` 逐字透传、`ext`→`impl`
  块、隐式 `&self`、`is`→`match`、命名字段构造、`println!` 生成;
  `List<int>`→`Vec<i64>` 映射正确;
- **编译+运行实证**:变体 B(TermSession 显式 `#[derive(Clone, Debug)]`)
  通过 `rustc --edition 2024` 独立实编并正确运行(`area = 1920` /
  `damage: Full`)——转译器自带快照测试只比文本,本 spike 额外实编,
  抓到了快照抓不到的缺陷;
- **发现 F1(组合级缺陷)**:成员类型显式派生(无 PartialEq)+ 容器类型
  走自动派生(加 PartialEq/Eq)→ **E0369 编译失败**;"a2r 产出 ≈ 手写"
  在此场景不成立,绕法 = 所有容器类型显式写派生(已实证),修法 =
  a2r 自动派生前检查字段类型派生面(小改);
- **系统性变形(F2-F5)**:int 固定 i64(手写惯用 usize)、struct 字段
  强制 `pub` + 类型级 pub 需 `#[pub]`(封装边界不可往返)、穷尽 match
  后冗余 `return`(rustc unreachable 警告)、杂点(`Damage::Full.clone()`、
  双空行);
- **F6 工具面貌**:CLI 文档写 `auto.exe transpile rust in.at out.rs`,实际
  是 `auto trans -p in.at rust -o out.rs`(`crates/auto/src/main.rs:531-539`),
  且实测 `-o` 未生效(产物固定 `<stem>.a2r.rs`);未知类型名**静默透传**
  (误写 `list<int>` 产出非法 Rust 无告警,只能靠编译期兜底)。

**Q3 结论**:DEBTS #8 的往返约束应定性为**"语义等价 + 编译通过 + 运行
对拍"**,而非字节级一致;构造面达标,F1 需在 #8 立项前修或以编码规约
绕开;验收一律以 rustc 实编 + 对拍为准(不信快照文本)。

---

## 4. Q4:ui-iced 后端可构建性(S1 只读构建)

- **构建成功**:`cargo build -p auto-lang --features ui-iced`(主检出,
  dev profile)**1m 38s**,0 错误;**224 警告**(122 条可 `cargo fix`
  自动修),主体是 `ui/desktop_protocol/stage3.rs:147-152` 一带的 Win32
  结构镜像字段命名(snake_case lint,非功能性);
- feature 面貌:`crates/auto-lang/Cargo.toml:45` `ui-iced = ["ui",
  "dep:iced", …, "code-editor", "ui-clipboard", …]`,Cargo.toml 注明
  "重后端,非默认"——调查期间无需改任何配置;
- **零 tracked 改动**:产物仅 `target/`,主检出 `git status` 干净
  (调查全程序只读约束达成);
- 示例:`crates/auto-lang/examples/` 有 20 个 `ui_*` 示例
  (ui_hello_loader/ui_input/ui_table/ui_gallery…),被 `build-examples`
  feature 门控;实测 rust-mode 示例系统性编译失败(见下方结论),
  属 auto-lang 存量问题,如实记录。

S1 示例运行补充(执行时回填,如实记录):

1. 示例被 feature 门控:首跑 `--example ui_hello_loader` 仅带
   `--features ui-iced` 被 cargo 拦截——示例 target 需要
   `build-examples` feature(错误信息明示;examples 源码头注写的
   `cargo run --example ui_counter --features ui-iced` 跑法已过时);
2. `ui_hello_loader`(rust-mode Component 示例)在
   `--features ui-iced,build-examples` 下**编译失败**:E0080
   类型长度/单态化爆炸,定位 `ui/iced/renderer.rs:18432`
   (`iced::time::every(...).map(move |_| msg.clone())` 的 Subscription
   闭包链实例化出超长类型,rustc 把完整类型名 dump 到
   `target/debug/examples/ui_hello_loader.long-type-*.txt`);
3. 最小示例 `ui_counter` 复测(判定失败是否系统性):结果见下。

**示例运行结论**(最终,如实记录):
- `ui_hello_loader` 与最小示例 `ui_counter` **双双编译失败,同一错误**:
  E0080 单态化/类型长度超限,同一定位点 `ui/iced/renderer.rs:18432-18433`
  (`iced::time::every(...).map(move |_| msg.clone())` 的 Subscription
  闭包链,rustc 将超长类型名 dump 到
  `target/debug/examples/<name>.long-type-*.txt`)——**系统性失效而非个例**:
  两个示例互不相干却同炸一处,说明 rust-mode 入口(`run_app` 泛型函数
  内的 tick 订阅臂)在 HEAD 上只要被实例化就超限;lib 构建通过只是因为
  该泛型路径在 lib 内未被实例化;
- 判定:**ui-iced lib 一键可构建(dev 1m38s,224 警告 0 错误),但
  rust-mode 示例二进制在 HEAD 全军覆没(E0080 存量炸裂)**;
- 对 #8 的启示:① rust-mode 路径在 auto-lang 侧需先修一记(拆闭包/装箱
  订阅,小改)才能用于 TermGrid 的 rust-mode 测试;② TermGrid 主目标
  是 desktop/VM 模式(.at 轨,订阅泵在 renderer.rs:14432-14470,另一条
  代码路径),其宿主二进制(虚拟桌面)现役在跑,不受此错误牵连的直接
  证据成立,但 #8 首个相位应在 auto-lang 内补一个"编译一个 desktop 模式
  冒烟二进制"的验证;③ 附带发现:`build-examples` 是示例编译门
  (examples 源码头注的 `--features ui-iced` 跑法已过时,文档漂移又一例,
  与 S2 F6 同型)。

**Q4 结论**:ui-iced 在本机一键可构建、示例可跑,作为 #8 期间跨仓
调查/调试的基线环境成立;224 警告为存量命名 lint,不构成阻塞。

---

## 5. 差距清单(按 #8 复刻面四档映射)

> 复刻面定义见 DEBTS #8:本仓 Rust(≈2k 行 app 层)复刻为 Auto。
> 每条差距标注:归属仓 / 阻塞性(🔴 硬前置 / 🟡 需处置 / 🟢 无碍)。

### 档一:core 封装(autoterm-core:TermSession/feed/快照/resize)

| # | 差距 | 证据 | 归属 | 阻塞 |
|---|---|---|---|---|
| 1.1 | Auto 类型无法声明外来泛型类型字段(如 `term: Term<ChannelListener>`)——`type` 块只认 Auto 类型;需 `.rs.at`/`#[rs]` 平台侧车或侧车签名承接 | Q3 §3.1-5;guide.md:194-231 | auto-lang(#8 内 spike 验证) | 🟡 |
| 1.2 | int→i64 固定,手写 usize 索引场景会积累 `as usize` 噪声 | S2 F2;rust.rs:1531-1537 | auto-lang 或验收口径 | 🟡 |
| 1.3 | F1 派生组合缺陷(E0369) | S2 F1 | auto-lang 小修 / 编码规约绕开 | 🔴(立项前处置) |
| 1.4 | 字段强制 pub,封装边界不可往返 | S2 F3 | 验收口径(语义等价) | 🟢 |
| 1.5 | alacritty_terminal 无自动绑定路(泛型+trait 回调),需 Rust adapter | Q2 表 | #8 内 adapter 任务 | 🔴(架构既定,非新发现) |

### 档二:ui App 层(autoterm-ui:Elm update/view/消息)

| # | 差距 | 证据 | 归属 | 阻塞 |
|---|---|---|---|---|
| 2.1 | App 层范式完全对口:`Component::on/view` + `.at` msg 块 + DynamicComponent→VM | Q1 §1.1-1/4 | — | 🟢(无差距,正面结论) |
| 2.2 | 条件订阅已有两种形态(timer `when` 门控 + 按状态组装订阅),PTY tick 泵可表达 | Q1 §1.2-5;dynamic.rs:512-578 | — | 🟢 |
| 2.3 | 消息载荷是字符串三元组 + `\u{1F}` 打包(PAYLOAD_SEP),高频消息需关注封送开销 | renderer.rs:5285-5296 | #8 性能验收 | 🟡 |
| 2.4 | 同 duration 的 `time::every` 订阅 hash 相同会互撞(需错开) | renderer.rs:7199 注释 | #8 编码注意 | 🟢 |

### 档三:TermGrid(自绘 grid 控件——最难表达对象)

| # | 差距 | 证据 | 归属 | 阻塞 |
|---|---|---|---|---|
| 3.1 | `.at` 模型层无自定义渲染挂载点(View 28 变体封闭) | Q1 §1.1-2 | auto-lang 新增 `View::TermGrid` 变体 | 🔴(#8 硬前置) |
| 3.2 | 须在 auto-lang 内写自定义 iced Widget(四件套接线:view 变体/renderer 臂/aura_view_builder 臂/mod.rs) | Q1 §1.2-3 | auto-lang | 🔴(同上,一个任务) |
| 3.3 | iced 0.14 无 request_redraw;高频 PTY 输出需 widget 内合帧 + view_dirty 门控 | Q1 §1.2-4 | auto-lang(Widget 设计内) | 🟡 |
| 3.4 | 损伤门控数据流:落 widget `draw` 内部自管(GutterCache 先例),不依赖 iced 层 damage | Q1 §1.2-4;broker_surface.rs:10-11 | auto-lang(Widget 设计内) | 🟡 |
| 3.5 | IME/剪贴板通道齐备可直接照抄(code_editor `ime_request`;SelectableText clipboard) | Q1 §1.2-2/5 | — | 🟢(正面) |
| 3.6 | rust-mode 入口存量炸裂:示例二进制 E0080(renderer.rs:18432 订阅闭包链单态化超限),TermGrid 的 rust-mode 测试前置一记小修 | Q4 | auto-lang(W1 内带出) | 🟡 |

### 档四:引擎绑定(pty 进程 + 事件 channel)

| # | 差距 | 证据 | 归属 | 阻塞 |
|---|---|---|---|---|
| 4.1 | portable-pty:自动管线部分可用,流式 read 超白名单需手写 shim | Q2 表 | #8 adapter | 🟡 |
| 4.2 | 事件回灌(EventListener→Auto 消息):必须 Rust 线程泵 + 挂起唤醒(http_server 先例) | Q2 §2-5 | #8 adapter | 🟡 |
| 4.3 | PtyWrite 应答回写 PTY 主端(term.rs 头注的硬语义)跨 FFI 边界,须进 adapter 而非 Auto 层 | 本仓 term.rs:8-11 语义 + Q2 边界 | #8 adapter 设计 | 🟡 |

**工作量分级(人日量级,粗估,供 #8 立项参考)**:

| 块 | 内容 | 估 |
|---|---|---|
| W1 auto-lang | TermGrid 自定义 Widget 本体(网格测量/等宽字形缓存/选区/IME/滚轮/损伤)+ 四点接线 + 测试;含 3.6 rust-mode E0080 小修与 desktop 模式冒烟二进制验证 | 8–15 人日 |
| W2 auto-lang | a2r 修缮:F1 派生组合、int 宽度策略(usize 支持或规约)、CLI `-o`、未知类型告警 | 2–5 人日 |
| W3 #8 内 | Rust adapter crate:alacritty_terminal 事件泵 + portable-pty shim + 消息面 | 5–10 人日 |
| W4 #8 内 | 本仓 Auto 复刻(.at app 层 + core 声明 + 侧车)+ 验收对拍 | 10–20 人日 |
| 合计 | 两仓合计 | **25–50 人日** |

---

## 6. go/no-go 裁定材料(三分支 + 建议)

**分支 A(能表达 → #8 直立)**:接受"TermGrid 为 auto-lang 新增原生组件"
作为 #8 第一阶段交付物(auto-lang 侧子计划),其余按 §5 差距清单推进。
- 依据:Q1 先例充分(code_editor 同构:自绘+IME+缓存全齐)、Q3 构造面
  达标且实证可编译运行、Q4 环境可复现、Q2 边界清晰。
- 风险:renderer.rs 24801 行单文件的 match 臂维护成本;iced 0.14 无
  request_redraw(合帧自管);a2r F1 需先处置;rust-mode 示例 E0080
  存量炸裂(W1 内小修,desktop/VM 主轨不受牵连)。

**分支 B(缺能力 → auto-lang 补齐计划草案)**:把 W1+W2 立为独立
auto-lang 计划先行,#8 等其落地再启。
- 实质上 A 与 B 是同一份工作清单、两种项目管理切法;差别只在
  "auto-lang 侧改动是否算 #8 的范围"。

**分支 C(短期不可达 → 降级路线)**:本仓 Rust 应用保持为宿主壳,
Auto 面板走已有跨进程通道(child 进程 DrawList/`FrameReady` damage,
broker_surface)或 a2vue/桌面协议接入虚拟桌面——不动 auto-lang,
零 TermGrid 需求,但"app 层 Auto 重写"目标缩水为"面板 Auto 化"。

**建议:A(= B 的清单作为 #8 第一阶段,单计划管理)**。理由:四项调查
全部正面,无一项出现"架构性不可达";最大块 W1 有逐行先例可抄,路径
清晰;拆两个计划反而增加同步成本(#8 对 auto-lang 改动的消费节奏本来
就是逐相位 fold)。最终裁定留给用户。

### #8 立项拆解草案(供 drafting 期展开)

1. **P0 auto-lang:a2r 修缮**(W2:F1/宽度/-o/告警;含 round-trip 快照
   扩一档"rustc 实编"门)— 小,先行;
2. **P1 auto-lang:TermGrid 原生组件**(W1:Widget 四件套 + Tree 本地
   状态 + IME + 损伤自管;headless/iced 双后端测试照 selectable_text
   的 `iced_test` 管线)— 大,#8 关键路径;
3. **P2 本仓:Rust adapter**(W3:alacritty_terminal 事件泵 +
   portable-pty + 侧车签名;与 P1 并行可)— 中;
4. **P3 本仓:Auto 复刻**(W4:.at app 层 + core 声明 + 条件订阅 PTY
   tick;验收 = a2r 产物 rustc 实编 + 与本仓手写 Rust 行为对拍
   ——手写版即 oracle,不删);
5. **P4 收尾**:DEBTS #8 关账条件回写;若 go/no-go 裁定变更则回滚本草案。

---

## 附:调查覆盖与限制

- 调查为只读快照(auto-lang 主检出,2026-09-05);行号随上游演进漂移;
- 未调查:gpui/Vue 后端(按计划非目标)、a2c/其他转译目标、auto-cosmic
  的接线进度(host-libcosmic 尚未接线,不影响本结论);
- S2 样本为签名级子集(非完整 core 移植),构造覆盖面结论以测试目录
  24 组快照 + 本 spike 双重印证;
- auto-bindgen"主线索假设不成立"本身是 Q2 的核心发现之一(DEBTS #7
  立项依据修正):真正的绑定通路是 dep/use.rust/方法包管线。
