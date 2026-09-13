# evidence/014 — 退化几何内存爆炸三案取证(2026-09-13)

| 文件 | 内容 |
|---|---|
| at-leak-191353.dmp | 19:13 案挂起现场 mini dump(观察哨自动 at-suspend + at-dump;cdb 分析见 at-dump-stacks.txt) |
| at-dump-stacks.txt | cdb `~*k 25` 全栈导出——主线程即凶器:`Tick→term_apply_resize→autoterm_engine_resize→Term::resize→Grid::shrink_columns→Vec::insert→RtlReAllocateHeap` |
| autoterm-run.log | 17:09 案(run1):心跳至冻结,commit 10.4GB,Rust 堆 7MB |
| autoterm-run2.log | 18:05 案(run2):仅 7 键,pending=0 全程,#10-13 resize 1x1↔136x48 翻转后爆 |
| autoterm-run3.log | 19:13 案(run3):ash+键入+最小化;mem_guard 4032MB 冻结报告 |
| autoterm-run4.log | **回归 PASS**:修复版完整配方,resize 计数恒 1,恢复无缝 |
| autoterm-proc-watch2.log | 外部观察哨曲线(17:00-18:06,含 18:05 爆发段 243MB→17.6GB 与系统级联) |
| autoterm-proc-watch3.log | 观察哨 v3(自动挂起+dump 武装;19:13 AUTO-SUSPEND 行在案) |
| auto-term-mem-report.txt | GuardAlloc 冻结报告(run3 末次):live=0.0MB vs commit 4032MB——DLL 堆盲区铁证 |

原始环境:%TEMP% 易失原件与此同源;分析工具链见 crates/autoterm-core/src/bin/。
复现配方自动化脚本:%TEMP%/at-ritual2.ps1(接受任意 pid)。
