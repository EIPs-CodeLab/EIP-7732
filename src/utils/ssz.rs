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

/// Minimal hash_tree_root stand-in: SHA256 over JSON serialization of the message.
/// This keeps signing deterministic and domain-separated until full SSZ is wired.
pub fn hash_tree_root_json<T: Serialize>(value: &T) -> [u8; 32] {
    let encoded = serde_json::to_vec(value).expect("serialize message");
    let mut hasher = Sha256::new();
    hasher.update(encoded);
    hasher.finalize().into()
}

/// signing_root = hash_tree_root(message) mixed with domain.
pub fn signing_root_json<T: Serialize>(message: &T, domain: [u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(hash_tree_root_json(message));
    hasher.update(domain);
    hasher.finalize().into()
}
