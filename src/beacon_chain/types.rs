/// EIP-7732 — Type aliases
/// Reference: https://eips.ethereum.org/EIPS/eip-7732#types
pub type BuilderIndex = u64;
pub type Slot = u64;
pub type Epoch = u64;
pub type Gwei = u64;
pub type Hash32 = [u8; 32];
pub type Root = [u8; 32];
pub type BLSPubkey = [u8; 48];
pub type BLSSignature = [u8; 96];
pub type ExecutionAddress = [u8; 20];
pub type Bytes32 = [u8; 32];
pub type KZGCommitment = [u8; 48];
pub type WithdrawalIndex = u64;
pub type ValidatorIndex = u64;
