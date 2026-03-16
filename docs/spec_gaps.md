# EIP-7732 — Spec Gaps & Implementation Notes

This document tracks every place where the EIP specification is ambiguous,
underspecified, or where this implementation made an explicit judgment call.
It is updated as new gaps are discovered during implementation.

---

## 1. BLS signature verification — domain separation

**Spec says:** Builders sign with `DOMAIN_BEACON_BUILDER`, PTC members sign
with `DOMAIN_PTC_ATTESTER`.

**Gap:** The spec does not define a full `compute_domain` procedure for these
new domains in the EIP itself. It defers to the consensus-specs repository
(`specs/gloas/beacon-chain.md`).

**Judgment call:** This implementation uses the standard Ethereum
`signing_root = sha256(hash_tree_root(msg) || padded_domain)` pattern
consistent with existing domains. Wire to `blst` for production.

---

## 2. `process_execution_payload` — when is it called?

**Spec says:** `process_execution_payload` is now called as a "separate helper
when receiving a `SignedExecutionPayloadEnvelope` on the P2P layer."

**Gap:** The spec does not specify the exact call site in the state transition
pipeline or whether it runs synchronously with P2P message delivery.

**Judgment call:** This implementation treats envelope processing as an
asynchronous event triggered by the P2P handler, updating fork-choice state
immediately but deferring CL state update (state_root verification) to a
queued job. This matches how Reth handles async CL inputs.

---

## 3. `AttestationData.index` field reuse

**Spec says:** The unused `index` field in `AttestationData` is repurposed:
`0` = no payload present, `1` = payload present.

**Gap:** The spec does not define what values other than `0` and `1` mean,
or whether attestations with `index > 1` should be rejected.

**Judgment call:** This implementation rejects attestations with `index > 1`
as malformed. This is stricter than the spec but safer.

---

## 4. Withdrawal stalling during Empty slots

**Spec says:** "In these cases, the consensus layer does not process any more
withdrawals until an execution payload has fulfilled the outstanding ones."

**Gap:** The spec doesn't define the maximum number of consecutive Empty slots
before withdrawal processing has downstream effects (e.g., validator exits
that depend on withdrawals being processed).

**Judgment call:** No cap is enforced in this implementation. A real client
would need to monitor consecutive empty slots and alert operators.

---

## 5. Builder deposit via `BUILDER_WITHDRAWAL_PREFIX`

**Spec says:** "deposit requests from new validator pubkeys, with withdrawal
credentials starting with the prefix `BUILDER_WITHDRAWAL_PREFIX`. These
deposits are immediately added to the beacon chain and processed as new builders."

**Gap:** The spec does not define the minimum deposit amount for builders
(it mentions "as little as 1ETH" in the abstract but this is not a formal
constant in the spec body).

**Judgment call:** This implementation enforces no minimum — it mirrors the
spec text loosely. A minimum of `1 ETH = 1_000_000_000 Gwei` is noted in
comments.

---

## 6. PTC equivocation handling

**Spec says:** "There is no penalty for PTC nor payload equivocation."

**Gap:** The spec acknowledges this is a conscious simplification. A future
EIP may add slashing conditions.

**Judgment call:** This implementation logs equivocations but does not penalize.
The `EpbsStore` tracks both `ptc_full` and `ptc_empty` flags independently,
allowing detection of a split-view scenario for monitoring.

---

## 7. `BUILDER_INDEX_FLAG` — not a real ValidatorIndex

**Spec says:** `BUILDER_INDEX_FLAG = uint64(2**40)` is a bitwise flag used to
indicate that a `ValidatorIndex` should be treated as a `BuilderIndex`.

**Gap:** The spec does not define which operations use this flag or how callers
are expected to detect and strip it.

**Judgment call:** This implementation treats `BuilderIndex` and `ValidatorIndex`
as distinct type aliases (`u64`). The flag is defined as a constant but no
automatic detection/stripping is implemented — the caller is responsible for
checking before indexing into the builder registry.

---

## 8. `state_root` in ExecutionPayloadEnvelope — circular dependency

**Spec says:** The envelope commits to the CL post-state-transition beacon
state root after processing the payload.

**Gap:** This creates a potential circular dependency: to compute the state
root you need to process the payload; but the envelope must commit to the
state root before it is broadcast.

**Resolution (from spec):** The builder runs the CL state transition function
themselves on the current state + the payload to compute `state_root`, then
includes it in the envelope. Validators verify by re-running the same
transition and comparing.

**Implementation note:** This requires the builder to implement the full CL
state transition function or use a trusted CL client API. This is out of scope
for this reference implementation — the `post_state_root` is a parameter passed
by the caller.