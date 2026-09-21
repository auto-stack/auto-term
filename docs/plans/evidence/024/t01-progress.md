# PLAN-024 T-01 进展记录(进行中)

日期:2026-09-19 | commit: auto-lang auto-term-dev `e22f575df`(基 e18066712)

## 已证(几何源面 = T-01 核心)

**shim override 实机生效**(P024_TRACE=1,`_desktop-t01c-boot.log`):

```
[P024-TRACE] window_width -> 766 (override=(766.0, 437.8) theme=1280)
[P024-TRACE] window_width -> 766 (override=(766.0, 409.40002) theme=1280)  ×持续
```

- override = 本 App(auto-term)vwin 内容区真值(rect 768×446.4 − chrome);
- theme = 1280(宿主视口,shell 族 view 构建写入)——**缺口 B 串扰源
  已被 override 短路**;
- 引擎 cols=80 ≈ 766/9.5 自洽(几何标定→引擎 resize 链通);
- 启动序列首值 384(=计算器 vwin 内容区?待核)后稳定 766——首拍
  注入时序待 T-01 收口时复核(疑似 layout grid 期间计算器拆借先行)。

门禁:cargo check(ui-iced)绿;p024 单测 2/2;`p022_stack` 5/5;
`terminal_input` 2/2(023 基线零回归)。

## 剩余缺陷(渲染层,阻断 T-01 收口)

**现象**:终端文本块仅内容区 ~57% 宽(868 物理 px vs 内容区 1520),
顶左对齐;T-00 基线同形(868px/cols=81)——**非本次改动回归**,
几何源修复后依旧(说明标定值正确但渲染不随投影 px 走)。

**证据链**:
- AURA 快照(_snap-t01c.txt):pane 容器无 px 类显示(其余容器
  style 行齐全);树形 = relative 容器 → col(w-full h-full) →
  [空容器 + 终端容器](absolute 拆分层形态,Plan 057 2.2);
- vtree(_vtree-t01.txt)bbox 与像素证据矛盾(根 1280×800/tab 行
  y744/子越父界),疑缓存陈旧,不足为凭;
- 网格/自由布局循环后形态进一步退化(1 行 prompt + 左缘竖线,
  banner 丢失)——grid→free 的重排路径下终端件重建异常。

**线索(下轮起点)**:
1. `dynamic_abs_layer_position`(renderer.rs:2808-2833):零偏移
   absolute 子(pane1 x=0,y=0 恒零)→ None → "原点满铺"语义——
   满铺目标/尺寸类(w-[Npx] h-[Npx])在该路径的消费未核;
2. 终端件 cell 度量疑似物理/逻辑 px 混算(宿主 DPI×2:2560 物理
   = 1280 逻辑;868 物理 = 434 逻辑 ≈ 766×0.567,比例未对上
   0.5,待实测 cell 宽);
3. f-string style(`style: f"absolute z-10 top-[${.y1}px]..."`)在
   in-proc 动态编译路径的求值未直接核验(快照不显,可能只是
   snapshot v2 展示面缺)。

## 取证基建(复用)

- `evidence/024/_mcp.py`——MCP 驱动(state/snap/shot/find/type/key/
  action/wait/desktop);
- `evidence/024/_t01_gate.py`——launch→稳态→layout grid→layout free
  序列(cols/rows 随动观测);
- 桌面启动(acceptance 通道):
  `cd D:/autostack/auto-os && AUTO_LANG_ROOT=D:\autostack\.wt\term-024\
  auto-lang P024_TRACE=1 AUTOUI_ACCEPTANCE=1 AUTOUI_MCP_PORT=9251
  AUTOTERM_ENGINE_DLL=D:\autostack\auto-term\target\debug\autoterm_core.dll
  bash scripts/desktop.sh iced`(钉 DLL 规避 auto-lang/target 下陈旧件);
- MCP bus 动词:launch\tauto-term / layout\tgrid|tfree / focus\tN /
  summon\tlauncher;launcher 关闭 = autoui_desktop action=handler
  app=launcher handler=Escape;**autoui_action press 受 primary-App
  单 App 语义限制,多 app 场景对非 primary app 件不可达**。

---

# 根修轮更新(2026-09-19 晚,commit a4a3237ad)

## 根因(取代上轮线索①②)

