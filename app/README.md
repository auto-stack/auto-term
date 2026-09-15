# app — AutoTerm 统一工程入口（PLAN-013 三形态 / PLAN-017 入口统一）

AutoTerm 复刻应用的 automan 标准工程形态：一份 .at 源，三条标准命令
三种后端形态 + auto-os 桌面 launcher 装载，驱动同一个
`autoterm_core.dll`。PLAN-017 起 `app/` 为 AutoTerm 唯一工程入口
（at-app/ 目录已退役，桌面壳旧形态一并移除）。

## 可见多终端 UI(PLAN-019)

Tab 条常显(点击激活 / 右键关闭 / `+` 新建 / 横分 / 竖分 / 关Pane /
Zoom / ◐ scheme 切换)+ 布局槽位消费(zoom 全屏 / 横向 row / 纵向
col / 单 Pane;V1 分屏上限 = 树深 1,已分屏 Tab 再分拒绝)。每
terminal 实例按 pane key 动态绑定;泵三档(焦点全量+键入 / 可见
非焦点全量投喂 / 不可见 drain-only);打字随动焦点。快捷键(WT
风格,Ctrl+Shift 前缀):T 新Tab / W 关Pane / E 横分 / O 纵分 /
Z zoom / Tab、← 前 Tab / → 后 Tab / K scheme;命中即应用动作,
未命中的键(含裸 Ctrl+C/Z)原样进终端。关末 Pane = 关其 Tab;
关末 Tab 拒绝(app 常驻)。契约详见 docs/specs/terminal-mux-model.md
V1 语义节。

## 布局

| 文件 | 属性 | 说明 |
|---|---|---|
| `pac.at` | 工程清单 | scene ui;api rust;`rust_sidecar` 侧车声明;window 802x482(整窗即终端);icon terminal/category system/17400 端口占位(伞形注册键) |
| `term.rs` | 侧车真身 | `crate::term` 引擎胶水(源自 at-gen/src/engine.rs,T2 机制供给 rust/vue 生成点)+ per-handle 会话态 + spawn_ex/定向泵(rows_for/pump_for/apply_resize_for,PLAN-018 D3)+ scheme 表装载(D10) |
| `src/front/app.at` | AutoUI 前端 | terminal 组件(props-feed 形态甲)整窗渲染 + `oninput` 直键入;四装载面同一源(rust/vm/vue + 桌面) |
| `src/back/api.at` | #[api] 契约 | 全标量/[]str 面;/api/term/* 14 路 + /api/mux/* 11 路(PLAN-018 结构操作);VM/desktop 直跑委托体 |
| `src/back/db.at` | 实现体 | TermApp 引擎驱动状态机 + **MuxCore 五层模型**(PLAN-018:Workspace/Tab/LayoutTree/Pane + Action 单入口 `mux_*`;契约见 docs/specs/terminal-mux-model.md);back crate 用户逻辑唯一吸收面 |

## 运行(auto-lang 仓构建的 auto CLI;cwd = app/)

```bash
auto run -r rust   # a2r iced 窗口(merged:db.at 吸收进 front crate,进程内直调)
auto run -r vm     # AutoVM 动态轨(aura_view_builder Terminal 两臂 + auto.term shims)
auto run -r vue    # Vite 页面(<pre> 最小视口)+ axum back(HTTP)
```

auto-os 桌面装载：`apps.manifest` auto-term 条目（repo `../auto-term/app`）
→ 宿主 `build_dynamic_component` 进程内编译 front，`use back.api` 本地
解析 `src/back/api.at` 委托体，引擎经 auto.term shims 直达宿主进程内
`autoterm_core.dll`（部署件契约见仓 README/scripts，PLAN-017 §5）。

前置:`cargo build -p autoterm-core`(auto-term 仓)产出
`autoterm_core.dll`;`AUTOTERM_ENGINE_DLL` 指向它(VM shims 与
term.rs 胶水同款解析序:env → exe 同目录 → 祖先 target)。

## 已知边界(PLAN-015 时点)

- 直键入(像 Windows Terminal):点击终端获得焦点后直接键入;widget 把
  按键翻译成 VT 串(语义对齐 autoterm-ui 冻结 oracle `key_to_bytes`)
  入 TerminalCore 队列,`.KeyIn` 消息触发 `term_pump_input()` 排空队列
  裸写引擎(载荷不经消息,rust 侧车与 VM shim 同语义);IME 提交串整串
  透传,Ctrl+C/Ctrl+D 等控制码原生可用;IME 每次聚焦强制英文起步
  (auto-lang c9cc31ab3→029d80557,TSF 权威切换;用户 Shift 仍可切中文);
- **右键菜单 Interrupt(PLAN-015)**:菜单第 4 项(载荷 3)→
  `term_menu_take()` → `term_interrupt()`(Break→C 双投递);显式触发
  不误伤 idle ash(裸 Ctrl+C 仍纯字节)。已知边界:ping 类在 26200
  build 不可中断(DEBTS #12 残留);菜单标签渲染已根修(iced 弱引用
  段落,DEBTS #17;同族 badge/preedit 待修);开发布局(app exe 住
  auto-lang target)须设 `AUTOTERM_CTRLC_BIN` 指向 helper,dist 三件套
  同目录免设;
- 定时器:`timer { Tick (every_ms: 50) }` 驱动流式刷新——rust 轨经
  run_app_with_task_devtools 的 tick 订阅(014 补齐 with-task 变体),
  VM 轨经动态 timer 注册表;命令输出不再依赖按钮 tick;
- 几何随动:widget layout 由可用空间反推网格,经注册表 pending →
  `term_apply_resize()`(tick 驱动)→ 引擎 resize → View 几何回流,
  窗口最大化/拖拽均跟随;光标格经 `term_cursor_row/col` 每拍回读喂入
  (非零才落位,(0,0) 为未喂入哨兵);
- vue 形态视口为只读最小实现(`/api/term/pump` 恒 0——键入队列在窗口
  进程;`/api/term/menu-take` 恒 -1——菜单在窗口进程;DEBTS 009 #4 的
  xterm.js 类交互留白维持);
- at/autoterm.at 与 at-gen 冻结为对拍 oracle,不随本工程改动。
