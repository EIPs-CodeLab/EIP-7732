/// Unit tests — fork choice store

use eip_7732::fork_choice::store::{EpbsStore, SlotPayloadStatus};

#[test]
fn slot_starts_skipped() {
    let store = EpbsStore::new();
    assert_eq!(store.slot_status(999), SlotPayloadStatus::Skipped);
}

#[test]
fn beacon_block_marks_empty() {
    let mut store = EpbsStore::new();
    store.on_beacon_block(100, [0xBBu8; 32]);
    assert_eq!(store.slot_status(100), SlotPayloadStatus::Empty);
}

#[test]
fn payload_reveal_marks_full() {
    let mut store = EpbsStore::new();
    store.on_beacon_block(100, [0xBBu8; 32]);
    store.on_execution_payload(100);
    assert_eq!(store.slot_status(100), SlotPayloadStatus::Full);
}

#[test]
fn payload_without_beacon_block_is_noop() {
    let mut store = EpbsStore::new();
    store.on_execution_payload(100); // no beacon block first
    assert_eq!(store.slot_status(100), SlotPayloadStatus::Skipped);
}

#[test]
fn reveal_safety_not_violated_when_full() {
    let mut store = EpbsStore::new();
    store.on_beacon_block(100, [0u8; 32]);
    store.on_execution_payload(100);
    store.on_ptc_threshold_reached(100, true);
    assert!(!store.check_reveal_safety(100));
}

#[test]
fn reveal_safety_violated_when_ptc_full_but_empty() {
    let mut store = EpbsStore::new();
    store.on_beacon_block(100, [0u8; 32]);
    // No payload revealed — slot stays Empty
    store.on_ptc_threshold_reached(100, true); // PTC says present!
    assert!(store.check_reveal_safety(100));
}

#[test]
fn multiple_slots_tracked_independently() {
    let mut store = EpbsStore::new();
    store.on_beacon_block(100, [0x01u8; 32]);
    store.on_beacon_block(101, [0x02u8; 32]);
    store.on_execution_payload(100);
    // 100 = Full, 101 = Empty, 102 = Skipped
    assert_eq!(store.slot_status(100), SlotPayloadStatus::Full);
    assert_eq!(store.slot_status(101), SlotPayloadStatus::Empty);
    assert_eq!(store.slot_status(102), SlotPayloadStatus::Skipped);
}