/// EIP-7732 — SSZ containers
/// All containers directly mirror the spec definitions.
/// Reference: https://eips.ethereum.org/EIPS/eip-7732#containers
use crate::beacon_chain::constants::MAX_PAYLOAD_ATTESTATIONS;
use crate::beacon_chain::types::*;
use serde::{Deserialize, Serialize};

// ── New containers ────────────────────────────────────────────────────────────

/// In-protocol staked builder registered in the beacon state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Builder {
    #[serde(with = "serde_arrays::bytes48")]
    pub pubkey: BLSPubkey,
    pub version: u8,
    pub execution_address: ExecutionAddress,
    pub balance: Gwei,
    pub deposit_epoch: Epoch,
    pub withdrawable_epoch: Epoch,
}

/// Pending payment from a builder to a proposer, tracked in beacon state
/// until the corresponding execution payload is processed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuilderPendingPayment {
    pub weight: Gwei,
    pub withdrawal: BuilderPendingWithdrawal,
}

/// A pending withdrawal to be credited on the execution layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuilderPendingWithdrawal {
    pub fee_recipient: ExecutionAddress,
    pub amount: Gwei,
    pub builder_index: BuilderIndex,
}

/// Attestation data broadcast by a PTC member.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayloadAttestationData {
    pub beacon_block_root: Root,
    pub slot: Slot,
    /// True if the builder revealed the payload in time.
    pub payload_present: bool,
    /// True if blob data was available according to the PTC member's view.
    pub blob_data_available: bool,
}

/// Aggregated PTC attestation included in a BeaconBlock.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayloadAttestation {
    /// Bitvector of length PTC_SIZE (512)
    pub aggregation_bits: Vec<bool>,
    pub data: PayloadAttestationData,
    #[serde(with = "serde_arrays::bytes96")]
    pub signature: BLSSignature,
}

/// Individual (unaggregated) PTC attestation broadcast on P2P.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayloadAttestationMessage {
    pub validator_index: ValidatorIndex,
    pub data: PayloadAttestationData,
    #[serde(with = "serde_arrays::bytes96")]
    pub signature: BLSSignature,
}

/// Indexed variant used for verification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexedPayloadAttestation {
    pub attesting_indices: Vec<ValidatorIndex>,
    pub data: PayloadAttestationData,
    #[serde(with = "serde_arrays::bytes96")]
    pub signature: BLSSignature,
}

/// A builder's signed commitment to reveal an execution payload for a slot.
/// The `ExecutionPayloadHeader` of pre-ePBS is renamed/restructured into this.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionPayloadBid {
    pub parent_block_hash: Hash32,
    pub parent_block_root: Root,
    /// The blockhash the builder commits to revealing.
    pub block_hash: Hash32,
    pub prev_randao: Bytes32,
    pub fee_recipient: ExecutionAddress,
    pub gas_limit: u64,
    pub builder_index: BuilderIndex,
    pub slot: Slot,
    /// Value (in Gwei) to be paid to the beacon block proposer.
    pub value: Gwei,
    pub execution_payment: Gwei,
    #[serde(with = "serde_arrays::vec_bytes48")]
    pub blob_kzg_commitments: Vec<KZGCommitment>,
}

/// Signed wrapper around ExecutionPayloadBid — broadcast on P2P and
/// included in BeaconBlockBody.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignedExecutionPayloadBid {
    pub message: ExecutionPayloadBid,
    #[serde(with = "serde_arrays::bytes96")]
    pub signature: BLSSignature,
}

/// The full execution payload revealed by the builder after the beacon block
/// has been broadcast. Includes the post-state-transition beacon state root
/// so the CL can verify correctness.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionPayloadEnvelope {
    /// The full EL execution payload.
    pub payload: ExecutionPayload,
    pub execution_requests: Vec<u8>, // opaque bytes; full type in EL spec
    pub builder_index: BuilderIndex,
    pub beacon_block_root: Root,
    pub slot: Slot,
    /// Hash tree root of the CL beacon state AFTER processing this payload.
    pub state_root: Root,
}

