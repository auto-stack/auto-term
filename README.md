# AutoTerm

AutoOS 的通用终端基础设施。产品名 **AutoTerm**;仓库 `auto-term`。

## 定位

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
spikes/               PLAN-001 归档(一次性探针,保留作证据,不再演进)
docs/plans/           实施计划(auto-plan 流)
docs/designs/         设计决策(000:渲染路线;001:Phase 1 架构与决策链)
```

## 运行

```powershell
cargo run -q -p autoterm-ui --                    # 交互终端(默认 pwsh)
cargo run -q -p autoterm-ui -- --shell <exe>      # 指定 shell(如 ash.exe)
cargo test --workspace                            # 全量回归
# 取证钩子(--dev-autotype/--dev-select 等)需 --features dev-tools 构建
```

## 交互(Phase 3,PLAN-004)

| 操作 | 行为 |
| --- | --- |
| 左键拖选 | 字符级选中,松开即复制(copy-on-select,默认开) |
| 双击 / 三击 | 词选(semantic)/ 整行选(lines),松开即复制 |
| Ctrl+Shift+C / Ctrl+Shift+V | 显式复制 / 粘贴(裸 Ctrl+C 仍走 PTY 中断,不劫持) |
| 右键 | 粘贴 |
| 滚轮 / PgUp / PgDn | 回滚浏览(键入自动回正,右上 `↑N` 偏移指示) |
| IME | 预编辑内联显示于光标处(带下划线,不上屏);提交/上屏才写入 PTY |

选中高亮随内容滚动保持锚定(绝对网格行);拖选暂不自动滚动
(Phase 4 候选)。细节与证据:`docs/designs/001-phase1-architecture.md`
(000→004 决策链)。

架构与设计决策:`docs/designs/001-phase1-architecture.md`;已知债务与
Phase 4 方向:`DEBTS.md`。

## 下一步(Phase 4+)

- 拖选到边缘自动滚动、块选、右键上下文菜单、选中色配置;
- Ctrl+C 稳定版复测(待环境,见 DEBTS #7);光标形状/闪烁;
- Unix 基座适配;iced↔auto-ui 生态对齐(`.at` 组件模型调查)。
