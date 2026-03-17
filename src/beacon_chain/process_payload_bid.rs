/// EIP-7732 — process_execution_payload_bid
///
/// Replaces the old `process_execution_payload` in BeaconBlock processing.
/// Validates the SignedExecutionPayloadBid included in the BeaconBlockBody,
/// deducts the committed value from the builder's balance and queues a
/// BuilderPendingPayment in the beacon state.
///
/// Reference: https://eips.ethereum.org/EIPS/eip-7732#beacon-chain-changes
use crate::beacon_chain::{
    constants::DOMAIN_BEACON_BUILDER,
    containers::{BuilderPendingPayment, BuilderPendingWithdrawal, SignedExecutionPayloadBid},
    types::{BLSPubkey, BuilderIndex, Gwei, Slot},
};
use crate::utils::{crypto, ssz};
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

    #[error("Builder pubkey missing for index {0}")]
    MissingPubkey(BuilderIndex),
}

/// Minimal beacon state surface needed by this function.
pub trait BeaconStateMut {
    fn builder_balance(&self, index: BuilderIndex) -> Option<Gwei>;
    fn builder_pubkey(&self, index: BuilderIndex) -> Option<BLSPubkey>;
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

    // Step 3 — balance check
    let balance = state
        .builder_balance(bid.builder_index)
        .ok_or(PayloadBidError::BuilderNotFound(bid.builder_index))?;

    if balance < bid.value {
        return Err(PayloadBidError::InsufficientBalance {
            balance,
            value: bid.value,
        });
    }

    // Step 4 — BLS signature
    verify_builder_signature(state, signed_bid)?;

    // Step 5 — deduct balance
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
fn verify_builder_signature<S: BeaconStateMut>(
    state: &S,
    signed_bid: &SignedExecutionPayloadBid,
) -> Result<(), PayloadBidError> {
    let message = &signed_bid.message;
    let pk = state
        .builder_pubkey(message.builder_index)
        .ok_or(PayloadBidError::MissingPubkey(message.builder_index))?;

    let domain = ssz::compute_domain_simple(DOMAIN_BEACON_BUILDER);
    let signing_root = ssz::signing_root(message, domain);
    crypto::bls_verify(&pk, &signing_root, &signed_bid.signature)
        .map_err(|_| PayloadBidError::InvalidSignature)
}