**launch 期 Init 先于 wm_add_win**(session.rs launch_app:
build_dynamic_component → Init 在此运行 → 之后才级联算 rect +
add_win)——Init 期 mux_window_* 的 override 持**上一 App 残值**
(TRACE 实录首值 384 = 计算器 vwin 内容区)→ 首拍投影错 → 引擎
**双重 resize** → banner 被重排抹掉(仅剩 prompt)。

修法:级联 rect 计算前置 + build 前注入本 App 真值(set_app_window_
px(vwin_content_size(rect)))。

## 上轮线索处置

- 线索①(DPI cell 度量混算)**证伪**:CELL_H=16/cell_w()≈9.5
  (实测 advance),cols=80=(766−2·PAD)/9.5 与 pane px 自洽;
  "文本块 57% 宽" = banner 最长行 44 字符 × 9.5 logical,非缩放错。
- 线索②(pane px 类未达 absolute 拆分层)**证伪**:同上自洽性。

## 复验(t01e,host 1723 logical 场)

- TRACE 首值即 766/1031.8(本 App 真值,无残值);
- engine_resize 序列 = **单次** 109×30(直达终态);
- **banner 5 行存活**(Version/copyright/prompt 视觉分析确认);
- layout grid→free:cols 121→101→111 随 rect 变更(几何随动链
  rect→override→投影→widget→引擎全通);
- 待观察:t01d(同修,2560×1600 场)banner 仍丢一例——疑首拍
  layout 瞬态,列入实机门观察项。

## 当前实机台

桌面已运行 worktree 构建.AUTO_LANG_ROOT=worktree P024_TRACE=1
AUTOUI_ACCEPTANCE=1 端口 9251,供用户实机手测(最大化/还原、宿主窗
resize、键鼠面 T-02/03)。

---

# 用户实机门缺陷:输出期滚动条抖动(2026-09-19 夜,commit 247864bc4)

用户验收("转 review"附带症状):桌面轨 ls 出滚动条后,滚动条**反复
抖动**;rust 独立窗轨无此症状。

## 根因(TRACE 直捕,_desktop-jitter2-boot.log)

```
observe view_y=0.0 hist=101 tgt=101 base=0  bound=0  off=0    ← 增长一步到 101,iced 绝对位不动
observe view_y=0.0 hist=101 tgt=101 base=101 bound=0  off=0    ← 回灌 +101 入队
observe view_y=0.0 hist=101 tgt=101 base=101 bound=101 off=101 ← 引擎跳到最旧端,稳态卡顶
```

iced scrollable 内容增长保持**绝对像素位**(不跟随底部)→ 贴底视口被
顶离底部 → 022 读出臂把位移误判为用户滚动 → 回灌引擎(offset 跳最旧
端);输出持续期,回灌与程序化绑定交替 = 滚动条反复抖动。

## 修法

- 读出臂 `observe_view_scroll` 增长漂移判别:**贴底基线(view_target
  ==0)+ 视图 y 未动 + 滚轮代数未进**下历史增长 = 内容增长漂移 →
  不回灌(保持贴底),挂 `scroll_repin_pending`;已滚向上(base>0)
  的内容锚定路径不变;
- 写臂 `bind_request_y`:repin 优先 → 绑到新画布底(offset 0 ↔
  history×CELL_H),回声抑制同常规绑定;
- 回声臂基线改取 `bind_echo_offset`(绑定登记值),不按现时 hist
  重算——重绑飞行期内容再增长时重算值=增长量,会污染贴底基线使
  判别失效(复发通道);
- 滚轮优先:代数已进(用户增量)照常回灌,不落入重绑(022 T-06
  同型病灶防线)。

## 测试

p024_growth_drift_repins_bottom_not_feed / p024_user_wheel_beats_
growth_repin 2 新例;virtual_scroll 8/8(022/023 滚动语义零回归);
p022 6/6。三轨共用组件,rust 轨同收益(输出跟随贴底 = 正确终端语义)。

## 验证状态

- 单测全绿;jitter3 重启 boot 干净(hist=0,上轮 101 环境性突发未
  复现);
- **ls 抖动实机复测待用户**(桌面已运行修复构建,TRACE 观测臂在案)。

---

# 抖动二段根修(2026-09-19 夜,commit d78e72021)

用户复测澄清:独立窗不抖 / 虚拟桌面(VM 轨)ls 后抖;首修(247864bc4)
后复测仍抖 → TRACE 复捕完整机制(_desktop-jitter3-boot.log 进场序列):

