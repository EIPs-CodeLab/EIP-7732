# EIP-7732 — References & Resources

## Primary Specification

| Resource | Link |
|---|---|
| EIP-7732 (canonical) | https://eips.ethereum.org/EIPS/eip-7732 |
| Ethereum Magicians discussion | https://ethereum-magicians.org/t/eip-7732-enshrined-proposer-builder-separation-epbs/19634 |
| Consensus specs (Gloas / ePBS) | https://github.com/ethereum/consensus-specs/tree/dev/specs/gloas |
| Beacon chain changes | https://github.com/ethereum/consensus-specs/blob/c94138e73e0e70eb4b27f9be4d4e9325fa1aebf7/specs/gloas/beacon-chain.md |
| Fork choice changes | https://github.com/ethereum/consensus-specs/blob/c94138e73e0e70eb4b27f9be4d4e9325fa1aebf7/specs/gloas/fork-choice.md |
| P2P interface changes | https://github.com/ethereum/consensus-specs/blob/c94138e73e0e70eb4b27f9be4d4e9325fa1aebf7/specs/gloas/p2p-interface.md |
| Honest validator guide | https://github.com/ethereum/consensus-specs/blob/c94138e73e0e70eb4b27f9be4d4e9325fa1aebf7/specs/gloas/validator.md |
| Honest builder guide | https://github.com/ethereum/consensus-specs/blob/c94138e73e0e70eb4b27f9be4d4e9325fa1aebf7/specs/gloas/builder.md |
| Fork logic | https://github.com/ethereum/consensus-specs/blob/c94138e73e0e70eb4b27f9be4d4e9325fa1aebf7/specs/gloas/fork.md |

## Related EIPs

| EIP | Title | Relation |
|---|---|---|
| EIP-7805 | FOCIL — Fork-Choice Enforced Inclusion Lists | Fully compatible with ePBS per spec §Compatible Designs |
| EIP-4844 | Shard Blob Transactions | Blob KZG commitments move to ExecutionPayloadEnvelope |
| EIP-7702 | Set EOA account code | AA interaction with builder payments |

## Client Implementations (Reference)

| Client | Language | Status |
|---|---|---|
| Prysm (ePBS branch) | Go | In progress |
| Lighthouse | Rust | Tracking |
| Lodestar | TypeScript | Tracking |

## Background Reading

- [ePBS — Unbundling PBS in ETH (Barnabé Monnot)](https://barnabe.substack.com/p/epbs)
- [Why ePBS? (Francesco D'Amato)](https://ethresear.ch/t/why-enshrine-pbs-a-viable-path-forward/19594)
- [PBS simulations — free option analysis](https://ethresear.ch/t/payload-timeliness-committee-ptc-an-epbs-design/16054)
- [MEV-Boost architecture (for comparison)](https://boost.flashbots.net/)
- [SSZ specification](https://github.com/ethereum/consensus-specs/blob/dev/ssz/simple-serialize.md)
- [BLS12-381 signatures](https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#bls-signatures)

## Devnet Tracking

| Devnet | Status | Notes |
|---|---|---|
| epbs-devnet-0 | Active (early 2026) | First multi-client ePBS devnet |
| Glamsterdam testnet | Planned Q2 2026 | |
| Glamsterdam mainnet | Planned H2 2026 | |