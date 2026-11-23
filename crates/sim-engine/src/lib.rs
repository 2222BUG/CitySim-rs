//! sim-engine：模拟主循环集成层（权威 crate）。
//!
//! 契约=docs/architecture/integration-design.md §1 tick 处理顺序：
//! 命令生效 → 寻路（TickBudget） → 路网事务（TxLimiter+TxRetryQueue） →
//! 交通步进 → 经济结算 → 预算/降级评估 → 摘要折叠 → 时钟推进。
//! 权威状态全部逐 tick 折入 GlobalDigest（设计 §3 清单——防 fold_digest
//! 漏字段教训）；禁止墙钟/浮点/无序容器（2.9 条 3/4）。

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use sim_agent::{AgentIdAllocator, AgentLifecycle};
use sim_core::budget::{
    BudgetDecl, BudgetRegistry, DegradationThresholds, OverflowBehavior, WorkUnit,
};
use sim_core::{CommandLog, CommandPayload, SimClock};
use sim_digest::GlobalDigest;
use sim_economy::EconomyLedger;
use sim_network::{Network, Tx, TxLimits, TxLimiter, TxRetryQueue};
use sim_pathfinding::{BudgetPolicy, FindOutcome, PathFinder, TickBudget, TICK_QUOTA};
use sim_replay::{InputEvent, Recorder};
use sim_traffic::GridTraffic;
use sim_world::{SimulationContext, WorldIdAllocator};

/// 预算系统 ID（ADR-NEW-14 C2 候选单位；阈值=已定案决策3 口径）。
pub const SYSTEM_PATHFINDING: u32 = 1;
pub const SYSTEM_TRAFFIC: u32 = 2;
pub const SYSTEM_NETWORK: u32 = 3;

/// 引擎构造配置。
#[derive(Clone, Debug)]
pub struct EngineConfig {
    pub seed: u64,
    /// 路网节点坐标（1mm 网格；寻路坐标表）。
    pub coords: Vec<(i64, i64)>,
    pub traffic_w: u32,
    pub traffic_h: u32,
    pub income_per_tick: i64,
    pub upkeep_per_tick: i64,
}

/// 确定性模拟引擎：权威状态单一持有者（存档/回放以本结构为准）。
/// 不导出 Clone/Debug（部分内部组件不实现；两跑一致性由 digest 判据保证）。
pub struct Engine {
    pub ctx: SimulationContext,
    pub clock: SimClock,
    pub command_log: CommandLog,
    pub agent_alloc: AgentIdAllocator,
    pub lifecycle: AgentLifecycle,
    pub net: Network,
    pub limiter: TxLimiter,
    pub retry: TxRetryQueue,
    pub finder: PathFinder,
    pub traffic: GridTraffic,
    pub economy: EconomyLedger,
    pub budget: BudgetRegistry,
    pub recorder: Recorder,
    pub coords: Vec<(i64, i64)>,
    /// 在途寻路请求（请求 ID 稳定序=BTreeMap 全序）：(agent_id, start, goal)。
    pub path_queue: BTreeMap<u64, (u64, u64, u64)>,
    pub next_request_id: u64,
    /// 新事务收件箱（提交即入，本 tick 步骤 3 按 tx_id 全序处理）。
    pub tx_inbox: Vec<Tx>,
    /// 已生效命令 ID（due_commands 为纯查询（effective_tick≤tick 全量返回），
    /// 引擎侧必须记录已应用集合，否则命令会逐 tick 重复生效）。
    pub applied_command_ids: BTreeSet<u64>,
    pub applied_commands: u64,
    pub applied_paths: u64,
    pub applied_txs: u64,
    /// 逐 tick 摘要序列（强读档等价判据：逐 tick 对比，不用累积态）。
    pub tick_digests: Vec<sim_digest::Digest256>,
    /// 滚动摘要链（避免 GlobalDigest 同键覆盖语义吞历史——审计3 D 发现）。
    pub chain: sim_digest::Digest256,
    /// tick 内事件缓冲（fold_tick 聚合成单条记录；同键 record 会覆盖，逐条须并集）。
    pub tick_cmd_payloads: Vec<Vec<u8>>,
    pub tick_path_payloads: Vec<Vec<u8>>,
    pub tick_tx_ids: Vec<u64>,
}

