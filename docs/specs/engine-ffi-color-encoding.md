# 引擎 FFI 颜色标量编码契约

> 来源:PLAN-016(SD-01,2026-09-14 复审通过)。交付锚定:autoterm-core
> 于 auto-term `09afd6e`。消费方:auto-lang VM shim
> (`crates/auto-lang/src/vm/ffi/term_engine.rs`)、at-app 侧车
> (`at-app/term.rs`)、at-gen 冻结胶水(`at-gen/src/engine.rs`)。
> PLAN-018(SD-03,2026-09-15)增补**配色方案面**(§scheme):kind_color
> 编码零改,渲染端 Default/base16 解析换表;face 17→19。

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

## 配色方案面(PLAN-018 D9/D10;scheme 表引擎单源)

PLAN-016 前:默认色/16 色板为渲染端写死深色常量(浅色桌面终端恒
黑底——2026-09-14 用户实机验收裁定,以 Windows Terminal 式可选
配色方案随引擎面扩建落地)。

**face 17→19**(kind_color 编码零改,Default/Indexed/RGB 语义穿传,
解析端换表):

| 符号 | 契约 |
|---|---|
| `autoterm_engine_set_palette(h, scheme_id) -> i32` | per-handle 方案选择(所有权铁律:scheme 属会话模型)。0=成功;-1=空句柄;-2=未知方案 |
| `autoterm_engine_palette_color(scheme_id, slot, is_fg) -> u32` | 纯函数无柄,0xRRGGBB。slot:0=def-fg、1=def-bg、2..=17=base16;is_fg 预留轴(0/1)。非法→0xFFFF_FFFF 哨兵(RGB 最高位必 0,无歧义;宿主以判定方案枚举终点) |

**scheme 表**(autoterm-core `palette.rs` 单源;渲染端回退表在
auto-lang `ui::terminal`,内置值同源):

- scheme 0 `CLASSIC_DARK`:def fg `#e8e8e8` / def bg `#060709`,
  base16 = xterm 标准盘——与 016 定型行为**逐字节一致**(零回归)。
- scheme 1 `LIGHT`:Windows Terminal "Solarized Light" 官方盘
  (def fg `#586e75` / def bg `#fdf6e3`,base16 深底变体)——浅底
  深字,浅色桌面可用。
- SCHEME_COUNT=2;用户自定义 scheme 上传为后续非目标。

**渲染端契约**(auto-lang `ui::terminal`):

- Term 组件 `scheme` prop(int):缺省 `-1` = 跟随桌面主题
  (`theme::dark_mode`:dark→0/light→1);≥0 显式覆盖。
- 引擎 glue(VM shim/侧车)装载一次:`palette_color` 逐槽查询 →
  注册表缓存(`terminal_palette_load`);未装载/旧 DLL 缺符号 →
  静默回落内置表(渲染不中断)。scheme 切换 = 重装载覆盖。
- 解析开销:每帧一次取 [18] 表,每格 O(1);Indexed 16-255 仍走
  xterm 256 全映射(表只定义 base16)。
- 光标块 = 方案前景色 @0.85 alpha(classic-dark 下与旧 0.91 常量
  逐字节同值,像素金样不受扰)。

**回归**:`cargo test -p autoterm-core --lib`(palette 三单测)+
ffi `palette_*` 三用例 + 016 像素金样(`--lib terminal`,dark 臂
逐字节)+ 双端 light 实机截图(evidence/018/rust-gui-light-scheme1.png、
vm-gui-light-scheme1.png)。
