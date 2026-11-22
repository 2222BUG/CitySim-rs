# STAGE2B3-BUNDLE PART 01（本卷文件：docs/architecture/review/AI-REVIEW-BRIEF-STAGE2B3.md, crates/sim-parallel/src/lib.rs, crates/sim-parallel/tests/thread_independence.rs）
<!-- commit 0bd384ab7874d9180273c7e0c87106b6e7d723e3；与 ZIP 同源；粘贴时请整卷一条消息发送 -->


===== FILE: docs/architecture/review/AI-REVIEW-BRIEF-STAGE2B3.md =====

# CitySim-rs 审计邀请：Stage 2B 并行确定性 + Stage 3 T0–T5 + ②′ 修测原语

- 日期：2026-11-22
- 类型：**债务/工程质量闭合审计**（对尚未经外部审计的工程面逐项裁决）
- 回传渠道：issue #28（https://github.com/2222BUG/CitySim-rs/issues/28）
- 署名：**推荐**具体型号/版本（如 DeepSeek-V4.1、qwen4-max、MiniMax-M3.1-Flash、GPT-6、Claude Sonnet 5.5）；泛称同样计票；完全未署名不计票
- 约定：只依据材料原文判断，材料没覆盖的点标"未见证据"；**拒绝和延后是合法裁决**；每轮至少一家做代码级审查（行号级证据）

## 背景（30 秒）

CitySim-rs 是确定性城市模拟（同种子逐 tick 可复现、强读档等价）。此前审计史：
五项修订 6 家 → ADR-24 第二轮 8 家 → 债务闭合审计 7 家 → 决策4/5/6（#31 三轮 10-11 家）。
**本包审计对象=从未送审的工程面**：Stage 2B 并行确定性、Stage 3 新增六交付（T0-T5）、
以及 #31 整改后的修测原语。设计面（ADR-14~26）已多轮评审，不在本轮范围。

## 审计范围（六项，每项单独裁决）

| 项 | 对象 | 关键文件 | 重点问题 |
|---|---|---|---|
| A | **Stage 2B 并行确定性**（从没审计） | crates/sim-parallel/{src/lib.rs,tests/thread_independence.rs}、sim-pathfinding TickBudget 段、docs/benchmarks/Stage2B-concurrent-stress.txt、Stage-2B-Exit-CHECKLIST.md | run_parallel 分片/归并是否真确定性；chaos 注入是否满足"测试自身不违 2.9"（忙等 vs sleep）；TickBudget 配额让步与 #27 判据隔离；1/2/8 线程摘要一致证据是否充分 |
| B | **Stage 3 T0-T5**（2026-11-22 刚完成，零审计） | crates/{sim-network TxRetryQueue 段, sim-agent, sim-replay, sim-economy, sim-world}、docs/architecture/{rfc-agent-state-machine.md, replay-schema.md, rfc-lod-draft.md, Stage-3-Task-List.md} | TxRetryQueue 全序/上限/诊断语义；AgentLifecycle 转移表完备性与受阻自动退役；回放 schema canonical 编码/版本 bump/回放等价测试真实含金量；经济 10^12 防护边界；WorldId 高水位口径 |
| C | **②′ 修测原语**（决策5 审计后新增/重写） | crates/sim-traffic/src/lib.rs（criterion_aligned_loss/point_resolution_ok/steady_window/fit_*_detailed 判据损失）、examples/mfd_prime.rs | 判据归一化损失实现是否真与判据同构；分辨率下限数学；稳态双口径；mfd_prime 是否严守预注册（种子段/族冻结/一次锁定/自动落①焊死） |
| D | **基础 crate 尾巴** | crates/{sim-test-support, sim-error, sim-codec, sim-digest}/src/lib.rs | 六模块契约测试含金量；错误码登记完备；canonical/BLAKE3 黄金向量是否仍匹配 |
| E | **CI 与依赖方向** | .github/workflows/ci.yml、.github/scripts/dep_graph_check.py、rust-toolchain.toml | 矩阵是否覆盖全部 21 crate；MSRV job 与 rust-toolchain 一致性；dep 规则 1-5 是否可被绕过 |
| F | **口径一致性** | decision-log.md、Stage-3-Exit-CHECKLIST.md、README.md | "6/6 达成"是否与证据一致；债务 #7/#10/#14 收口口径是否准确；状态词表 0.2 合规 |

## 请重点回答的问题

