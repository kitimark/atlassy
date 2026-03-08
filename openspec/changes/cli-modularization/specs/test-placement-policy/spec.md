## MODIFIED Requirements

### Requirement: Tests requiring private access SHALL reside under src/

Tests that call private (non-`pub`) functions, methods, or types SHALL be placed in dedicated test files or `#[cfg(test)]` blocks under `src/` within the same crate. For facade crates with domain modules, tests for a module's private methods MAY reside in a `#[cfg(test)] mod tests { ... }` block within that module file, using `use super::*;` to access the module's private items.

#### Scenario: Pipeline tests access private compute_section_bytes
- **WHEN** `atlassy-pipeline` tests call the private `compute_section_bytes` function
- **THEN** those tests reside in a `#[cfg(test)] mod tests` block within `crates/atlassy-pipeline/src/util.rs`

#### Scenario: Pipeline state_tracker test accesses transition enforcement
- **WHEN** `atlassy-pipeline` tests validate `StateTracker` transition enforcement behavior
- **THEN** those tests reside in a `#[cfg(test)] mod tests` block within `crates/atlassy-pipeline/src/state_tracker.rs`

#### Scenario: Confluence live module tests access private build methods
- **WHEN** `atlassy-confluence` tests call private `LiveConfluenceClient::build_publish_payload` or `build_create_payload`
- **THEN** those tests reside in a `#[cfg(test)] mod tests` block within `crates/atlassy-confluence/src/live.rs`

#### Scenario: CLI test accessing private map_live_startup_error
- **WHEN** `atlassy-cli` tests call the private `map_live_startup_error` function
- **THEN** that test resides in a `#[cfg(test)] mod tests` block within `crates/atlassy-cli/src/commands/run.rs`

### Requirement: Domain modules with private logic SHALL include inline unit tests

Domain module files containing private (`fn`) or `pub(crate)` functions SHALL include a `#[cfg(test)] mod tests { ... }` block with unit tests that exercise those functions directly via `use super::*`. Modules with only `pub` functions and no private logic are exempt.

#### Scenario: ADF scope module has inline tests for private functions
- **WHEN** `crates/atlassy-adf/src/scope.rs` is inspected
- **THEN** it contains a `#[cfg(test)] mod tests` block
- **THEN** that block contains tests exercising `full_page_resolution`, `find_heading_paths`, `find_block_paths`, `heading_level`, and `expand_heading_to_section`

#### Scenario: ADF index module has inline tests for private and pub(crate) functions
- **WHEN** `crates/atlassy-adf/src/index.rs` is inspected
- **THEN** it contains a `#[cfg(test)] mod tests` block
- **THEN** that block contains tests exercising `build_node_path_index_inner` and `collect_text`

#### Scenario: ADF path module has inline tests for private and pub(crate) functions
- **WHEN** `crates/atlassy-adf/src/path.rs` is inspected
- **THEN** it contains a `#[cfg(test)] mod tests` block
- **THEN** that block contains tests exercising `compare_path_segments`, `is_json_pointer`, `escape_pointer_segment`, and `parent_path`

#### Scenario: Contracts validation module has inline tests for private functions
- **WHEN** `crates/atlassy-contracts/src/validation.rs` is inspected
- **THEN** it contains a `#[cfg(test)] mod tests` block
- **THEN** that block contains tests exercising `is_valid_git_sha` and `is_within_scope`

#### Scenario: Modules without private logic are exempt
- **WHEN** `crates/atlassy-adf/src/bootstrap.rs`, `patch.rs`, or `table_guard.rs` is inspected
- **THEN** they MAY omit `#[cfg(test)] mod tests` blocks (all functions are `pub`, tested via `tests/`)

#### Scenario: CLI commands/run module has inline test for private helper
- **WHEN** `crates/atlassy-cli/src/commands/run.rs` is inspected
- **THEN** it contains a `#[cfg(test)] mod tests` block
- **THEN** that block contains a test exercising the private `map_live_startup_error` function

### Requirement: Tests using only public API SHALL reside under tests/

Tests that use only `pub` items from a library crate SHALL be placed under the crate's `tests/` directory as integration-style tests. These files use `use <crate_name>::*;` imports.

#### Scenario: ADF tests use only public API
- **WHEN** all 42 `atlassy-adf` tests call only `pub` functions and reference only `pub` types
- **THEN** those tests reside in files under `crates/atlassy-adf/tests/`

#### Scenario: Contracts tests use only public API
- **WHEN** all 10 `atlassy-contracts` tests call only `pub` functions and reference only `pub` types
- **THEN** those tests reside in `crates/atlassy-contracts/tests/contract_validation.rs`

