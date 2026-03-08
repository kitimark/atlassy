## MODIFIED Requirements

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