1. **A 项**：sim-parallel 的确定性声明在 8 线程+chaos 下是否有未覆盖的竞态/未定义行为路径（如排序键碰撞、分片边界、panic 传播）？
2. **B 项**：T0-T5 六交付中哪些是"测试通过但契约有缺口"？（例：回放等价测试的玩具模拟 vs 真实权威循环的差距；AgentLifecycle 与 EC-01/ADR-17 受阻语义的衔接）
3. **C 项**：修测原语是否还有"估计目标≠判据"的残余形态？mfd_prime 的归因分解表是否可信？
4. **D/E 项**：基础 crate 与 CI 是否存在会掩盖回归的薄弱点？
5. **F 项**：记录与证据是否一致（不一致逐条列出）？

## 裁决格式（请原样返回）

```
## Stage 2B/3 审计裁决记录
- 署名（推荐具体型号）：
- 逐项裁决：A= / B= / C= / D= / E= / F=（可入账 / 有条件入账 / 需重做）
- 各项理由（1-3 句）与验证条件/修改要求（逐条）：
- 代码级发现（行号级，如有）：
- 新增风险（如有）：
- 总体：可入账 / 有条件入账 / 需重做
```

## 材料清单（多通道）

附件/单文本包/Release 内含：A-F 全部关键文件源码+测试、相关 benchmarks 数据
（Stage2B-concurrent-stress.txt 等）、退出核对表、MANIFEST（口径说明）。
仓库：https://github.com/2222BUG/CitySim-rs 。


===== FILE: crates/sim-parallel/src/lib.rs =====

//! # sim-parallel —— 确定性并行执行框架（2.9 条 6）
//!
//! 契约（主文档 2.9 第 6 条 / ADR-NEW-15）：并行任务**只读 tick 开始快照**，
//! 产出结果按**稳定键全序**归并后**串行提交**。线程数、调度顺序、注入延迟
//! 只影响耗时，不影响提交序列与最终摘要（线程数无关性）。
//!
//! 稳定键 = `(phase_id, task_key)` 复合全序（ADR-NEW-15 P1-02 口径：
//! 全序提交与系统执行顺序分开定义；键相等由调用方保证不发生）。

#![forbid(unsafe_code)]

/// 权威 crate：2.9 契约适用（clippy 分层见 crate-authority-list.md）。
pub fn authoritative() -> bool {
    true
}

/// 稳定提交全序键：(阶段 ID, 任务键)。
pub type StableKey = (u32, u64);

/// 并行任务：对共享快照的纯计算（`func` 不得捕获外部状态）。
pub struct Task<S: ?Sized, In, Out> {
    pub key: StableKey,
    pub input: In,
    pub func: fn(&S, &In) -> Out,
}

