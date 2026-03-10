.PHONY: build test lint fmt fmt-check check qa-setup qa-check setup

build:
	cargo build --workspace

test:
	cargo test --workspace

lint:
	cargo clippy --workspace --all-targets -- -D warnings

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

check:
	cargo check --workspace

qa-setup:
	bash qa/scripts/setup-confluence-env.sh

qa-check:
	bash qa/scripts/check-env.sh

setup:
	git config core.hooksPath .githooks
