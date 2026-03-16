/// EIP-7732 — process_execution_payload_bid
///
/// Replaces the old `process_execution_payload` in BeaconBlock processing.
/// Validates the SignedExecutionPayloadBid included in the BeaconBlockBody,
/// deducts the committed value from the builder's balance and queues a
/// BuilderPendingPayment in the beacon state.
///
/// Reference: https://eips.ethereum.org/EIPS/eip-7732#beacon-chain-changes
use crate::beacon_chain::{
    containers::{BuilderPendingPayment, BuilderPendingWithdrawal, SignedExecutionPayloadBid},
    types::{BuilderIndex, Gwei, Slot},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PayloadBidError {
    #[error("Builder index {0} not found in registry")]
    BuilderNotFound(BuilderIndex),

    #[error("Builder balance {balance} insufficient for bid value {value}")]
    InsufficientBalance { balance: Gwei, value: Gwei },

    #[error("Bid slot {bid_slot} does not match block slot {block_slot}")]
    SlotMismatch { bid_slot: Slot, block_slot: Slot },

    #[error("Invalid builder BLS signature")]
    InvalidSignature,

    #[error("Parent block hash mismatch")]
    ParentHashMismatch,
}

/// Minimal beacon state surface needed by this function.
pub trait BeaconStateMut {
    fn builder_balance(&self, index: BuilderIndex) -> Option<Gwei>;
    fn deduct_builder_balance(&mut self, index: BuilderIndex, amount: Gwei);
    fn push_pending_payment(&mut self, payment: BuilderPendingPayment);
    fn current_slot(&self) -> Slot;
    fn latest_block_hash(&self) -> [u8; 32];
}

/// Process a `SignedExecutionPayloadBid` from the beacon block body.
///
/// Steps (per spec):
/// 1. Verify the bid slot matches the current block slot.
/// 2. Verify the parent block hash matches `state.latest_block_hash`.
/// 3. Verify the builder's BLS signature.
/// 4. Check the builder has sufficient balance.
/// 5. Deduct `bid.value` from the builder's balance.
/// 6. Queue a `BuilderPendingPayment` in the beacon state.
pub fn process_execution_payload_bid<S: BeaconStateMut>(
    state: &mut S,
    signed_bid: &SignedExecutionPayloadBid,
    // In a full impl, pass genesis_validators_root + fork for domain computation
) -> Result<(), PayloadBidError> {
    let bid = &signed_bid.message;

    // Step 1 — slot check
    if bid.slot != state.current_slot() {
        return Err(PayloadBidError::SlotMismatch {
            bid_slot: bid.slot,
            block_slot: state.current_slot(),
        });
    }

    // Step 2 — parent block hash
    if bid.parent_block_hash != state.latest_block_hash() {
        return Err(PayloadBidError::ParentHashMismatch);
    }

    // Step 3 — BLS signature (stub — wire to blst crate in full impl)
    verify_builder_signature(signed_bid)?;

    // Step 4 + 5 — balance check and deduction
    let balance = state
        .builder_balance(bid.builder_index)
        .ok_or(PayloadBidError::BuilderNotFound(bid.builder_index))?;

    if balance < bid.value {
        return Err(PayloadBidError::InsufficientBalance {
            balance,
            value: bid.value,
        });
    }

    state.deduct_builder_balance(bid.builder_index, bid.value);

    // Step 6 — queue pending payment
    let pending = BuilderPendingPayment {
        weight: bid.value,
        withdrawal: BuilderPendingWithdrawal {
            fee_recipient: bid.fee_recipient,
            amount: bid.value,
            builder_index: bid.builder_index,
        },
    };
    state.push_pending_payment(pending);

    Ok(())
}

/// Stub — replace with blst domain-separated BLS verify in full impl.
fn verify_builder_signature(
    _signed_bid: &SignedExecutionPayloadBid,
) -> Result<(), PayloadBidError> {
    // TODO: compute signing_root with DOMAIN_BEACON_BUILDER and verify
    Ok(())
}