fn budget_decls() -> Vec<BudgetDecl> {
    vec![
        BudgetDecl {
            system_id: SYSTEM_PATHFINDING,
            unit: WorkUnit::NodeExpansions,
            per_tick_cap: TICK_QUOTA,
            overflow: OverflowBehavior::YieldToNextTick,
        },
        BudgetDecl {
            system_id: SYSTEM_TRAFFIC,
            unit: WorkUnit::EntityUpdates,
            per_tick_cap: 1_000_000,
            overflow: OverflowBehavior::YieldToNextTick,
        },
        BudgetDecl {
            system_id: SYSTEM_NETWORK,
            unit: WorkUnit::TxApplies,
            per_tick_cap: 1_024,
            overflow: OverflowBehavior::QueueToNextTick,
        },
    ]
}

impl Engine {
    pub fn new(cfg: EngineConfig) -> Self {
        let mut worlds = WorldIdAllocator::new();
        let ctx = SimulationContext::new(worlds.allocate(), cfg.seed);
        Self {
            ctx,
            clock: SimClock::new(),
            command_log: CommandLog::new(),
            agent_alloc: AgentIdAllocator::new(),
            lifecycle: AgentLifecycle::new(3),
            net: Network::new(),
            limiter: TxLimiter::new(TxLimits {
                topology_per_tick: 1_024,
                cost_per_tick: 1_024,
                max_edge_ops: 8_192,
            }),
            retry: TxRetryQueue::new(),
            finder: PathFinder::new(BudgetPolicy::default()),
            traffic: GridTraffic::new(cfg.traffic_w, cfg.traffic_h, 4, cfg.seed),
            economy: EconomyLedger::new(cfg.seed, cfg.income_per_tick, cfg.upkeep_per_tick),
            budget: BudgetRegistry::new(budget_decls(), DegradationThresholds::default()),
            recorder: Recorder::new(cfg.seed, 1, 30),
            coords: cfg.coords,
            path_queue: BTreeMap::new(),
            next_request_id: 1,
            tx_inbox: Vec::new(),
            applied_command_ids: BTreeSet::new(),
            applied_commands: 0,
            applied_paths: 0,
            applied_txs: 0,
            tick_digests: Vec::new(),
            chain: sim_digest::digest("sim-engine", "TickChain", b"init"),
            tick_cmd_payloads: Vec::new(),
            tick_path_payloads: Vec::new(),
            tick_tx_ids: Vec::new(),
        }
    }

    /// 派生 Agent（身份=AgentIdAllocator 高水位口径）。
    pub fn spawn_agent(&mut self) -> u64 {
        let id = self.agent_alloc.alloc().raw();
        let tick = self.clock.tick();
        let _ = self.lifecycle.spawn(id, tick);
        let _ = self.lifecycle.activate(id, tick);
        id
    }

    /// 提交玩家命令（带生效 tick；写 CommandLog + 回放 Recorder）。
    pub fn submit_command(
        &mut self,
        agent_id: u64,
        effective_tick: u64,
        payload: CommandPayload,
    ) -> Result<u64, sim_core::SubmitError> {
        let current = self.clock.tick();
        let id = self
            .command_log
            .submit(agent_id, effective_tick, current, payload)?;
        let (opcode, arg0, arg1) = match payload {
            CommandPayload::MoveTo { x, y } => (0u32, x, y),
            CommandPayload::SetDestination { node } => (1, node as i64, 0),
            CommandPayload::CancelRequest { request_id } => (2, request_id as i64, 0),
        };
        let _ = self.recorder.record(InputEvent::Command {
            command_id: id,
            effective_tick,
            agent_id,
            opcode,
            arg0,
            arg1,
        });
        Ok(id)
    }

    /// 提交路网事务（本 tick 步骤 3 按 tx_id 全序处理）。
    pub fn submit_tx(&mut self, tx: Tx) {
        self.tx_inbox.push(tx);
    }