```
observe view_y=0.0 hist=0  tgt=0  base=0   ← 贴底稳态
observe view_y=0.0 hist=99 tgt=99 base=0   ← 增长一跳 → repin(首修正确生效)
bind    scroll_to y=1584 off=0             ← 贴底重绑发出
observe view_y=0.0 hist=99 tgt=99 base=0   ← 着陆延迟拍:视图未动,旧抑制臂
                                            "未对齐即清除"→ 回灌 +99(点火)
bind    scroll_to y=0.0 off=99             ← 引擎跳最旧端
... 每帧 ±98 回灌/绑定交替,视图 0↔1568 永续跳(持续环)
```

二段修法(commit d78e72021):
- **在途耐心**(杀点火):抑制挂起 + 无滚轮 + 未对齐 + 视图未动 =
  scroll_to 在途 → 持抑制等着陆,不回灌;
- **前回声匹配**(杀持续环):视图已动、不匹配当前回声但匹配前一
  绑定回声 = 交替在途迟落点 → 按前绑定 offset 对齐基线吞掉;
- 真实观察语义(已动 + 无回声匹配)与 022 T-06 用户增量语义不变。

测试:p024 在途/交替 2 新例(6/6);virtual_scroll 10/10、p022 6/6、
p023 4/4 零回归。**待用户 ls 复测**(jitter4 桌面已运行本构建)。

---

# 三段修:不跟随 + 缺 prompt 行 + 方角超界(2026-09-19 深夜,commit cd4c60f43)

用户复测(二段修构建):抖动消失 ✓;新三症状。TRACE 逐帧解码
(_desktop-jitter4-boot.log):

- **症状①②同根**:offset 换算用视口顶——画布底部 PAD(2×4px)+ 
  scrollable max 钳位使"几何贴底"的视口顶算出 **offset=1**(16px 恰
  =1 行)。稳态卡 offset 1 → **引擎锚定**(alacritty 语义:offset≠0
  时新输出保持绝对位置)→ ls 输出 116 行后引擎 offset 0→116(泵回
  写实录,observe 双拍 aligned 吞没零回灌——引擎自己动的)→ 视图停
  半路;拖到底也只到 offset 1 → 最新 prompt 行永在快照窗外。
- **症状③**:scrollable 容器右缘余量条/画布 bg 方角 + thumb
  radius 3,在窗框圆角 16 处探出 app 边界。

修法(commit cd4c60f43):
- observe 换算改**底隙量化** gap_to_offset((gap+4)/16 整除,
  gap=canvas−view_y−viewport_h)——几何贴底(±半行容差)⇔ offset 0,
  引擎不锚定 → 跟随恢复 + 拖到底见最新行;对齐判定同步 offset 口径;
- 容器余量条底色圆角 16 + thumb radius 8(同色叠层,角落共同透明);
  上翻态画布中段方角 = iced 无圆角裁剪原语,挂 DEBT。
- 测试:observe 系全改底隙口径(6+10+6+4+2 全绿)+ 着陆残隙量化
  回归例。基线 flaky 记录:terminal_pixel_preedit 三跑绿绿红(基线
  同现,与本轮无关)。

---

# 四段修:底部圆角缺失(2026-09-19 深夜,commit 3afe5b97a)

用户复测(三段修构建):**滚动跟随/最新行可见/滚动条全部正常 ✓**
(横分双 pane 截图在案)。余底部圆角缺失。

根因:贴底 bind 目标 = 快照窗顶(hist×CELL_H),scrollable 钳位真值
= canvas−视口高,差 8px(底部 PAD)→ 视口底停画布底上方 8px →
圆角弧(16px)下半段被裁,剩上段视觉≈直角;容器/画布 bg 圆角同画
在画布底,一并被裁。

修法:bind 贴底臂(repin + 引擎 offset 0)目标改钳位真值
(canvas−viewport_h,注册表 miss 回落);offset 换算底隙口径下两种
y 观察同为 0,自洽零回归。terminal_canvas_height glue 镜像新增。

测试:6+10+6 全绿(fallback 路径断言不变)。**待用户复测圆角**
(jitter6 桌面在跑 3afe5b97a)。

---

# 五段修:圆角三项视觉缺陷(2026-09-20 凌晨,commit 2a96108f0)

用户复测(四段修构建,刚开态截图 + 像素逐行扫描):
**滚动/输出全部正常 ✓**;余 ①内容圆角与窗框弧间浅色月牙 ②app 浅色
背景探出窗框圆角(顶部)③全局圆 thumb 突兀。

