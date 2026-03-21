# EIP-7732 — Enshrined Proposer-Builder Separation (ePBS)

> **EIPs-CodeLab** · EIP specifications translated into working code  
> Status: 🟡 Draft · Target fork: **Glamsterdam**

---

## Overview

EIP-7732 fundamentally changes how an Ethereum block is validated by **decoupling execution validation from consensus validation** — both logically and temporally.

Before this EIP, every beacon block included a full `ExecutionPayload`. Validators had only ~4 seconds (the attesting deadline) to:
- Run the consensus state transition
- Run the execution state transition
- Check blob data availability
- Evaluate the new chain head

ePBS splits this into two separate objects revealed at different times:

| Object | Who broadcasts | When |
|---|---|---|
| `BeaconBlock` (with `SignedExecutionPayloadBid`) | Consensus proposer | t = 0s |
| `SignedExecutionPayloadEnvelope` | Builder | t = 1s – 4s |

This means validators only run the **consensus state transition** in the critical 4-second window. Execution validation is deferred to the rest of the slot.

### Three slot states

Under ePBS, every slot is in exactly one of three states:

```
Full    → beacon block + execution payload both on-chain  (normal case)
Skipped → no beacon block at all                          (missed slot)
Empty   → beacon block on-chain, builder withheld payload (builder penalty)
```

### Key new entities

- **Builder** — a new in-protocol staked entity (≥ 1 ETH stake, no validator duties)
- **PTC (Payload Timeliness Committee)** — 512-validator subset that attests to whether the builder revealed the payload in time
- **`SignedExecutionPayloadBid`** — replaces `ExecutionPayload` in the `BeaconBlockBody`; a signed commitment from the builder to reveal a specific payload for a specific value

---

## Repository Structure

```
EIP-7732/
├── src/
│   ├── lib.rs                  # Crate root
│   ├── types/
│   │   └── mod.rs              # All SSZ containers from the spec
│   ├── state/
│   │   └── mod.rs              # State transition functions
│   ├── fork_choice/
│   │   └── mod.rs              # Modified LMD-GHOST for ePBS
│   ├── p2p/
│   │   └── mod.rs              # New gossip topics & validation rules
│   ├── builder/
│   │   └── mod.rs              # Honest builder guide
│   └── utils/
│       └── mod.rs              # PTC selection, slot status helpers
├── tests/
│   ├── unit/
│   │   └── mod.rs              # Unit tests for each state transition
│   ├── integration/            # End-to-end slot lifecycle tests
│   └── fuzzing/                # Fuzz targets (proptest)
├── cli/
│   └── main.rs                 # epbs-cli: bid / verify / sim / status
├── benches/
│   └── payload_processing.rs   # Criterion benchmarks
├── configs/
│   └── gloas_preset.json       # Mainnet-equivalent constants
├── docs/
│   └── TECHNICAL.md            # Deep-dive article (see below)
├── Cargo.toml
├── Makefile
└── README.md
```

---

## Prerequisites

- **Rust** ≥ 1.78 (stable recommended; repo currently built with 1.87)  
  Install: https://rustup.rs
- **Cargo-fuzz** + **nightly toolchain** (only if you run fuzz targets)

```bash
rustup update stable
rustup toolchain install nightly        # required for make fuzz
cargo install cargo-fuzz                # required for make fuzz
```

---

## Getting Started

### Build
```bash
git clone https://github.com/EIPs-CodeLab/EIP-7732
cd EIP-7732
make build            # release build
# or: cargo build      # debug build
```

### Test matrix
```bash
make test             # all tests
make test-unit        # unit tests only
make test-integration # integration tests only
```

### Lint / format
```bash
make fmt              # rustfmt
make fmt-check        # rustfmt --check
make lint             # cargo clippy --all-targets --all-features -D warnings
```

### Fuzz (requires nightly + cargo-fuzz)
```bash
make fuzz             # runs fuzz_bid and fuzz_envelope (~30s each)
make fuzz-bid         # bid harness only (~30s)
make fuzz-envelope    # envelope harness only (~30s)
```
Fuzz setup:
```bash
rustup toolchain install nightly
cargo install cargo-fuzz
cd fuzz && cargo +nightly fuzz run fuzz_bid
```

