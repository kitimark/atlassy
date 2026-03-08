## Why

Five production files contain 77 inline `#[test]` functions (786 lines in `atlassy-adf/src/lib.rs` alone). These long `mod tests { ... }` blocks inflate production files, slow navigation, and make it harder to distinguish production logic from test setup. This is Phase 1 of the code quality roadmap (`roadmap/15-code-quality-and-readability.md`) — test extraction must land before module restructuring (Phases 2-4) can begin safely.

## What Changes

- Move 77 inline `#[test]` functions from 5 production `src/` files into dedicated test files.
- Production files retain only thin `#[cfg(test)] mod tests;` declarations (no inline test bodies).
- Tests requiring private access (`atlassy-pipeline`, `atlassy-confluence`, `atlassy-cli`) move to `src/tests.rs` within the same crate.
- Tests using only public API (`atlassy-adf`, `atlassy-contracts`) move to `tests/` as integration-style tests.
- `atlassy-adf` tests split into 5 domain-grouped files (scope resolution, target discovery, patch ops, path classification, emptiness/bootstrap).
- `atlassy-cli` test helpers (`fixture_path`, `execute_batch_from_manifest_file`) extract to `src/test_helpers.rs`.
- Inline ADF JSON fixtures remain duplicated per-test (no shared builder helpers) for test readability.
- Zero behavior change. Zero visibility change. Zero new tests.

## Capabilities

### New Capabilities

- `test-placement-policy`: Structural rules governing where tests live, how production files reference them, and verification criteria for enforcement.

### Modified Capabilities

None. No existing spec-level behavior changes — this is a behavior-preserving structural refactor.

## Impact

- **Code:** All 5 workspace crates gain new test files; 5 production `src/` files shrink significantly.
- **CI:** `cargo test --workspace` must remain green. `cargo fmt` and `cargo clippy` gates unchanged.
- **APIs:** No public API changes. No visibility widening.
- **Dependencies:** No new crate or external dependencies.
- **Downstream:** No output schema changes. Pipeline integration tests and CLI integration tests unchanged.
