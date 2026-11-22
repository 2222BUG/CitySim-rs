# STAGE2B3 逐文件 raw 直链清单（替代粘贴渠道）

> 富文本/粘贴渠道会转换标记并切片——raw 链接是唯一保真渠道（行号完整、无转换、无截断）。
> 每条链接打开即该文件全文；行号请以 raw 端为准。

- `crates/sim-parallel/src/lib.rs`（179 行 / 6397 B）: https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/crates/sim-parallel/src/lib.rs
- `crates/sim-parallel/tests/thread_independence.rs`（135 行 / 4608 B）: https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/crates/sim-parallel/tests/thread_independence.rs
- `crates/sim-pathfinding/src/lib.rs`（597 行 / 25901 B）: https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/crates/sim-pathfinding/src/lib.rs
- `crates/sim-network/src/lib.rs`（842 行 / 29849 B）: https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/crates/sim-network/src/lib.rs
- `crates/sim-agent/src/lib.rs`（549 行 / 18398 B）: https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/crates/sim-agent/src/lib.rs
- `crates/sim-replay/src/lib.rs`（534 行 / 16312 B）: https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/crates/sim-replay/src/lib.rs
- `crates/sim-economy/src/lib.rs`（200 行 / 7998 B）: https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/crates/sim-economy/src/lib.rs
- `crates/sim-world/src/lib.rs`（105 行 / 3652 B）: https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/crates/sim-world/src/lib.rs
- `crates/sim-traffic/src/lib.rs`（1005 行 / 45231 B）: https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/crates/sim-traffic/src/lib.rs
- `crates/sim-traffic/examples/mfd_prime.rs`（460 行 / 19205 B）: https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/crates/sim-traffic/examples/mfd_prime.rs
- `crates/sim-test-support/src/chaos.rs`（65 行 / 2651 B）: https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/crates/sim-test-support/src/chaos.rs
- `crates/sim-codec/src/lib.rs`（153 行 / 4960 B）: https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/crates/sim-codec/src/lib.rs
- `crates/sim-digest/src/lib.rs`（197 行 / 7511 B）: https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/crates/sim-digest/src/lib.rs
- `crates/sim-error/src/lib.rs`（209 行 / 6882 B）: https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/crates/sim-error/src/lib.rs
- `.github/workflows/ci.yml`（116 行 / 3767 B）: https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/.github/workflows/ci.yml
- `.github/scripts/dep_graph_check.py`（56 行 / 2514 B）: https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/.github/scripts/dep_graph_check.py
- `rust-toolchain.toml`（7 行 / 390 B）: https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/rust-toolchain.toml
- `docs/architecture/review/AI-REVIEW-BRIEF-STAGE2B3.md`（51 行 / 4562 B）: https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/docs/architecture/review/AI-REVIEW-BRIEF-STAGE2B3.md
- `docs/architecture/adr/ADR-NEW-25-MFD交通模型与验证方案.md`（257 行 / 21718 B）: https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/docs/architecture/adr/ADR-NEW-25-MFD%E4%BA%A4%E9%80%9A%E6%A8%A1%E5%9E%8B%E4%B8%8E%E9%AA%8C%E8%AF%81%E6%96%B9%E6%A1%88.md
- `docs/benchmarks/workspace-test-baseline.txt`（517 行 / 21376 B）: https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/docs/benchmarks/workspace-test-baseline.txt
- `docs/benchmarks/globaldigest-golden-vectors.txt`（31 行 / 1479 B）: https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/docs/benchmarks/globaldigest-golden-vectors.txt

## 证据补件

- 测试基线（F-3）: docs/benchmarks/workspace-test-baseline.txt（本次 cargo test --workspace 全输出含 ignored/filtered 计数）
  https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/docs/benchmarks/workspace-test-baseline.txt
- 预注册时序（C-3）: git log --oneline 线性序列 7160955（Z3 预注册规格入仓）→ aa36d42（P1 修测原语）→ 78ed6cb（P2 fixture+取数+P4）→ 1db87d2（S4）→ 19e1f3a（材料 v2）。预注册 commit 严格早于 fixture 实装与取数，可用 `git log --format=%H%x20%ci docs/architecture/adr/ADR-NEW-25-MFD交通模型与验证方案.md` 复核。
- F-1 答复: 源码 .rs 不含任何 HTML 标签——`<strong>` 是粘贴渠道富文本转换残留，raw 链接可对照验证（mfd_prime.rs 头注释纯文本）。同一原因导致"5 段残缺"：粘贴渠道对长文本任意切片，本清单起改 raw 直投。
- C-1 答复: 头注释"三段独立"指 fit/holdout/back-half 三个种子段；back-half 段内含 fit(+16..24)/hold(+24..32) 两子段共 4 个数组槽——措辞不严谨属实，以 mfd_prime.rs 内 SEED_BASE 派生代码为准。
- C-4: 密度数组 FIT=[1,2,3,4,6,8,12,16,24,32,48,64,96,128,192,256,384,512]、HOLDOUT=[2,3,5,7,9,13,18,26,36,52,72,104,144,208,288,416,576,768]（mfd_prime.rs 源码可核）。
- C-2: steady_window 析取口径（≤2‰ 或 ≤15 辆）确为取宽，是否记录生效口径待核——记为待核实发现，源码见 sim-traffic/src/lib.rs。
