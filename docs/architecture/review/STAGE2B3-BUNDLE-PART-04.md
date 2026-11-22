# STAGE2B3-BUNDLE PART 04（crates/sim-network/src/lib.rs 续段 2）
<!-- commit 0bd384ab7874d9180273c7e0c87106b6e7d723e3 -->
ity_id, 7);
    }

    #[test]
    fn limiter_rejection_feeds_retry_queue() {
        // 与 TxLimiter（BudgetExceeded）衔接：预算拒绝→入队→下 tick 重试成功。
        let mut lim = TxLimiter::new(TxLimits {
            topology_per_tick: 1,
            cost_per_tick: 1,
            max_edge_ops: 16,
        });
        let net = Network::new();
        let t1 = Tx { id: 1, ops: vec![Op::SetDynamicCost { id: 1, cost: 5 }] };
        let t2 = Tx { id: 2, ops: vec![Op::SetDynamicCost { id: 1, cost: 6 }] };
        assert!(lim.admit(&t1, &net).is_ok());
        assert!(matches!(
            lim.admit(&t2, &net),
            Err(TxError::BudgetExceeded { .. })
        ));
        let mut q = TxRetryQueue::new();
        assert_eq!(q.offer(t2), RetryOutcome::Requeued { retry_count: 1 });
        // 下 tick：限流器复位，due 取回原事务（原 tx_id），admit 成功。
        lim.reset();
        let due = q.due();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].id, 2);
        assert!(lim.admit(&due[0], &net).is_ok());
    }
}
