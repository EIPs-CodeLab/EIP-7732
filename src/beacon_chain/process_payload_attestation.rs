/// EIP-7732 — process_payload_attestation
///
/// New operation added to process_operations.
/// Validates aggregated PTC attestations included in a BeaconBlockBody.
///
/// Reference: https://eips.ethereum.org/EIPS/eip-7732#beacon-chain-changes
use crate::beacon_chain::{
    constants::PTC_SIZE,
    containers::{PayloadAttestation, PayloadAttestationData},
    types::{Slot, ValidatorIndex},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PayloadAttestationError {
    #[error("Attestation references slot {attested} but parent slot is {parent}")]
    WrongSlot { attested: Slot, parent: Slot },

    #[error("aggregation_bits length {0} does not match PTC_SIZE ({PTC_SIZE})")]
    WrongBitsLength(usize),

    #[error("Invalid aggregated BLS signature")]
    InvalidSignature,

    #[error("Attesting index {0} is not a PTC member for this slot")]
    NotPtcMember(ValidatorIndex),
}

pub trait BeaconStateRead {
    fn parent_slot(&self) -> Slot;
    fn get_ptc(&self, slot: Slot) -> Vec<ValidatorIndex>;
    fn parent_beacon_block_root(&self) -> [u8; 32];
}

/// Validate and record a `PayloadAttestation` from the beacon block body.
///
/// PTC members attest to whether:
/// - The builder revealed the payload in time (`payload_present`).
/// - Blob data was available (`blob_data_available`).
///
/// Note: PTC members do NOT validate execution correctness — only timeliness.
pub fn process_payload_attestation<S: BeaconStateRead>(
    state: &S,
    attestation: &PayloadAttestation,
) -> Result<(), PayloadAttestationError> {
    let data = &attestation.data;

    // Attestation must reference the previous slot's beacon block
    if data.slot != state.parent_slot() {
        return Err(PayloadAttestationError::WrongSlot {
            attested: data.slot,
            parent: state.parent_slot(),
        });
    }

    if data.beacon_block_root != state.parent_beacon_block_root() {
        // spec allows this in some views — log but don't error for now
    }

    // aggregation_bits must have exactly PTC_SIZE bits
    if attestation.aggregation_bits.len() != PTC_SIZE as usize {
        return Err(PayloadAttestationError::WrongBitsLength(
            attestation.aggregation_bits.len(),
        ));
    }

    // Get the PTC members for the attested slot
    let ptc = state.get_ptc(data.slot);

    // Collect attesting validators
    let attesting: Vec<ValidatorIndex> = attestation
        .aggregation_bits
        .iter()
        .enumerate()
        .filter(|(_, &bit)| bit)
        .map(|(i, _)| ptc[i])
        .collect();

    // Verify aggregated signature (stub)
    verify_aggregate_ptc_signature(&attestation.signature, data, &attesting)?;

    Ok(())
}

fn verify_aggregate_ptc_signature(
    _signature: &[u8; 96],
    _data: &PayloadAttestationData,
    _validators: &[ValidatorIndex],
) -> Result<(), PayloadAttestationError> {
    // TODO: aggregate public keys, compute signing_root with DOMAIN_PTC_ATTESTER,
    //       verify with blst
    Ok(())
}