    /// 执行一个 tick（顺序契约=integration-design.md §1，不得漂移）。
    pub fn run_tick(&mut self) {
        let tick = self.clock.tick();

        // 1. 命令生效：(effective_tick, command_id) 全序。
        for cmd in self.command_log.due_commands(tick) {
            if self.applied_command_ids.insert(cmd.command_id) {
                self.apply_command(cmd);
            }
        }

        // 2. 寻路：请求 ID 稳定序 + 全局每 tick 配额（TickBudget 每 tick 重建=重置）。
        let mut tb = TickBudget::new(TICK_QUOTA);
        let queue = std::mem::take(&mut self.path_queue);
        for (rid, (agent, start, goal)) in queue {
            let out = self
                .finder
                .request_with_budget(&self.net, &self.coords, start, goal, &mut tb);
            self.budget.spend(SYSTEM_PATHFINDING, out.expanded());
            self.applied_paths += 1;
            let class: u8 = match out {
                FindOutcome::Complete { .. } => 0,
                FindOutcome::Yielded { .. } => 1,
                FindOutcome::Unreachable { .. } => 2,
            };
            let mut payload = Vec::new();
            payload.extend_from_slice(&rid.to_le_bytes());
            payload.extend_from_slice(&agent.to_le_bytes());
            payload.push(class);
            payload.extend_from_slice(&out.expanded().to_le_bytes());
            self.tick_path_payloads.push(payload);
        }

        // 3. 路网事务：重试批次先行（tx_id 全序）→ 新事务；被拒进重试队列。
        let mut batch = self.retry.due();
        batch.append(&mut std::mem::take(&mut self.tx_inbox));
        batch.sort_by_key(|t| t.id);
        for tx in batch {
            let id = tx.id;
            if self.limiter.admit(&tx, &self.net).is_ok() {
                if self.net.publish(tx).is_ok() {
                    self.applied_txs += 1;
                    self.budget.spend(SYSTEM_NETWORK, 1);
                    self.tick_tx_ids.push(id);
                }
            } else {
                self.retry.offer(tx);
            }
        }
        self.limiter.reset();

        // 4. 交通步进（确定性 id 序；fold_digest 含 last_step_population 等全字段）。
        self.traffic.step();
        self.budget
            .spend(SYSTEM_TRAFFIC, self.traffic.active_count());

        // 5. 经济结算（EC-04 L2/L3 暂停语义由调用方 pause/resume 驱动）。
        let _ = self.economy.settle_tick();

        // 6. 预算/降级评估（确定性工作量单位，ADR-NEW-14；需求通道默认冻结）。
        let _level = self.budget.end_tick();

        // 7. 摘要折叠（设计 §3 清单；任何权威字段变更必须同步此处）。
        self.fold_tick(tick);

        // 8. 时钟推进（u64 tick，唯一时间入口）。
        self.clock.advance();
    }

    fn apply_command(&mut self, cmd: sim_core::PlayerCommand) {
        self.applied_commands += 1;
        match cmd.payload {
            CommandPayload::SetDestination { node } => {
                if (node as usize) < self.coords.len() {
                    let rid = self.next_request_id;
                    self.next_request_id += 1;
                    self.path_queue
                        .insert(rid, (cmd.agent_id, 0, node));
                }
            }
            CommandPayload::CancelRequest { request_id } => {
                self.path_queue.remove(&request_id);
            }
            CommandPayload::MoveTo { .. } => {}
        }
        let mut payload = Vec::new();
        payload.extend_from_slice(&cmd.command_id.to_le_bytes());
        payload.extend_from_slice(&cmd.effective_tick.to_le_bytes());
        payload.extend_from_slice(&cmd.agent_id.to_le_bytes());
        self.tick_cmd_payloads.push(payload);
    }

