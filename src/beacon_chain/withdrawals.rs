/// EIP-7732 — Asynchronous withdrawal processing
///
/// Pre-ePBS, withdrawals were deducted and credited in a single synchronous
/// step inside process_execution_payload. ePBS makes this asynchronous:
///
///   1. When the BeaconBlock is processed:
///      - Withdrawals are DEDUCTED from the beacon chain.
///      - The resulting list is committed to `state.payload_expected_withdrawals`.
///
///   2. When the builder's ExecutionPayloadEnvelope is processed:
///      - The EL payload MUST include exactly the withdrawals committed in (1).
///      - On success, the commitment is cleared.
///
/// If a slot is "empty" (beacon block present, no payload revealed) the beacon
/// chain halts further withdrawal processing until the committed list is honored
/// by a future payload.
///
/// Reference: https://eips.ethereum.org/EIPS/eip-7732#withdrawals
use crate::beacon_chain::{
    containers::Withdrawal,
    types::{Gwei, ValidatorIndex},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WithdrawalError {
    #[error("Payload withdrawals do not match state commitment")]
    WithdrawalsMismatch,

    #[error("Outstanding withdrawals must be honored before processing new ones")]
    OutstandingWithdrawals,
}

pub trait WithdrawalState {
    /// Returns the list committed during beacon block processing.
    fn payload_expected_withdrawals(&self) -> &[Withdrawal];

    /// Deducts withdrawals from CL balances and stores them as expected.
    fn deduct_and_commit_withdrawals(&mut self, withdrawals: Vec<Withdrawal>);

    /// Clears the committed withdrawal list after a successful payload.
    fn clear_expected_withdrawals(&mut self);

    fn has_pending_withdrawals(&self) -> bool {
        !self.payload_expected_withdrawals().is_empty()
    }
}

/// Called during BeaconBlock processing (before payload is known).
/// Deducts from beacon chain and commits to state.
pub fn process_withdrawals_consensus<S: WithdrawalState>(
    state: &mut S,
    withdrawals: Vec<Withdrawal>,
) -> Result<(), WithdrawalError> {
    if state.has_pending_withdrawals() {
        return Err(WithdrawalError::OutstandingWithdrawals);
    }
    state.deduct_and_commit_withdrawals(withdrawals);
    Ok(())
}

/// Called when the ExecutionPayloadEnvelope is received and processed.
/// Verifies the EL payload honored the committed withdrawal list.
pub fn verify_payload_withdrawals<S: WithdrawalState>(
    state: &mut S,
    payload_withdrawals: &[Withdrawal],
) -> Result<(), WithdrawalError> {
    let expected = state.payload_expected_withdrawals();
    if payload_withdrawals != expected {
        return Err(WithdrawalError::WithdrawalsMismatch);
    }
    state.clear_expected_withdrawals();
    Ok(())
}

/// Helper: compute the next set of validator withdrawals from beacon state.
/// Full logic mirrors the Electra sweep — truncated here for clarity.
pub fn compute_next_withdrawals(
    validator_balances: &[(ValidatorIndex, Gwei)],
    max_per_payload: usize,
) -> Vec<Withdrawal> {
    validator_balances
        .iter()
        .take(max_per_payload)
        .map(|(idx, balance)| Withdrawal {
            index: 0, // withdrawal index tracking omitted
            validator_index: *idx,
            address: [0u8; 20], // resolved from withdrawal credentials
            amount: *balance,
        })
        .collect()
}
