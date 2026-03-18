/// EIP-7732 — Builder bid construction
///
/// An honest builder constructs a `SignedExecutionPayloadBid` and broadcasts
/// it on the `/signed_execution_payload_bid` P2P topic immediately after
/// receiving a beacon block from the proposer.
///
/// The bid commits to:
///   - The exact blockhash of the payload the builder will reveal.
///   - A value (in Gwei) to be paid to the proposer.
///   - KZG commitments for any blobs included.
///
/// Reference: https://eips.ethereum.org/EIPS/eip-7732#honest-builder-guide
use crate::beacon_chain::{
    constants::DOMAIN_BEACON_BUILDER,
    containers::{ExecutionPayloadBid, SignedExecutionPayloadBid},
    types::{
        BLSSignature, BuilderIndex, ExecutionAddress, Gwei, Hash32, KZGCommitment, Root, Slot,
    },
};
use crate::utils::ssz;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BidError {
    #[error("Bid value {value} exceeds builder balance {balance}")]
    ValueExceedsBalance { value: Gwei, balance: Gwei },

    #[error("Block hash commitment is zero — builder must commit to a real payload")]
    ZeroBlockHash,

    #[error("Signing failed: {0}")]
    SigningFailed(String),
}

/// All information a builder needs to construct a bid for a slot.
#[derive(Debug, Clone)]
pub struct BidParams {
    pub builder_index: BuilderIndex,
    pub builder_balance: Gwei,
    pub slot: Slot,
    pub parent_block_hash: Hash32,
    pub parent_block_root: Root,
    /// The blockhash of the payload the builder has already built and will reveal.
    pub committed_block_hash: Hash32,
    pub prev_randao: [u8; 32],
    /// Proposer's preferred fee recipient address.
    pub fee_recipient: ExecutionAddress,
    pub gas_limit: u64,
    /// Value offered to the proposer.
    pub bid_value: Gwei,
    /// Any execution-layer payment on top of the beacon-chain payment.
    pub execution_payment: Gwei,
    pub blob_kzg_commitments: Vec<KZGCommitment>,
}

/// Construct and sign an `ExecutionPayloadBid`.
///
/// In production:
///   - The builder MUST have already built the full payload before bidding.
///   - `committed_block_hash` must be the real blockhash of that payload.
///   - The builder signs with DOMAIN_BEACON_BUILDER.
pub fn construct_bid(
    params: &BidParams,
    sign_fn: impl Fn(&[u8]) -> Result<BLSSignature, String>,
) -> Result<SignedExecutionPayloadBid, BidError> {
    // Sanity: never commit to a zero blockhash
    if params.committed_block_hash == [0u8; 32] {
        return Err(BidError::ZeroBlockHash);
    }

    // Builder must have enough balance to back the bid
    if params.bid_value > params.builder_balance {
        return Err(BidError::ValueExceedsBalance {
            value: params.bid_value,
            balance: params.builder_balance,
        });
    }

    let message = ExecutionPayloadBid {
        parent_block_hash: params.parent_block_hash,
        parent_block_root: params.parent_block_root,
        block_hash: params.committed_block_hash,
        prev_randao: params.prev_randao,
        fee_recipient: params.fee_recipient,
        gas_limit: params.gas_limit,
        builder_index: params.builder_index,
        slot: params.slot,
        value: params.bid_value,
        execution_payment: params.execution_payment,
        blob_kzg_commitments: params.blob_kzg_commitments.clone(),
    };

    let domain = ssz::compute_domain_simple(DOMAIN_BEACON_BUILDER);
    let signing_root = ssz::signing_root(&message, domain);

    let signature = sign_fn(&signing_root).map_err(BidError::SigningFailed)?;

    Ok(SignedExecutionPayloadBid { message, signature })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_signer(_msg: &[u8]) -> Result<BLSSignature, String> {
        Ok([0u8; 96])
    }

    #[test]
    fn bid_zero_hash_rejected() {
        let params = BidParams {
            builder_index: 1,
            builder_balance: 100_000_000_000,
            slot: 100,
            parent_block_hash: [1u8; 32],
            parent_block_root: [1u8; 32],
            committed_block_hash: [0u8; 32], // invalid
            prev_randao: [0u8; 32],
            fee_recipient: [0u8; 20],
            gas_limit: 30_000_000,
            bid_value: 1_000_000_000,
            execution_payment: 0,
            blob_kzg_commitments: vec![],
        };
        assert!(matches!(
            construct_bid(&params, dummy_signer),
            Err(BidError::ZeroBlockHash)
        ));
    }

    #[test]
    fn bid_exceeds_balance_rejected() {
        let params = BidParams {
            builder_index: 1,
            builder_balance: 500,
            slot: 100,
            parent_block_hash: [1u8; 32],
            parent_block_root: [1u8; 32],
            committed_block_hash: [2u8; 32],
            prev_randao: [0u8; 32],
            fee_recipient: [0u8; 20],
            gas_limit: 30_000_000,
            bid_value: 1_000, // more than balance
            execution_payment: 0,
            blob_kzg_commitments: vec![],
        };
        assert!(matches!(
            construct_bid(&params, dummy_signer),
            Err(BidError::ValueExceedsBalance { .. })
        ));
    }

    #[test]
    fn valid_bid_constructed() {
        let params = BidParams {
            builder_index: 1,
            builder_balance: 100_000_000_000,
            slot: 100,
            parent_block_hash: [1u8; 32],
            parent_block_root: [1u8; 32],
            committed_block_hash: [2u8; 32],
            prev_randao: [0u8; 32],
            fee_recipient: [0xABu8; 20],
            gas_limit: 30_000_000,
            bid_value: 1_000_000_000,
            execution_payment: 0,
            blob_kzg_commitments: vec![],
        };
        let bid = construct_bid(&params, dummy_signer).unwrap();
        assert_eq!(bid.message.slot, 100);
        assert_eq!(bid.message.value, 1_000_000_000);
        assert_eq!(bid.message.block_hash, [2u8; 32]);
    }
}