    /// 逐 tick 权威状态折叠（GlobalDigest 纳入清单=设计 §3）。每 tick 新建
    /// GlobalDigest：同键 record 覆盖语义不能吞历史（审计3 D 发现）——tick 内
    /// 多事件聚合成单条记录后入档，逐 tick 摘要进 tick_digests 并滚动进 chain。
    fn fold_tick(&mut self, tick: u64) {
        let mut g = GlobalDigest::new();
        g.record("sim-engine", "Tick", &tick.to_le_bytes());
        g.record(
            "sim-engine",
            "Counters",
            &[
                self.applied_commands.to_le_bytes(),
                self.applied_paths.to_le_bytes(),
                self.applied_txs.to_le_bytes(),
                self.next_request_id.to_le_bytes(),
            ]
            .concat(),
        );
        g.record(
            "sim-engine",
            "CommandAppliedSet",
            &self.tick_cmd_payloads.concat(),
        );
        g.record(
            "sim-engine",
            "PathOutcomeSet",
            &self.tick_path_payloads.concat(),
        );
        let txs: Vec<u8> = self
            .tick_tx_ids
            .iter()
            .flat_map(|i| i.to_le_bytes())
            .collect();
        g.record("sim-engine", "TxAppliedSet", &txs);
        self.tick_cmd_payloads.clear();
        self.tick_path_payloads.clear();
        self.tick_tx_ids.clear();
        self.traffic.fold_digest(&mut g);
        let e = self.economy.digest().hex();
        g.record("sim-engine", "EconomyDigest", e.as_bytes());
        let w = self.lifecycle.digest().hex();
        g.record("sim-engine", "LifecycleDigest", w.as_bytes());
        let n = self.net.digest().hex();
        g.record("sim-engine", "NetworkDigest", n.as_bytes());
        let mut cw = sim_codec::CanonicalWriter::new("sim-engine", "Budget");
        self.budget.encode_state(&mut cw);
        let b = cw.into_bytes();
        g.record("sim-engine", "BudgetState", &b);
        let d = g.finalize();
        self.tick_digests.push(d);
        let mut chain_in = Vec::new();
        chain_in.extend_from_slice(&self.chain.0);
        chain_in.extend_from_slice(&d.0);
        self.chain = sim_digest::digest("sim-engine", "TickChain", &chain_in);
    }

    /// 滚动摘要链（两跑逐 tick tick_digests 相等则 chain 相等）。
    pub fn digest(&self) -> sim_digest::Digest256 {
        self.chain
    }

    /// 引擎全状态快照（EC-03 最小集引擎侧；纯数据，可序列化）。
    pub fn snapshot(&self) -> EngineState {
        EngineState {
            seed: self.ctx.seed,
            coords: self.coords.clone(),
            clock_tick: self.clock.tick(),
            commands: self.command_log.export_commands(),
            cmd_high_water: self.command_log.id_high_water(),
            applied_ids: self.applied_command_ids.clone(),
            agent_high_water: self.agent_alloc.clone().high_water(),
            lifecycle: self.lifecycle.export_state(),
            net: self.net.export_state(),
            retry: self.retry.export_state(),
            budget: self.budget.export_state(),
            traffic: self.traffic.export_state(),
            economy: self.economy.export_state(),
            recorder_events: self.recorder.events(),
            path_queue: self.path_queue.clone(),
            next_request_id: self.next_request_id,
            tx_inbox: self.tx_inbox.clone(),
            applied_commands: self.applied_commands,
            applied_paths: self.applied_paths,
            applied_txs: self.applied_txs,
            finder_budget_yield_count: self.finder.budget_yield_count,
            finder_quota_yield_count: self.finder.quota_yield_count,
            tick_digests: self.tick_digests.clone(),
            chain: self.chain,
        }
    }

    /// 从快照恢复（强读档等价"保存并恢复"语义——EC-03/ADR-NEW-16）。
    pub fn restore(s: EngineState) -> Self {
        let mut e = Engine::new(EngineConfig {
            seed: s.seed,
            coords: s.coords.clone(),
            traffic_w: s.traffic.w,
            traffic_h: s.traffic.h,
            income_per_tick: s.economy.income_per_tick,
            upkeep_per_tick: s.economy.upkeep_per_tick,
        });
        e.clock.restore_tick(s.clock_tick);
        e.command_log = CommandLog::from_parts(s.commands, s.cmd_high_water);
        e.applied_command_ids = s.applied_ids;
        e.agent_alloc.restore(s.agent_high_water);
        e.lifecycle.restore_state(s.lifecycle);
        e.net.restore_state(s.net);
        e.retry.restore_state(s.retry);
        e.budget.restore_state(s.budget);
        e.traffic.restore_state(s.traffic);
        e.economy.restore_state(s.economy);
        for ev in s.recorder_events {
            let _ = e.recorder.record(ev);
        }
        e.path_queue = s.path_queue;
        e.next_request_id = s.next_request_id;
        e.tx_inbox = s.tx_inbox;
        e.applied_commands = s.applied_commands;
        e.applied_paths = s.applied_paths;
        e.applied_txs = s.applied_txs;
        e.finder.budget_yield_count = s.finder_budget_yield_count;
        e.finder.quota_yield_count = s.finder_quota_yield_count;
        e.tick_digests = s.tick_digests;
        e.chain = s.chain;
        e
    }
}

