/// EIP-7732 — CLI tool
///
/// Interactive command-line inspector for ePBS data structures.
/// Run with: `make cli` or `cargo run --bin epbs-cli -- --help`
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "epbs-cli",
    about = "EIP-7732 ePBS inspector & simulator",
    version = "0.1.0"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print all EIP-7732 constants
    Constants,

    /// Show the structure of all new SSZ containers
    Containers,

    /// Inspect a mock SignedExecutionPayloadBid for a given slot
    Bid {
        #[arg(long, default_value = "100")]
        slot: u64,
        #[arg(long, default_value = "1")]
        builder: u64,
        #[arg(long, default_value = "1000000000")]
        value: u64,
    },

    /// Show the three ePBS slot states and their fork-choice meaning
    SlotStates,

    /// Explain the PTC threshold calculation
    Ptc {
        #[arg(long, default_value = "400")]
        yes_votes: usize,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Constants => print_constants(),
        Commands::Containers => print_containers(),
        Commands::Bid {
            slot,
            builder,
            value,
        } => print_bid(slot, builder, value),
        Commands::SlotStates => print_slot_states(),
        Commands::Ptc { yes_votes } => print_ptc(yes_votes),
    }
}

fn print_constants() {
    use eip_7732::beacon_chain::constants::*;
    println!("EIP-7732 Constants\n");
    println!(
        "  BUILDER_INDEX_FLAG                 = 0x{:X}",
        BUILDER_INDEX_FLAG
    );
    println!(
        "  BUILDER_INDEX_SELF_BUILD            = {} (u64::MAX)",
        BUILDER_INDEX_SELF_BUILD
    );
    println!(
        "  BUILDER_PAYMENT_THRESHOLD           = {}/{}",
        BUILDER_PAYMENT_THRESHOLD_NUMERATOR, BUILDER_PAYMENT_THRESHOLD_DENOMINATOR
    );
    println!("  PTC_SIZE                            = {}", PTC_SIZE);
    println!(
        "  MAX_PAYLOAD_ATTESTATIONS            = {}",
        MAX_PAYLOAD_ATTESTATIONS
    );
    println!(
        "  BUILDER_REGISTRY_LIMIT              = {}",
        BUILDER_REGISTRY_LIMIT
    );
    println!(
        "  BUILDER_PENDING_WITHDRAWALS_LIMIT   = {}",
        BUILDER_PENDING_WITHDRAWALS_LIMIT
    );
    println!(
        "  MAX_BUILDERS_PER_WITHDRAWALS_SWEEP  = {}",
        MAX_BUILDERS_PER_WITHDRAWALS_SWEEP
    );
    println!(
        "  MIN_BUILDER_WITHDRAWABILITY_DELAY   = {} epochs",
        MIN_BUILDER_WITHDRAWABILITY_DELAY
    );
    println!(
        "  NEXT_PROPOSER_VALIDATION_WINDOW     = {}s",
        NEXT_PROPOSER_VALIDATION_WINDOW
    );
    println!(
        "  ATTESTERS_VALIDATION_WINDOW         = {}s",
        ATTESTERS_VALIDATION_WINDOW
    );
    println!();
    println!(
        "  DOMAIN_BEACON_BUILDER      = 0x{}",
        hex::encode(DOMAIN_BEACON_BUILDER)
    );
    println!(
        "  DOMAIN_PTC_ATTESTER        = 0x{}",
        hex::encode(DOMAIN_PTC_ATTESTER)
    );
    println!(
        "  DOMAIN_PROPOSER_PREFERENCES= 0x{}",
        hex::encode(DOMAIN_PROPOSER_PREFERENCES)
    );
    println!(
        "  BUILDER_WITHDRAWAL_PREFIX  = 0x{:02X}",
        BUILDER_WITHDRAWAL_PREFIX
    );
}

