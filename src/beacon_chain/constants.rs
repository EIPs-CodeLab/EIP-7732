/// EIP-7732 — Beacon chain constants
/// Reference: https://eips.ethereum.org/EIPS/eip-7732#beacon-chain-changes
// ── Index flags ──────────────────────────────────────────────────────────────
/// Bitwise flag indicating a ValidatorIndex should be treated as a BuilderIndex
pub const BUILDER_INDEX_FLAG: u64 = 1 << 40;

// ── Domain types ─────────────────────────────────────────────────────────────
pub const DOMAIN_BEACON_BUILDER: [u8; 4] = [0x0B, 0x00, 0x00, 0x00];
pub const DOMAIN_PTC_ATTESTER: [u8; 4] = [0x0C, 0x00, 0x00, 0x00];
pub const DOMAIN_PROPOSER_PREFERENCES: [u8; 4] = [0x0D, 0x00, 0x00, 0x00];

// ── Misc ──────────────────────────────────────────────────────────────────────
/// Sentinel value indicating the proposer built the payload themselves
pub const BUILDER_INDEX_SELF_BUILD: u64 = u64::MAX;

pub const BUILDER_PAYMENT_THRESHOLD_NUMERATOR: u64 = 6;
pub const BUILDER_PAYMENT_THRESHOLD_DENOMINATOR: u64 = 10;

// ── Withdrawal prefix ─────────────────────────────────────────────────────────
pub const BUILDER_WITHDRAWAL_PREFIX: u8 = 0x03;

// ── Preset ────────────────────────────────────────────────────────────────────
/// Size of the Payload Timeliness Committee
pub const PTC_SIZE: u64 = 512; // 2^9

/// Max payload attestations per block
pub const MAX_PAYLOAD_ATTESTATIONS: usize = 4;

pub const BUILDER_REGISTRY_LIMIT: u64 = 1 << 40;
pub const BUILDER_PENDING_WITHDRAWALS_LIMIT: u64 = 1 << 20;
pub const MAX_BUILDERS_PER_WITHDRAWALS_SWEEP: u64 = 1 << 14;

// ── Time parameters ───────────────────────────────────────────────────────────
/// Minimum delay (in epochs) before a builder can withdraw their stake
pub const MIN_BUILDER_WITHDRAWABILITY_DELAY: u64 = 64; // 2^6

// ── Payload timeliness intervals ──────────────────────────────────────────────
/// Seconds per slot (mainnet)
pub const SECONDS_PER_SLOT: u64 = 12;
pub const INTERVALS_PER_SLOT: u64 = 4;

/// Seconds the next proposer has to validate the revealed payload
pub const NEXT_PROPOSER_VALIDATION_WINDOW: u64 = SECONDS_PER_SLOT * 2 / INTERVALS_PER_SLOT; // 6s

/// Seconds other validators have to validate the revealed payload  
pub const ATTESTERS_VALIDATION_WINDOW: u64 = SECONDS_PER_SLOT * 3 / INTERVALS_PER_SLOT; // 9s
