#!/usr/bin/env python
# ledger_p021_projection.py — PLAN-021 六节投影入 .autoos/specs.json(offline 原子写)。
# 模板 = P019/P020 条目;归档路径按收据顺序(ledger_refreshed 时归档未发生,
# file 字段按 P019/P020 先例直接引用最终 archived 路径)。
import json, os, shutil

P = r'D:\autostack\auto-term\.autoos\specs.json'
ARC = 'docs/plans/archived/021-ffi-sustained-concurrency-stability.md'
SPEC = 'docs/specs/engine-ffi-color-encoding.md'
REL = ['PLAN-021']

def item(i, section, title, content, file_):
    return {'id': f'P021-{i}', 'section': section, 'title': title, 'content': content,
            'file': file_, 'related': REL, 'status': 'published',
            'created': '2026-09-17', 'last_modified': '2026-09-18'}

TITLE = 'VM 视图/事件管线稳定性专项(FFI 堆破坏 + 指针事件链 + rust 轨 codegen 整备)'
items = [
    item(1, 'reports', f'{TITLE} — 变更摘要',
         '双线专项:①线A FFI sustained 堆破坏(vue 页面轮询 0xc0000374,020 复审 P1)——'
         '根因引擎 DLL 导出面裸 &mut 别名零线程承诺,根修 = FFI 边界每柄串行化;'
         '②线B VM 指针事件链 press 全死(020 移交)——根因 iced 0.14 mouse_area '
         '自身 0×0 bounds,根修 = 内容侧类型化镜像(双臂);Phase2 = rust 轨 codegen '
         '整备(db 进程内吸收优先 + 返回类型化 + 阻塞标量 POST)。裁定改判:自绘滚动条'
         '打磨/SD-02/020 AC-04 移交官方滚动条计划;T-10 移交 auto-lang 缺陷计划。',
         ARC),
    item(2, 'goals', f'{TITLE} — 目标',
         'G1 仪器定因堆破坏(DLL 导出面)/G2 修复后 3×≥10min 浸泡零崩/G3 复现器+金样'
         '防回归/G4 指针事件链修复实点交付;Phase2:rust 轨从零可编译 + 部署态重建;'
         '滚动条/分屏打磨与官方滚动条(虚拟滚动)计划合并推进(用户裁定)。',
         ARC),
    item(3, 'architecture', f'{TITLE} — 架构方案',
         '引擎 FFI 并发契约 = FFI 边界每柄串行化(ENGINE_LOCKS,含读类导出;锁序 = 柄锁'
         '→ring 单向;导出不重入)——规范权威源 = docs/specs/engine-ffi-color-encoding.md '
         '§并发契约(SD-01)。指针事件链:iced 0.14 mouse_area layout 直通子件、事件面 '
         'is_over(自身 bounds) 早退——交互 wrapper 必须使命中区=可视区(内容侧类型化镜像)。'
         'rust 轨 codegen:api:rust 单 exe 优先 db.at 进程内吸收(017 merged 承诺),'
         'wrapper 按 api.at 声明类型化。', SPEC),
    item(4, 'designs', f'{TITLE} — 详细设计',
         'D1 无头复现器 scripts/repro/vue_crash_repro.ps1(WER Event-1000 判崩);'
         'D2 仪器(glue FfiGuard 五包点 + DLL DllGuard 变异类,AUTO_FFI_TRACE 门);'
         'D3 每柄串行化(ENGINE_LOCKS,Box::leak 单次锁);D5 mouse_area 双臂修复'
         '(renderer.rs VM 动态臂 + IntoIcedElement 臂)+ AUTO_MA_DBG;T-09 codegen 三处'
         '(generate_api_client 吸收优先/split 返回类型化/POST 标量阻塞反序列化)。',
         ARC),
    item(5, 'tests', f'{TITLE} — 测试设计',
         '金样 autoterm-core::ffi_concurrency_serialization(6 线程同柄混合流;去锁红相 '
         '= 进程级 0xc0000005);复现器浸泡(修复前 3/3 崩 65/65/140s,修复后 3×≥10min '
         '零崩);ma_press 无头回归钉×2(块中心必发 on_click/条外必静默);全量门 '
         'workspace 78/0;rust 轨从零构建 0 错;实机三查(shell/Tab/内容)。', ARC),
    item(6, 'reviews', f'{TITLE} — 验收标准',
         'AC-01..06 达成(见 §8 证据);AC-07/AC-08 两轮用户裁定改判移交官方滚动条计划;'
         '复审 pass(rev3:auto-term 526ff51 / lang-021 6c6950f75 / lang-021-p2 '
         'ca1d1260e+ef5e0538c)。移交清单:官方滚动条(虚拟滚动)计划 = 自绘滚动条缺陷簇'
         '(thumb 尺寸/对齐/比例/跟随)+ wheel 半屏钳位 + rust 轨分屏渲染 + SD-02 + '
         '020 AC-04 + T-06 换装;auto-lang 缺陷计划 = T-10 dev 跑法 api 委托断供 + '
         'codegen 整备未尽面(merged CRUD 原型退役)。遗留:F2 vue.rs 环境性失败'
         '(auto-os mirror 缺失,与 021 无涉)。', ARC),
]

tmp = P + '.tmp'
shutil.copyfile(P, tmp + '.bak')
d = json.load(open(P, encoding='utf-8'))
for sec_name in d['sections']:
    lst = d['sections'][sec_name]
    lst[:] = [i for i in lst if not str(i.get('id', '')).startswith('P021')]
for it in items:
    d['sections'][it['section']].append(it)
with open(tmp, 'w', encoding='utf-8', newline='\n') as f:
    json.dump(d, f, ensure_ascii=False, indent=1)
# validate temp before atomic replace
json.load(open(tmp, encoding='utf-8'))
os.replace(tmp, P)
print('ledger P021-1..6 projected; sections:', {k: len(v) for k, v in d['sections'].items()})