### Examples & CLI
```bash
make cli              # prints epbs-cli --help
make sim-builder      # single-slot builder simulation
make sim-ptc          # PTC voting scenarios

# direct CLI usage
cargo run --release --bin epbs-cli -- bid --slot 100 --builder-index 0 --value 1000000000
cargo run --release --bin epbs-cli -- verify --bid /path/to/bid.json
```

---

## Implementation Status

| Component | Status | Notes |
|---|---|---|
| SSZ type definitions | ✅ Complete | All containers from consensus-specs |
| `process_execution_payload_bid` | ✅ Complete | Balance check, pending payment queue |
| `process_payload_attestation` | ✅ Complete | PTC threshold, availability bitvector |
| `process_execution_payload` | ✅ Complete | Blockhash check, withdrawal match |
| `process_builder_pending_payments` | ✅ Complete | Epoch-boundary withhold penalty |
| Fork choice (LMD-GHOST) | ✅ Complete | Full/Empty/Skipped ordering |
| P2P gossip validation | ✅ Complete | Deduplication + validation rules |
| Honest builder guide | ✅ Complete | Bid creation, envelope reveal |
| BLS signature verification | ✅ Complete | Backed by `blst` crate |
| `hash_tree_root` | ✅ Complete | SSZ helpers implemented |
| Full beacon state integration | ✅ Complete | End-to-end ePBS flow wired |

---

## Security Considerations

The following security properties are analysed in the EIP and preserved in this implementation:

### Free option problem
A rational builder can withhold their payload if it is profitable to do so (e.g. to benefit from a timing advantage). The spec mitigates this through:
- `process_builder_pending_payments`: builders who withhold receive only `BUILDER_PAYMENT_THRESHOLD_NUMERATOR / BUILDER_PAYMENT_THRESHOLD_DENOMINATOR` (6/10) of their committed value back — they lose 40%.
- The PTC attests to payload presence independently of the builder, so the proposer is paid regardless.

### Builder reveal safety
If a builder reveals a valid payload in time (as attested by the PTC), the fork-choice implementation **must prefer the Full slot** over an Empty slot at equal validator weight. This is enforced in `fork_choice::ForkChoiceStore::find_best_child` via `slot_status_ord`.

### Builder withhold safety
If the beacon block containing a builder's bid is withheld and revealed late, the builder is **not penalised** — the `MIN_SEED_LOOKAHEAD` window ensures builders can determine whether their slot is canonical before committing.

### Malicious PTC
The expected time for an attacker controlling 35% of stake to gain majority control of the PTC is 205,000 years. The `PTC_SIZE = 512` constant is the primary parameter governing this.

---

## References

| Resource | Link |
|---|---|
| EIP-7732 specification | https://eips.ethereum.org/EIPS/eip-7732 |
| Ethereum Magicians discussion | https://ethereum-magicians.org/t/eip-7732-enshrined-proposer-builder-separation-epbs/19634 |
| consensus-specs (Gloas fork) | https://github.com/ethereum/consensus-specs/tree/c94138e73e0e70eb4b27f9be4d4e9325fa1aebf7/specs/gloas |
| Beacon chain changes | https://github.com/ethereum/consensus-specs/blob/c94138e73e0e70eb4b27f9be4d4e9325fa1aebf7/specs/gloas/beacon-chain.md |
| Fork choice changes | https://github.com/ethereum/consensus-specs/blob/c94138e73e0e70eb4b27f9be4d4e9325fa1aebf7/specs/gloas/fork-choice.md |
| Honest builder guide | https://github.com/ethereum/consensus-specs/blob/c94138e73e0e70eb4b27f9be4d4e9325fa1aebf7/specs/gloas/builder.md |
| P2P interface changes | https://github.com/ethereum/consensus-specs/blob/c94138e73e0e70eb4b27f9be4d4e9325fa1aebf7/specs/gloas/p2p-interface.md |
| EIPs-CodeLab org | https://github.com/EIPs-CodeLab |

---

## Technical Deep-Dive

*For a detailed explanation of how ePBS works, the design decisions behind each component, and how this implementation maps to the spec, see [docs/TECHNICAL.md](docs/TECHNICAL.md).*

---

## License

Apache-2.0 — see [LICENSE](LICENSE)
