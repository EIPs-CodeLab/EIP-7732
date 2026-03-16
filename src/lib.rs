//! # EIP-7732 — Enshrined Proposer-Builder Separation (ePBS)
//!
//! Reference implementation of EIP-7732 targeting the Reth execution client.
//!
//! ## Structure
//!
//! - [`beacon_chain`] — SSZ containers, state transition functions, constants
//! - [`builder`]      — Honest builder guide: bid construction & envelope reveal
//! - [`fork_choice`]  — Extended fork-choice store (Full / Empty / Skipped slots)
//! - [`p2p`]          — New gossip topics and message handlers
//! - [`utils`]        — BLS crypto helpers and SSZ stubs
//!
//! ## EIP Reference
//! <https://eips.ethereum.org/EIPS/eip-7732>

pub mod beacon_chain;
pub mod builder;
pub mod fork_choice;
pub mod p2p;
pub mod utils;
