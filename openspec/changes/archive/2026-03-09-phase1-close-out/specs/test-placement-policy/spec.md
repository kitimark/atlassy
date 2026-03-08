## ADDED Requirements

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

### Requirement: Confluence public-API tests SHALL reside in tests/ directory

Tests exercising `StubConfluenceClient` through the public `ConfluenceClient` trait SHALL reside in `crates/atlassy-confluence/tests/`, not in `src/tests.rs`.

#### Scenario: Stub client tests are integration tests
- **WHEN** `crates/atlassy-confluence/tests/stub_client.rs` is inspected
- **THEN** it contains tests for `create_page` (insert, reject missing parent, reject duplicate title)
- **THEN** it imports via `use atlassy_confluence::*` (not `use super::*`)

#### Scenario: No test module declaration in confluence lib.rs
- **WHEN** `crates/atlassy-confluence/src/lib.rs` is inspected
- **THEN** it does NOT contain `#[cfg(test)] mod tests;`

#### Scenario: src/tests.rs does not exist
- **WHEN** `crates/atlassy-confluence/src/` is listed
- **THEN** no `tests.rs` file exists

## MODIFIED Requirements

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
