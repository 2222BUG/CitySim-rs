# -*- coding: utf-8 -*-
"""依赖方向检查（Stage 1 T1，债务 #15 约束执行）。
规则见 docs/architecture/crate-authority-list.md：
 1. sim-core 无依赖；
 2. 权威 crate 不依赖非权威 crate；
 3. 任何 crate 不依赖 sim-view；
 4. crates/ 不依赖 spikes/；
 5. sim-test-support 只出现在 dev-dependencies。
exit 0 = 全部通过。
"""
import sys, os, tomllib

sys.stdout.reconfigure(encoding='utf-8', errors='replace')
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

AUTHORITATIVE = {'sim-core', 'sim-agent', 'sim-network', 'sim-pathfinding',
                 'sim-traffic', 'sim-economy', 'sim-error', 'sim-codec',
                 'sim-digest', 'sim-rng', 'simscalar', 'sim-save'}
NON_AUTHORITATIVE = {'sim-view', 'sim-test-support', 'dep-verify'}
SPIKES = {'simscalar-spike', 'rng-spike', 'network-tx-spike', 'save-restore-spike'}

errors = []

def members():
    with open(os.path.join(ROOT, 'Cargo.toml'), 'rb') as f:
        ws = tomllib.load(f)['workspace']
    for rel in ws['members']:
        manifest = os.path.join(ROOT, rel, 'Cargo.toml')
        with open(manifest, 'rb') as f:
            data = tomllib.load(f)
        yield data['package']['name'], data

for name, data in members():
    deps = set(data.get('dependencies', {}).keys())
    dev_deps = set(data.get('dev-dependencies', {}).keys())

    if name == 'sim-core' and deps:
        errors.append(f'规则1：sim-core 不得有依赖，实际 {sorted(deps)}')
    if name in AUTHORITATIVE:
        bad = (deps | dev_deps) & NON_AUTHORITATIVE
        if bad:
            errors.append(f'规则2：权威 {name} 依赖非权威 {sorted(bad)}')
    if name != 'sim-view' and 'sim-view' in deps | dev_deps:
        errors.append(f'规则3：{name} 不得依赖 sim-view')
    if not name.endswith('-spike') and (deps | dev_deps) & SPIKES:
        errors.append(f'规则4：{name} 不得依赖 spikes（{sorted((deps | dev_deps) & SPIKES)}）')
    if 'sim-test-support' in deps:
        errors.append(f'规则5：{name} 的 sim-test-support 必须仅在 dev-dependencies')

if errors:
    print('依赖方向检查 FAILED：')
    for e in errors:
        print(' -', e)
    sys.exit(1)
print('依赖方向检查 PASS（规则 1-5）')
