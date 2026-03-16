/// EIP-7732 — Fork Choice store extensions
///
/// ePBS introduces three slot states that fork choice must distinguish:
///
///   Full    — beacon block + execution payload both canonical.
///   Skipped — no beacon block for this slot.
///   Empty   — beacon block canonical, but builder did NOT reveal payload.
///
/// Attesters signal their view by setting the `index` field of
/// AttestationData: 0 = no payload present (or past block), 1 = payload present.
///
/// This module defines the store types needed to track these states and
/// enforce the three fork-choice safety properties:
///   1. Proposer unconditional payment  (proposer paid even in Empty slots)
///   2. Builder reveal safety           (timely payload → canonical)
///   3. Builder withhold safety         (withheld beacon block → builder not charged)
///
/// Reference: https://eips.ethereum.org/EIPS/eip-7732#fork-choice-changes
use crate::beacon_chain::types::{Root, Slot};
use std::collections::HashMap;

/// The three possible states for an ePBS slot.
#[derive(Debug, Clone, PartialEq)]
pub enum SlotPayloadStatus {
    /// Both beacon block and execution payload are canonical.
    Full,
    /// No beacon block was seen for this slot.
    Skipped,
    /// Beacon block is canonical but execution payload was NOT revealed.
    Empty,
}

/// Per-slot ePBS fork-choice data tracked in the store.
#[derive(Debug, Clone)]
pub struct EpbsSlotData {
    pub slot: Slot,
    pub beacon_root: Root,
    pub payload_status: SlotPayloadStatus,
    /// True if PTC reached threshold for payload_present.
    pub ptc_full: bool,
    /// True if PTC reached threshold for payload_absent.
    pub ptc_empty: bool,
}

/// Fork-choice store extension for ePBS.
#[derive(Debug, Default)]
pub struct EpbsStore {
    /// Slot → ePBS slot data
    slots: HashMap<Slot, EpbsSlotData>,
    /// Latest finalized slot
    pub finalized_slot: Slot,
}

impl EpbsStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a beacon block for a slot (slot transitions from Skipped → Empty
    /// until a payload arrives).
    pub fn on_beacon_block(&mut self, slot: Slot, beacon_root: Root) {
        self.slots.insert(
            slot,
            EpbsSlotData {
                slot,
                beacon_root,
                payload_status: SlotPayloadStatus::Empty,
                ptc_full: false,
                ptc_empty: false,
            },
        );
    }

    /// Record a revealed execution payload (slot transitions Empty → Full).
    pub fn on_execution_payload(&mut self, slot: Slot) {
        if let Some(data) = self.slots.get_mut(&slot) {
            data.payload_status = SlotPayloadStatus::Full;
        }
    }

    /// Update PTC vote tallies for a slot.
    pub fn on_ptc_threshold_reached(&mut self, slot: Slot, payload_present: bool) {
        if let Some(data) = self.slots.get_mut(&slot) {
            if payload_present {
                data.ptc_full = true;
            } else {
                data.ptc_empty = true;
            }
        }
    }

    pub fn slot_status(&self, slot: Slot) -> SlotPayloadStatus {
        match self.slots.get(&slot) {
            Some(d) => d.payload_status.clone(),
            None => SlotPayloadStatus::Skipped,
        }
    }

    /// Builder reveal safety: if PTC confirmed payload_present, the payload
    /// must be canonical (Full). Returns true if a reorg risk is detected.
    pub fn check_reveal_safety(&self, slot: Slot) -> bool {
        if let Some(data) = self.slots.get(&slot) {
            // If PTC voted full but slot isn't Full → reveal safety violation
            return data.ptc_full && data.payload_status != SlotPayloadStatus::Full;
        }
        false
    }
}
