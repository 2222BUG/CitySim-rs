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
    // 稳定键唯一性契约（审计3 A-1/CF-1 修复）：排序后相邻重复即键碰撞——
    // 碰撞会让提交序取决于线程调度（确定性静默破口），必须在入口确定性 panic。
    {
        let mut it = tasks.iter();
        if let Some(first) = it.next() {
            let mut prev = first.key;
            for t in it {
                assert!(
                    t.key != prev,
                    "StableKey collision ({:?})：稳定键必须全局唯一（契约违反，键相等由调用方保证不发生）",
                    t.key
                );
                prev = t.key;
            }
        }
    }
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

    /// 提交键序列线程数无关（审计3 A-2）：1/2/8/16 线程下提交的**键顺序**完全
    /// 相同（不止结果相等——顺序契约显式断言）。
    #[test]
    fn commit_key_sequence_is_identical_across_thread_counts() {
        fn key_seq(threads: usize) -> Vec<StableKey> {
            let snap = 7u64;
            fn t(_: &u64, i: &u64) -> u64 {
                *i
            }
            let keys: Vec<StableKey> = (0..16u64).rev().map(|i| (0, i)).collect();
            let tasks: Vec<Task<u64, u64, u64>> =
                keys.into_iter().map(|key| Task { key, input: key.1, func: t }).collect();
            let mut seq = Vec::new();
            run_parallel(&snap, tasks, threads, |k, _| seq.push(k));
            seq
        }
        let s1 = key_seq(1);
        assert_eq!(s1, key_seq(2));
        assert_eq!(s1, key_seq(8));
        assert_eq!(s1, key_seq(16));
        assert!(s1.windows(2).all(|w| w[0] < w[1]), "提交键序列必须为严格升序全序");
    }

    /// 稳定键碰撞即确定性 panic（审计3 A-1/CF-1）：重复键不得静默进入
    /// 分片——碰撞时提交序会退化为线程到达序。
    #[test]
    #[should_panic(expected = "StableKey collision")]
    fn stable_key_collision_is_rejected() {
        let snap = 0u64;
        fn t(_: &u64, i: &u64) -> u64 {
            *i
        }
        let tasks: Vec<Task<u64, u64, u64>> = vec![
            Task { key: (0, 1), input: 1, func: t },
            Task { key: (0, 1), input: 2, func: t },
        ];
        let _ = &tasks;
        run_parallel(&snap, tasks, 2, |_, _| {});
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
