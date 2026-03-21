#![no_main]

use arbitrary::Arbitrary;
use eip_7732::{
    beacon_chain::containers::{ExecutionPayload, Withdrawal},
    beacon_chain::types::{BuilderIndex, ExecutionAddress, Hash32, Slot, ValidatorIndex},
    builder::envelope::{construct_envelope, EnvelopeParams},
};
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct WithdrawalInput {
    index: u64,
    validator_index: ValidatorIndex,
    address: ExecutionAddress,
    amount: u64,
}

#[derive(Arbitrary, Debug)]
struct Input {
    block_hash: Hash32,
    parent_hash: Hash32,
    fee_recipient: ExecutionAddress,
    gas_used: u64,
    gas_limit: u64,
    timestamp: u64,
    extra_data: Vec<u8>,
    transactions: Vec<Vec<u8>>,
    withdrawals: Vec<WithdrawalInput>,
    execution_requests: Vec<u8>,
    builder_index: BuilderIndex,
    beacon_block_root: Hash32,
    slot: Slot,
    post_state_root: Hash32,
}

fn dummy_signer(_: &[u8]) -> Result<[u8; 96], String> { Ok([0u8; 96]) }

fuzz_target!(|input: Input| {
    let withdrawals: Vec<Withdrawal> = input
        .withdrawals
        .into_iter()
        .map(|w| Withdrawal {
            index: w.index,
            validator_index: w.validator_index,
            address: w.address,
            amount: w.amount,
        })
        .collect();

    let payload = ExecutionPayload {
        block_hash: input.block_hash,
        parent_hash: input.parent_hash,
        fee_recipient: input.fee_recipient,
        gas_used: input.gas_used,
        gas_limit: input.gas_limit.max(1),
        timestamp: input.timestamp,
        extra_data: input.extra_data,
        transactions: input.transactions,
        withdrawals: withdrawals.clone(),
    };

    // Keep the envelope internally consistent to exercise downstream logic.
    let params = EnvelopeParams {
        payload,
        execution_requests: input.execution_requests,
        builder_index: input.builder_index,
        beacon_block_root: input.beacon_block_root,
        slot: input.slot,
        post_state_root: input.post_state_root,
        committed_hash: input.block_hash,
        expected_withdrawals: withdrawals,
    };

    let _ = construct_envelope(&params, dummy_signer);
});
