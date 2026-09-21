# PLAN-024 T-00 几何源决策工件

日期:2026-09-19 | 取证人:work 会话 | 关联:AC-01/02,§2 缺口 A/B/C

## 1. 现行为取证(T-00a)

净场:无 ui_desktop.exe/auto-term.exe 残留实例(tasklist 核对);
`bash D:/autostack/auto-os/scripts/desktop.sh iced` + `AUTOUI_ACCEPTANCE=1
AUTOUI_MCP_PORT=9251 AUTOTERM_ENGINE_DLL=D:\autostack\auto-term\target\debug\
autoterm_core.dll`(钉 9-18 18:34 构建,规避 auto-lang/target/debug 下
9-17 陈旧 DLL);MCP bus `launch\tauto-term`(autoui_desktop bus verb)
→ `[session] launch_app(inproc) auto-term`,Init/Tick 活。

### 1.1 mux 形态渲染现状

**形态在**。快照(`_snap-term.txt`):Tab 条(shell 1 + `+` 新建 + 横分/
竖分/关Pane/Zoom/◐)+ relative 容器 + 终端件摘要
`terminal key=pane-1 cols=81 rows=22 lines=22`(引擎几何真值,非模型
缺省 100×30)。截图 `t00-a-clean3.png`:_wm_wins wid=2 "终端" focused,
标题条/Tab/终端内容(Windows shell banner)齐。

### 1.2 投影错位形态(实测)

t00-a-clean3.png(2560×1600 物理)像素测量:

