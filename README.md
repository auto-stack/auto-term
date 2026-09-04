# AutoTerm

AutoOS 的通用终端基础设施。产品名 **AutoTerm**;仓库 `auto-term`。

> ⚠️ **Spike 阶段声明**:`spikes/` 下的全部代码是打通全链路的一次性探针
> (PTY → 仿真核心 → 渲染),**非正式架构,允许整体重写**。架构决策见
> `docs/designs/000-render-route.md`。

## 定位

- **AutoOS 虚拟桌面的通用终端**:一个长得在宿主桌面里的普通进程,经
  基座 OS 的 PTY 服务(Windows: ConPTY / Unix: openpty)驱动任意
  shell 子进程,以 `alacritty_terminal` 为仿真核心渲染 VT 字节流。
- **与 auto-shell(ash)零构建依赖**:ash 只是 AutoTerm 可承载的任一
  子进程,以**运行时配置**(默认 shell 路径)耦合,不进入本仓依赖树。
  ash 是最苛刻的首批 VT 客户端(动态块/渐进表格/真彩色),用作冒烟
  标尺,但本仓构建永不依赖它。

## 布局

```
spikes/pty-probe     宿主 PTY 申请 + VT 字节流读取(无 GUI)
spikes/term-probe    alacritty_terminal 仿真核心嵌入(headless,含集成测试)
spikes/render-probe  最小 iced 窗口渲染网格 + 键盘回写(路线 A 探针)
docs/plans/          实施计划(auto-plan 流)
docs/designs/        设计决策(000:渲染路线 A/B)
```

## 运行(Windows 基座)

```powershell
cargo run -q -p pty-probe -- --duration 3          # PTY 字节流 + 字节率
cargo test -p term-probe                            # 真 PTY → 网格断言
cargo run -q -p render-probe -- --shell pwsh        # 可交互窗口
```

## 下一步

- **渲染路线已拍板:方案 A(iced 一等应用)**——判据核对与证据数字见
  `docs/designs/000-render-route.md`;
- **Phase 1 另立计划**:单窗口 MVP(正式 crate 结构 + 仿真正确性回归
  种子化 + 损伤重绘/字形图集/事件驱动清偿,清单见 `DEBTS.md`);
- Unix 基座(macOS/Linux)适配后续计划验证,portable-pty 已承诺路径。

