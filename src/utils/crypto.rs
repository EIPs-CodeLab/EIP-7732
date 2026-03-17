// PR #1: BLS signing helpers for bids/envelopes/PTC
use blst::min_pk::{PublicKey, SecretKey, Signature};
use blst::BLST_ERROR;

use crate::beacon_chain::types::BLSPubkey;

pub const ETH_DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";

pub fn bls_verify(pubkey: &BLSPubkey, message: &[u8], signature: &[u8; 96]) -> Result<(), String> {
    let pk = PublicKey::from_bytes(pubkey).map_err(|e| format!("invalid pubkey: {:?}", e))?;
    let sig =
        Signature::from_bytes(signature).map_err(|e| format!("invalid signature: {:?}", e))?;
    match sig.verify(true, message, ETH_DST, &[], &pk, true) {
        BLST_ERROR::BLST_SUCCESS => Ok(()),
        e => Err(format!("bls verify failed: {:?}", e)),
    }
}

pub fn bls_verify_aggregate(
    pubkeys: &[BLSPubkey],
    message: &[u8],
    signature: &[u8; 96],
) -> Result<(), String> {
    let sig =
        Signature::from_bytes(signature).map_err(|e| format!("invalid signature: {:?}", e))?;
    let mut pubs = Vec::with_capacity(pubkeys.len());
    for pk_bytes in pubkeys {
        pubs.push(PublicKey::from_bytes(pk_bytes).map_err(|e| format!("invalid pubkey: {:?}", e))?);
    }
    let pub_refs: Vec<&PublicKey> = pubs.iter().collect();
    match sig.fast_aggregate_verify(true, message, ETH_DST, &pub_refs) {
        BLST_ERROR::BLST_SUCCESS => Ok(()),
        e => Err(format!("aggregate bls verify failed: {:?}", e)),
    }
}

/// Convenience for tests: sign message with a fixed secret key.
pub fn bls_sign(sk: &SecretKey, message: &[u8]) -> [u8; 96] {
    sk.sign(message, ETH_DST, &[]).to_bytes()
}
