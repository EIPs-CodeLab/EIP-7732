use eip_7732::beacon_chain::{
    constants::{
        BUILDER_PENDING_WITHDRAWALS_LIMIT, BUILDER_REGISTRY_LIMIT,
        MAX_BUILDERS_PER_WITHDRAWALS_SWEEP,
    },
    containers::{Builder, BuilderPendingWithdrawal},
    registry::{register_builder, request_builder_withdrawal, sweep_pending_withdrawals, RegistryError, RegistryState},
    types::*,
};
use std::collections::{HashMap, VecDeque};

struct MockRegistry {
    builders: HashMap<BuilderIndex, Builder>,
    pending: VecDeque<BuilderPendingWithdrawal>,
    pending_len_override: Option<u64>,
}

impl MockRegistry {
    fn new() -> Self {
        Self { builders: HashMap::new(), pending: VecDeque::new(), pending_len_override: None }
    }
    fn builder(index: BuilderIndex, balance: Gwei, withdrawable_epoch: Epoch) -> Builder {
        Builder {
            pubkey: [0u8; 48],
            version: 0,
            execution_address: [0u8; 20],
            balance,
            deposit_epoch: 0,
            withdrawable_epoch,
        }
    }
}

impl RegistryState for MockRegistry {
    fn builder_count(&self) -> u64 { self.builders.len() as u64 }
    fn get_builder(&self, index: BuilderIndex) -> Option<Builder> {
        self.builders.get(&index).cloned()
    }
    fn insert_builder(&mut self, index: BuilderIndex, builder: Builder) {
        self.builders.insert(index, builder);
    }
    fn debit_builder_balance(&mut self, index: BuilderIndex, amount: Gwei) {
        if let Some(b) = self.builders.get_mut(&index) {
            b.balance -= amount;
        }
    }
    fn push_pending_withdrawal(&mut self, w: BuilderPendingWithdrawal) {
        self.pending.push_back(w);
    }
    fn pending_withdrawals_len(&self) -> u64 {
        self.pending_len_override.unwrap_or(self.pending.len() as u64)
    }
    fn pop_pending_withdrawals(&mut self, max: u64) -> Vec<BuilderPendingWithdrawal> {
        let mut out = Vec::new();
        for _ in 0..max {
            if let Some(w) = self.pending.pop_front() {
                out.push(w);
            } else {
                break;
            }
        }
        out
    }
}

#[test]
fn register_builder_succeeds() {
    let mut reg = MockRegistry::new();
    let b = MockRegistry::builder(1, 10_000, 5);
    assert!(register_builder(&mut reg, 1, b).is_ok());
    assert_eq!(reg.builder_count(), 1);
}

#[test]
fn register_builder_duplicate_rejected() {
    let mut reg = MockRegistry::new();
    let b = MockRegistry::builder(1, 10_000, 5);
    register_builder(&mut reg, 1, b.clone()).unwrap();
    let err = register_builder(&mut reg, 1, b).unwrap_err();
    assert!(matches!(err, RegistryError::BuilderExists(1)));
}

#[test]
fn withdrawal_not_withdrawable_yet() {
    let mut reg = MockRegistry::new();
    register_builder(&mut reg, 1, MockRegistry::builder(1, 10_000, 10)).unwrap();
    let err = request_builder_withdrawal(&mut reg, 1, 1_000, 5).unwrap_err();
    assert!(matches!(err, RegistryError::NotWithdrawable { .. }));
}

#[test]
fn withdrawal_insufficient_balance() {
    let mut reg = MockRegistry::new();
    register_builder(&mut reg, 1, MockRegistry::builder(1, 500, 0)).unwrap();
    let err = request_builder_withdrawal(&mut reg, 1, 1_000, 0).unwrap_err();
    assert!(matches!(err, RegistryError::InsufficientBalance { .. }));
}

#[test]
fn pending_queue_limit_enforced() {
    let mut reg = MockRegistry::new();
    register_builder(&mut reg, 1, MockRegistry::builder(1, 10_000, 0)).unwrap();
    reg.pending_len_override = Some(BUILDER_PENDING_WITHDRAWALS_LIMIT);
    let err = request_builder_withdrawal(&mut reg, 1, 1, 0).unwrap_err();
    assert!(matches!(err, RegistryError::PendingQueueFull(_)));
}

#[test]
fn sweep_respects_max_builders_per_sweep() {
    let mut reg = MockRegistry::new();
    register_builder(&mut reg, 1, MockRegistry::builder(1, 10_000, 0)).unwrap();
    for _ in 0..(MAX_BUILDERS_PER_WITHDRAWALS_SWEEP + 2) {
        request_builder_withdrawal(&mut reg, 1, 1, 0).unwrap();
    }
    let swept = sweep_pending_withdrawals(&mut reg);
    assert_eq!(swept.len() as u64, MAX_BUILDERS_PER_WITHDRAWALS_SWEEP);
}
