.PHONY: build test lint fmt check qa-setup qa-check

build:
	cargo build --workspace

test:
	cargo test --workspace

lint:
	cargo clippy --workspace --all-targets -- -D warnings

fmt:
	cargo fmt --all

check:
	cargo check --workspace

qa-setup:
	bash qa/scripts/setup-confluence-env.sh

qa-check:
	bash qa/scripts/check-env.sh
