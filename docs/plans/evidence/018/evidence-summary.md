# PLAN-018 T-06 集成取证汇总(2026-09-15)

## 证据清单

| 文件 | 内容 | 断言 |
|---|---|---|
| `curl-mux-script.log` | AC-06 模型行为权威剧本(axum back = rust-workspace/app-back,HTTP :17401) | 初始单 Pane→split(树 3 节点/2 Pane/focus 不变)→focus(pane-2)→zoom toggle(1→0)→close(兄弟收编+焦点回落+handle 释放)→末 Pane 拒绝(0,结构不变)→new-tab/activate/close-tab→末 Tab 拒绝;cols/rows=100/30;focus-key=pane-N |
| `rust-gui-dark.png` | rust 轨实机(1293×836,PrintWindow) | 内容区 dominant RGB(6,7,9)=classic-dark 0x060709(917 网格样本);文字 0xE8E8E8;跟随主题缺省臂 |
| `rust-gui-after-minimize.png` | 014 最小化/恢复配方后复查 | Responding=True;dominant 仍 (6,7,9)×917——无重排爆炸、无复燃 |
| `rust-gui-light-scheme1.png` | rust 轨显式 scheme:1 | dominant RGB(253,246,227)=0xFDF6E3 light bg;文字 0x586E75(solarized fg)——浅底深字 |
| `vm-gui-dark.png` | VM 轨实机(815×518) | dominant (6,7,9);文字 0xE8E8E8;跟随主题臂 |
| `vm-gui-light-scheme1.png` | VM 轨显式 scheme:1 | dominant 0xFDF6E3;文字 0x586E75 |

像素采样口径:全图网格(步进 20/25-30/40)Count 计数;dark 臂 (6,7,9) 均为最大计数簇;边角 (243,243,243)/(0,0,0) 为窗框 chrome 与圆角,非终端内容。

## 测试命令与结果

| 门 | 命令 | 结果 |
|---|---|---|
| 引擎 ffi(AC-01/02/10) | `cargo test -p autoterm-engine-ffi-tests --test engine_ffi_integration` | 7/7 绿 2.62s(cwd/argv/双柄隔离 + 旧 spawn 全量回归 + classic-dark 18 槽逐值/light 浅底/set_palette 契约) |
| core 单测 | `cargo test -p autoterm-core --lib` | 5/5(pty 2 + palette 3) |
| parity(016 色契约) | `cargo test -p autoterm-parity` | 5/5 绿 6.17s |
| widget/VM 泵面(AC-04) | `cargo test -p auto-lang --features ui-iced,iced-layout-tests --lib terminal` | 29/29 绿(016 像素金样三件 + per-key 定向 2 件 + palette 解析 1 件) |
| VM shim 冒烟(AC-05) | `auto crates/auto-lang/test/term_mux_pumps/spawn_ex_pumps_vm.at` | SPAWNEX_OK(argv 透传回显见证 + 缺 key pump/resize no-op) |
| VM shim 注册 | `cargo test -p auto-lang --lib catalog_integrity` | 3/3(2983-2986 入册) |
| 模型行为(AC-06) | curl 剧本(见上) | 全断言过 |
| GUI 等价(AC-07/11) | 实机截图 + 像素采样(见上)+ 014 抽查 | 过 |
| 全量 | `cargo test --workspace`(auto-term) | T-08 收口时运行,见 §9 收尾记录 |

## 执行注记

- AC-06 运输形态注记:剧本经 app-back(axum,api.at→db.rs 同一 a2r axum 发射器)驱动;vue 前端 Vite 页面因 a2vue 生成 App.vue 重复导入 `onUnmounted`(auto-lang codegen 既有缺陷,与本次 .at 无关,app.at 未改)未能整链起服——模型行为断言面(back HTTP)不受影响;缺陷修复留后续 auto-lang 计划。
- 光标块颜色经 scheme 前景 alpha 化后,classic-dark 下与旧 0.91 常量逐字节同值(0xE8=232),像素金样不受扰。
- VM light 臂截图时窗框存在 (26,34,39) 底栏(VM 壳 chrome),非终端内容区。
