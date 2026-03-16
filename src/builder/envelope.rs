/// EIP-7732 — Builder payload reveal (ExecutionPayloadEnvelope)
///
/// After the proposer includes the builder's bid in a beacon block, the builder
/// MUST reveal the full payload by broadcasting a `SignedExecutionPayloadEnvelope`
/// on the P2P layer.
///
/// The builder SHOULD reveal as soon as they see the beacon block. Revealing
/// late (after the PTC attestation deadline) results in a PTC vote of
/// `payload_present = false`, which means the builder is NOT paid.
///
/// The envelope commits to the CL post-state-transition beacon state root,
/// allowing the consensus layer to verify correctness without re-executing.
///
/// Reference: https://eips.ethereum.org/EIPS/eip-7732#honest-builder-guide
use crate::beacon_chain::{
    containers::{
        ExecutionPayload, ExecutionPayloadEnvelope, SignedExecutionPayloadEnvelope, Withdrawal,
    },
    types::{BuilderIndex, Hash32, Root, Slot},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnvelopeError {
    #[error("Payload blockhash {revealed:?} does not match committed blockhash {committed:?}")]
    BlockHashMismatch { revealed: Hash32, committed: Hash32 },

    #[error("Payload slot {payload_slot} does not match bid slot {bid_slot}")]
    SlotMismatch { payload_slot: Slot, bid_slot: Slot },

    #[error("Withdrawals in payload do not match beacon state commitment")]
    WithdrawalsMismatch,

    #[error("Signing failed: {0}")]
    SigningFailed(String),
}

/// Parameters for building the envelope.
#[derive(Debug, Clone)]
pub struct EnvelopeParams {
    pub payload: ExecutionPayload,
    pub execution_requests: Vec<u8>,
    pub builder_index: BuilderIndex,
    pub beacon_block_root: Root,
    pub slot: Slot,
    /// The CL state root AFTER applying this payload — committed in envelope.
    pub post_state_root: Root,
    /// The blockhash the builder committed to in their bid.
    pub committed_hash: Hash32,
    /// The withdrawals the beacon state expects (must match payload).
    pub expected_withdrawals: Vec<Withdrawal>,
}

/// Construct and sign a `SignedExecutionPayloadEnvelope`.
///
/// The builder calls this after receiving the beacon block and verifying
/// their bid was included. They then broadcast the result immediately.
pub fn construct_envelope(
    params: &EnvelopeParams,
    sign_fn: impl Fn(&[u8]) -> Result<[u8; 96], String>,
) -> Result<SignedExecutionPayloadEnvelope, EnvelopeError> {
    // Blockhash must match what was committed in the bid
    if params.payload.block_hash != params.committed_hash {
        return Err(EnvelopeError::BlockHashMismatch {
            revealed: params.payload.block_hash,
            committed: params.committed_hash,
        });
    }

    // Withdrawals must match the beacon state commitment
    if params.payload.withdrawals != params.expected_withdrawals {
        return Err(EnvelopeError::WithdrawalsMismatch);
    }

    let message = ExecutionPayloadEnvelope {
        payload: params.payload.clone(),
        execution_requests: params.execution_requests.clone(),
        builder_index: params.builder_index,
        beacon_block_root: params.beacon_block_root,
        slot: params.slot,
        state_root: params.post_state_root,
    };

    let signing_root = compute_envelope_signing_root(&message);
    let signature = sign_fn(&signing_root).map_err(EnvelopeError::SigningFailed)?;

    Ok(SignedExecutionPayloadEnvelope { message, signature })
}

fn compute_envelope_signing_root(_msg: &ExecutionPayloadEnvelope) -> Vec<u8> {
    // TODO: ssz hash_tree_root(msg) XOR compute_domain(DOMAIN_BEACON_BUILDER, ...)
    vec![0u8; 32]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beacon_chain::containers::ExecutionPayload;

    fn make_payload(hash: Hash32) -> ExecutionPayload {
        ExecutionPayload {
            block_hash: hash,
            parent_hash: [1u8; 32],
            fee_recipient: [0u8; 20],
            gas_used: 1_000_000,
            gas_limit: 30_000_000,
            timestamp: 1_700_000_000,
            extra_data: vec![],
            transactions: vec![],
            withdrawals: vec![],
        }
    }

    fn dummy_signer(_msg: &[u8]) -> Result<[u8; 96], String> {
        Ok([0u8; 96])
    }

    #[test]
    fn blockhash_mismatch_rejected() {
        let params = EnvelopeParams {
            payload: make_payload([0xAAu8; 32]),
            execution_requests: vec![],
            builder_index: 1,
            beacon_block_root: [1u8; 32],
            slot: 100,
            post_state_root: [2u8; 32],
            committed_hash: [0xBBu8; 32], // different!
            expected_withdrawals: vec![],
        };
        assert!(matches!(
            construct_envelope(&params, dummy_signer),
            Err(EnvelopeError::BlockHashMismatch { .. })
        ));
    }

    #[test]
    fn valid_envelope_constructed() {
        let hash = [0xAAu8; 32];
        let params = EnvelopeParams {
            payload: make_payload(hash),
            execution_requests: vec![],
            builder_index: 1,
            beacon_block_root: [1u8; 32],
            slot: 100,
            post_state_root: [2u8; 32],
            committed_hash: hash,
            expected_withdrawals: vec![],
        };
        let envelope = construct_envelope(&params, dummy_signer).unwrap();
        assert_eq!(envelope.message.slot, 100);
        assert_eq!(envelope.message.payload.block_hash, hash);
    }
}
