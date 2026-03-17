/// Integration test — full ePBS round
///
/// Tests the complete slot lifecycle end-to-end:
/// bid → beacon block → envelope → PTC → fork choice

use blst::min_pk::SecretKey;
use eip_7732::{
    beacon_chain::{
        containers::ExecutionPayload,
        process_payload_bid::{process_execution_payload_bid, BeaconStateMut},
        types::*,
    },
    builder::{
        bid::{construct_bid, BidParams},
        envelope::{construct_envelope, EnvelopeParams},
        guide::HonestBuilder,
    },
    fork_choice::{handlers as fc, store::{EpbsStore, SlotPayloadStatus}},
    utils::crypto,
};

fn test_secret_key() -> SecretKey {
    SecretKey::from_bytes(&[9u8; 32]).expect(\"valid sk\")
}

fn bls_signer(msg: &[u8]) -> Result<[u8; 96], String> {
    Ok(crypto::bls_sign(&test_secret_key(), msg))
}

struct SimpleState {
    balance:      u64,
    payments:     Vec<eip_7732::beacon_chain::containers::BuilderPendingPayment>,
    slot:         u64,
    latest_hash:  [u8; 32],
    pubkey:       BLSPubkey,
}

impl BeaconStateMut for SimpleState {
    fn builder_balance(&self, _: u64) -> Option<u64> { Some(self.balance) }
    fn deduct_builder_balance(&mut self, _: u64, a: u64) { self.balance -= a; }
    fn push_pending_payment(&mut self, p: eip_7732::beacon_chain::containers::BuilderPendingPayment) {
        self.payments.push(p);
    }
    fn current_slot(&self) -> u64 { self.slot }
    fn latest_block_hash(&self) -> [u8; 32] { self.latest_hash }
    fn builder_pubkey(&self, _: u64) -> Option<BLSPubkey> { Some(self.pubkey) }
}

#[test]
fn full_slot_happy_path() {
    let slot              = 200u64;
    let committed_hash    = [0xAAu8; 32];
    let parent_hash       = [0x01u8; 32];
    let beacon_block_root = [0xBBu8; 32];

    // 1. Builder submits bid
    let mut builder = HonestBuilder::new(1, 32_000_000_000);
    let bid = builder.submit_bid(
        BidParams {
            builder_index:        1,
            builder_balance:      32_000_000_000,
            slot,
            parent_block_hash:    parent_hash,
            parent_block_root:    [0x02u8; 32],
            committed_block_hash: committed_hash,
            prev_randao:          [0u8; 32],
            fee_recipient:        [0xFEu8; 20],
            gas_limit:            30_000_000,
            bid_value:            500_000_000,
            execution_payment:    0,
            blob_kzg_commitments: vec![],
        },
        bls_signer,
    ).unwrap();

    // 2. Beacon state processes bid
    let mut state = SimpleState {
        balance:     32_000_000_000,
        payments:    vec![],
        slot,
        latest_hash: parent_hash,
        pubkey: test_secret_key().sk_to_pk().to_bytes(),
    };
    process_execution_payload_bid(&mut state, &bid).unwrap();
    assert_eq!(state.payments.len(), 1);
    assert_eq!(state.payments[0].weight, 500_000_000);

    // 3. Fork choice records beacon block
    let mut store = EpbsStore::new();
    fc::on_beacon_block(&mut store, slot, beacon_block_root);
    assert_eq!(store.slot_status(slot), SlotPayloadStatus::Empty);

    // 4. Builder reveals envelope
    builder.on_bid_included(slot);
    assert!(builder.is_ready_to_reveal());

    builder.reveal_envelope(
        EnvelopeParams {
            payload: ExecutionPayload {
                block_hash:    committed_hash,
                parent_hash,
                fee_recipient: [0xFEu8; 20],
                gas_used:      10_000_000,
                gas_limit:     30_000_000,
                timestamp:     1_700_000_000,
                extra_data:    vec![],
                transactions:  vec![],
                withdrawals:   vec![],
            },
            execution_requests:   vec![],
            builder_index:        1,
            beacon_block_root,
            slot,
            post_state_root:      [0xCCu8; 32],
            committed_hash,
            expected_withdrawals: vec![],
        },
        bls_signer,
    ).unwrap();

    // 5. Fork choice records payload
    fc::on_execution_payload(&mut store, slot);
    assert_eq!(store.slot_status(slot), SlotPayloadStatus::Full);

    // 6. PTC votes present
    fc::on_ptc_threshold(&mut store, slot, true);
    builder.on_ptc_result(slot, true, 500_000_000);

    assert!(!store.check_reveal_safety(slot));
    assert!(matches!(
        builder.slot_state,
        eip_7732::builder::guide::BuilderSlotState::Paid { amount: 500_000_000, .. }
    ));
}

#[test]
fn empty_slot_builder_not_paid() {
    let slot = 201u64;
    let mut store = EpbsStore::new();

    fc::on_beacon_block(&mut store, slot, [0xBBu8; 32]);
    // No envelope revealed
    fc::on_ptc_threshold(&mut store, slot, false);

    assert_eq!(store.slot_status(slot), SlotPayloadStatus::Empty);
    assert!(store.check_reveal_safety(slot) == false); // no reveal safety issue — builder just didn't reveal
}
