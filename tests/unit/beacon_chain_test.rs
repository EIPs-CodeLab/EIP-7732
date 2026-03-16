/// Unit tests — beacon chain state transition functions

use eip_7732::beacon_chain::{
    constants::*,
    containers::*,
    process_payload_bid::{process_execution_payload_bid, BeaconStateMut, PayloadBidError},
    process_payload_attestation::{process_payload_attestation, BeaconStateRead, PayloadAttestationError},
    types::*,
    withdrawals::{process_withdrawals_consensus, verify_payload_withdrawals, WithdrawalError, WithdrawalState},
};

// ── Mock beacon state ──────────────────────────────────────────────────────────

struct MockState {
    builders:           std::collections::HashMap<BuilderIndex, Gwei>,
    pending_payments:   Vec<BuilderPendingPayment>,
    slot:               Slot,
    latest_block_hash:  Hash32,
    expected_withdrawals: Vec<Withdrawal>,
}

impl MockState {
    fn new(slot: Slot) -> Self {
        let mut builders = std::collections::HashMap::new();
        builders.insert(1u64, 32_000_000_000u64);
        Self {
            builders,
            pending_payments:    vec![],
            slot,
            latest_block_hash:  [0x01u8; 32],
            expected_withdrawals: vec![],
        }
    }
}

impl BeaconStateMut for MockState {
    fn builder_balance(&self, index: BuilderIndex) -> Option<Gwei> {
        self.builders.get(&index).copied()
    }
    fn deduct_builder_balance(&mut self, index: BuilderIndex, amount: Gwei) {
        if let Some(bal) = self.builders.get_mut(&index) {
            *bal -= amount;
        }
    }
    fn push_pending_payment(&mut self, payment: BuilderPendingPayment) {
        self.pending_payments.push(payment);
    }
    fn current_slot(&self) -> Slot { self.slot }
    fn latest_block_hash(&self) -> Hash32 { self.latest_block_hash }
}

impl BeaconStateRead for MockState {
    fn parent_slot(&self) -> Slot { self.slot - 1 }
    fn get_ptc(&self, _slot: Slot) -> Vec<ValidatorIndex> {
        (0..PTC_SIZE as u64).collect()
    }
    fn parent_beacon_block_root(&self) -> [u8; 32] { [0xBBu8; 32] }
}

impl WithdrawalState for MockState {
    fn payload_expected_withdrawals(&self) -> &[Withdrawal] {
        &self.expected_withdrawals
    }
    fn deduct_and_commit_withdrawals(&mut self, w: Vec<Withdrawal>) {
        self.expected_withdrawals = w;
    }
    fn clear_expected_withdrawals(&mut self) {
        self.expected_withdrawals.clear();
    }
}

fn make_valid_bid(slot: Slot) -> SignedExecutionPayloadBid {
    SignedExecutionPayloadBid {
        message: ExecutionPayloadBid {
            parent_block_hash:    [0x01u8; 32],
            parent_block_root:    [0x02u8; 32],
            block_hash:           [0xAAu8; 32],
            prev_randao:          [0u8; 32],
            fee_recipient:        [0xFEu8; 20],
            gas_limit:            30_000_000,
            builder_index:        1,
            slot,
            value:                1_000_000_000,
            execution_payment:    0,
            blob_kzg_commitments: vec![],
        },
        signature: [0u8; 96],
    }
}

// ── process_execution_payload_bid tests ───────────────────────────────────────

#[test]
fn valid_bid_accepted() {
    let mut state = MockState::new(100);
    let bid       = make_valid_bid(100);
    assert!(process_execution_payload_bid(&mut state, &bid).is_ok());
    assert_eq!(state.pending_payments.len(), 1);
    assert_eq!(state.pending_payments[0].weight, 1_000_000_000);
}

#[test]
fn bid_wrong_slot_rejected() {
    let mut state = MockState::new(100);
    let bid       = make_valid_bid(99); // wrong slot
    let err = process_execution_payload_bid(&mut state, &bid).unwrap_err();
    assert!(matches!(err, PayloadBidError::SlotMismatch { .. }));
}

#[test]
fn bid_unknown_builder_rejected() {
    let mut state = MockState::new(100);
    let mut bid   = make_valid_bid(100);
    bid.message.builder_index = 999; // unknown
    let err = process_execution_payload_bid(&mut state, &bid).unwrap_err();
    assert!(matches!(err, PayloadBidError::BuilderNotFound(999)));
}

