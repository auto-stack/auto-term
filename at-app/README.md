# at-app — AutoTerm 标准 Auto 工程(PLAN-013)

AutoTerm 复刻应用的 automan 标准工程形态:一份 .at 源,三条标准命令
三种后端形态,驱动同一个 `autoterm_core.dll`。

## 布局

| 文件 | 属性 | 说明 |
|---|---|---|
| `pac.at` | 工程清单 | scene ui;api rust;`rust_sidecar` 侧车声明 |
| `term.rs` | 侧车真身 | `crate::term` 引擎胶水(源自 at-gen/src/engine.rs,T2 机制供给 rust/vue 生成点) |
| `src/front/app.at` | AutoUI 前端 | terminal 组件(props-feed 形态甲)+ 输入行;三形态同一源 |
| `src/back/api.at` | #[api] 契约 | 全标量/[]str 面;VM 直跑委托体 |
| `src/back/db.at` | 实现体 | TermApp 引擎驱动状态机(移植自冻结的 at/autoterm.at);back crate 用户逻辑唯一吸收面 |

## 运行(auto-lang 仓构建的 auto CLI;cwd = at-app/)

```bash
auto run -r rust   # a2r iced 窗口(merged:db.at 吸收进 front crate,进程内直调)
auto run -r vm     # AutoVM 动态轨(aura_view_builder Terminal 两臂 + auto.term shims)
auto run -r vue    # Vite 页面(<pre> 最小视口)+ axum back(HTTP)
```

前置:`cargo build -p autoterm-core`(auto-term 仓)产出
`autoterm_core.dll`;`AUTOTERM_ENGINE_DLL` 指向它(VM shims 与
term.rs 胶水同款解析序:env → exe 同目录 → 祖先 target)。

## 已知边界(013 收口时点)

- 定时器(sched.*)是 VM 渲染靶专属——rust/vue 形态 tick 由
  Send/Ctrl+C/Refresh 按钮驱动,实时流式刷新留后续;
- vue 形态视口为只读最小实现(DEBTS 009 #4 的 xterm.js 类交互留白
  维持);
- at/autoterm.at 与 at-gen 冻结为对拍 oracle,不随本工程改动。