- 终端虚拟窗 bg(#060709)包围盒 **x 260..1784,y 408..1124**
  → 外沿 1524×716;内容区(扣 chrome BORDER=1×2,TITLEBAR_H=36)
  ≈ **1520×679 物理**;
- 终端文本块 **x 269..1137,y 450..967** ≈ **868×517 物理**,顶左对齐;
- 即:**终端按 ~868px 宽标定渲染(81 cols),内容区实际 1520px** ——
  欠尺寸 + 右侧/底部死区。启动态即错(无需宿主 resize)。

机理:标定源链(shim 2998/2999 → theme thread_local)在 Tick 时刻读到
的值 ≠ 该 app 虚拟窗内容区(见 §2 语义混沌);终端件按 pane px 自测
→ 引擎 resize 随 pane px 自洽,故呈现"小终端在大窗"而非溢出。

### 1.3 键鼠到达情况(初探)

- `autoui_desktop` key action(chars "echo p024-live" + Enter VK13):
  队列受理,但无任何 TERM_PRESS/Keyboard VM 事件踪迹 → **注入未达
  终端件**(预期:需真实点击取得组件键盘焦点,同 §10.7 DEBT 域)。
  留 T-02 实机门(实机键入录证,017/020 同法)。
- MCP `autoui_action` press 对 auto-term 按钮(横分)报
  "No 'press' handler found":action 通道打标 primary App(T8 单 App
  语义),桌面多 app 下 shell 持 primary → auto-term 件不可达。
  **T-03 鼠标门必须实机鼠标直接操作**(020 惯例),MCP 仅取证快照/截图。

### 1.4 多 app 现场要素

启动场自带 011-calculator(wid 1)+ auto-term(wid 2);桌面 widget
(watch/待办清单等)在壁纸区,像素测量时注意区分(勿当文本溢出)。
另有无关报错注记:017-chat 框架 demo 链接失败(api.send_message
未定义),与本计划无关。

## 2. 462 WM 虚拟窗 resize 语义定界(T-00b)

**(1) 虚拟窗支持用户独立 resize**:session.rs:1300-1370 八向边缘拖拽
(East/South/West/North + 角),MIN 120×120,增长向钳制不越宿主可用区;
resize 完成 → `v.rect` + **`v.window_size = rect 外沿 w/h`**(session.rs:
1360,注释自称"响应式布局 window_width 随缩放更新")。

**(2) 尺寸事件载荷路径**(三写入方 + 一缺位):

| 写入方 | 位置 | v.window_size 写入值 | 语义 |
|---|---|---|---|
| 开窗 | session.rs:810-825 add_win | rect 外沿 w/h | 外沿 |
| WM 用户 resize | session.rs:1360 | rect 外沿 w/h | 外沿 |
| 宿主窗 resize | renderer.rs:19142-19154 | **宿主 OS 窗全尺寸**(载荷 `__window_resized` = 宿主逻辑尺寸,全体虚拟窗覆写) | 宿主全窗 |
| fit(Plan 512) | renderer.rs:12825 | 内容区(不含 chrome) | 内容 |
| **最大化/还原** | session.rs:1264-1280 toggle_maximize_win | **不同步**(只写 v.rect) | — |

→ `v.window_size` 五写入方三语义 + 最大化缺位:对"该 app 虚拟窗内容区"
这一 mux 标定语义,**该字段不可用**(启动即错位、宿主 resize 后全错、
最大化后冻结旧值)。

**(3) session.rs:1359 窗口级字段在桌面的到达值**:每 app 视图经
`split_ref_at/split_mut_at`(session.rs:4664/4613)拆借
`window_size = &v.window_size`(desktop 分支 4675/4631);
`dynamic_view_impl`(renderer.rs:20282)每 app view 构建前把
theme thread_local 写为该值。**shell 族**(shell/launcher/switcher/
notification/desktop fields)的 window_size = 宿主 viewport
(sync_shell_windows renderer.rs:13950-13963)→ **shell 每帧 view 构建
把 theme 覆写为宿主全窗** —— 进程级串扰(缺口 B)的常驻源:
auto-term Tick 读 theme 的值取决于"最后一个完成 view 构建的是哪个 app",
帧序非确定。

**(4) rust/vm 轨现状**(对照):run_app WindowResized(T-03 ①,renderer.rs:
24496)与动态轨 `__window_resized` 拦截(T-03 ②,renderer.rs:24766)
写 theme —— 单窗轨语义正确;桌面轨复用 ② 路径但载荷 = 宿主全窗(缺口 A)。

## 3. 候选对照决策(T-00c)

| 维度 | A shim 会话感知(rect 派生) | B 窗口级字段前馈 | C 组件测量回填 |
|---|---|---|---|
| 真相源 | v.rect(全 WM 写入方一致维护的外沿几何,**单一真相**) | v.window_size + 注入环 | 组件实测 bounds |
| 前提成立? | ✓ | **✗ 实机证伪**:§2 表,响应式字段五写入方三语义,须先修字段维护再建前馈环 | 部分(fit 测量基建在,但为 fit_pending 专用) |
| auto-lang 改动面 | shim 读臂 + 拆借点注入(~2 处) | 字段维护修复(5 写入方)+ 模型注入面 + (无 .at 新面) | 测量回填环路(新) |
| auto-term 改动面 | **零** | db.at/api.at/app.at 三件 + 新路由 | app.at 回填逻辑 |
| per-app 隔离 | 天然(注入随拆借,每 app 每次派发刷新) | 天然(前馈 per-app) | 天然 |
| 三轨侵入 | 零(rust/vm 轨不设 override,行为不变) | 低但动 shim 读法承诺面 | 低 |
| a2r 可表达性 | 不涉及(零 .at 改动) | 需新 api 路由 + 模型变量 | 需回填约定 |
| 失败域 | 注入点未覆盖的 VM 入口(MCP fixture 臂)回落 theme(现状兼容) | 字段维护再漏一处即错 | 回填环时序/帧延迟 |

**决策:候选 A,桌面限定形态。**

- 落地形态:`term_engine` 增 per-app 几何 override thread_local
  (`set_app_window_px(w,h)`,0=清除);shim 2998/2999 读序 =
  override(≥1 有效)→ theme(现状)。**override 仅桌面轨设置**:
  split_ref_at/split_mut_at desktop 分支(session.rs:4631/4675 邻位)
  从 `v.rect` 派生内容区 `(rect.w−2·BORDER, rect.h−TITLEBAR_H−BORDER)`
  `.max(1)` 写入。rust 轨(T-03 ①)/动态轨(②)/vm 轨(缺省)零变化。
- 下游自动随动:app.at Tick 轮询 mux_window_width(app.at:228)→ 重投影
  → 终端件自测 → 引擎 resize/光标随动,均既有链,零 .at 改动。
- 022 T-03 ② 拦截路(theme 写)桌面臂处置:T-01 实施时评估
  `is_desktop` 收口(退役或保号),以复测定;
- 遗留注记(不入本计划范围,SD-03 记注):`v.window_size` 语义混沌
  (§2 表)对**其他桌面 app 的响应式布局**(Plan 409 grid 列数等,
  renderer.rs:20282 消费)同样成立——本计划不动该面(范围纪律),
  错位留档供后续计划裁决。

### 决策依据核对(§2 维度)

- 虚拟窗 resize 语义:有独立 resize(§2-1),载荷路径已定界(§2-2)
  → AC-02 验证口径 = 虚拟窗独立 resize + 宿主 resize 双随动,无缩句。
- 多 app 隔离:A 构造性隔离(每 app 派发注入)。
- 最小侵入:三轨契约零触碰(rust/vm 不设 override)。
- a2r 可表达性:零 .at 改动,天然满足。

## 4. 证据清单

- `t00-a-boot-mux-form.png` 启动首屏(launcher 遮罩期)
- `t00-a-clean3.png` 净屏(launcher 已关)——§1.2 测量源
- `_snap-term.txt` MCP 快照(mux 形态 + terminal cols=81 rows=22)
- `_state-term.txt` shell 态(__wm_wins wid2 "终端" focused)
- `_desktop-boot.log` 启动/launch_app(inproc)/Tick 踪迹
- `_mcp.py` 取证驱动(bus verb launch / key / handler 通道)

## 5. T-01 施工要点(从本决策导出)

1. auto-lang worktree(组 term-024)+ Category B 门禁;
2. override 面:term_engine.rs(APP_WINDOW_PX thread_local + setter +
   shim 读序);session.rs 拆借点 desktop 分支注入(rect 派生内容区);
3. 注入覆盖核对:split_mut_at(update 派发)/split_ref_at(view/订阅闭包)
   两口即覆盖全部 app 域 VM 入口;MCP fixture 臂回落 theme(兼容现状);
4. 验证:桌面单窗——启动即满幅投影(快照 cols 变化)+ vwin 边缘拖拽
   resize + 最大化/还原 + 宿主窗 resize 三态录证;calculator 同场
   开窗/resize 前后标定值不变(串扰面,AC-03);
5. 三轨零回归面:rust/vm/vue 冒烟 + p022_stack 5/5 + terminal_input 2/2
   (override 仅桌面设,三轨读路径不变)。