#[test]
fn bid_insufficient_balance_rejected() {
    let mut state = MockState::new(100);
    state.builders.insert(1, 500); // too low
    let mut bid = make_valid_bid(100);
    bid.message.value = 1_000; // bid > balance
    let err = process_execution_payload_bid(&mut state, &bid).unwrap_err();
    assert!(matches!(err, PayloadBidError::InsufficientBalance { .. }));
}

#[test]
fn bid_parent_hash_mismatch_rejected() {
    let mut state         = MockState::new(100);
    state.latest_block_hash = [0xFFu8; 32]; // different
    let bid               = make_valid_bid(100);
    let err = process_execution_payload_bid(&mut state, &bid).unwrap_err();
    assert!(matches!(err, PayloadBidError::ParentHashMismatch));
}

#[test]
fn builder_balance_deducted_after_bid() {
    let mut state     = MockState::new(100);
    let initial_bal   = *state.builders.get(&1).unwrap();
    let bid           = make_valid_bid(100);
    let bid_value     = bid.message.value;
    process_execution_payload_bid(&mut state, &bid).unwrap();
    let new_bal = *state.builders.get(&1).unwrap();
    assert_eq!(new_bal, initial_bal - bid_value);
}

// ── Withdrawal tests ──────────────────────────────────────────────────────────

#[test]
fn withdrawals_committed_on_consensus() {
    let mut state = MockState::new(100);
    let ws = vec![Withdrawal {
        index: 0, validator_index: 1, address: [0u8; 20], amount: 1_000_000,
    }];
    process_withdrawals_consensus(&mut state, ws.clone()).unwrap();
    assert_eq!(state.payload_expected_withdrawals(), ws.as_slice());
}

#[test]
fn withdrawal_mismatch_rejected() {
    let mut state = MockState::new(100);
    let committed = vec![Withdrawal {
        index: 0, validator_index: 1, address: [0u8; 20], amount: 1_000_000,
    }];
    process_withdrawals_consensus(&mut state, committed).unwrap();
    let wrong = vec![Withdrawal {
        index: 0, validator_index: 1, address: [0u8; 20], amount: 999,
    }];
    let err = verify_payload_withdrawals(&mut state, &wrong).unwrap_err();
    assert!(matches!(err, WithdrawalError::WithdrawalsMismatch));
}

#[test]
fn withdrawal_cleared_after_valid_payload() {
    let mut state = MockState::new(100);
    let ws = vec![Withdrawal {
        index: 0, validator_index: 1, address: [0u8; 20], amount: 1_000_000,
    }];
    process_withdrawals_consensus(&mut state, ws.clone()).unwrap();
    verify_payload_withdrawals(&mut state, &ws).unwrap();
    assert!(state.payload_expected_withdrawals().is_empty());
}

// ── PTC attestation tests ─────────────────────────────────────────────────────

#[test]
fn valid_ptc_attestation_accepted() {
    let state = MockState::new(100);
    let att = PayloadAttestation {
        aggregation_bits: vec![true; PTC_SIZE as usize],
        data: PayloadAttestationData {
            beacon_block_root:   [0xBBu8; 32],
            slot:                99, // parent slot
            payload_present:     true,
            blob_data_available: true,
        },
        signature: [0u8; 96],
    };
    assert!(process_payload_attestation(&state, &att).is_ok());
}

#[test]
fn ptc_wrong_bits_length_rejected() {
    let state = MockState::new(100);
    let att = PayloadAttestation {
        aggregation_bits: vec![true; 10], // wrong length
        data: PayloadAttestationData {
            beacon_block_root:   [0xBBu8; 32],
            slot:                99,
            payload_present:     true,
            blob_data_available: true,
        },
        signature: [0u8; 96],
    };
    let err = process_payload_attestation(&state, &att).unwrap_err();
    assert!(matches!(err, PayloadAttestationError::WrongBitsLength(10)));
}

#[test]
fn ptc_wrong_slot_rejected() {
    let state = MockState::new(100); // parent_slot = 99
    let att = PayloadAttestation {
        aggregation_bits: vec![false; PTC_SIZE as usize],
        data: PayloadAttestationData {
            beacon_block_root:   [0u8; 32],
            slot:                50, // wrong
            payload_present:     false,
            blob_data_available: false,
        },
        signature: [0u8; 96],
    };
    let err = process_payload_attestation(&state, &att).unwrap_err();
    assert!(matches!(err, PayloadAttestationError::WrongSlot { .. }));
}