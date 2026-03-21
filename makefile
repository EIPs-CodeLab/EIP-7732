.PHONY: build test lint fmt check clean sim-builder sim-ptc cli docs fuzz fuzz-bid fuzz-envelope

# ─── Build ────────────────────────────────────────────────────────────────────
build:
	cargo build --release

# ─── Test ─────────────────────────────────────────────────────────────────────
test:
	cargo test --all

test-unit:
	cargo test --test unit

test-integration:
	cargo test --test integration

fuzz: fuzz-bid fuzz-envelope

fuzz-bid:
	cd fuzz && cargo +nightly fuzz run fuzz_bid -- -max_total_time=30

fuzz-envelope:
	cd fuzz && cargo +nightly fuzz run fuzz_envelope -- -max_total_time=30

# ─── Quality ──────────────────────────────────────────────────────────────────
lint:
	cargo clippy --all-targets --all-features -- -D warnings

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

check: fmt-check lint test

# ─── Examples ─────────────────────────────────────────────────────────────────
# Simulates a full ePBS round: proposer selects a builder bid,
# builder reveals payload, PTC attests
sim-builder:
	cargo run --example builder_sim

# Simulates a 512-member Payload Timeliness Committee voting on
# payload availability
sim-ptc:
	cargo run --example ptc_sim

# Interactive CLI — inspect bids, envelopes, PTC attestations
cli:
	cargo run --bin epbs-cli -- --help

# ─── Docs ─────────────────────────────────────────────────────────────────────
docs:
	cargo doc --no-deps --open

# ─── Clean ────────────────────────────────────────────────────────────────────
clean:
	cargo clean
