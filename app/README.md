# app — AutoTerm 统一工程入口（PLAN-013 三形态 / PLAN-017 入口统一）

AutoTerm 复刻应用的 automan 标准工程形态：一份 .at 源，三条标准命令
三种后端形态 + auto-os 桌面 launcher 装载，驱动同一个
`autoterm_core.dll`。PLAN-017 起 `app/` 为 AutoTerm 唯一工程入口
（at-app/ 目录已退役，桌面壳旧形态一并移除）。

## 布局

| 文件 | 属性 | 说明 |
|---|---|---|
| `pac.at` | 工程清单 | scene ui;api rust;`rust_sidecar` 侧车声明;window 802x482(整窗即终端);icon terminal/category system/17400 端口占位(伞形注册键) |
| `term.rs` | 侧车真身 | `crate::term` 引擎胶水(源自 at-gen/src/engine.rs,T2 机制供给 rust/vue 生成点)+ 键入泵 `engine_pump_input`(读 auto_lang 键入队列裸写引擎) |
| `src/front/app.at` | AutoUI 前端 | terminal 组件(props-feed 形态甲)整窗渲染 + `oninput` 直键入;四装载面同一源(rust/vm/vue + 桌面) |
| `src/back/api.at` | #[api] 契约 | 全标量/[]str 面;VM/desktop 直跑委托体 |
| `src/back/db.at` | 实现体 | TermApp 引擎驱动状态机(移植自冻结的 at/autoterm.at);back crate 用户逻辑唯一吸收面 |

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

## 已知边界(014 直键入收口时点)

- 直键入(像 Windows Terminal):点击终端获得焦点后直接键入;widget 把
  按键翻译成 VT 串(语义对齐 autoterm-ui 冻结 oracle `key_to_bytes`)
  入 TerminalCore 队列,`.KeyIn` 消息触发 `term_pump_input()` 排空队列
  裸写引擎(载荷不经消息,rust 侧车与 VM shim 同语义);IME 提交串整串
  透传,Ctrl+C/Ctrl+D 等控制码原生可用;
- 定时器:`timer { Tick (every_ms: 50) }` 驱动流式刷新——rust 轨经
  run_app_with_task_devtools 的 tick 订阅(014 补齐 with-task 变体),
  VM 轨经动态 timer 注册表;命令输出不再依赖按钮 tick;
- 几何随动:widget layout 由可用空间反推网格,经注册表 pending →
  `term_apply_resize()`(tick 驱动)→ 引擎 resize → View 几何回流,
  窗口最大化/拖拽均跟随;光标格经 `term_cursor_row/col` 每拍回读喂入
  (非零才落位,(0,0) 为未喂入哨兵);
- vue 形态视口为只读最小实现(`/api/term/pump` 恒 0——键入队列在窗口
  进程;DEBTS 009 #4 的 xterm.js 类交互留白维持);
- at/autoterm.at 与 at-gen 冻结为对拍 oracle,不随本工程改动。
