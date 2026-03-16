/// EIP-7732 — Payload Timeliness Committee simulation
///
/// Simulates a 512-member PTC for a single slot under three scenarios:
///   A. Happy path       — payload revealed in time, 400/512 vote present.
///   B. Withheld payload — builder never reveals, 0/512 vote present.
///   C. Split view       — exactly on the 60% threshold boundary.
///
/// Run with: `make sim-ptc`
use eip_7732::beacon_chain::{
    constants::{
        BUILDER_PAYMENT_THRESHOLD_DENOMINATOR, BUILDER_PAYMENT_THRESHOLD_NUMERATOR, PTC_SIZE,
    },
    types::{Slot, ValidatorIndex},
};

#[derive(Debug)]
struct PtcMember {
    _validator_index: ValidatorIndex,
    /// Did this member observe the payload in time?
    observed_payload: bool,
}

#[derive(Debug)]
struct PtcResult {
    present_votes: usize,
    absent_votes: usize,
    threshold: usize,
    outcome: PtcOutcome,
}

#[derive(Debug, PartialEq)]
enum PtcOutcome {
    PayloadPresent,
    PayloadAbsent,
    BelowThreshold,
}

fn run_ptc(slot: Slot, members: &[PtcMember]) -> PtcResult {
    let present_votes = members.iter().filter(|m| m.observed_payload).count();
    let absent_votes = members.len() - present_votes;
    let threshold = (PTC_SIZE as usize * BUILDER_PAYMENT_THRESHOLD_NUMERATOR as usize)
        / BUILDER_PAYMENT_THRESHOLD_DENOMINATOR as usize;

    let outcome = if present_votes >= threshold {
        PtcOutcome::PayloadPresent
    } else if absent_votes >= threshold {
        PtcOutcome::PayloadAbsent
    } else {
        PtcOutcome::BelowThreshold
    };

    println!("  Slot            : {}", slot);
    println!("  PTC size        : {}", PTC_SIZE);
    println!("  Threshold (60%) : {}", threshold);
    println!("  Votes present   : {}", present_votes);
    println!("  Votes absent    : {}", absent_votes);
    println!("  Outcome         : {:?}", outcome);

    PtcResult {
        present_votes,
        absent_votes,
        threshold,
        outcome,
    }
}

fn build_committee(size: usize, present_count: usize) -> Vec<PtcMember> {
    (0..size)
        .map(|i| PtcMember {
            _validator_index: i as ValidatorIndex,
            observed_payload: i < present_count,
        })
        .collect()
}

fn main() {
    println!("═══════════════════════════════════════════════");
    println!("  EIP-7732 ePBS — PTC Simulation");
    println!("═══════════════════════════════════════════════\n");

    // ── Scenario A: Happy path ─────────────────────────────────────────────────
    println!("── Scenario A: Happy path (400/512 vote present) ──");
    let committee_a = build_committee(PTC_SIZE as usize, 400);
    let result_a = run_ptc(100, &committee_a);
    assert_eq!(result_a.outcome, PtcOutcome::PayloadPresent);
    println!(
        "  Totals        : present={} absent={} threshold={}",
        result_a.present_votes, result_a.absent_votes, result_a.threshold
    );
    println!("  → Builder PAID, payload canonical\n");

    // ── Scenario B: Withheld payload ───────────────────────────────────────────
    println!("── Scenario B: Payload withheld (0/512 vote present) ──");
    let committee_b = build_committee(PTC_SIZE as usize, 0);
    let result_b = run_ptc(101, &committee_b);
    assert_eq!(result_b.outcome, PtcOutcome::PayloadAbsent);
    println!(
        "  Totals        : present={} absent={} threshold={}",
        result_b.present_votes, result_b.absent_votes, result_b.threshold
    );
    println!("  → Builder NOT paid, slot marked Empty\n");

    // ── Scenario C: Split view ─────────────────────────────────────────────────
    let threshold = (PTC_SIZE as usize * 6) / 10; // 307
    let split_votes = threshold - 1; // 306 — just below threshold
    println!(
        "── Scenario C: Split view ({}/{} — just below threshold) ──",
        split_votes, PTC_SIZE
    );
    let committee_c = build_committee(PTC_SIZE as usize, split_votes);
    let result_c = run_ptc(102, &committee_c);
    assert_eq!(result_c.outcome, PtcOutcome::BelowThreshold);
    println!(
        "  Totals        : present={} absent={} threshold={}",
        result_c.present_votes, result_c.absent_votes, result_c.threshold
    );
    println!("  → No threshold reached — fork choice ambiguous\n");

    // ── Scenario D: Exact threshold ────────────────────────────────────────────
    println!(
        "── Scenario D: Exact threshold ({}/{}) ──",
        threshold, PTC_SIZE
    );
    let committee_d = build_committee(PTC_SIZE as usize, threshold);
    let result_d = run_ptc(103, &committee_d);
    assert_eq!(result_d.outcome, PtcOutcome::PayloadPresent);
    println!(
        "  Totals        : present={} absent={} threshold={}",
        result_d.present_votes, result_d.absent_votes, result_d.threshold
    );
    println!("  → Threshold met, payload canonical\n");

    println!("═══════════════════════════════════════════════");
    println!("  All PTC scenarios passed ✓");
    println!("═══════════════════════════════════════════════");
}