/// Signed wrapper around ExecutionPayloadEnvelope — broadcast on P2P.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignedExecutionPayloadEnvelope {
    pub message: ExecutionPayloadEnvelope,
    #[serde(with = "serde_arrays::bytes96")]
    pub signature: BLSSignature,
}

/// Minimal stand-in for the EL ExecutionPayload type.
/// Replace with a full SSZ container when integrating with an EL client.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionPayload {
    pub block_hash: Hash32,
    pub parent_hash: Hash32,
    pub fee_recipient: ExecutionAddress,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub timestamp: u64,
    pub extra_data: Vec<u8>,
    pub transactions: Vec<Vec<u8>>,
    pub withdrawals: Vec<Withdrawal>,
}

/// A withdrawal from the beacon chain to be credited on the EL.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Withdrawal {
    pub index: WithdrawalIndex,
    pub validator_index: ValidatorIndex,
    pub address: ExecutionAddress,
    pub amount: Gwei,
}

// ── Modified BeaconBlockBody ──────────────────────────────────────────────────

/// Relevant ePBS fields added to / changed in BeaconBlockBody.
/// Fields unchanged from pre-Gloas are omitted for brevity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BeaconBlockBodyEpbs {
    // … all pre-existing fields omitted (randao_reveal, eth1_data, etc.) …

    // REMOVED: execution_payload, blob_kzg_commitments, execution_requests
    // (moved to ExecutionPayloadEnvelope)
    /// [New in EIP-7732] Builder's signed commitment to reveal a payload.
    pub signed_execution_payload_bid: SignedExecutionPayloadBid,

    /// [New in EIP-7732] PTC attestations from the *previous* slot.
    pub payload_attestations: Vec<PayloadAttestation>, // max MAX_PAYLOAD_ATTESTATIONS
}

impl BeaconBlockBodyEpbs {
    pub fn validate_payload_attestations_count(&self) -> bool {
        self.payload_attestations.len() <= MAX_PAYLOAD_ATTESTATIONS
    }
}

// ── Serde helpers for fixed-size byte arrays > 32 bytes ──────────────────────
mod serde_arrays {
    use serde::{de, ser::SerializeSeq, Deserialize, Deserializer, Serializer};

    pub mod bytes48 {
        use super::*;

        pub fn serialize<S>(value: &[u8; 48], serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_bytes(value)
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 48], D::Error>
        where
            D: Deserializer<'de>,
        {
            let bytes: Vec<u8> = Deserialize::deserialize(deserializer)?;
            if bytes.len() != 48 {
                return Err(de::Error::invalid_length(bytes.len(), &"48 bytes"));
            }
            let mut arr = [0u8; 48];
            arr.copy_from_slice(&bytes);
            Ok(arr)
        }
    }

    pub mod bytes96 {
        use super::*;

        pub fn serialize<S>(value: &[u8; 96], serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_bytes(value)
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 96], D::Error>
        where
            D: Deserializer<'de>,
        {
            let bytes: Vec<u8> = Deserialize::deserialize(deserializer)?;
            if bytes.len() != 96 {
                return Err(de::Error::invalid_length(bytes.len(), &"96 bytes"));
            }
            let mut arr = [0u8; 96];
            arr.copy_from_slice(&bytes);
            Ok(arr)
        }
    }

    pub mod vec_bytes48 {
        use super::*;

        pub fn serialize<S>(value: &Vec<[u8; 48]>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut seq = serializer.serialize_seq(Some(value.len()))?;
            for arr in value {
                // serialize as byte array to avoid requiring Serialize on [u8; 48]
                seq.serialize_element(&arr.as_slice())?;
            }
            seq.end()
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<[u8; 48]>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let items: Vec<Vec<u8>> = Deserialize::deserialize(deserializer)?;
            let mut out = Vec::with_capacity(items.len());
            for bytes in items {
                if bytes.len() != 48 {
                    return Err(de::Error::invalid_length(bytes.len(), &"48 bytes"));
                }
                let mut arr = [0u8; 48];
                arr.copy_from_slice(&bytes);
                out.push(arr);
            }
            Ok(out)
        }
    }
}
