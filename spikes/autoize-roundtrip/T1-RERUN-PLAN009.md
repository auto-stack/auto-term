# T1 spike 重跑记录(PLAN-009 P0 / 006 S2 归档diff)

日期:2026-09-06 · 转译器:auto-lang worktree `.wt/auto-009/auto-lang`
(auto-term-dev 分支,F1 修复后,in-process `transpile_rust` API)

## 方法说明

006 时代用 CLI(`auto trans`)驱动;本次 CLI 面 `auto trans` 在 master
f2ae1cb29 起挂起(60 行样本 3 分钟无返回,新旧 exe、主检出与 worktree
一致,存量回归与 PLAN-009 无关,已记 PLAN-009 待澄清③)。改用与 CLI
同源的 `transpile_rust`(同一 RustTrans::trans + post_process 管线)重跑,
另以临时测试生成产物后已删除,工作区无残留。

## 产物

- `sample.t1.a2r.rs`(新)/ `sample.a2r.rs`(006 归档)
- `sample2.t1.a2r.rs`(新)/ `sample2.a2r.rs`(006 归档)

## diff(新 vs 006)

- **sample**:仅 1 行——TermSession 自动派生
  `#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]` →
  `#[derive(Clone, Debug)]`。这正是 F1 修复本体:GridSize 显式派生面
  (`Clone, Copy, Debug`)缺比较 trait,容器自动派生经交集收窄。
- **sample2**(全部类型显式派生):**字节级零差异**——显式透传行为未被
  扰动。

## rustc 实编证据

```
rustc --edition 2021 --crate-name spike_sample --crate-type bin \
  --emit=metadata -A warnings sample.t1.a2r.rs   # exit=0
rustc ... spike_sample2 ... sample2.t1.a2r.rs     # exit=0
```

006 S2 结论"原样本(GridSize 显式派生、TermSession 走自动派生)编译
失败 E0369"——**现已编译通过**,F1 修复在真实 spike 样本上闭环。

## NOTES.md 发现清单回填

| # | 006 状态 | T1 后 |
|---|---|---|
| F1 | E0369 组合缺陷 | **已修**(转译器侧,自动派生×显式派生面交集;链式递归+upgrade 防回扩) |
| F2 | int→i64 变形 | 不变(非本相位范围) |
| F3 | 字段强制 pub | 不变 |
| F4 | is 冗余出口 | 不变 |
| F5 | 杂点噪声 | 不变 |
| F6 | CLI 面不符合文档 | 恶化:CLI `trans` 现挂起(见上);in-process 面正常 |
