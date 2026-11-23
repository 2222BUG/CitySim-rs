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

/// L2 降级寻路配额封顶（候选值，随 ADR-NEW-14 阈值定案回填；确定性常量）。
pub const DEGRADATION_L2_QUOTA_CAP: u64 = TICK_QUOTA / 4;

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
    /// 输入事件收件箱（W1）：键=(生效 tick, 提交序) 全序。
    pub event_inbox: BTreeMap<(u64, u64), InputEvent>,
    pub event_seq: u64,
    /// 本 tick 新增的寻路请求 ID（L1 降级=新请求延迟；tick 末清空）。
    pub new_this_tick: BTreeSet<u64>,
    /// Agent LOD 级别（W2：0=L0 全精度/1=L1/2=L2；入摘要入档）。
    pub lod_levels: BTreeMap<u64, u8>,
    /// 配置变更记录（ConfigChange 事件；(tick, key_id, value) 追加序）。
    pub config_changes: Vec<(u64, u64, i64)>,
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
            event_inbox: BTreeMap::new(),
            event_seq: 0,
            new_this_tick: BTreeSet::new(),
            lod_levels: BTreeMap::new(),
            config_changes: Vec::new(),
        }
    }

    /// 提交输入事件（W1）：Command 保留原 command_id 直插命令日志；
    /// ConfigChange/BudgetAdjustment/LodSwitch 入事件收件箱按 (tick, 提交序) 生效。
    pub fn submit_input_event(&mut self, ev: InputEvent) {
        // 录制（回放等价判据源）：Command 按提交序、其余按 tick 边界语义。
        let _ = self.recorder.record(ev);
        match ev {
            InputEvent::Command {
                command_id,
                effective_tick,
                agent_id,
                opcode,
                arg0,
                arg1,
            } => {
                let payload = match opcode {
                    0 => CommandPayload::MoveTo { x: arg0, y: arg1 },
                    1 => CommandPayload::SetDestination {
                        node: arg0.max(0) as u64,
                    },
                    _ => CommandPayload::CancelRequest {
                        request_id: arg0.max(0) as u64,
                    },
                };
                self.command_log.insert_recorded(sim_core::PlayerCommand {
                    command_id,
                    effective_tick,
                    agent_id,
                    payload,
                });
            }
            ev => {
                let seq = self.event_seq;
                self.event_seq += 1;
                self.event_inbox.insert((ev.tick(), seq), ev);
            }
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
        // 降级等级取本 tick 起点值（由上一 tick end_tick 评估；工作量单位确定性，ADR-NEW-14）。
        let level = self.budget.level();
        let degrade_l1 = matches!(
            level,
            sim_core::budget::DegradationLevel::L1 | sim_core::budget::DegradationLevel::L2
        );
        let degrade_l3 = matches!(level, sim_core::budget::DegradationLevel::L3);
        let degrade_l2_plus = matches!(
            level,
            sim_core::budget::DegradationLevel::L2 | sim_core::budget::DegradationLevel::L3
        );

        // 1. 命令生效：(effective_tick, command_id) 全序。
        for cmd in self.command_log.due_commands(tick) {
            if self.applied_command_ids.insert(cmd.command_id) {
                self.apply_command(cmd);
            }
        }
        // 1b. 输入事件（W1 实接）：(生效 tick, 提交序) 全序即时生效。
        //     ConfigChange→配置记录；BudgetAdjustment→预算上限调整；LodSwitch→LOD 级别。
        let due: Vec<_> = self
            .event_inbox
            .range(..=(tick, u64::MAX))
            .map(|(k, v)| (*k, *v))
            .collect();
        for (k, ev) in due {
            self.event_inbox.remove(&k);
            self.apply_input_event(ev, tick);
        }
        // L3=仅维护已有状态：新命令刚产生的请求进 new_this_tick 集，L1 延迟、L3 全延。
        let new_this_tick = std::mem::take(&mut self.new_this_tick);

        // 2. 寻路：请求 ID 稳定序 + 全局每 tick 配额（TickBudget 每 tick 重建=重置）。
        //    L1=新请求延迟入队（本轮只处理既存请求）；L3=全部延迟。
        let quota = if degrade_l2_plus {
            DEGRADATION_L2_QUOTA_CAP
        } else {
            TICK_QUOTA
        };
        let mut tb = TickBudget::new(quota);
        let queue = std::mem::take(&mut self.path_queue);
        for (rid, (agent, start, goal)) in queue {
            if degrade_l3 || (degrade_l1 && new_this_tick.contains(&rid)) {
                self.path_queue.insert(rid, (agent, start, goal));
                continue;
            }
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
        //    L3=仅维护已有状态：新事务与重试均延后。
        if !degrade_l3 {
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
        }

        // 4. 交通步进（确定性 id 序；fold_digest 含 last_step_population 等全字段）。
        // 4) 交通：LOD 分频移动（lod_levels 实接 rfc-lod-draft；L0 全精度/L1 每 2/L2 每 4 tick）
        let lod = self.lod_levels.clone();
        self.traffic.step_with_lod(|id| lod.get(&id).copied().unwrap_or(0));
        self.budget
            .spend(SYSTEM_TRAFFIC, self.traffic.active_count());

        // 5. 经济结算（EC-04 L2/L3 暂停语义由降级等级驱动：进入 L2+ 暂停、回到 L1- 恢复）。
        if degrade_l2_plus {
            if !self.economy.paused() {
                let _ = self.economy.pause(tick);
            }
        } else if self.economy.paused() {
            let _ = self.economy.resume(tick);
        }
        let _ = self.economy.settle_tick();

        // 6. 预算/降级评估（确定性工作量单位，ADR-NEW-14；需求通道默认冻结）。
        let _level = self.budget.end_tick();

        // 7. 摘要折叠（设计 §3 清单；任何权威字段变更必须同步此处）。
        self.fold_tick(tick);

        // 8. 时钟推进（u64 tick，唯一时间入口）。
        self.clock.advance();
    }

    /// 输入事件处理（W1）：三类非 Command 事件即时生效（确定性白名单）。
    fn apply_input_event(&mut self, ev: InputEvent, tick: u64) {
        match ev {
            InputEvent::Command { .. } => {
                // Command 走 command_log 全序通道，此处不重复处理。
            }
            InputEvent::ConfigChange { key, value, .. } => {
                self.config_changes.push((tick, key, value));
            }
            InputEvent::BudgetAdjustment {
                system_id, per_tick_cap, ..
            } => {
                let _ = self.budget.set_per_tick_cap(system_id as u32, per_tick_cap);
            }
            InputEvent::LodSwitch {
                entity_id,
                to_level,
                ..
            } => {
                self.lod_levels.insert(entity_id, to_level);
            }
        }
    }

    fn apply_command(&mut self, cmd: sim_core::PlayerCommand) {
        self.applied_commands += 1;
        match cmd.payload {
            CommandPayload::SetDestination { node } => {
                if (node as usize) < self.coords.len() {
                    let rid = self.next_request_id;
                    self.next_request_id += 1;
                    self.new_this_tick.insert(rid);
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
        // LOD 级别（W2）与配置变更（W1）：键序确定性（BTreeMap/追加序）。
        let mut lw = sim_codec::CanonicalWriter::new("sim-engine", "LodState");
        lw.u64(self.lod_levels.len() as u64);
        for (agent, lvl) in &self.lod_levels {
            lw.u64(*agent);
            lw.u8(*lvl);
        }
        let l = lw.into_bytes();
        g.record("sim-engine", "LodState", &l);
        let mut kw = sim_codec::CanonicalWriter::new("sim-engine", "ConfigChanges");
        kw.u64(self.config_changes.len() as u64);
        for (t, key, value) in &self.config_changes {
            kw.u64(*t);
            kw.u64(*key);
            kw.i64(*value);
        }
        let k = kw.into_bytes();
        g.record("sim-engine", "ConfigChanges", &k);
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
            event_inbox: self.event_inbox.clone(),
            event_seq: self.event_seq,
            lod_levels: self.lod_levels.clone(),
            config_changes: self.config_changes.clone(),
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
        e.event_inbox = s.event_inbox;
        e.event_seq = s.event_seq;
        e.lod_levels = s.lod_levels;
        e.config_changes = s.config_changes;
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
    pub event_inbox: BTreeMap<(u64, u64), InputEvent>,
    pub event_seq: u64,
    pub lod_levels: BTreeMap<u64, u8>,
    pub config_changes: Vec<(u64, u64, i64)>,
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

    /// 回归：snapshot→restore→snapshot 纯往返无损（逐字段 Debug 相等）。
    #[test]
    fn snapshot_roundtrip_is_lossless() {
        let f = sim_test_support::fixtures::Fixture::smoke();
        let mut coords = Vec::new();
        for gy in 0..f.grid_nodes_y {
            for gx in 0..f.grid_nodes_x {
                coords.push((gx as i64 * f.spacing_mm as i64, gy as i64 * f.spacing_mm as i64));
            }
        }
        let nodes = coords.len() as u64;
        let mut cfg = config(f.seed);
        cfg.coords = coords;
        let mut e = Engine::new(cfg.clone());
        let agents: Vec<u64> = (0..10u64).map(|_| e.spawn_agent()).collect();
        for (i, id) in agents.iter().enumerate() {
            let goal = ((i as u64).wrapping_mul(0x9E37) + 13) % nodes;
            e.submit_command(*id, 1, CommandPayload::SetDestination { node: goal }).unwrap();
        }
        e.submit_tx(sim_network::Tx { id: 100, ops: vec![sim_network::Op::AddEdge(sim_network::Edge { id: 1, a: 0, b: 1, cost: 1000, geom: vec![(0, 0), (1000, 0)] })] });
        e.traffic.inject(1);
        for _ in 0..3 {
            e.run_tick();
        }
        let s1 = e.snapshot();
        let s2 = Engine::restore(s1.clone()).snapshot();
        assert_eq!(format!("{:?}", s1), format!("{:?}", s2), "snapshot roundtrip must be lossless");
    }

    /// I4：R2 fixture 缩小场景端到端（smoke 档）——确定性 + 强读档等价在场景级成立。
    /// 全循环驱动：命令注入、寻路、事务、交通、经济、降级、摘要。
    #[test]
    fn r2_smoke_scenario_end_to_end() {
        let f = sim_test_support::fixtures::Fixture::smoke();
        let mut coords = Vec::new();
        for gy in 0..f.grid_nodes_y {
            for gx in 0..f.grid_nodes_x {
                coords.push((gx as i64 * f.spacing_mm as i64, gy as i64 * f.spacing_mm as i64));
            }
        }
        let nodes = coords.len() as u64;
        let mut cfg = config(f.seed);
        cfg.coords = coords;

        let build = |cfg: &EngineConfig| {
            let mut e = Engine::new(cfg.clone());
            let agents: Vec<u64> = (0..f.agent_count.min(100)).map(|_| e.spawn_agent()).collect();
            for (i, id) in agents.iter().enumerate() {
                let goal = ((i as u64).wrapping_mul(0x9E37) + 13) % nodes;
                e.submit_command(*id, 1, CommandPayload::SetDestination { node: goal }).unwrap();
                e.submit_command(*id, 3, CommandPayload::MoveTo { x: (i % 7) as i64, y: 0 }).unwrap();
            }
            for k in 0..3u64 {
                let edge = sim_network::Edge {
                    id: k + 1,
                    a: k,
                    b: k + 1,
                    cost: 1000,
                    geom: vec![(0, 0), (1000, 0)],
                };
                e.submit_tx(sim_network::Tx {
                    id: 100 + k,
                    ops: vec![sim_network::Op::AddEdge(edge)],
                });
            }
            for t in 0..2u64 {
                e.traffic.inject(t + 1);
            }
            e
        };

        let ticks = f.tick_count.min(30);
        let mut a = build(&cfg);
        let mut b = build(&cfg);
        let mut c = build(&cfg);
        // 三引擎同节奏跑到中途；c 在此 snapshot/restore（强读档等价，场景级）
        let mid = ticks / 2;
        for _ in 0..mid {
            a.run_tick();
            b.run_tick();
            c.run_tick();
        }
        let restored = Engine::restore(c.snapshot());
        let mut c = restored;

        for _ in 0..(ticks - mid) {
            a.run_tick();
            b.run_tick();
            c.run_tick();
        }
        assert_eq!(a.digest(), b.digest(), "two-run determinism at fixture scale");
        assert_eq!(a.digest(), c.digest(), "strong readback at fixture scale");
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

    /// W1：三类输入事件即时生效且确定性（ConfigChange 记录/BudgetAdjustment 调 cap/LodSwitch 入状态）。
    #[test]
    fn input_events_apply_deterministically() {
        let run = |seed: u64| {
            let mut e = scenario(seed);
            e.submit_input_event(InputEvent::ConfigChange {
                tick: 1,
                key: 7,
                value: 42,
            });
            e.submit_input_event(InputEvent::LodSwitch {
                tick: 2,
                entity_id: 1,
                from_level: 0,
                to_level: 2,
            });
            e.submit_input_event(InputEvent::BudgetAdjustment {
                tick: 1,
                system_id: SYSTEM_PATHFINDING as u64,
                per_tick_cap: 1_000,
            });
            for _ in 0..10 {
                e.run_tick();
            }
            e
        };
        let a = run(0xABCD);
        let b = run(0xABCD);
        assert_eq!(a.digest(), b.digest(), "events must be deterministic");
        assert_eq!(a.config_changes, vec![(1, 7, 42)]);
        assert_eq!(a.lod_levels.get(&1), Some(&2));
        assert_eq!(a.applied_commands, b.applied_commands);
    }

    /// W1：回放等价含四类事件（Command 保号直插；submit 即录制；spawn 为环境设置由两侧同序执行）。
    #[test]
    fn replay_equivalence_with_all_event_kinds() {
        let setup = |seed: u64| {
            let mut e = Engine::new(config(seed));
            for _ in 0..3 {
                e.spawn_agent();
            }
            e
        };
        let mut a = setup(0x77);
        // 命令也走 submit_input_event（保号+录制）。
        a.submit_input_event(InputEvent::Command {
            command_id: 0,
            effective_tick: 1,
            agent_id: 1,
            opcode: 1,
            arg0: 3,
            arg1: 0,
        });
        a.submit_input_event(InputEvent::ConfigChange {
            tick: 1,
            key: 3,
            value: 9,
        });
        a.submit_input_event(InputEvent::LodSwitch {
            tick: 1,
            entity_id: 2,
            from_level: 0,
            to_level: 1,
        });
        a.submit_input_event(InputEvent::BudgetAdjustment {
            tick: 2,
            system_id: SYSTEM_NETWORK as u64,
            per_tick_cap: 2_048,
        });
        for _ in 0..12 {
            a.run_tick();
        }
        // 重放引擎只喂录制输入（spawn 环境同序复刻）。
        let mut b = setup(0x77);
        for ev in a.recorder.events() {
            b.submit_input_event(ev);
        }
        for _ in 0..12 {
            b.run_tick();
        }
        assert_eq!(a.tick_digests, b.tick_digests, "replay must reproduce");
    }

    /// W1：降级分支确定性（L1 延迟新请求/L2 暂停经济），两跑摘要一致。
    #[test]
    fn degradation_branches_are_deterministic() {
        let run = |seed: u64| {
            let mut e = Engine::new(config(seed));
            for i in 0..4 {
                let a = e.spawn_agent();
                let _ = e.submit_command(
                    a,
                    1 + (i % 3),
                    CommandPayload::SetDestination { node: 3 },
                );
            }
            // 压缩寻路上限至 1（每次搜索展开 2-3 个即 200-300% ≥90%）→
            // 连续 tick 后触 L2（EC-04：L2+ 暂停经济）。语义按预算比例实算。
            e.submit_input_event(InputEvent::BudgetAdjustment {
                tick: 0,
                system_id: SYSTEM_PATHFINDING as u64,
                per_tick_cap: 1,
            });
            let mut paused_seen = false;
            let mut l2_plus_seen = false;
            for _ in 0..10 {
                e.run_tick();
                paused_seen |= e.economy.paused();
                l2_plus_seen |= e.budget.level() != sim_core::budget::DegradationLevel::Normal;
            }
            (e, paused_seen, l2_plus_seen)
        };
        let (a, a_pause, a_lvl) = run(0xD00D);
        let (b, b_pause, b_lvl) = run(0xD00D);
        assert_eq!(a.digest(), b.digest(), "degradation must be deterministic");
        assert_eq!(a_lvl, b_lvl);
        assert_eq!(a_pause, b_pause);
        assert!(a_lvl, "L2+ must be reached under cap=1 workload");
        assert!(
            a_pause,
            "L2+ must pause economy within the run (EC-04 mapping；恢复后自动 resume）"
        );
    }

    /// W2：LodSwitch 事件实接交通分频——同种子同事件两跑一致，且相对无切换
    /// 运行产生差异（分频真实生效，非恒等通过）。
    #[test]
    fn lod_switch_gates_traffic_deterministically() {
        let config = EngineConfig {
            seed: 0x10D,
            coords: (0..64).map(|i| ((i % 8) as i64, (i / 8) as i64)).collect(),
            traffic_w: 8,
            traffic_h: 8,
            income_per_tick: 1,
            upkeep_per_tick: 1,
        };
        let run = |with_lod: bool| {
            let mut e = Engine::new(config.clone());
            e.traffic.inject(6); // 车辆 id 1..=6（next_id 分配器）
            for i in 0..6u64 {
                let a = e.spawn_agent();
                e.submit_command(a, 1 + (i % 3), CommandPayload::MoveTo { x: 7, y: 7 });
            }
            if with_lod {
                // tick 2 起车辆实体 L2（每 4 tick 才移动更新）
                for id in 1..=6u64 {
                    e.submit_input_event(InputEvent::LodSwitch {
                        tick: 2,
                        entity_id: id,
                        from_level: 0,
                        to_level: 2,
                    });
                }
            }
            for _ in 0..12 {
                e.run_tick();
            }
            e.digest()
        };
        let a1 = run(true);
        let a2 = run(true);
        let b = run(false);
        assert_eq!(a1, a2, "lod switches must be deterministic");
        assert_ne!(a1, b, "lod gating must actually alter evolution (non-degenerate)");
    }
}
