# 主题接入（theme-adoption）— auto-term

> 来源：auto-os/PLAN-061（主合同）。宿主策略定义见 OS
> `docs/specs/shell/host-theme-policy-and-delivery.md`；批次登记见 OS
> `docs/specs/apps/product-theme-batch-061.md`。本页只记本产品的实际表面
> 与合法固定值，不重复宿主机制。

## 接入形态（2026-11-05，PLAN-061 T-03）

- `app/vendor/themes`：UI 管理渠道固定版本快照（themes 0.2.0，内容 digest
  `9fa07742dc68aa5377ae06cb2d966386002063f184c515b6c5dddf8a6bef838e`）。
- `app/pac.at`：原显式 `theme: "dark"` 标量键退役 = **follow 宿主**；
  `dep themes { path: "vendor/themes" }`。
- `app/src/front/app.at`：`use themes.themes: Auto` + `dark_mode` 契约位
  （默认 true 保现暗色默认）+ `design { theme: "Auto", mode: if
  .dark_mode … }`；◐ scheme 开关与 Shortcut(9) 统一翻 dark_mode 位，
  `scheme` 由 mode 位派生（Init 同步 + 六槽表达式 prop）。

## 公共 role 面（PTY/VT 与公共壳分开）

- 公共壳：根/内容区 `bg-background`（原有 token 消费）、状态栏
  `bg-card`（原 fix-statusbar-style #282d34/#e9e4d8 硬编码退役）、Tab
  条原生按钮默认样式随主题。采样 oracle = 状态栏中心条。
- PTY/VT ANSI 调色 = `terminal` 原生组件 palette
  （`terminal_effective_palette`，引擎所有）：**内容域豁免不映射**；
  `scheme` prop 组件契约形态不变（0=classic-dark 1=light）。

## 合法固定值（内容/业务面，不随主题）

- VT 16 色 ANSI 输出、光标、选中反白：终端内容身份。
- mux 分屏拓扑/Pane 几何/resize 随动：业务面（cargo workspace 原门
  49 passed/0 failed 守护）。

## 主题腿实录（app/tests/test_061_theme.py，5/5）

A 显式暗状态栏 `#0f0f12` 精确 / B 显式亮 `#ffffff` + standalone ◐ 真实
开关翻回 `#0f0f12`（toggle 权威域=standalone/override 轨；active policy
下被 policy re-assert 为 053 优先级正确行为，实录）/ C Solarized/dark
宿主候选 `#073642` 精确 / D 活体翻转守恒 / E 冷恢复 `#eee8d5` 精确。

## 具名边界（预存，非本批）

CLI VM split-over-HTTP 形态 shim extern `term/get_lines` 500 → 独立 CLI
轨无 PTY（owner=VM 宿主平台 shims；桌面=宿主进程内装载不受影响）；
app/rust-workspace a2r 构建断（auto-lang `ui-iced` feature 漂移）；boot
种子与 Init scheme 派生同步缝（◐一次闭合）。主题门采样 = 公共壳（本页
oracle 面），PTY/mux 业务门 = cargo workspace。
