## Context

Phase 1 leaf crate modularization is structurally complete. Module boundaries, facades, and re-exports match the `leaf-crate-module-structure` spec. Two test architecture gaps remain:

1. `crates/atlassy-confluence/src/tests.rs` contains 3 public-API tests that belong in `tests/` per the `test-placement-policy` spec.
2. ADF modules (`scope.rs`, `index.rs`, `path.rs`) and contracts `validation.rs` contain private/`pub(crate)` functions with no inline unit tests. These functions are exercised through integration tests, but the roadmap requires inline `#[cfg(test)] mod tests` for private logic.

Closing these gaps completes Phase 1 and unblocks Phase 2 (pipeline modularization).

## Goals / Non-Goals

**Goals:**

- Comply with `test-placement-policy` for confluence test placement.
- Add inline unit tests for all private and `pub(crate)` functions in ADF and contracts domain modules.
- Unblock Phase 2 with a clean Phase 1 baseline.

**Non-Goals:**

- Payload extraction from `live.rs`. Evaluated during explore and rejected: 49 lines total, single caller each, would require `pub(crate)` visibility widening. The `leaf-crate-module-structure` spec explicitly places payload builders in `live.rs`.
- New integration tests. Existing `tests/` coverage is thorough and unchanged.
- Any API visibility changes.

## Decisions

### 1. Confluence test move: file placement and naming

Move `src/tests.rs` → `tests/stub_client.rs`. Named after what it tests (the stub client's `create_page` behavior), not generically.

- Replace `use super::*` with `use atlassy_confluence::*`
- Remove `#[cfg(test)] mod tests;` from `lib.rs`
- No `tests/common/mod.rs` needed (no shared test helpers)

### 2. Inline test scope: function-level contracts, not flow duplication

Each inline test targets a single private/`pub(crate)` function directly. Tests focus on:
- Edge cases specific to the function's contract (e.g., `heading_level` defaulting to 6 when attrs missing)
- Boundary conditions (e.g., `parent_path` on root `"/"`)
- Invariants (e.g., `escape_pointer_segment` handling `~` and `/`)

Tests do NOT duplicate integration test scenarios. Integration tests exercise end-to-end flows; inline tests exercise function-level contracts.

### 3. No visibility changes

All functions remain at current visibility. Inline tests access private functions via `use super::*`. No `pub(crate)` backdoors.

### 4. Modules that do NOT get inline tests

- `adf/bootstrap.rs` — 2 public functions, zero private logic. Fully covered by `tests/emptiness_bootstrap.rs`.
- `adf/patch.rs` — 4 public functions, zero private logic. Fully covered by `tests/patch_ops.rs`.
- `adf/table_guard.rs` — 4 public functions, zero private logic. Fully covered by `tests/target_discovery.rs`.
- `contracts/constants.rs` — pure `pub const` declarations, no logic.
- `contracts/types.rs` — public structs/enums with trivial method impls, no private logic.
- `confluence/stub.rs` — 1 private function (`synthetic_page_id`), but it's a hash-based ID generator with no interesting contract to unit test.

## Risks / Trade-offs

- [Partial coverage overlap with integration tests] → Some inline tests will exercise paths already covered by `tests/`. → Acceptable: inline tests pin function-level contracts; integration tests pin flow-level behavior. Both serve different regression purposes.
- [Test count in `test-placement-policy` spec becomes stale] → Current count is 107. New inline tests will increase this. → Mitigation: update the spec's test count after implementation.
