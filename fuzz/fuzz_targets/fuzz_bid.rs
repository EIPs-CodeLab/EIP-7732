#![no_main]

use arbitrary::Arbitrary;
use eip_7732::beacon_chain::{
    containers::{ExecutionPayloadBid, SignedExecutionPayloadBid},
    process_payload_bid::{process_execution_payload_bid, BeaconStateMut},
    types::{BLSPubkey, BuilderIndex, Gwei, KZGCommitment, Slot},
};
use libfuzzer_sys::fuzz_target;
use std::collections::HashMap;

#[derive(Arbitrary, Debug)]
struct Input {
    parent_hash: [u8; 32],
    parent_root: [u8; 32],
    block_hash: [u8; 32],
    prev_randao: [u8; 32],
    fee_recipient: [u8; 20],
    builder_index: BuilderIndex,
    slot: Slot,
    value: Gwei,
    execution_payment: Gwei,
    blobs: Vec<KZGCommitment>,
    signature: [u8; 96],
}

struct MockState {
    balances: HashMap<BuilderIndex, Gwei>,
    pubkeys: HashMap<BuilderIndex, BLSPubkey>,
    slot: Slot,
    latest: [u8; 32],
}

impl BeaconStateMut for MockState {
    fn builder_balance(&self, index: BuilderIndex) -> Option<Gwei> {
        self.balances.get(&index).copied()
    }
    fn builder_pubkey(&self, index: BuilderIndex) -> Option<BLSPubkey> {
        self.pubkeys.get(&index).copied()
    }
    fn deduct_builder_balance(&mut self, index: BuilderIndex, amount: Gwei) {
        if let Some(b) = self.balances.get_mut(&index) {
            *b = b.saturating_sub(amount);
        }
    }
    fn push_pending_payment(
        &mut self,
        _payment: eip_7732::beacon_chain::containers::BuilderPendingPayment,
    ) {
    }
    fn current_slot(&self) -> Slot {
        self.slot
    }
    fn latest_block_hash(&self) -> [u8; 32] {
        self.latest
    }
}

fuzz_target!(|input: Input| {
    let bid = SignedExecutionPayloadBid {
        message: ExecutionPayloadBid {
            parent_block_hash: input.parent_hash,
            parent_block_root: input.parent_root,
            block_hash: input.block_hash,
            prev_randao: input.prev_randao,
            fee_recipient: input.fee_recipient,
            gas_limit: 30_000_000,
            builder_index: input.builder_index,
            slot: input.slot,
            value: input.value,
            execution_payment: input.execution_payment,
            blob_kzg_commitments: input.blobs.into_iter().take(8).collect(),
        },
        signature: input.signature,
    };

    let mut state = MockState {
        balances: HashMap::new(),
        pubkeys: HashMap::new(),
        slot: input.slot,
        latest: bid.message.parent_block_hash,
    };
    state
        .balances
        .insert(bid.message.builder_index, input.value.saturating_add(1));
    state.pubkeys.insert(bid.message.builder_index, [0u8; 48]);

    let _ = process_execution_payload_bid(&mut state, &bid);
});
