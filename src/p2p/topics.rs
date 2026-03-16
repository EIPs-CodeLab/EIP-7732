/// EIP-7732 — New P2P gossip topics
///
/// Three new global topics are added to the P2P layer:
///
///   /eth2/<fork_digest>/signed_execution_payload_bid/ssz_snappy
///       Broadcast by builders when submitting a bid.
///
///   /eth2/<fork_digest>/payload_attestation_message/ssz_snappy
///       Broadcast by PTC members attesting to payload timeliness.
///
///   /eth2/<fork_digest>/proposer_preferences/ssz_snappy
///       Broadcast by proposers advertising their builder preferences.
///
/// Reference: https://eips.ethereum.org/EIPS/eip-7732#p2p-changes
pub const TOPIC_SIGNED_EXECUTION_PAYLOAD_BID: &str = "signed_execution_payload_bid";

pub const TOPIC_PAYLOAD_ATTESTATION_MESSAGE: &str = "payload_attestation_message";

pub const TOPIC_PROPOSER_PREFERENCES: &str = "proposer_preferences";

/// Build the full topic string for a given fork digest (4 bytes, hex-encoded).
pub fn topic_for_fork(base: &str, fork_digest_hex: &str) -> String {
    format!("/eth2/{}/{}/ssz_snappy", fork_digest_hex, base)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topic_format() {
        let t = topic_for_fork(TOPIC_SIGNED_EXECUTION_PAYLOAD_BID, "aabbccdd");
        assert_eq!(t, "/eth2/aabbccdd/signed_execution_payload_bid/ssz_snappy");
    }
}