fn print_containers() {
    println!("EIP-7732 New SSZ Containers\n");
    println!("  Builder                      — in-protocol staked builder in BeaconState");
    println!("  BuilderPendingPayment        — pending proposer payment after bid accepted");
    println!("  BuilderPendingWithdrawal     — withdrawal queued to execution layer");
    println!("  PayloadAttestationData       — PTC attestation data (slot, present, blobs)");
    println!("  PayloadAttestation           — aggregated PTC attestation (in BeaconBlock)");
    println!("  PayloadAttestationMessage    — unaggregated PTC message (P2P)");
    println!("  IndexedPayloadAttestation    — indexed variant for verification");
    println!("  ExecutionPayloadBid          — builder's commitment (replaces ExecPayloadHeader)");
    println!("  SignedExecutionPayloadBid    — signed bid broadcast on P2P");
    println!("  ExecutionPayloadEnvelope     — full payload + CL post-state-root");
    println!("  SignedExecutionPayloadEnvelope — signed envelope broadcast on P2P");
    println!();
    println!("Modified containers:");
    println!("  BeaconBlockBody  — execution_payload REMOVED, signed_execution_payload_bid ADDED");
    println!(
        "  BeaconState      — builders[], builder_pending_payments[], latest_block_hash ADDED"
    );
}

fn print_bid(slot: u64, builder: u64, value: u64) {
    println!("Mock SignedExecutionPayloadBid\n");
    println!("  slot          : {}", slot);
    println!("  builder_index : {}", builder);
    println!(
        "  value         : {} Gwei ({:.6} ETH)",
        value,
        value as f64 / 1e9
    );
    println!("  block_hash    : 0x{}", hex::encode([0xAAu8; 32]));
    println!("  fee_recipient : 0x{}", hex::encode([0xFEu8; 20]));
    println!("  gas_limit     : 30000000");
    println!("  blobs         : 0");
    println!("  signature     : 0x{}", hex::encode([0u8; 96]));
}

fn print_slot_states() {
    println!("EIP-7732 Slot States\n");
    println!("  FULL    — Beacon block + execution payload both canonical.");
    println!("            Builder was paid. Normal operation.\n");
    println!("  SKIPPED — No beacon block for this slot.");
    println!("            No builder involved. Validator offline.\n");
    println!("  EMPTY   — Beacon block canonical, builder did NOT reveal payload.");
    println!("            Proposer still receives payment (unconditional payment guarantee).");
    println!("            Builder forfeits their bid value.");
    println!("            Withdrawals are stalled until next payload.\n");
    println!("Fork choice uses the attestation `index` field to encode view:");
    println!("  index=0 → no payload present (Skipped or Empty)");
    println!("  index=1 → payload present (Full)");
}

fn print_ptc(yes_votes: usize) {
    use eip_7732::beacon_chain::constants::{
        BUILDER_PAYMENT_THRESHOLD_DENOMINATOR, BUILDER_PAYMENT_THRESHOLD_NUMERATOR, PTC_SIZE,
    };
    let threshold = (PTC_SIZE as usize * BUILDER_PAYMENT_THRESHOLD_NUMERATOR as usize)
        / BUILDER_PAYMENT_THRESHOLD_DENOMINATOR as usize;
    let no_votes = PTC_SIZE as usize - yes_votes;

    println!("PTC Vote Analysis\n");
    println!("  PTC size   : {}", PTC_SIZE);
    println!(
        "  Threshold  : {} ({}/{})",
        threshold, BUILDER_PAYMENT_THRESHOLD_NUMERATOR, BUILDER_PAYMENT_THRESHOLD_DENOMINATOR
    );
    println!("  Yes votes  : {}", yes_votes);
    println!("  No votes   : {}", no_votes);

    if yes_votes >= threshold {
        println!("  Outcome    : PAYLOAD PRESENT — builder paid, slot Full");
    } else if no_votes >= threshold {
        println!("  Outcome    : PAYLOAD ABSENT  — builder not paid, slot Empty");
    } else {
        println!("  Outcome    : BELOW THRESHOLD — ambiguous, fork choice decides");
    }
}
