/// Unit tests — builder bid and envelope construction

use eip_7732::builder::{
    bid::{construct_bid, BidError, BidParams},
    envelope::{construct_envelope, EnvelopeError, EnvelopeParams},
    guide::{BuilderSlotState, HonestBuilder},
};
use eip_7732::beacon_chain::containers::{ExecutionPayload, Withdrawal};

fn dummy_signer(_: &[u8]) -> Result<[u8; 96], String> { Ok([0u8; 96]) }
fn failing_signer(_: &[u8]) -> Result<[u8; 96], String> { Err("key not found".into()) }

fn base_bid_params() -> BidParams {
    BidParams {
        builder_index:        1,
        builder_balance:      32_000_000_000,
        slot:                 100,
        parent_block_hash:    [0x01u8; 32],
        parent_block_root:    [0x02u8; 32],
        committed_block_hash: [0xAAu8; 32],
        prev_randao:          [0u8; 32],
        fee_recipient:        [0xFEu8; 20],
        gas_limit:            30_000_000,
        bid_value:            1_000_000_000,
        execution_payment:    0,
        blob_kzg_commitments: vec![],
    }
}

fn base_envelope_params() -> EnvelopeParams {
    EnvelopeParams {
        payload: ExecutionPayload {
            block_hash:    [0xAAu8; 32],
            parent_hash:   [0x01u8; 32],
            fee_recipient: [0xFEu8; 20],
            gas_used:      1_000_000,
            gas_limit:     30_000_000,
            timestamp:     1_700_000_000,
            extra_data:    vec![],
            transactions:  vec![],
            withdrawals:   vec![],
        },
        execution_requests:   vec![],
        builder_index:        1,
        beacon_block_root:    [0xBBu8; 32],
        slot:                 100,
        post_state_root:      [0xCCu8; 32],
        committed_hash:       [0xAAu8; 32],
        expected_withdrawals: vec![],
    }
}

// ── Bid tests ─────────────────────────────────────────────────────────────────

#[test]
fn valid_bid_fields_preserved() {
    let p   = base_bid_params();
    let bid = construct_bid(&p, dummy_signer).unwrap();
    assert_eq!(bid.message.slot,          p.slot);
    assert_eq!(bid.message.builder_index, p.builder_index);
    assert_eq!(bid.message.value,         p.bid_value);
    assert_eq!(bid.message.block_hash,    p.committed_block_hash);
    assert_eq!(bid.message.fee_recipient, p.fee_recipient);
}

#[test]
fn bid_signing_failure_propagated() {
    let p   = base_bid_params();
    let err = construct_bid(&p, failing_signer).unwrap_err();
    assert!(matches!(err, BidError::SigningFailed(_)));
}

#[test]
fn bid_value_zero_accepted() {
    let mut p = base_bid_params();
    p.bid_value = 0; // free bid — valid
    assert!(construct_bid(&p, dummy_signer).is_ok());
}

#[test]
fn bid_with_blobs() {
    let mut p = base_bid_params();
    p.blob_kzg_commitments = vec![[0xCCu8; 48], [0xDDu8; 48]];
    let bid = construct_bid(&p, dummy_signer).unwrap();
    assert_eq!(bid.message.blob_kzg_commitments.len(), 2);
}

// ── Envelope tests ────────────────────────────────────────────────────────────

#[test]
fn valid_envelope_fields_preserved() {
    let p   = base_envelope_params();
    let env = construct_envelope(&p, dummy_signer).unwrap();
    assert_eq!(env.message.slot,               p.slot);
    assert_eq!(env.message.builder_index,      p.builder_index);
    assert_eq!(env.message.state_root,         p.post_state_root);
    assert_eq!(env.message.payload.block_hash, p.committed_hash);
}

#[test]
fn envelope_withdrawal_mismatch_rejected() {
    let mut p = base_envelope_params();
    p.expected_withdrawals = vec![Withdrawal {
        index: 0, validator_index: 1, address: [0u8; 20], amount: 1_000,
    }];
    // payload.withdrawals is empty — mismatch
    let err = construct_envelope(&p, dummy_signer).unwrap_err();
    assert!(matches!(err, EnvelopeError::WithdrawalsMismatch));
}

#[test]
fn envelope_withdrawal_match_accepted() {
    let mut p = base_envelope_params();
    let w = Withdrawal { index: 0, validator_index: 1, address: [0u8; 20], amount: 1_000 };
    p.expected_withdrawals = vec![w.clone()];
    p.payload.withdrawals  = vec![w];
    assert!(construct_envelope(&p, dummy_signer).is_ok());
}

// ── HonestBuilder lifecycle tests ─────────────────────────────────────────────

#[test]
fn builder_lifecycle_happy_path() {
    let mut builder = HonestBuilder::new(1, 32_000_000_000);
    assert_eq!(builder.slot_state, BuilderSlotState::Idle);

    // Submit bid
    builder.submit_bid(base_bid_params(), dummy_signer).unwrap();
    assert!(matches!(builder.slot_state, BuilderSlotState::BidSubmitted { .. }));

    // Bid included
    builder.on_bid_included(100);
    assert!(builder.is_ready_to_reveal());

    // Reveal envelope
    builder.reveal_envelope(base_envelope_params(), dummy_signer).unwrap();
    assert!(matches!(builder.slot_state, BuilderSlotState::EnvelopeRevealed { .. }));

    // PTC says present
    builder.on_ptc_result(100, true, 1_000_000_000);
    assert!(matches!(builder.slot_state, BuilderSlotState::Paid { amount: 1_000_000_000, .. }));
}

#[test]
fn builder_lifecycle_withheld_payload() {
    let mut builder = HonestBuilder::new(1, 32_000_000_000);
    builder.submit_bid(base_bid_params(), dummy_signer).unwrap();
    builder.on_bid_included(100);
    builder.reveal_envelope(base_envelope_params(), dummy_signer).unwrap();
    builder.on_ptc_result(100, false, 1_000_000_000);
    assert!(matches!(builder.slot_state, BuilderSlotState::Unpaid { .. }));
}