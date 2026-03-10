## 1. Makefile

- [ ] 1.1 Add `fmt-check` target to `Makefile` running `cargo fmt --all -- --check`
- [ ] 1.2 Add `fmt-check` to the `.PHONY` declaration

## 2. CI Workflow

- [ ] 2.1 Replace `cargo fmt --all -- --check` with `make fmt-check` in `.github/workflows/ci.yml`
- [ ] 2.2 Replace `cargo clippy --workspace --all-targets -- -D warnings` with `make lint`
- [ ] 2.3 Replace `cargo test --workspace` with `make test`

## 3. Verify

- [ ] 3.1 Run `make fmt-check`, `make lint`, `make test` locally to confirm targets work