根因:①②= **非整行残差**(画布高与视口高差 ~22px@DPI1.25)在
scrollable 底部露 app 根浅色(余量区无涂色),方角在窗框圆角区探出;
顶部两角 = 014 根圆角机制只圆底部的历史缺口。③= 上轮 thumb 8 全局。

修法(2a96108f0):scrollable 外包视口级底色容器(Fill,余量色,四角
圆 16 同心)+ 根四角圆(RoundedT 随 RoundedB)+ 画布 quad 四角圆
(CANVAS_RADIUS)+ thumb 回 3。**最右下 pane thumb 圆角 prop** 涉
.at 语言层(Terminal 视图变体+三轨发射器)→ 024 §10 待澄清(用户
方案在案:round prop 只给最右下 splitpane)。

测试:根圆角例四角断言 + 全套绿。**待用户复测**(jitter7 在跑)。

---

# 六段修:顶部圆角回退 + 右侧"突出"像素澄清(2026-09-20,f28f4d6e2)

用户复测(五段修):底部几乎贴合(余 1-2px = 焦点环 2px + 多层同径
弧抗锯齿复合,像素级极限,接受);**顶部明确不要圆角**;右侧"浅色
突出"——像素定位澄清:终端暗色 (6,7,9) 紧贴焦点环内侧**无溢出**,
环外浅灰带(y60-310)= **后方其它窗口露出的边缘**(其下即壁纸)——
多窗叠加正常现象;计算器"不突出" = 浅色背景与 chrome 同色系不可见,
终端暗色高对比才显形。

修法:顶部三处回退(根机制去 RoundedT/画布 quad 顶角回方/外层容器
radius 底角 only),常量回 BOTTOM_RADIUS。测试回底角 only 断言。

**待用户复测**(jitter8 在跑 f28f4d6e2):底部圆角保持、顶部方角
恢复、整体观感。

---

# 七段:statusbar 状态栏方案落地 + 影子 manifest 取证通道(2026-09-20)

用户横分实录:per-pane 底角圆在**中间缝两角误圆**——圆角是窗框属性
非 pane 属性。**用户裁定 statusbar hack 落地**:

- auto-term plan-024-dev `4d1dde4`:app.at 加底部状态栏(h-24,
  rounded-b-2xl=窗框圆角,左'◉ AutoTerm mux'/右'cols×rows'),
  投影公式扣 statusH(6 处),实机 rows 22→21 验证;
- auto-lang auto-term-dev `390ad623e`:pane 全方角(画布 quad 方 +
  外层容器去 radius);根圆角机制保留(状态栏同弧)。

**取证通道升级(影子 manifest 根)**:桌面动态装载 apps.manifest
repo 相对路径指主检出——statusbar 提交后主检出处还原即丢失。方案:
`D:/autostack/.wt/term-024/os-shadow/apps.manifest`(auto-term repo=
worktree app 绝对路径;manifest_root.join 绝对径直用) +
`DESKTOP_OS_ROOT=shadow` 启动(82 行 export AUTO_OS_ROOT 无条件覆盖
传入 env 的坑,DESKTOP_OS_ROOT 才是 OS_ROOT 正门)。二分实证:
AUTO_OS_ROOT=空目录下 auto-term 仍装载 = env 被覆盖非语义问题。
现状:**桌面全栈 worktree**(auto-lang auto-term-dev + auto-term
plan-024-dev),statusbar 在快照,launch 状态栏渲染在案。

'右侧浅色突出'终判:用户坚持跟 term 走;像素证据(环内暗色无溢出/
环外带下缘 y310 截止)指后方窗露边——statusbar 方案后观感重估,
若仍在再开有界诊断。

---

# 生产桌面 025 符号撞击 + 影子台恢复(2026-09-21 00:40)

用户报生产桌面 auto-term 启动失败:Undefined symbol
term.config_spawn_program。根因:**并行计划 PLAN-025** 已推进
auto-term master(231ddac:db.at 增 config_profiles/spawn 系 6 符号
+ app.at 右键 profile 子菜单),生产桌面装载主检出新源码,而
autoterm_core.dll 两处均旧(9-17/9-18,早于 025)→ 链接失败。
DLL 重建被 025 会话自己的验收进程(auto.exe 36196)锁住——**归属
025 解除**,不代劳。

