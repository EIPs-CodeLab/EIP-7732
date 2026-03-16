# EIP-7732 — Architecture

## ePBS Slot Timeline

```
t=0s  Slot N starts
       │
       ├─ Proposer broadcasts SignedBeaconBlock
       │    └─ BeaconBlockBody contains SignedExecutionPayloadBid (no full payload)
       │
t=4s  Attestation deadline (validators attest to beacon block)
       │    └─ Only CL state transition needed — EL deferred
       │
       ├─ Builder broadcasts SignedExecutionPayloadEnvelope
       │    └─ Contains full ExecutionPayload + post-state-root
       │
t=6s  PTC attestation deadline  (SECONDS_PER_SLOT * 2 / INTERVALS_PER_SLOT)
       │    └─ 512 PTC members vote: payload_present + blob_data_available
       │    └─ Next proposer has seen payload — can validate (6s window)
       │
t=9s  All validators have had time to validate payload (9s window)
       │
t=12s Slot N+1 starts
```

## Module Map

```
src/
├── lib.rs                        — crate root, public API
│
├── beacon_chain/
│   ├── constants.rs              — PTC_SIZE, domains, delays, thresholds
│   ├── types.rs                  — BuilderIndex, Slot, Gwei, etc.
│   ├── containers.rs             — All new/modified SSZ containers
│   ├── process_payload_bid.rs    — process_execution_payload_bid()
│   ├── process_payload_attestation.rs — process_payload_attestation()
│   └── withdrawals.rs            — Async withdrawal split (CL deducts / EL honors)
│
├── builder/
│   ├── bid.rs                    — construct_bid() — builds SignedExecutionPayloadBid
│   ├── envelope.rs               — construct_envelope() — builds SignedExecutionPayloadEnvelope
│   └── guide.rs                  — HonestBuilder — orchestrates the full lifecycle
│
├── fork_choice/
│   ├── store.rs                  — EpbsStore — tracks Full/Empty/Skipped slot states
│   └── handlers.rs               — on_beacon_block(), on_execution_payload(), on_ptc_threshold()
│
├── p2p/
│   ├── topics.rs                 — 3 new gossip topic constants
│   └── handlers.rs               — P2P message validation and routing
│
└── utils/
    ├── crypto.rs                 — BLS signing root helpers (per domain)
    └── ssz.rs                    — SSZ hash_tree_root stubs

examples/
├── builder_sim/main.rs           — Full ePBS round simulation (make sim-builder)
├── ptc_sim/main.rs               — PTC vote simulation across 4 scenarios (make sim-ptc)
└── cli/main.rs                   — Interactive inspector (make cli)

tests/
├── unit/
│   ├── beacon_chain_test.rs      — process_payload_bid, PTC, withdrawals
│   ├── builder_test.rs           — bid construction, envelope construction, lifecycle
│   └── fork_choice_test.rs       — EpbsStore state transitions
└── integration/
    └── epbs_flow_test.rs         — Full slot end-to-end test
```

## Three Slot States

| State   | Beacon Block | Execution Payload | Builder Payment |
|---------|:---:|:---:|:---:|
| Full    | ✓ | ✓ | Paid |
| Skipped | ✗ | ✗ | N/A |
| Empty   | ✓ | ✗ | **Paid** (unconditional guarantee) |

The key innovation of ePBS is the **Empty** case: the proposer receives payment
from the builder even if the builder never reveals their payload.
This is enforced via the beacon chain balance deduction in
`process_execution_payload_bid`.

## Fork Choice Safety Properties

1. **Proposer unconditional payment** — proposer gets paid in Full and Empty slots.
2. **Builder reveal safety** — if PTC confirms `payload_present=true`, the
   payload will be the canonical head regardless of proposer action.
3. **Builder withhold safety** — if a beacon block is withheld and revealed
   late, the builder is not charged for the bid value.

## What Changed vs Pre-ePBS

| Component | Before | After |
|---|---|---|
| `BeaconBlockBody` | Contains full `ExecutionPayload` | Contains `SignedExecutionPayloadBid` only |
| `ExecutionPayloadHeader` | Tracks latest payload | Renamed to `ExecutionPayloadBid` |
| Block validation hot path | CL + EL validation together (4s) | CL only (4s), EL deferred (9s) |
| Builder trust model | Trusted relay (MEV-Boost) | In-protocol staked builder |
| Withdrawals | Synchronous in `process_execution_payload` | Async: CL deducts, EL honors |
| P2P topics | — | + bid, + payload_attestation, + proposer_preferences |