/// 执行一批任务并确定性提交。
///
/// 1. 任务按稳定键升序排序（提交全序）；
/// 2. 连续块确定性分片到 `threads` 个工作线程，各线程只读快照并行纯计算；
/// 3. 输出按稳定键归并排序后，在调用线程**串行**调用 `commit(key, out)`。
///
/// `threads <= 1` 走同一代码路径（单分片），保证并行/单线程可对比。
pub fn run_parallel<S: Sync + ?Sized, In: Send, Out: Send>(
    snapshot: &S,
    mut tasks: Vec<Task<S, In, Out>>,
    threads: usize,
    mut commit: impl FnMut(StableKey, Out),
) {
    tasks.sort_by_key(|t| t.key);
    let n_threads = threads.max(1).min(tasks.len().max(1));
    if tasks.is_empty() {
        return;
    }
    let per = tasks.len().div_ceil(n_threads);
    let mut outputs: Vec<(StableKey, Out)> = Vec::with_capacity(tasks.len());
    // 按块转移所有权进工作线程（避免 &[Task] 共享对 In 的 Sync 要求）
    let mut work: Vec<Vec<Task<S, In, Out>>> = Vec::new();
    let mut it = tasks.into_iter();
    loop {
        let chunk: Vec<Task<S, In, Out>> = it.by_ref().take(per).collect();
        if chunk.is_empty() {
            break;
        }
        work.push(chunk);
    }
    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for chunk in work {
            handles.push(scope.spawn(move || {
                chunk
                    .into_iter()
                    .map(|t| (t.key, (t.func)(snapshot, &t.input)))
                    .collect::<Vec<(StableKey, Out)>>()
            }));
        }
        for h in handles {
            outputs.extend(h.join().expect("worker panicked"));
        }
    });
    // 串行提交：与线程数/到达顺序无关的稳定全序
    outputs.sort_by_key(|(k, _)| *k);
    for (k, out) in outputs {
        commit(k, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试快照：只读基值。
    struct Snap {
        base: u64,
    }

    fn mul_task(s: &Snap, i: &u64) -> u64 {
        s.base.saturating_mul(*i).saturating_add(*i)
    }

    fn shuffled_tasks() -> Vec<Task<Snap, u64, u64>> {
        // 乱序构造，键 (phase, key)：(1,3) (1,1) (1,2) (0,9) (0,8)
        let keys: Vec<StableKey> = vec![(1, 3), (1, 1), (1, 2), (0, 9), (0, 8)];
        keys.into_iter()
            .map(|key| Task { key, input: key.1, func: mul_task })
            .collect()
    }

    fn committed(tasks: Vec<Task<Snap, u64, u64>>, threads: usize) -> Vec<(StableKey, u64)> {
        let snap = Snap { base: 7 };
        let mut out = Vec::new();
        run_parallel(&snap, tasks, threads, |k, v| out.push((k, v)));
        out
    }

    #[test]
    fn role_is_pinned() {
        assert!(authoritative());
    }

    /// 退出条件"任务提交顺序有明确契约"：commit 按稳定键全序调用。
    #[test]
    fn commit_order_is_stable_key_order() {
        let seq = committed(shuffled_tasks(), 2);
        let keys: Vec<StableKey> = seq.iter().map(|(k, _)| *k).collect();
        assert_eq!(keys, vec![(0, 8), (0, 9), (1, 1), (1, 2), (1, 3)]);
    }

    /// 提交顺序无关性：任务到达顺序不同，提交序列逐项一致。
    #[test]
    fn submission_order_independence() {
        let a = committed(shuffled_tasks(), 3);
        let mut rev = shuffled_tasks();
        rev.reverse();
        let b = committed(rev, 3);
        assert_eq!(a, b, "到达顺序不得影响提交序列");
    }

    /// 线程数无关性（退出条件 2 的契约级）：1/2/8 线程提交序列一致。
    #[test]
    fn thread_count_independence() {
        let r1 = committed(shuffled_tasks(), 1);
        let r2 = committed(shuffled_tasks(), 2);
        let r8 = committed(shuffled_tasks(), 8);
        assert_eq!(r1, r2);
        assert_eq!(r1, r8);
    }

    /// 注入延迟（确定性忙等）不影响结果——仅模拟调度扰动。
    #[test]
    fn chaos_injection_does_not_affect_result() {
        use sim_test_support::chaos::chaos_spin;

        struct ChaosSnap {
            base: u64,
            chaos_seed: u64,
        }
        fn chaos_task(s: &ChaosSnap, i: &u64) -> u64 {
            chaos_spin(*i, s.chaos_seed); // 确定性忙等，打乱完成顺序
            s.base.saturating_add(*i)
        }
        let keys: Vec<StableKey> = (0..16).map(|i| (0, i)).collect();
        let run = |seed: u64| {
            let snap = ChaosSnap { base: 11, chaos_seed: seed };
            let tasks: Vec<Task<ChaosSnap, u64, u64>> = keys
                .iter()
                .map(|&key| Task { key, input: key.1, func: chaos_task })
                .collect();
            let mut out = Vec::new();
            run_parallel(&snap, tasks, 4, |k, v| out.push((k, v)));
            out
        };
        assert_eq!(run(0), run(42), "注入延迟不得影响提交序列");
        assert_eq!(run(0).len(), 16);
    }

    /// 边界：空任务集与单任务。
    #[test]
    fn empty_and_single_task() {
        let snap = Snap { base: 1 };
        let mut calls = 0u32;
        run_parallel(&snap, Vec::<Task<Snap, u64, u64>>::new(), 4, |_, _| calls += 1);
        assert_eq!(calls, 0);
        let one = vec![Task { key: (0, 1), input: 5, func: mul_task }];
        let mut got = Vec::new();
        run_parallel(&snap, one, 8, |k, v| got.push((k, v)));
        assert_eq!(got, vec![((0, 1), 10)]); // 1*5+5=10
    }
}


===== FILE: crates/sim-parallel/tests/thread_independence.rs =====

//! B3 场景级线程无关性验证（Stage 2B 退出条件 1/2/3/4）。
//!
//! 场景：100 个 Agent × 50 tick 的确定性随机游走。
//! 每 tick：只读快照上并行执行纯函数任务（含混沌忙等注入），
//! 按稳定键排序串行提交，逐 tick 折叠 GlobalDigest。
//! 断言：1/2/8 线程摘要一致、混沌种子无关、任务到达顺序无关、与单线程参考一致。

use sim_codec::CanonicalWriter;
use sim_digest::GlobalDigest;
use sim_parallel::{run_parallel, StableKey, Task};
use sim_rng::event_u64;
use sim_test_support::chaos::chaos_spin;

const SEED: u64 = 0x1234_5678_9ABC_DEF0;
const AGENTS: u64 = 100;
const TICKS: u64 = 50;

#[derive(Clone)]
struct World {
    agents: Vec<(i64, i64)>, // 索引 = agent_id - 1
}

type In = (u64, u64, u64); // (agent_id, tick, chaos_seed)
type Out = (i64, i64);

/// 纯函数任务：只读快照 + 确定性派生位移 + 混沌忙等（只影响耗时，不影响结果）。
fn step_task(snap: &World, input: &In) -> Out {
    let (aid, tick, chaos_seed) = *input;
    let _ = chaos_spin(aid.saturating_mul(tick), chaos_seed);
    let (x, y) = snap.agents[(aid - 1) as usize];
    let r = event_u64(
        SEED,
        "sim-parallel",
        aid.saturating_mul(1000).saturating_add(tick),
        0,
    );
    let dx = (r % 5) as i64 - 2;
    let dy = ((r >> 8) % 5) as i64 - 2;
    (x.saturating_add(dx), y.saturating_add(dy))
}

fn record_tick(gd: &mut GlobalDigest, tick: u64, world: &World) {
    let mut w = CanonicalWriter::new("sim-parallel", "AgentState");
    w.u64(tick);
    for p in &world.agents {
        w.i64(p.0);
        w.i64(p.1);
    }
    gd.record("sim-parallel", "AgentState", w.as_bytes());
}

/// 场景执行：并行框架路径。reversed=true 时任务构建顺序颠倒（检验提交契约）。
fn run_scenario(threads: usize, chaos_seed: u64, reversed: bool) -> String {
    let mut world = World {
        agents: (1..=AGENTS).map(|i| (i as i64 * 7, i as i64 * 11)).collect(),
    };
    let mut gd = GlobalDigest::new();
    for tick in 1..=TICKS {
        let snap = world.clone();
        let mut tasks: Vec<Task<World, In, Out>> = (1..=AGENTS)
            .map(|aid| Task {
                key: (0, aid),
                input: (aid, tick, chaos_seed),
                func: step_task,
            })
            .collect();
        if reversed {
            tasks.reverse();
        }
        run_parallel(&snap, tasks, threads, |k, out| {
            // 串行提交（run_parallel 已按稳定键排序逐项回调）
            world.agents[(k.1 - 1) as usize] = out;
        });
        record_tick(&mut gd, tick, &world);
    }
    gd.finalize().hex()
}

/// 单线程参考实现：同样的纯函数与排序提交，完全不走并行框架。
fn run_serial(chaos_seed: u64) -> String {
    let mut world = World {
        agents: (1..=AGENTS).map(|i| (i as i64 * 7, i as i64 * 11)).collect(),
    };
    let mut gd = GlobalDigest::new();
    for tick in 1..=TICKS {
        let snap = world.clone();
        let tasks: Vec<Task<World, In, Out>> = (1..=AGENTS)
            .map(|aid| Task {
                key: (0, aid),
                input: (aid, tick, chaos_seed),
                func: step_task,
            })
            .collect();
        let mut results: Vec<(StableKey, Out)> = tasks
            .iter()
            .map(|t| (t.key, (t.func)(&snap, &t.input)))
            .collect();
        results.sort_by_key(|(k, _)| *k);
        for (k, out) in results {
            world.agents[(k.1 - 1) as usize] = out;
        }
        record_tick(&mut gd, tick, &world);
    }
    gd.finalize().hex()
}

#[test]
fn thread_count_independence_1_2_8() {
    let d1 = run_scenario(1, 0, false);
    let d2 = run_scenario(2, 0, false);
    let d8 = run_scenario(8, 0, false);
    assert_eq!(d1, d2, "1 线程与 2 线程摘要必须一致");
    assert_eq!(d1, d8, "1 线程与 8 线程摘要必须一致");
}

#[test]
fn chaos_injection_does_not_change_result() {
    let d0 = run_scenario(4, 0, false);
    let d42 = run_scenario(4, 42, false);
    assert_eq!(d0, d42, "混沌注入只影响耗时，不得影响结果");
}

#[test]
fn submission_order_independence_scenario() {
    let forward = run_scenario(2, 0, false);
    let reversed = run_scenario(2, 0, true);
    assert_eq!(forward, reversed, "任务到达顺序不得影响提交结果");
}

#[test]
fn parallel_matches_serial_reference() {
    let par = run_scenario(4, 0, false);
    let ser = run_serial(0);
    assert_eq!(par, ser, "并行与单线程参考结果必须一致");
}
