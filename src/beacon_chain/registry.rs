//! Builder registry and withdrawal flows (consensus-specs inspired).
//!
//! This module keeps the state surface minimal while enforcing:
//! - bounded registry size (`BUILDER_REGISTRY_LIMIT`)
//! - bounded pending withdrawals (`BUILDER_PENDING_WITHDRAWALS_LIMIT`)
//! - withdrawability gating by epoch (`withdrawable_epoch`)
//!
//! It does not implement deposits; tests inject builders directly via the
//! state trait to keep the scope small.

use crate::beacon_chain::{
    constants::{
        BUILDER_PENDING_WITHDRAWALS_LIMIT, BUILDER_REGISTRY_LIMIT,
        MAX_BUILDERS_PER_WITHDRAWALS_SWEEP,
    },
    containers::{Builder, BuilderPendingWithdrawal},
    types::{BuilderIndex, Epoch, Gwei},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("builder registry full (limit {0})")]
    RegistryFull(u64),

    #[error("builder {0} already exists")]
    BuilderExists(BuilderIndex),

    #[error("unknown builder {0}")]
    UnknownBuilder(BuilderIndex),

    #[error("builder {index} not withdrawable until epoch {required}, current {current}")]
    NotWithdrawable {
        index: BuilderIndex,
        required: Epoch,
        current: Epoch,
    },

    #[error("builder {index} balance {balance} insufficient for withdrawal {amount}")]
    InsufficientBalance {
        index: BuilderIndex,
        balance: Gwei,
        amount: Gwei,
    },

    #[error("pending withdrawals queue full (limit {0})")]
    PendingQueueFull(u64),
}

pub trait RegistryState {
    fn builder_count(&self) -> u64;
    fn get_builder(&self, index: BuilderIndex) -> Option<Builder>;
    fn insert_builder(&mut self, index: BuilderIndex, builder: Builder);
    fn debit_builder_balance(&mut self, index: BuilderIndex, amount: Gwei);

    fn push_pending_withdrawal(&mut self, w: BuilderPendingWithdrawal);
    fn pending_withdrawals_len(&self) -> u64;
    fn pop_pending_withdrawals(&mut self, max: u64) -> Vec<BuilderPendingWithdrawal>;
}

/// Register a builder if space permits.
pub fn register_builder<S: RegistryState>(
    state: &mut S,
    index: BuilderIndex,
    builder: Builder,
) -> Result<(), RegistryError> {
    if state.builder_count() >= BUILDER_REGISTRY_LIMIT {
        return Err(RegistryError::RegistryFull(BUILDER_REGISTRY_LIMIT));
    }
    if state.get_builder(index).is_some() {
        return Err(RegistryError::BuilderExists(index));
    }
    state.insert_builder(index, builder);
    Ok(())
}

/// Request a withdrawal of builder balance into the execution layer.
pub fn request_builder_withdrawal<S: RegistryState>(
    state: &mut S,
    index: BuilderIndex,
    amount: Gwei,
    current_epoch: Epoch,
) -> Result<BuilderPendingWithdrawal, RegistryError> {
    let builder = state
        .get_builder(index)
        .ok_or(RegistryError::UnknownBuilder(index))?;

    if builder.withdrawable_epoch > current_epoch {
        return Err(RegistryError::NotWithdrawable {
            index,
            required: builder.withdrawable_epoch,
            current: current_epoch,
        });
    }
    if builder.balance < amount {
        return Err(RegistryError::InsufficientBalance {
            index,
            balance: builder.balance,
            amount,
        });
    }
    if state.pending_withdrawals_len() >= BUILDER_PENDING_WITHDRAWALS_LIMIT {
        return Err(RegistryError::PendingQueueFull(
            BUILDER_PENDING_WITHDRAWALS_LIMIT,
        ));
    }

    let withdrawal = BuilderPendingWithdrawal {
        fee_recipient: builder.execution_address,
        amount,
        builder_index: index,
    };

    state.debit_builder_balance(index, amount);
    state.push_pending_withdrawal(withdrawal.clone());
    Ok(withdrawal)
}

/// Pop up to `MAX_BUILDERS_PER_WITHDRAWALS_SWEEP` pending withdrawals for inclusion.
pub fn sweep_pending_withdrawals<S: RegistryState>(state: &mut S) -> Vec<BuilderPendingWithdrawal> {
    state.pop_pending_withdrawals(MAX_BUILDERS_PER_WITHDRAWALS_SWEEP)
}
