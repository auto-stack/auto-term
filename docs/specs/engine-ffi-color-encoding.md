# 引擎 FFI 颜色标量编码契约

> 来源:PLAN-016(SD-01,2026-09-14 复审通过)。交付锚定:autoterm-core
> 于 auto-term `09afd6e`。消费方:auto-lang VM shim
> (`crates/auto-lang/src/vm/ffi/term_engine.rs`)、at-app 侧车
> (`at-app/term.rs`)、at-gen 冻结胶水(`at-gen/src/engine.rs`)。

## 契约

`autoterm_engine_row_style` 以 **u32 标量色**逐格交出 fg/bg(交错,
`2×cols` 容量),编码 `(kind << 24) | value`:

| kind | 含义 | value 域 |
|---|---|---|
| 0 | Default | 0(宿主按主题取默认前/背景) |
| 1 | Indexed | 0–255(xterm 256 色;含 vte `NamedColor` 0–15 基础 16 色) |
| 2 | RGB | 0xRRGGBB |

**不变量(防复发)**:

1. vte/alacritty 的 `Color::Named(n)`:仅 `n < 256` 的判别值属于
   Indexed 契约(0–15 与 base16 对齐);**`n >= 256` 的语义色
   (Foreground=256、Background=257、Cursor、Dim*、BrightForeground
   等)一律归并 kind=0 Default**,由宿主主题解析。禁止按原值传出。
2. 消费端解码 **不得截断解释 value**:`Indexed(value as u8)` 类
   截断会把 257 折成 1(xterm base16 暗红 RGB(128,0,0))。
3. oracle 侧(autoterm-parity `kind()` 镜像、at-gen、侧车)与
   ffi.rs `kind_color` 必须同语义;单边修改视为契约漂移。

## 依据(事故档案)

cmd 会话不发 SGR,整屏默认格携带 `fg=Named(256)/bg=Named(257)`;
修复前按 Indexed 原值传出,消费端截断后 257→1=暗红、256→0=黑,
呈现为整屏黑字红底(013 起 VM 轨截图一直如此)。修复后宿主主题
默认前/背景(浅灰 #e8e8e8 / 近黑 #060709)接管。

## 回归

`cargo test -p autoterm-parity parity_color_attestation`
(oracle rlib vs a2r FFI STYLE 协议行逐格对拍,含 ESC[0m 复位行,
即语义色归并路径)。