024 验收台恢复:影子 worktree 的 auto-down 兄弟被并行清理删除,
重建(detached)后影子桌面重拉(jitter12b):statusbar xl(12px)
在案。**024 验收与生产桌面互不影响**;合流注意:025 动了
app.at(右键 profile 子菜单)/db.at——024 merge 期与 statusbar
改动同文件融合。

---

# 八段:状态栏圆角精确校准 r=15(2026-09-21,a67cd569b + 5472f76)

用户复测 xl(12px):"还差一点点"。**几何推导定精确值**:窗框可见
弧(焦点环 2+边框 1,r=16,圆心 rect 内缩 16)与状态栏弧(客户区
内缩 1)重合条件 = **r=15**(同心内缩 1px,间隙均匀=边框宽);
命名刻度 12/16 两侧均不中。

落地:auto-lang RoundedSize::Px(f32) 变体(消费侧全走 to_pixels
零改动)+ rounded-{dir}-[Npx] 方向性任意值 parse(2 新例);
app.at 状态栏 rounded-b-[15px]。影子桌面冷载部署(jitter13)。

插曲:worktree incremental 缓存 5.8G 撑爆磁盘(os 112),清理后过;
生产桌面 025 符号撞击(Undefined symbol config_spawn_program)=
并行计划 DLL 陈旧,归属 025 解除(其验收进程锁 DLL);影子通道
auto-down 兄弟曾被并行清理,重建后恢复。

**待用户终验 15px 圆角**。

---

# "浅色月牙"归属终判(2026-09-21,决定性实验)

用户坚持月牙"跟着 term 走"。影子桌面决定性实验(jitter13 台):
- 基线截图(with-calc):term 窗 x326-2014,calculator(wid1)在场;
- bus close\1 关闭 calculator → 复测:**窗框右下弧外浅色像素 = 0**
  (with/no-calc 两帧同测,弧外 50×75 区域零残留);
- 反向验证(launch calc)因桌面被关未完成,单向证据 + 层分析
  (rect 外无 term 绘制层;环/边框/阴影全在 rect 内)已足。

**结论:月牙 = calculator(后方窗)的圆角角从 term 焦点弧旁露出**。
"跟着走"观感 = calc 紧贴 term 后方、相对位置稳定。自行验证:拖走
term(月牙留在原地)或关闭 calc(月牙消失)。

状态栏 15px 同心解(几何推导 + Px 任意值刻度)不受此影响——弧线
本身已精确。T-01 视觉链收口。

---

# T-05 三轨零回归冒烟(2026-09-21,worktree 源)

- **VM 轨**(`auto run -r vm`,worktree app):Init 走通、零 panic
  零 error(_smoke-vm.log;partial-db 警告 = 022 遗留 known)。
- **rust 轨**:`auto build -r rust` **Finished 零错**(EXIT=0,
  2m50s;statusbar .at → a2r 发射门 = 027 教训的"真 .at 重生成+
  编译"门)——exe 运行(DLL pinned)+ MCP:终端 cols=84 rows=25
  (几何随动 ✓)+ 状态栏渲染 ✓(_smoke-rust-run.log)。
  插曲:载具构建输出落在 lang worktree target(磁盘曾爆,重跑过)。
- **vue 轨**:`auto run -r vue` → VITE ready 195ms 编译零错
  (exit 143 = 限时器正常终止)(_smoke-vue.log)。
- **auto-lang 作用域门禁**:p024 6/6 + virtual_scroll 10/10 +
  p022 6/6 + p023 4/2/4 + terminal_input 2/2 + rounded parse 3/3。
- 已知 flaky:terminal_pixel_preedit(基线同现,三跑两绿一红)。

# T-02/03/04 实测覆盖记录(诚实口径)

- **用户已实测**(多轮复验):聚焦直键入(ls/dir 回显)、命令执行、
  滚动全链(跟随/贴底/滚轮/滚动条)、横分分屏、状态栏、圆角。
- **待用户补验**(实机门遗留,不阻塞 work 交付):Ctrl+C 中断、
  WT 快捷键全表(C-S-T/W/E/O/Z/Tab/←→/K)、嵌套分屏(≤6 超限
  拒绝)、分隔条磁吸钳位实点、Tab 点击/右键关/+新建、虚拟窗边缘
  拖拽 resize 与最大化/还原随动录证。
- **T-04 串扰**:TRACE 实证(calculator 同场时 term override 恒
  766 稳定不漂,jitter13 日志)+ 本轮 calc 关/开对照;
  resize 场串扰随遗留项。
