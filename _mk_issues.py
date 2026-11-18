# -*- coding: utf-8 -*-
"""创建两个决策 issue（决策1 状态口径 / 决策2 Rust 基线）。"""
import subprocess, sys

sys.stdout.reconfigure(encoding='utf-8', errors='replace')
GH = r'C:\Program Files\GitHub CLI\gh.exe'
REPO = '2222BUG/CitySim-rs'
BRIEF = 'https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/docs/architecture/review/DECISION-REQUEST.md'

issues = [
    ('[决策1] ADR 状态口径：Accepted（附条件） vs 严格 Proposed',
     '请拍板 10 个 ADR 的状态记法（详见决策请求文档：' + BRIEF + '）\n\n'
     '- **A（维护者推荐）**：Accepted（附条件）——裁决=接受，验证条件作为 Stage 1/2A 开工门禁在 decision-log 追踪\n'
     '- **B（严格口径，评审 F 主张）**：条件满足前保持 Proposed，条件逐项满足后一次性转 Accepted\n\n'
     '影响面：10 个 ADR 状态列、decision-log 裁决表、Stage-0-Exit-CHECKLIST 表述；不改变任何技术决策内容。\n\n'
     '回传格式：\n```\n决策 1：A / B\n（理由可选）\n```'),
    ('[决策2] Rust 工具链基线：1.89（Bevy MSRV） vs 1.100.0（实测）',
     '请拍板 Rust 工具链基线（债务 #22，详见决策请求文档：' + BRIEF + '）\n\n'
     '- **A**：锁 1.89——与文档 2.1 一致（Bevy 0.18.1 MSRV，Step 8 实测吻合）；ordered-float 钉 5.4.0 成长期约束\n'
     '- **B**：升 1.100.0——实测工具链、CI 主 job 现用；2.1 基线句改写 + MSRV 三区分（文档基线/CI 最低/开发工具链）\n\n'
     '影响面：rust-toolchain.toml、CI 矩阵（msrv-check job）、2.1 文档、债务 #22 关闭方式。\n\n'
     '回传格式：\n```\n决策 2：A / B\n（理由可选）\n```'),
]

for title, body in issues:
    r = subprocess.run([GH, 'issue', 'create', '--repo', REPO, '--title', title, '--body', body],
                       capture_output=True, text=True, encoding='utf-8')
    print(('OK ' + r.stdout.strip()) if r.returncode == 0 else ('FAIL ' + r.stderr.strip()))