/// 引擎全状态快照（纯数据；save() 字节格式在 I4 以 canonical+BLAKE3 封装）。
#[derive(Clone, Debug)]
pub struct EngineState {
    pub seed: u64,
    pub coords: Vec<(i64, i64)>,
    pub clock_tick: u64,
    pub commands: Vec<sim_core::PlayerCommand>,
    pub cmd_high_water: u64,
    pub applied_ids: BTreeSet<u64>,
    pub agent_high_water: u64,
    pub lifecycle: sim_agent::AgentLifecycleState,
    pub net: sim_network::NetworkState,
    pub retry: sim_network::RetryState,
    pub budget: sim_core::budget::BudgetState,
    pub traffic: sim_traffic::GridTrafficState,
    pub economy: sim_economy::EconomyState,
    pub recorder_events: Vec<InputEvent>,
    pub path_queue: BTreeMap<u64, (u64, u64, u64)>,
    pub next_request_id: u64,
    pub tx_inbox: Vec<Tx>,
    pub applied_commands: u64,
    pub applied_paths: u64,
    pub applied_txs: u64,
    pub finder_budget_yield_count: u64,
    pub finder_quota_yield_count: u64,
    pub tick_digests: Vec<sim_digest::Digest256>,
    pub chain: sim_digest::Digest256,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(seed: u64) -> EngineConfig {
        EngineConfig {
            seed,
            coords: vec![(0, 0), (1000, 0), (0, 1000), (1000, 1000)],
            traffic_w: 4,
            traffic_h: 4,
            income_per_tick: 10,
            upkeep_per_tick: 1,
        }
    }

    fn scenario(seed: u64) -> Engine {
        let mut e = Engine::new(config(seed));
        for _ in 0..3 {
            let a = e.spawn_agent();
            let _ = e.submit_command(a, 1, CommandPayload::SetDestination { node: 3 });
        }
        let _ = e.submit_command(1, 2, CommandPayload::MoveTo { x: 5, y: 5 });
        e.traffic.inject(5);
        e
    }

    #[test]
    fn two_runs_same_seed_digest_equal() {
        let mut a = scenario(0x1234);
        let mut b = scenario(0x1234);
        for _ in 0..20 {
            a.run_tick();
            b.run_tick();
        }
        assert_eq!(a.digest(), b.digest(), "same seed must reproduce");
        assert_eq!(a.applied_commands, b.applied_commands);
        assert_eq!(a.applied_paths, b.applied_paths);
    }

    #[test]
    fn different_seed_produces_different_digest() {
        let mut a = scenario(0x1234);
        let mut b = scenario(0x5678);
        for _ in 0..20 {
            a.run_tick();
            b.run_tick();
        }
        assert_ne!(a.digest(), b.digest(), "seed must enter the digest domain");
    }

    #[test]
    fn strong_readback_equivalence_via_snapshot() {
        // EC-11 强形式：run 至 N → snapshot → restore → 双方跑至 M，
        // 逐 tick 摘要必须一致（"保存并恢复"语义，ADR-NEW-16 定案）。
        let mut direct = scenario(0xC0FFEE);
        for _ in 0..10 {
            direct.run_tick();
        }
        let mut restored = Engine::restore(direct.snapshot());
        for _ in 0..15 {
            direct.run_tick();
            restored.run_tick();
        }
        assert_eq!(direct.tick_digests, restored.tick_digests);
        assert_eq!(direct.digest(), restored.digest());
        assert_eq!(direct.applied_commands, restored.applied_commands);
    }

    #[test]
    fn snapshot_roundtrip_preserves_counters() {
        let mut e = scenario(0xABCD);
        for _ in 0..5 {
            e.run_tick();
        }
        let s = e.snapshot();
        let r = Engine::restore(s.clone());
        assert_eq!(r.applied_commands, e.applied_commands);
        assert_eq!(r.applied_paths, e.applied_paths);
        assert_eq!(r.clock.tick(), e.clock.tick());
        assert_eq!(r.tick_digests, e.tick_digests);
    }

