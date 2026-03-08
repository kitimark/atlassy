## Why

Phase 1 leaf crate modularization is structurally complete (module splits, facades, re-exports) but has two gaps against the roadmap's test architecture standard: (1) confluence public-API tests are misplaced in `src/` instead of `tests/`, violating the existing `test-placement-policy` spec, and (2) ADF and contracts domain modules have no inline unit tests for private/`pub(crate)` logic. Closing these gaps completes Phase 1 and unblocks Phase 2 (pipeline modularization).

## What Changes

- Move `crates/atlassy-confluence/src/tests.rs` (3 tests exercising `StubConfluenceClient` through public API) to `crates/atlassy-confluence/tests/stub_client.rs`.
- Add inline `#[cfg(test)] mod tests { ... }` blocks in `crates/atlassy-adf/src/scope.rs` covering 5 private functions (`full_page_resolution`, `find_heading_paths`, `find_block_paths`, `heading_level`, `expand_heading_to_section`).
- Add inline `#[cfg(test)] mod tests { ... }` block in `crates/atlassy-adf/src/index.rs` covering 1 private function (`build_node_path_index_inner`) and 1 `pub(crate)` function (`collect_text`).
- Add inline `#[cfg(test)] mod tests { ... }` block in `crates/atlassy-adf/src/path.rs` covering 1 private function (`compare_path_segments`) and 3 `pub(crate)` functions (`is_json_pointer`, `escape_pointer_segment`, `parent_path`).
- Add inline `#[cfg(test)] mod tests { ... }` block in `crates/atlassy-contracts/src/validation.rs` covering 2 private functions (`is_valid_git_sha`, `is_within_scope`).
- Remove `#[cfg(test)] mod tests;` declaration from `crates/atlassy-confluence/src/lib.rs`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `test-placement-policy`: Test count increases from 107 (new inline unit tests are added, not extracted). The count-preservation requirement applies to extraction operations; this change adds net-new coverage.

## Impact

- **Files modified:** `confluence/src/lib.rs`, `adf/src/scope.rs`, `adf/src/index.rs`, `adf/src/path.rs`, `contracts/src/validation.rs`
- **Files created:** `confluence/tests/stub_client.rs`
- **Files deleted:** `confluence/src/tests.rs`
- **No API changes:** No visibility widening, no new public items, no import path changes for downstream crates.
- **No behavior changes:** All existing tests remain unchanged. New tests cover existing private logic only.
