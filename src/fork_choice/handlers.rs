use crate::beacon_chain::types::{Root, Slot};
/// EIP-7732 — Fork choice event handlers
///
/// Wires incoming P2P messages to EpbsStore state transitions.
use crate::fork_choice::store::EpbsStore;

/// Called when a SignedBeaconBlock is received and passes initial validation.
pub fn on_beacon_block(store: &mut EpbsStore, slot: Slot, block_root: Root) {
    store.on_beacon_block(slot, block_root);
}

/// Called when a SignedExecutionPayloadEnvelope is received and validated.
pub fn on_execution_payload(store: &mut EpbsStore, slot: Slot) {
    store.on_execution_payload(slot);
}

/// Called when the PTC vote tally for a slot crosses the threshold.
pub fn on_ptc_threshold(store: &mut EpbsStore, slot: Slot, payload_present: bool) {
    store.on_ptc_threshold_reached(slot, payload_present);
}