    /// 回归：Recorder 对 Command 事件按提交序记录不丢事件（生效 tick 可倒序）——
    /// 2026-11-23 I3 集成发现按 effective_tick 单调校验会静默丢命令致回放分歧。
    #[test]
    fn replay_recorder_keeps_out_of_order_commands() {
        let mut a = Engine::new(config(0x7777));
        let ids: Vec<u64> = (0..3).map(|_| a.spawn_agent()).collect();
        for (k, id) in ids.iter().enumerate() {
            a.submit_command(*id, 1, CommandPayload::SetDestination { node: 3 }).unwrap();
            a.submit_command(*id, 2, CommandPayload::MoveTo { x: k as i64, y: 0 }).unwrap();
        }
        let events = a.recorder.events();
        let mut b = Engine::new(config(0x7777));
        for _ in 0..3 {
            b.spawn_agent();
        }
        for ev in &events {
            match *ev {
                InputEvent::Command { agent_id, opcode, arg0, arg1, .. } => {
                    let p = match opcode {
                        0 => CommandPayload::MoveTo { x: arg0, y: arg1 },
                        1 => CommandPayload::SetDestination { node: arg0 as u64 },
                        _ => CommandPayload::CancelRequest { request_id: arg0 as u64 },
                    };
                    b.submit_command(agent_id, ev.tick(), p).unwrap();
                }
                _ => panic!(),
            }
        }
        a.run_tick();
        b.run_tick();
        assert_eq!(
            format!("{:?}", a.snapshot()),
            format!("{:?}", b.snapshot()),
            "snapshot diff at tick 1"
        );
    }

    #[test]
    fn replay_equivalence_via_recorded_commands() {
        // 回放等价（replay-schema 口径）：录制的命令输入驱动第二引擎重放，
        // 逐 tick 摘要必须与原运行一致。
        fn payload_of(ev: &InputEvent) -> (u64, CommandPayload) {
            match *ev {
                InputEvent::Command {
                    agent_id,
                    opcode,
                    arg0,
                    arg1,
                    ..
                } => {
                    let p = match opcode {
                        0 => CommandPayload::MoveTo { x: arg0, y: arg1 },
                        1 => CommandPayload::SetDestination {
                            node: arg0 as u64,
                        },
                        _ => CommandPayload::CancelRequest {
                            request_id: arg0 as u64,
                        },
                    };
                    (agent_id, p)
                }
                _ => panic!("scenario 只录制 Command 事件"),
            }
        }

        let mut a = Engine::new(config(0x7777));
        let ids: Vec<u64> = (0..3).map(|_| a.spawn_agent()).collect();
        for (k, id) in ids.iter().enumerate() {
            a.submit_command(*id, 1, CommandPayload::SetDestination { node: 3 })
                .unwrap();
            a.submit_command(*id, 2, CommandPayload::MoveTo { x: k as i64, y: 0 })
                .unwrap();
        }
        let events = a.recorder.events();
        for _ in 0..12 {
            a.run_tick();
        }

        let mut b = Engine::new(config(0x7777));
        for _ in 0..3 {
            b.spawn_agent();
        }
        for ev in &events {
            let (agent_id, p) = payload_of(ev);
            let eff = ev.tick();
            b.submit_command(agent_id, eff, p).unwrap();
        }
        for _ in 0..12 {
            b.run_tick();
        }
        assert_eq!(a.tick_digests, b.tick_digests, "replay must reproduce");
        assert_eq!(a.digest(), b.digest());
    }

    #[test]
    fn tick_advances_and_commands_apply_in_order() {
        let mut e = scenario(0x1234);
        e.run_tick();
        // tick0：effective_tick=1 的 3 条未到；clock 前进到 1。
        assert_eq!(e.applied_commands, 0);
        assert_eq!(e.clock.tick(), 1);
        e.run_tick();
        // tick1：3 条 SetDestination 生效（(effective_tick, command_id) 全序）。
        assert_eq!(e.applied_commands, 3);
        assert_eq!(e.clock.tick(), 2);
        e.run_tick();
        // tick2：MoveTo 生效。
        assert_eq!(e.applied_commands, 4);
        assert_eq!(e.clock.tick(), 3);
    }
}
