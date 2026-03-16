/// EIP-7732 — Builder simulation
///
/// Simulates a complete ePBS round for a single slot:
///
///   1. Builder constructs a payload and submits a bid.
///   2. Proposer receives the bid and includes it in the beacon block.
///   3. Builder sees their bid included and reveals the payload envelope.
///   4. PTC votes on payload timeliness.
///   5. Fork choice records the slot as Full / Empty based on PTC outcome.
///
/// Run with: `make sim-builder`
use eip_7732::{
    beacon_chain::{
        constants::PTC_SIZE,
        containers::ExecutionPayload,
        types::{BuilderIndex, Gwei},
    },
    builder::{
        bid::BidParams,
        envelope::EnvelopeParams,
        guide::{BuilderSlotState, HonestBuilder},
    },
    fork_choice::{handlers as fc, store::EpbsStore},
};

fn dummy_signer(_: &[u8]) -> Result<[u8; 96], String> {
    Ok([0u8; 96])
}

fn main() {
    println!("═══════════════════════════════════════════════");
    println!("  EIP-7732 ePBS — Builder Simulation");
    println!("  Slot: 100");
    println!("═══════════════════════════════════════════════\n");

    // ── Setup ──────────────────────────────────────────────────────────────────
    let builder_index: BuilderIndex = 42;
    let builder_balance: Gwei = 32_000_000_000; // 32 ETH in Gwei
    let slot = 100u64;
    let parent_hash = [0x01u8; 32];
    let parent_root = [0x02u8; 32];
    let committed_hash = [0xAAu8; 32];
    let beacon_block_root = [0xBBu8; 32];
    let post_state_root = [0xCCu8; 32];

    let mut builder = HonestBuilder::new(builder_index, builder_balance);
    let mut store = EpbsStore::new();

    // ── Step 1: Submit bid ─────────────────────────────────────────────────────
    println!("▶ Step 1: Builder constructs payload and submits bid");
    let bid_params = BidParams {
        builder_index,
        builder_balance,
        slot,
        parent_block_hash: parent_hash,
        parent_block_root: parent_root,
        committed_block_hash: committed_hash,
        prev_randao: [0u8; 32],
        fee_recipient: [0xFEu8; 20],
        gas_limit: 30_000_000,
        bid_value: 100_000_000, // 0.1 ETH
        execution_payment: 0,
        blob_kzg_commitments: vec![],
    };

    let signed_bid = builder.submit_bid(bid_params, dummy_signer).unwrap();
    println!(
        "   ✓ Bid submitted  slot={} builder={} value={} Gwei",
        signed_bid.message.slot, signed_bid.message.builder_index, signed_bid.message.value
    );
    println!(
        "   ✓ Committed hash: 0x{}",
        hex::encode(signed_bid.message.block_hash)
    );

    // ── Step 2: Proposer includes bid in beacon block ──────────────────────────
    println!("\n▶ Step 2: Proposer includes bid in BeaconBlockBody");
    fc::on_beacon_block(&mut store, slot, beacon_block_root);
    builder.on_bid_included(slot);
    println!("   ✓ Beacon block produced  slot={}", slot);
    println!("   ✓ Slot status: {:?}", store.slot_status(slot));
    assert!(builder.is_ready_to_reveal());

    // ── Step 3: Builder reveals envelope ──────────────────────────────────────
    println!("\n▶ Step 3: Builder reveals ExecutionPayloadEnvelope");
    let payload = ExecutionPayload {
        block_hash: committed_hash,
        parent_hash,
        fee_recipient: [0xFEu8; 20],
        gas_used: 15_000_000,
        gas_limit: 30_000_000,
        timestamp: 1_700_000_000 + slot * 12,
        extra_data: b"EIPs-CodeLab".to_vec(),
        transactions: vec![vec![0x02, 0x00, 0x01]], // 1 dummy tx
        withdrawals: vec![],
    };

    let envelope_params = EnvelopeParams {
        payload,
        execution_requests: vec![],
        builder_index,
        beacon_block_root,
        slot,
        post_state_root,
        committed_hash,
        expected_withdrawals: vec![],
    };

    let envelope = builder
        .reveal_envelope(envelope_params, dummy_signer)
        .unwrap();
    fc::on_execution_payload(&mut store, slot);

    println!(
        "   ✓ Envelope revealed  slot={} builder={}",
        envelope.message.slot, envelope.message.builder_index
    );
    println!(
        "   ✓ Payload hash:  0x{}",
        hex::encode(envelope.message.payload.block_hash)
    );
    println!(
        "   ✓ State root:    0x{}",
        hex::encode(envelope.message.state_root)
    );

    // ── Step 4: PTC votes ──────────────────────────────────────────────────────
    println!("\n▶ Step 4: Payload Timeliness Committee votes");
    let ptc_yes_votes = 400usize;
    let ptc_no_votes = PTC_SIZE as usize - ptc_yes_votes;
    let threshold = (PTC_SIZE as f64 * 0.6) as usize;
    let payload_present = ptc_yes_votes >= threshold;

    println!("   PTC size      : {}", PTC_SIZE);
    println!("   Threshold     : {} votes (60%)", threshold);
    println!("   Votes present : {}", ptc_yes_votes);
    println!("   Votes absent  : {}", ptc_no_votes);
    println!("   Payload present: {}", payload_present);

    fc::on_ptc_threshold(&mut store, slot, payload_present);
    builder.on_ptc_result(slot, payload_present, 100_000_000);

    // ── Step 5: Final state ────────────────────────────────────────────────────
    println!("\n▶ Step 5: Final slot state");
    println!("   Fork choice slot status : {:?}", store.slot_status(slot));
    println!("   Builder slot state      : {:?}", builder.slot_state);
    println!(
        "   Reveal safety violation : {}",
        store.check_reveal_safety(slot)
    );

    println!("\n═══════════════════════════════════════════════");
    match &builder.slot_state {
        BuilderSlotState::Paid { amount, .. } => println!("  ✓ Builder PAID  {} Gwei", amount),
        BuilderSlotState::Unpaid { .. } => {
            println!("  ✗ Builder UNPAID — payload not seen in time")
        }
        other => println!("  ? Unexpected state: {:?}", other),
    }
    println!("═══════════════════════════════════════════════");
}
