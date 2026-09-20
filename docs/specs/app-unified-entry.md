# app/ 唯一工程入口契约(三形态同源 + 桌面装载形态)

> 来源:PLAN-017(SD-01,2026-09-14 R3 复审通过)。交付锚定:auto-term
> `c157c80`(置换)/`e3095ec`(限定名切换)/`4346580`(R2 门复验)。
> 裁定:用户 2026-09-14 会话内明示(以 at-app 三形态工程置换 app/
> 桌面壳,未来统一 `app/` 为唯一工程入口)。

## 唯一入口契约

`app/` 是 AutoTerm **唯一工程入口**,三种装载形态同源单仓:

| 形态 | 命令 | 产物/宿主 |
|---|---|---|
| rust(独立入口) | `auto build -r rust`(cwd=app/) | `auto-term.exe`(用户日常独立入口) |
| vm(merged) | `auto run -r vm` | CLI 宿主进程内解释,backend in-process |
| vue(split) | `auto build/run -r vue` | front 17400 / back 17401 |

约束:

1. 新工作一律落 `app/`;双前端并存不再合法。`at-app/` 已退役删除,
   历史经 `git log --follow app/src/front/app.at` 可溯
   (`0770f3f` 013 T7 → `c157c80` 017 T-01 git mv)。
2. 三形态键同源 `app/pac.at`(api:rust / exe_name / rust_sidecar /
   window / theme);旧壳输入框+Send 形态已删除,交互=整窗 terminal
   点击直键入(014 形态)。

## 桌面装载形态

auto-os 桌面 launcher 指向 `../auto-term/app`(apps.manifest 行:
kind=repo、ports 17400/17401 占位、无 daemon、status=active),装载 =
**进程内动态编译 front + 本地 src/back merged 委托体 + auto.term
shims 进程内引擎**;`#[api]` 调用必须走**限定名形态**
(`use back.api` 模块导入 + `api.X()`)——裸名导入在 merged/桌面动态
编译路径落入 PLAN-053 no-op 桩(端到端死),限定名直落本地字节码为
唯一进程内可达形态(rust/vue 发射器限定名支持由 auto-lang PLAN-627
交付,master `3b2f7cf56`)。

引擎 FFI:`app/term.rs` 经 libloading 加载
`autoterm_core.dll`(解析序 env `AUTOTERM_ENGINE_DLL` → exe 同目录 →
exe 祖先 `target/{debug,release}`),色彩契约见
[engine-ffi-color-encoding.md](engine-ffi-color-encoding.md)。

## 配置文件与 profiles(PLAN-025 SD-02)

默认 shell 与 profile 集经 `config.toml` 定义,crate
`crates/autoterm-config` 单源解析(仅 autoterm-ui CLI 与 app-back
消费;autoterm-core 零依赖面不动):

- **位置**:`AUTOTERM_CONFIG` 环境变量(非空)→ 缺省 exe 同目录
  `config.toml`(三件套同目录分发契约)。
- **Schema V1**(TOML):顶层 `default_profile = "<名>"`;`[[profiles]]`
  数组,每条 `name`(必)/`commandline`(必,引号包带空格路径)/
  `starting_directory`(可选,空 = 继承宿主)。`ctrl_c_mode` V1 不进
  schema(两轨 Ctrl+C 策略不同源,PLAN-025 §10.3 DEBT)。
- **降级语义(G6)**:文件缺席/解析失败 = 空 profile 集(行为等价无
  配置现状,spawn 回落 COMSPEC/cmd 兜底,不崩);单条 profile 字段
  非法 = 跳过该条 + stderr 告警;重名 = 后者覆盖 + 告警;
  `default_profile` 未命中名 = 忽略(不回落首个)。
- **解析优先级**:显式参数(`--shell`) > `--profile` > 配置文件
  `default_profile` > 轨道缺省(app 轨 COMSPEC 兜底;CLI 维持既有
  pwsh 兼容缺省)。`--profile` 未命中 = 报错退出 1(信息含配置搜索
  路径)。
- **spawn 投递**:profile → `(program, argv, cwd)` 三元组
  (`Config::resolve` 拆分 commandline)→ 既有 SpawnSpec
  (`engine_spawn_ex` / `PtySession::spawn_in`)生效,引擎层零改动。
- **消费面**:back registry Init 期一次装载(term.rs sidecar
  `config_*` 函数族);HTTP 契约 `/api/term/profiles`
  ("name|commandline|cwd" 记录数组)与 `/api/term/profile-names`;
  vm/desktop 臂同契约后补(PLAN-025 非目标,§10.2)。

## 验收与回归

- 桌面门三面:整窗 terminal 形态 + 几何随动(快照 cols/rows ≠ 模型
  缺省 100×30)+ 色彩契约(整屏零 RGB(128,0,0);背景 #060709)。
  证据:`docs/plans/evidence/017/desktop-terminal-live-r2.png`
  (2026-09-14 R2 门,验收通道 AUTOUI_ACCEPTANCE=1)。
- 独立入口:`auto build -r rust` Finished 零错、exe 可运行
  (2026-09-14 基线复现,post-627 master CLI)。
- 单入口断言:`at-app/` 目录不存在;`git grep at-app` 功能残留为零
  (命中仅出处注记/DEBTS #14③ 冻结 oracle 字符串/史志文档——
  注记为 §5.1/5.2 git mv 保历史+并合表的计划自身要求,非残留)。

## 教训注记(防复发)

长驻桌面进程跨引擎 DLL 重建会保留**陈旧内存映像**(Windows 文件
替换不溯及已加载模块)——016 红底曾在门证据中以 RGB(128,0,0) 复现,
根因即此。桌面门取证前必须确认宿主进程启动时间晚于引擎 DLL 构建
时间,或重启桌面进程后再取证。
