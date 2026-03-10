## Why

The CI workflow and Makefile duplicate the same cargo commands (`clippy`, `test`), and `fmt` differs only by the `--check` flag. If a command changes (e.g., new clippy flags), both files must be updated independently. Consolidating CI to call Makefile targets creates a single source of truth for build commands.

## What Changes

- Add `fmt-check` target to `Makefile` running `cargo fmt --all -- --check`.
- Update `.github/workflows/ci.yml` to call `make fmt-check`, `make lint`, and `make test` instead of raw cargo commands.
- Add `fmt-check` to the `.PHONY` declaration.

## Capabilities

### New Capabilities
_(none — no new behavioral capability is introduced)_

### Modified Capabilities
- `ci-test-workflow`: CI format, lint, and test steps delegate to Makefile targets instead of inlining cargo commands.

## Impact

- Modified files: `Makefile`, `.github/workflows/ci.yml`.
- No changes to CI behavior — same commands run in the same order.
- No changes to any Rust crate source code.
