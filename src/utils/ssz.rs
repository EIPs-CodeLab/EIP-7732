use serde::Serialize;
use sha2::{Digest, Sha256};

/// Compute an EIP-4881-style domain by padding the 4-byte domain type with zeros.
/// Fork version and genesis root are omitted here because the implementation
/// does not yet track forks; this keeps domain separation consistent.
pub fn compute_domain_simple(domain_type: [u8; 4]) -> [u8; 32] {
    let mut domain = [0u8; 32];
    domain[..4].copy_from_slice(&domain_type);
    domain
}

/// Compute a signing root by hashing serialized message bytes plus domain.
/// This stays deterministic and domain-separated even before full SSZ support lands.
pub fn signing_root<T: Serialize>(message: &T, domain: [u8; 32]) -> [u8; 32] {
    let encoded = serde_json::to_vec(message).expect("serialize message for signing");
    let mut hasher = Sha256::new();
    hasher.update(encoded);
    hasher.update(domain);
    hasher.finalize().into()
}
