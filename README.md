# AutoTerm

AutoOS 的通用终端基础设施。产品名 **AutoTerm**;仓库 `auto-term`。

## 定位

> **PLAN-009 转型声明(2026-09-06)**:Auto 化第一相位落地后,本仓
> 双重身份——
> - **引擎 adapter(活)**:`autoterm-core`(PTY + alacritty 仿真)是
>   未来 AutoTerm 的引擎本体,经 cdylib FFI 面(`ffi.rs`,libloading
>   运行期加载)供 Auto 侧消费;这是本仓的主产品形态。
> - **Rust 参考实现 / 验收 oracle(冻结)**:`autoterm-ui` 的 widget.rs
>   与 App **冻结**——真身已迁 auto-lang `src/ui/terminal/`(terminal
>   组件);Rust 版仅作对拍基准(`crates/autoterm-parity` 门禁的
>   oracle 侧),只修对拍阻断项,不加产品功能。
> - **Auto 复刻应用**:`at/autoterm.at`(Auto 源)→ a2r 转译 →
>   `at-gen/`(产物 crate,挂 terminal 组件的真窗口应用)。
> - **无环铁律**:auto-lang 对本仓零 Cargo 依赖;本仓仅 `at-gen` 一处
>   依赖 auto-lang 运行时(cargo tree 断言在案,PLAN-009 T11)。

- **AutoOS 虚拟桌面的通用终端**:一个长得在宿主桌面里的普通进程,经
  基座 OS 的 PTY 服务(Windows: ConPTY / Unix: openpty)驱动任意
  shell 子进程,以 `alacritty_terminal` 为仿真核心渲染 VT 字节流。
- **与 auto-shell(ash)零构建依赖**:ash 只是 AutoTerm 可承载的任一
  子进程,以**运行时配置**(默认 shell 路径)耦合,不进入本仓依赖树。
  ash 是最苛刻的首批 VT 客户端(动态块/渐进表格/真彩色),用作冒烟
  尺规,但本仓构建永不依赖它。

## 支持声明

- 当前:**Windows 基座**,要求 Win10 1809+ (ConPTY `CreatePseudoConsole`
  的最低系统版本)。
- Unix(macOS/Linux)路径由 portable-pty 承诺,尚未验证——另立计划。

## 布局

```
crates/autoterm-core  终端会话层:PTY 会话 + alacritty_terminal 封装
                      (11 个仿真回归用例:真 PTY / 纯 VT 矩阵 / 生命周期 / 选中)
crates/autoterm-ui    单窗口终端(iced):事件驱动、实测字形度量、
                      回滚 UI、反色光标、选中/复制粘贴、IME 预编辑;
                      bin = autoterm
at-engine-face/       引擎 12 符号面的 Auto 版替身 cdylib(PLAN-012,
                      并存形态:at/engine_face.at 真身→a2r 转译入库;
                      parity 六场景已验,不替换 dist 生产物)
app/                 AutoTerm 统一工程入口(PLAN-013 三形态 + PLAN-017
                      入口统一:auto run -r rust|vm|vue 同源 + auto-os
                      桌面 launcher 装载;契约见 docs/designs/005;引擎经
                      侧车 term.rs/auto.term shims 驱动同一 DLL)
spikes/               PLAN-001 归档(一次性探针,保留作证据,不再演进)
docs/plans/           实施计划(auto-plan 流)
docs/designs/         设计决策(000:渲染路线;001:Phase 1 架构与决策链;
                      002:Auto 化前提调查)
```

## 运行

```powershell
# 标准 Auto 工程三形态(PLAN-013/017,详见 app/README.md):
cd app && auto run -r rust   # a2r iced 窗口(merged:db 吸收+侧车)
cd app && auto run -r vm     # AutoVM 动态轨(auto.term shims)
cd app && auto run -r vue    # Vite 页面 + axum back(HTTP)

cargo run -q -p autoterm-ui --                    # 交互终端(默认 pwsh)
cargo run -q -p autoterm-ui -- --shell <exe>      # 指定 shell(如 ash.exe)
cargo test --workspace                            # 全量回归
# 取证钩子(--dev-autotype/--dev-select 等)需 --features dev-tools 构建
```

### 用 ash 跑(虚拟桌面目标形态,PLAN-007)

```powershell
cargo run -q -p autoterm-ui -- --shell D:/autostack/auto-shell/ash/target/release/ash.exe
# ash 门禁回归(ash 在场即全量断言;AUTOTERM_ASH_BIN 可显式指定产物路径):
cargo test -p autoterm-core --test ash_integration
```

兼容性结论与已知限制(见 DEBTS #12/#13 与 003 §4.1):
`docs/designs/003-ash-compatibility.md`。

### Ctrl+C 中断(PLAN-008)

裸 Ctrl+C 为**双投递**:控制台事件(经辅助进程 `autoterm-ctrlc.exe`
广播,中断运行中的命令)+ 0x03 字节(raw 行编辑器废弃输入行)。
`--ctrl-c-mode auto|byte|event|both`(默认 `auto`:ash 豁免为仅字节
——事件会整体终止无 handler 的 ash 进程,其余 shell 双投递)。
已知边界:ping 类(对 Ctrl+Break 免疫)与 ash 内建命令(F1)仍不可
中断,见 003 §4.1。**部署注意:`autoterm-ctrlc.exe` 须与
`autoterm.exe` 同目录**(缺失自动降级字节路径,不崩)。

## 交互(Phase 3/4,PLAN-004/005)

| 操作 | 行为 |
| --- | --- |
| 左键拖选 | 字符级选中,松开即复制(copy-on-select,默认开);拖到视口上/下边缘自动滚动(松开或离开边缘即停) |
| Alt+左键拖选 | 块选:矩形列带选中,复制按行截断(与多击计数正交) |
| 双击 / 三击 | 词选(semantic)/ 整行选(lines),松开即复制 |
| Ctrl+Shift+C / Ctrl+Shift+V | 显式复制 / 粘贴(裸 Ctrl+C 走中断双投递,不劫持;见"Ctrl+C 中断"节) |
| 右键 | 上下文菜单(复制 / 粘贴 / 全选);ESC 或菜单外点击关闭 |
| 滚轮 / PgUp / PgDn | 回滚浏览(键入自动回正,右上 `↑N` 偏移指示) |
| IME | 预编辑内联显示于光标处(带下划线,不上屏);提交/上屏才写入 PTY |
| 光标 | 形状随 shell DECSCUSR(Block/Underline/Beam);聚焦时 500ms 闪烁;失焦/IME 组合中/菜单开着时常亮 |

选中高亮随内容滚动保持锚定(绝对网格行);高亮色可配
`--selection-color RRGGBB[AA]`(默认 `e8e8e8@25%`,非法值回退默认)。
细节与证据:`docs/designs/001-phase1-architecture.md`(000→005 决策链)。

架构与设计决策:`docs/designs/001-phase1-architecture.md`;已知债务与
后续方向:`DEBTS.md`。

## 下一步

- ping 类(Break 免疫)命令中断的残留缺口待健康 build 验证
  (DEBTS #12 清偿方向);
- ash 装 ctrl handler(auto-shell 侧,#13)后撤 auto 豁免;
- Unix 基座适配;
- Auto 化(DEBTS #8,用户裁定必须项):Rust 版功能已齐全,#7 调查
  已关账(TermGrid 须以 auto-lang 内原生组件落地,rust-mode 示例
  存在存量编译问题,见 `docs/designs/002-autoize-feasibility.md`)。
