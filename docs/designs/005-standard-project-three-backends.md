# 005 — 标准 Auto 工程三形态运行契约（PLAN-013）

> 性质:PLAN-013 沉淀型设计记录(2026-09-11 执行实证)。
> 关联:004(C 通道与分期)、DEBTS #10、009 #4(Vue/web 留白)。

## 1. 三形态与统一引擎

`at-app/`(标准 automan 工程)一份 .at 源,三条命令三种宿主,
同一个 `autoterm_core.dll`:

| 命令 | 宿主 | 引擎调用链 |
| --- | --- | --- |
| `auto run -r rust` | iced 窗口(a2r 合并进程内) | 生成工程内 `crate::term` 侧车 → DLL |
| `auto run -r vm` | AutoVM 动态轨(aura_view_builder) | `auto.term.*` shims → DLL |
| `auto run -r vue` | Vite 页面 + axum back(HTTP) | back crate 内 `crate::term` 侧车 → DLL |

## 2. 裁定与实证

### D1 调用面统一 = `use auto.term` + 裸函数调用

VM 原生命名空间(native_catalog 2943-2949,规范名与 at-gen 胶水同形);
a2r 对其下降 `use crate::term::{…}`(trans/rust.rs)。闭合条件(均落
auto-lang master):

- **Plan 347 影子抑制按 file_modules 判定**:原 auto_modules 把
  `use auto.term` 自己的限名符也注册,裸调用被自我抑制改道 reloc →
  link Undefined symbol。现只有**文件模块 use**(如 `use base64`)才可
  抑制同名 native;native 命名空间(`auto.*`)不再自遮。
- **#[vm] 无体声明不影子**:codegen 增 vm_fn_names(模块编译期收集,
  session 汇集,根 codegen 播种);无体声明只配 native。
- **List<str> 负哨兵解码**:bridge 增 `read_str_list_value`(内联
  Array/堆 ListData<i32> 的 `-(idx)-1` 哨兵→字符串表),接入
  convert_terminal——否则视口整屏空串。

应用面约束(T1 实测,写入 term_engine_shims_bare_call_roundtrip 注记):
裸调用的 List 返回须落**显式类型 var**(`var lines List<str> = …`);
裸调用不得内联于 if 条件位(597 §9 挂起缺陷,先赋值再比较规避)。

### D2 .rs 侧车机制(auto-man sidecar)

pac.at `rust_sidecar { modules: ["term:term.rs"]; deps: ["libloading:0.8"] }`
→ 两个生成点(front 工程 generate_rust_ui/regenerate_code_only、back
crate generate_api)复制模块入位 + `mod` 声明 + Cargo deps 注入,幂等。
侧车内容 `term.rs` = at-gen/src/engine.rs 同语义(crate::term)。

**merged db 吸收**:rust merged 形态原只产 JSON CRUD 桩;现当
src/back/db.at 存在、可转译、endpoint 全标量/[]str 面且 db 有同名实现
时,转译件以 `mod db` 嵌入 + 端点 fn 委托 `db::<fn>(…)`;
back crate(axum)同理:无 primary 类型的标量服务 API 落委托处理器
(原一律 TODO 骨架)。

### D3 terminal 生成器臂(最小)

- rust:`terminal{key,cols,rows,lines}` → `View::Terminal{…}` 直达;
- vue:最小只读 `<pre>`(v-for 逐行,HTML 转义);交互面(选中/回滚
  UI/xterm.js 类)维持 009 #4 留白。

## 3. 应用面已知边界(013 收口时点)

- 定时器(sched.*)是 VM 渲染靶专属;rust/vue 形态 tick 由
  Send/Ctrl+C/Refresh 按钮驱动——输出晚一拍,Refresh 补看;
- api fn 命名:`list_`/`get_` 前缀零参 fn 才被 async-init 引导识别
  (extract_init_api_func);at-app 用 `get_lines()`;
- a2r 对 `Time.sleep_ms` 存在路径限定缺陷(跨 fn 泄漏上一调用名,
  E0433 实证)——boot 等待用有界忙等规避,缺陷归 auto-lang 独立账;
- 引导快照可能早于 shell banner:db 首调带 400 次有界忙等重试。

## 4. 实证(2026-09-11,证据 docs/plans/evidence/013/)

- vm:`auto run -r vm` 真机窗口,banner/echo 回显/提示符上屏
  (vm-echo-final.png);MCP autoui_type/action 驱动。
- rust:`auto run -r rust` iced 窗口,生成码含 `mod db`(转译状态机)
  + `mod term` + `View::Terminal`;echo 上屏(rust-echo-final2.png)。
- vue:`auto run -r vue` Vite 页面 + axum back;HTTP 回环断言
  (`POST /api/term/send` 200;`/api/term/tick` JSON 视口含 echo)与
  页面截图(vue-page.png)。
