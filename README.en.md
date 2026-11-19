# CitySim-rs

**An open-source city simulation engine built with Rust + Bevy — deterministic-first, documentation-driven.**

[![CI](https://github.com/2222BUG/CitySim-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/2222BUG/CitySim-rs/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Rust](https://img.shields.io/badge/Rust-1.100.0-orange.svg)](rust-toolchain.toml)
![Tests](https://img.shields.io/badge/tests-124%20passed-brightgreen.svg)

> English | [简体中文](README.md)

**What makes it different**: identical digests across two runs with the same seed, and **strong read-back equivalence** — save at tick N, load, run to tick M, and the result is bit-identical to running straight through. Zero floating point, zero wall-clock in authoritative state. These are not aspirations; they are **tested commitments**. The whole design is driven by ADRs + a decision-debt ledger + strict evidence discipline, with verdicts from six independent AI reviews — fully traceable.

## Status

| Layer | State |
| --- | --- |
| Design | [Technical Spec V1.9.3](docs/architecture/CitySim-rs-Technical-Spec-V1.9.3.md) (Chinese, the single current master document); 10 ADRs (NEW-14~23) formally adjudicated (Accepted, with conditions); [26 decision debts](docs/architecture/decisions/decision-log.md) tracked |
| Stage 0 | ✅ Closed (2026-11-19): parameter baseline decided, 5 spikes verified, dependency verification, CI skeleton |
| Stage 1 | ✅ Complete (2026-11-19): layered crate skeleton, SimScalar, RNG protocol, GlobalDigest, save/load, error taxonomy |
| Stage 2 | Not started (2A road network / A*, 2B parallel determinism, 2C traffic & degradation) |
| The game itself | Not yet — this is the engine foundation and verifiable docs. Design help welcome. |

## Quick start

```bash
git clone https://github.com/2222BUG/CitySim-rs.git
cd CitySim-rs
cargo test --workspace        # 124 tests green (Rust 1.100.0, pinned via rust-toolchain.toml)
cargo test -p sim-save        # strong read-back equivalence demo
```

No external dependencies required. `cargo +1.89 check` (the Bevy 0.18.1 MSRV reference line) must also pass (CI gate).

## How to contribute

- 🟢 **[good first issue](https://github.com/2222BUG/CitySim-rs/labels/good%20first%20issue)** — small, scoped tasks: statistics tests, benchmarks, translation, design RFCs
- 💬 **ADR reviews & design discussions** — [Issues](https://github.com/2222BUG/CitySim-rs/issues) & [Discussions](https://github.com/2222BUG/CitySim-rs/discussions): many decisions are **still open**; commenting *is* contributing
- 📐 **Architecture design** — 26 decision debts (agent state machine, replay schema, LOD …) are up for grabs, see [decision-log](docs/architecture/decisions/decision-log.md)
- See [CONTRIBUTING.md](CONTRIBUTING.md) (Chinese)

Note: most docs are currently in Chinese — translation help is very welcome (and a great first contribution).

## Documentation map

| Want to know | Read |
| --- | --- |
| Overall design | [Technical Spec V1.9.3](docs/architecture/CitySim-rs-Technical-Spec-V1.9.3.md) |
| How decisions are made, what's still open | [decision-log](docs/architecture/decisions/decision-log.md) |
| Individual architecture decisions | [`docs/architecture/adr/`](docs/architecture/adr/) |
| Evidence for the determinism claims | [determinism-evidence.md](docs/architecture/determinism-evidence.md), [benchmarks/](docs/benchmarks/) |
| Join a review | [AI-REVIEW-BRIEF](docs/architecture/review/AI-REVIEW-BRIEF.md) |

## Repository layout

```
crates/     # production crates (layered per spec §2.2; see crate-authority-list.md)
spikes/     # Stage 0 verification prototypes (kept reproducible)
docs/       # spec, ADRs, decision log, task lists, review material
```

Dependency direction is CI-enforced (`dep_graph_check`): the simulation core never depends on UI, authoritative layers never depend on tooling, and spikes never leak into production dependencies.

## License

Dual-licensed under MIT OR Apache-2.0, at your option. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).