#### Scenario: CLI batch and readiness tests use public API
- **WHEN** `atlassy-cli` tests exercise `execute_batch_from_manifest_file`, `rebuild_batch_report_from_artifacts`, `generate_readiness_outputs_from_artifacts`, `verify_decision_packet_replay`, or `ensure_readiness_unblocked`
- **THEN** those tests reside in files under `crates/atlassy-cli/tests/`
- **THEN** those tests import via `use atlassy_cli::*` (not `use super::*`)

### Requirement: CLI test helpers SHALL use tests/common/mod.rs

The `fixture_path` helper function SHALL reside in `crates/atlassy-cli/tests/common/mod.rs` following the Rust Book Ch. 11-3 convention for shared test helpers. The `execute_batch_from_manifest_file` wrapper SHALL NOT be a test helper — the function it wraps is now part of the public API and callable directly.

#### Scenario: fixture_path is in tests/common/mod.rs
- **WHEN** `crates/atlassy-cli/tests/common/mod.rs` is inspected
- **THEN** it contains the `fixture_path` helper function
- **THEN** integration test files include `mod common;` and call `common::fixture_path`

#### Scenario: No src/test_helpers.rs exists
- **WHEN** `crates/atlassy-cli/src/` is listed
- **THEN** no `test_helpers.rs` file exists

#### Scenario: No src/tests.rs exists
- **WHEN** `crates/atlassy-cli/src/` is listed
- **THEN** no `tests.rs` file exists

#### Scenario: main.rs has no test module declarations
- **WHEN** `crates/atlassy-cli/src/main.rs` is inspected
- **THEN** it does NOT contain `#[cfg(test)] mod tests;` or `#[cfg(test)] mod test_helpers;`

### Requirement: Public API visibility SHALL NOT be widened for test access

No item's visibility SHALL be changed from private to `pub` or `pub(crate)` solely to allow tests to access it from a different file location.

#### Scenario: Private function stays private after extraction
- **WHEN** `compute_section_bytes` in `atlassy-pipeline` is private before extraction
- **THEN** it remains private after extraction (tests access it via inline `#[cfg(test)] mod tests` in `util.rs` using `use super::*`)

#### Scenario: CLI entry-point functions become pub because they are the API
- **WHEN** `execute_batch_from_manifest_file`, `generate_readiness_outputs_from_artifacts`, and `rebuild_batch_report_from_artifacts` are extracted to library modules
- **THEN** they become `pub fn` because `main()` dispatches to them and they form the crate's public interface
- **THEN** this is NOT visibility widening for test access — these are the same functions `main()` calls

#### Scenario: CLI internal helpers stay private after extraction
- **WHEN** `map_live_startup_error` in `commands/run.rs` is private before extraction
- **THEN** it remains private after extraction (tested via inline `#[cfg(test)] mod tests` using `use super::*`)

### Requirement: Test count SHALL be preserved exactly

The total number of `#[test]` functions across the workspace SHALL remain identical before and after extraction. No tests SHALL be removed or renamed during extraction operations. New inline unit tests added for private logic coverage are additions, not extractions, and increase the total count.

#### Scenario: Test count parity check after extraction
- **WHEN** a refactor moves existing tests between files without adding or removing tests
- **THEN** `grep -r '#\[test\]' crates/ --include='*.rs' | wc -l` returns the same count before and after

#### Scenario: All workspace tests pass
- **WHEN** `cargo test --workspace` is run after any test-related change
- **THEN** all tests pass with zero failures

#### Scenario: New inline tests increase the count
- **WHEN** `grep -r '#\[test\]' crates/ --include='*.rs' | wc -l` is run after adding inline unit tests
- **THEN** the count is greater than 107 (the pre-change baseline)

### Requirement: Quality gates SHALL pass after extraction

`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` SHALL all pass after extraction with no new warnings or errors.

#### Scenario: Format check passes
- **WHEN** `cargo fmt --all -- --check` is run after extraction
- **THEN** zero formatting issues are reported

#### Scenario: Clippy check passes
- **WHEN** `cargo clippy --workspace --all-targets -- -D warnings` is run after extraction
- **THEN** zero warnings or errors are reported

#### Scenario: Full test suite passes
- **WHEN** `cargo test --workspace` is run after extraction
- **THEN** all tests pass

## REMOVED Requirements

### Requirement: CLI test helpers SHALL be in a separate module

**Reason**: After CLI modularization, `src/test_helpers.rs` no longer exists. The `fixture_path` helper moves to `tests/common/mod.rs` (Rust Book convention). The `execute_batch_from_manifest_file` wrapper is no longer needed as the underlying function becomes part of the public API.

**Migration**: `fixture_path` is now in `tests/common/mod.rs`. Integration tests use `mod common;` and call `common::fixture_path`. Direct calls to `execute_batch_from_manifest_file` replace the test helper wrapper.
