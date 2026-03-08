## Purpose

Define deterministic test placement rules so production source files stay focused on runtime logic while test coverage, visibility boundaries, and quality gates remain unchanged.

## Requirements

### Requirement: Production files SHALL NOT contain inline test bodies

Production source files (`src/lib.rs`, `src/main.rs`) SHALL NOT contain `mod tests { ... }` blocks with test function bodies. Production files MAY contain only thin test module declarations: `#[cfg(test)] mod tests;` or `#[cfg(test)] mod test_helpers;`. Domain module files (e.g., `src/live.rs`, `src/scope.rs`) MAY contain `#[cfg(test)] mod tests { ... }` blocks when those tests require private access to the module's internals.

#### Scenario: Production file with thin declaration only
- **WHEN** a `src/lib.rs` or `src/main.rs` file references a test module
- **THEN** it contains only `#[cfg(test)] mod tests;` (a declaration, not a block with braces and body)

#### Scenario: Grep verification finds no inline test blocks in entry files
- **WHEN** `grep -rn 'mod tests {' crates/*/src/lib.rs crates/*/src/main.rs` is run
- **THEN** zero matches are returned

#### Scenario: Domain module with inline tests for private access
- **WHEN** `crates/atlassy-confluence/src/live.rs` contains a `#[cfg(test)] mod tests { ... }` block
- **THEN** those tests call only private methods of `LiveConfluenceClient` defined in that same file

### Requirement: Tests requiring private access SHALL reside under src/

Tests that call private (non-`pub`) functions, methods, or types SHALL be placed in dedicated test files or `#[cfg(test)]` blocks under `src/` within the same crate. For facade crates with domain modules, tests for a module's private methods MAY reside in a `#[cfg(test)] mod tests { ... }` block within that module file, using `use super::*;` to access the module's private items.

#### Scenario: Pipeline tests access private compute_section_bytes
- **WHEN** `atlassy-pipeline` tests call the private `compute_section_bytes` function
- **THEN** those tests reside in `crates/atlassy-pipeline/src/tests.rs`

#### Scenario: Confluence live module tests access private build methods
- **WHEN** `atlassy-confluence` tests call private `LiveConfluenceClient::build_publish_payload` or `build_create_payload`
- **THEN** those tests reside in a `#[cfg(test)] mod tests` block within `crates/atlassy-confluence/src/live.rs`

#### Scenario: CLI tests access binary crate internals
- **WHEN** `atlassy-cli` tests use private types (`BatchReport`, `RunManifest`, `DecisionPacket`) or private functions (`map_live_startup_error`, `rebuild_batch_report_from_artifacts`, etc.)
- **THEN** those tests reside in `crates/atlassy-cli/src/tests.rs`

### Requirement: Tests using only public API SHALL reside under tests/

Tests that use only `pub` items from a library crate SHALL be placed under the crate's `tests/` directory as integration-style tests. These files use `use <crate_name>::*;` imports.

#### Scenario: ADF tests use only public API
- **WHEN** all 42 `atlassy-adf` tests call only `pub` functions and reference only `pub` types
- **THEN** those tests reside in files under `crates/atlassy-adf/tests/`

#### Scenario: Contracts tests use only public API
- **WHEN** all 10 `atlassy-contracts` tests call only `pub` functions and reference only `pub` types
- **THEN** those tests reside in `crates/atlassy-contracts/tests/contract_validation.rs`

### Requirement: ADF tests SHALL be grouped by functional domain

The 42 `atlassy-adf` tests SHALL be split across 5 files organized by functional domain, not arbitrarily or by single function name.

#### Scenario: Scope resolution tests grouped together
- **WHEN** a test exercises `resolve_scope` (heading selectors, block selectors, fallbacks, multi-selector union)
- **THEN** it resides in `crates/atlassy-adf/tests/scope_resolution.rs`

#### Scenario: Target discovery tests grouped together
- **WHEN** a test exercises `discover_target_path` (prose/table discovery, scope boundary, index bounds, error cases)
- **THEN** it resides in `crates/atlassy-adf/tests/target_discovery.rs`

#### Scenario: Patch operation tests grouped together
- **WHEN** a test exercises `build_patch_ops`, `apply_patch_ops`, or `canonicalize_mapped_path`
- **THEN** it resides in `crates/atlassy-adf/tests/patch_ops.rs`

#### Scenario: Path classification tests grouped together
- **WHEN** a test exercises `is_table_cell_text_path`, `is_table_shape_or_attr_path`, `document_order_sort`, or the `SCOPE_ANCHOR_TYPES` invariant
- **THEN** it resides in `crates/atlassy-adf/tests/path_classification.rs`

#### Scenario: Emptiness and bootstrap tests grouped together
- **WHEN** a test exercises `is_page_effectively_empty` or `bootstrap_scaffold`
- **THEN** it resides in `crates/atlassy-adf/tests/emptiness_bootstrap.rs`

### Requirement: CLI test helpers SHALL be in a separate module

The `#[cfg(test)]` helper functions (`fixture_path`, `execute_batch_from_manifest_file`) SHALL reside in `crates/atlassy-cli/src/test_helpers.rs`, not inline in `main.rs` or in the test file.

#### Scenario: main.rs references test_helpers module
- **WHEN** `main.rs` is inspected
- **THEN** it contains `#[cfg(test)] mod test_helpers;` and `#[cfg(test)] mod tests;`
- **THEN** it does NOT contain the body of `execute_batch_from_manifest_file`

#### Scenario: Tests import helpers from test_helpers module
- **WHEN** `src/tests.rs` calls `fixture_path` or `execute_batch_from_manifest_file`
- **THEN** those functions are imported from `super::test_helpers`

### Requirement: Public API visibility SHALL NOT be widened for test access

No item's visibility SHALL be changed from private to `pub` or `pub(crate)` solely to allow tests to access it from a different file location.

#### Scenario: Private function stays private after extraction
- **WHEN** `compute_section_bytes` in `atlassy-pipeline` is private before extraction
- **THEN** it remains private after extraction (tests access it via `src/tests.rs` using `use super::*`)

#### Scenario: Binary crate items stay private after extraction
- **WHEN** `BatchReport`, `map_live_startup_error`, and other CLI internals are private before extraction
- **THEN** they remain private after extraction

### Requirement: Test count SHALL be preserved exactly

The total number of `#[test]` functions across the workspace SHALL remain identical before and after extraction. No tests are added, removed, or renamed.

#### Scenario: Test count parity check
- **WHEN** `grep -r '#\[test\]' crates/ --include='*.rs' | wc -l` is run before and after extraction
- **THEN** both counts equal 107

#### Scenario: All workspace tests pass
- **WHEN** `cargo test --workspace` is run after extraction
- **THEN** all tests pass with zero failures

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
- **THEN** all 107 tests pass
