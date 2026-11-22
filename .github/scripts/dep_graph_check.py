# -*- coding: utf-8 -*-
"""依赖方向检查（Stage 1 T1，债务 #15 约束执行；审计3 E-1/E-2 加固）。
规则见 docs/architecture/crate-authority-list.md：
 1. sim-core 仅可依赖基础设施 crate（sim-codec、sim-error），不得依赖其他 crate；
 2. 权威 crate 不依赖非权威 crate；
 3. 任何 crate 不依赖 sim-view；
 4. crates/ 不依赖 spikes/；
 5. sim-test-support 只出现在 dev-dependencies。
加固（审计3）：权威名单全覆盖+未分类成员 fail-closed；依赖解析覆盖改名依赖
（package 键）、build-dependencies、target.<cfg>.<section>、workspace 依赖转发；
`--self-test` 用合成 manifest 自测各违规检出。
exit 0 = 全部通过（或自测通过）。
"""
import sys, os, tomllib

sys.stdout.reconfigure(encoding='utf-8', errors='replace')
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

AUTHORITATIVE = {'sim-core', 'sim-agent', 'sim-network', 'sim-pathfinding',
                 'sim-traffic', 'sim-economy', 'sim-error', 'sim-codec',
                 'sim-digest', 'sim-rng', 'simscalar', 'sim-save',
                 'sim-parallel', 'sim-replay', 'sim-world'}
NON_AUTHORITATIVE = {'sim-view', 'sim-test-support', 'dep-verify'}
SPIKES = {'simscalar-spike', 'rng-spike', 'network-tx-spike', 'save-restore-spike'}
KNOWN = AUTHORITATIVE | NON_AUTHORITATIVE | SPIKES

SECTIONS = ('dependencies', 'dev-dependencies', 'build-dependencies')


def dep_packages(data, section):
    """收集一个依赖节的**真实包名**（改名依赖取 package 键，E-2 绕过向量）。"""
    out = set()
    for key, val in data.get(section, {}).items():
        if isinstance(val, dict) and 'package' in val:
            out.add(val['package'])
        else:
            out.add(key)
    return out


def all_deps(data):
    """运行时依赖（含 build-deps 与 target.<cfg>.<section>——E-2 绕过向量）。"""
    out = set()
    for sec in SECTIONS:
        out |= dep_packages(data, sec)
    for cfg, sections in data.get('target', {}).items():
        for sec in SECTIONS:
            out |= dep_packages(sections, sec)
    return out


def dev_deps(data):
    out = dep_packages(data, 'dev-dependencies')
    for cfg, sections in data.get('target', {}).items():
        out |= dep_packages(sections, 'dev-dependencies')
    return out


def check_member(name, data, errors):
    runtime = all_deps(data)
    dev = dev_deps(data)
    non_dev = runtime - dev  # dev-only 依赖不算运行时依赖

    if name == 'sim-core' and non_dev - {'sim-codec', 'sim-error'}:
        errors.append(f'规则1：sim-core 仅可依赖 sim-codec/sim-error，实际 {sorted(non_dev)}')
    if name in AUTHORITATIVE:
        bad = non_dev & NON_AUTHORITATIVE
        if bad:
            errors.append(f'规则2：权威 {name} 运行时依赖非权威 {sorted(bad)}')
    if name != 'sim-view' and 'sim-view' in runtime | dev:
        errors.append(f'规则3：{name} 不得依赖 sim-view')
    if not name.endswith('-spike') and (runtime | dev) & SPIKES:
        errors.append(f'规则4：{name} 不得依赖 spikes（{sorted((runtime | dev) & SPIKES)}）')
    if (dep_packages(data, 'dependencies') | dep_packages(data, 'build-dependencies')) & {'sim-test-support'}:
        errors.append(f'规则5：{name} 的 sim-test-support 必须仅在 dev-dependencies')
    for cfg, sections in data.get('target', {}).items():
        if dep_packages(sections, 'build-dependencies') & {'sim-test-support'}:
            errors.append(f'规则5：{name} 的 sim-test-support 不得进 build-dependencies（target.{cfg}）')


def members():
    with open(os.path.join(ROOT, 'Cargo.toml'), 'rb') as f:
        ws = tomllib.load(f)['workspace']
    for rel in ws['members']:
        manifest = os.path.join(ROOT, rel, 'Cargo.toml')
        if not os.path.isfile(manifest):
            yield rel.replace('/', '-'), None
            continue
        with open(manifest, 'rb') as f:
            data = tomllib.load(f)
        yield data['package']['name'], data


def self_test():
    """合成 manifest 自测（审计3 依赖检查器自测）：违规须全部检出、干净须全过。"""
    fails = []

    def expect(name, data, want_sub):
        errs = []
        check_member(name, data, errs)
        if want_sub:
            if not any(want_sub in e for e in errs):
                fails.append(f'self-test：{name}/{want_sub} 未检出（errs={errs}）')
        elif errs:
            fails.append(f'self-test：{name} 干净却报 {errs}')

    # 干净的权威 crate
    expect('sim-core', {'dependencies': {'sim-codec': '1'}}, None)
    # 规则 1
    expect('sim-core', {'dependencies': {'sim-parallel': '1'}}, '规则1')
    # 规则 2（含改名依赖绕过：键为别名、package 为真名）
    expect('sim-traffic', {'dependencies': {'view_alias': {'package': 'sim-view', 'version': '1'}}}, '规则3')
    expect('sim-traffic', {'build-dependencies': {'sim-test-support': '1'}}, '规则5')
    expect('sim-agent', {'target': {'cfg(unix)': {'dependencies': {'alias': {'package': 'sim-test-support', 'version': '1'}}}}}, '规则2')
    # 规则 3/4/5
    expect('sim-economy', {'dependencies': {'sim-view': '1'}}, '规则3')
    expect('sim-economy', {'dependencies': {'rng-spike': '1'}}, '规则4')
    expect('sim-save', {'dependencies': {'sim-test-support': '1'}}, '规则5')

    if fails:
        print('dep_graph_check 自测 FAILED：')
        for f in fails:
            print(' -', f)
        return 1
    print('dep_graph_check 自测 PASS（7 用例）')
    return 0


if '--self-test' in sys.argv:
    sys.exit(self_test())

errors = []
for name, data in members():
    if data is None:
        errors.append(f'fail-closed：{name} 的 Cargo.toml 不存在')
        continue
    if name not in KNOWN:
        errors.append(f'fail-closed：成员 {name} 未在权威清单分类（crate-authority-list.md 同步后方可加入）')
        continue
    check_member(name, data, errors)

if errors:
    print('依赖方向检查 FAILED：')
    for e in errors:
        print(' -', e)
    sys.exit(1)
print('依赖方向检查 PASS（规则 1-5；名单 15 权威+3 非权威+4 spike；fail-closed 启用）')
