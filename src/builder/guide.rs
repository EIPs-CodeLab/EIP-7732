/// EIP-7732 — Honest Builder Guide (high-level orchestration)
///
/// This module mirrors the structure of the spec's honest builder guide:
/// https://github.com/ethereum/consensus-specs/blob/c94138e73e0e70eb4b27f9be4d4e9325fa1aebf7/specs/gloas/builder.md
///
/// The honest builder lifecycle per slot:
///
///  [Slot start]
///    1. Build a full execution payload optimistically.
///    2. Submit a SignedExecutionPayloadBid immediately.
///
///  [On receiving BeaconBlock with own bid]
///    3. Verify the bid is included correctly.
///    4. Reveal the SignedExecutionPayloadEnvelope ASAP (before PTC deadline).
///
///  [After PTC deadline - SECONDS_PER_SLOT * 2 / INTERVALS_PER_SLOT seconds]
///    5. If PTC voted payload_present = true → payment guaranteed.
///    6. If PTC voted payload_present = false → builder was not paid.
///
/// Key safety rule: The builder MUST NOT reveal a withheld payload after the
/// PTC votes `payload_present = false`. Revealing late costs the builder
/// their bid value with no benefit.
use crate::beacon_chain::{
    containers::{SignedExecutionPayloadBid, SignedExecutionPayloadEnvelope},
    types::{BuilderIndex, Gwei, Slot},
};
use crate::builder::{
    bid::{construct_bid, BidError, BidParams},
    envelope::{construct_envelope, EnvelopeError, EnvelopeParams},
};

/// Current state of the builder's lifecycle for a given slot.
#[derive(Debug, Clone, PartialEq)]
pub enum BuilderSlotState {
    /// Builder has not yet submitted a bid for this slot.
    Idle,
    /// Bid submitted, waiting for beacon block confirmation.
    BidSubmitted { slot: Slot, bid_value: Gwei },
    /// Beacon block seen with our bid included — envelope should be revealed now.
    BidIncluded { slot: Slot },
    /// Envelope revealed — waiting for PTC result.
    EnvelopeRevealed { slot: Slot },
    /// PTC confirmed payload_present = true — payment guaranteed.
    Paid { slot: Slot, amount: Gwei },
    /// PTC confirmed payload_present = false — builder was not paid.
    Unpaid { slot: Slot },
}

pub struct HonestBuilder {
    pub builder_index: BuilderIndex,
    pub balance: Gwei,
    pub slot_state: BuilderSlotState,
}

impl HonestBuilder {
    pub fn new(index: BuilderIndex, balance: Gwei) -> Self {
        Self {
            builder_index: index,
            balance,
            slot_state: BuilderSlotState::Idle,
        }
    }

    /// Step 1+2: Build payload and immediately submit a bid.
    pub fn submit_bid(
        &mut self,
        params: BidParams,
        sign_fn: impl Fn(&[u8]) -> Result<[u8; 96], String>,
    ) -> Result<SignedExecutionPayloadBid, BidError> {
        let bid = construct_bid(&params, sign_fn)?;
        self.slot_state = BuilderSlotState::BidSubmitted {
            slot: params.slot,
            bid_value: params.bid_value,
        };
        Ok(bid)
    }

    /// Step 3: Called when we see our bid included in the beacon block.
    pub fn on_bid_included(&mut self, slot: Slot) {
        self.slot_state = BuilderSlotState::BidIncluded { slot };
    }

    /// Step 4: Reveal the envelope as fast as possible.
    pub fn reveal_envelope(
        &mut self,
        params: EnvelopeParams,
        sign_fn: impl Fn(&[u8]) -> Result<[u8; 96], String>,
    ) -> Result<SignedExecutionPayloadEnvelope, EnvelopeError> {
        let envelope = construct_envelope(&params, sign_fn)?;
        self.slot_state = BuilderSlotState::EnvelopeRevealed { slot: params.slot };
        Ok(envelope)
    }

    /// Step 5/6: Update state based on PTC outcome.
    pub fn on_ptc_result(&mut self, slot: Slot, payload_present: bool, bid_value: Gwei) {
        self.slot_state = if payload_present {
            BuilderSlotState::Paid {
                slot,
                amount: bid_value,
            }
        } else {
            BuilderSlotState::Unpaid { slot }
        };
    }

    pub fn is_ready_to_reveal(&self) -> bool {
        matches!(self.slot_state, BuilderSlotState::BidIncluded { .. })
    }
}
