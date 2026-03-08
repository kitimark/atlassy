## Purpose

Define the module boundaries, naming conventions, and facade re-export pattern for leaf crates (`atlassy-adf`, `atlassy-contracts`, `atlassy-confluence`). These rules establish the structural pattern that later phases (pipeline, CLI) will follow.

## Requirements

### Requirement: Leaf crate lib.rs SHALL be a facade

Each leaf crate's `src/lib.rs` SHALL contain only module declarations (`mod <name>;`), re-exports (`pub use <module>::*;`), and cross-cutting type/constant definitions. It SHALL NOT contain domain logic functions.

#### Scenario: atlassy-contracts lib.rs is a facade
- **WHEN** `crates/atlassy-contracts/src/lib.rs` is inspected
- **THEN** it contains `mod constants;`, `mod types;`, `mod validation;` declarations
- **THEN** it contains `pub use constants::*;`, `pub use types::*;`, `pub use validation::*;` re-exports
- **THEN** it contains no `fn` definitions (excluding `#[cfg(test)]` blocks)

#### Scenario: atlassy-confluence lib.rs is a facade
- **WHEN** `crates/atlassy-confluence/src/lib.rs` is inspected
- **THEN** it contains `mod types;`, `mod stub;`, `mod live;` declarations
- **THEN** it contains `pub use types::*;`, `pub use stub::*;`, `pub use live::*;` re-exports
- **THEN** it contains no `fn` definitions (excluding `#[cfg(test)]` blocks)

#### Scenario: atlassy-adf lib.rs is a facade with cross-cutting types
- **WHEN** `crates/atlassy-adf/src/lib.rs` is inspected
- **THEN** it contains `mod path;`, `mod index;`, `mod scope;`, `mod patch;`, `mod table_guard;`, `mod bootstrap;` declarations
- **THEN** it contains glob re-exports for each module
- **THEN** it MAY contain shared type definitions (`AdfError`, `ScopeResolution`, `PatchCandidate`, `PatchOperation`, `TargetRoute`) and constants (`EDITABLE_PROSE_TYPES`, `SCOPE_ANCHOR_TYPES`) that are referenced across multiple modules
- **THEN** it contains no domain logic functions

### Requirement: Each module SHALL have a single primary responsibility

Each extracted module SHALL correspond to one concern cluster. A module SHALL NOT mix unrelated concerns.

#### Scenario: atlassy-adf modules match concern clusters
- **WHEN** the `atlassy-adf` source directory is inspected
- **THEN** `path.rs` contains only JSON pointer manipulation functions (`document_order_sort`, `compare_path_segments`, `is_within_allowed_scope`, `is_path_within_or_descendant`, `canonicalize_mapped_path`, `is_json_pointer`, `escape_pointer_segment`, `parent_path`)
- **THEN** `index.rs` contains only node path index functions (`build_node_path_index`, `build_node_path_index_inner`, `path_has_ancestor_type`, `collect_text`)
- **THEN** `scope.rs` contains only scope resolution functions (`resolve_scope`, `full_page_resolution`, `find_heading_paths`, `find_block_paths`, `expand_heading_to_section`, `heading_level`)
- **THEN** `patch.rs` contains only patch operation functions (`normalize_changed_paths`, `build_patch_ops`, `apply_patch_ops`, `ensure_paths_in_scope`)
- **THEN** `table_guard.rs` contains only target discovery and table classification functions (`discover_target_path`, `is_table_cell_text_path`, `is_table_shape_or_attr_path`, `markdown_for_path`)
- **THEN** `bootstrap.rs` contains only emptiness and scaffolding functions (`is_page_effectively_empty`, `bootstrap_scaffold`)

#### Scenario: atlassy-contracts modules match concern layers
- **WHEN** the `atlassy-contracts` source directory is inspected
- **THEN** `constants.rs` contains all `pub const` declarations (version constants, error codes, flow identifiers, pattern identifiers, runtime mode identifiers)
- **THEN** `types.rs` contains all `pub struct`, `pub enum`, and inherent `impl` blocks
- **THEN** `validation.rs` contains all validation `pub fn` functions and their private helpers

#### Scenario: atlassy-confluence modules match architectural boundaries
- **WHEN** the `atlassy-confluence` source directory is inspected
- **THEN** `types.rs` contains `FetchPageResponse`, `PublishPageResponse`, `CreatePageResponse`, `ConfluenceError`, and the `ConfluenceClient` trait
- **THEN** `stub.rs` contains `StubPage`, `StubConfluenceClient`, and its `ConfluenceClient` trait implementation
- **THEN** `live.rs` contains `LiveConfluenceClient` and its `ConfluenceClient` trait implementation including private payload builder methods

### Requirement: Public API surface SHALL be preserved via facade re-exports

All items that are `pub` before modularization SHALL remain importable at the crate root path after modularization. Downstream crates SHALL NOT require any import path changes.

#### Scenario: Pipeline imports unchanged after adf modularization
- **WHEN** `atlassy-pipeline` uses `use atlassy_adf::{resolve_scope, build_node_path_index, AdfError, ...};`
- **THEN** those imports compile without changes after `atlassy-adf` is modularized

#### Scenario: Pipeline imports unchanged after contracts modularization
- **WHEN** `atlassy-pipeline` uses `use atlassy_contracts::{RunSummary, PipelineState, StateEnvelope, ...};`
- **THEN** those imports compile without changes after `atlassy-contracts` is modularized

#### Scenario: CLI imports unchanged after confluence modularization
- **WHEN** `atlassy-cli` uses `use atlassy_confluence::{LiveConfluenceClient, StubConfluenceClient, StubPage, ...};`
- **THEN** those imports compile without changes after `atlassy-confluence` is modularized

### Requirement: Module-internal visibility SHALL follow minimum-necessary principle

Functions that were private and are only used within their new module SHALL remain private. Functions that were private but are now called from other modules within the same crate SHALL become `pub(crate)`. No item SHALL become `pub` or `pub(crate)` solely for test access.

#### Scenario: ADF path utilities become pub(crate)
- **WHEN** private functions `is_json_pointer`, `escape_pointer_segment`, and `parent_path` are moved to the `path` module
- **THEN** they become `pub(crate)` because they are called from other modules (`index`, `scope`, `patch`, `table_guard`)
- **THEN** they are NOT re-exported as `pub` from the crate root

#### Scenario: ADF scope internals stay private
- **WHEN** private functions `heading_level` and `expand_heading_to_section` are moved to the `scope` module
- **THEN** they remain private (`fn`, not `pub` or `pub(crate)`) because they are only called within `scope`

#### Scenario: Contracts validation helpers stay private
- **WHEN** private functions `is_valid_git_sha` and `is_within_scope` are moved to the `validation` module
- **THEN** they remain private because they are only called within `validation`

### Requirement: Module dependency graph SHALL be acyclic

No module within a crate SHALL depend on another module that depends back on it. Dependencies SHALL flow in one direction.

#### Scenario: ADF module dependencies are acyclic
- **WHEN** the `atlassy-adf` module dependency graph is traced
- **THEN** `path` has zero intra-crate module dependencies
- **THEN** `index` depends only on `path`
- **THEN** `scope` depends on `path` and `index`
- **THEN** `patch` depends on `path`
- **THEN** `table_guard` depends on `path` and `index`
- **THEN** `bootstrap` has zero intra-crate module dependencies
- **THEN** no circular dependency exists

#### Scenario: Contracts module dependencies are acyclic
- **WHEN** the `atlassy-contracts` module dependency graph is traced
- **THEN** `constants` has zero intra-crate dependencies
- **THEN** `types` depends only on `constants` (if any constant is referenced)
- **THEN** `validation` depends on `constants` and `types`
- **THEN** no circular dependency exists

### Requirement: Quality gates SHALL pass after modularization

`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` SHALL all pass after each crate is modularized.

#### Scenario: All quality gates pass after contracts modularization
- **WHEN** `atlassy-contracts` modularization is complete
- **THEN** `cargo fmt --all -- --check` reports zero issues
- **THEN** `cargo clippy --workspace --all-targets -- -D warnings` reports zero warnings
- **THEN** `cargo test --workspace` passes all tests

#### Scenario: All quality gates pass after confluence modularization
- **WHEN** `atlassy-confluence` modularization is complete
- **THEN** all three quality gate commands pass

#### Scenario: All quality gates pass after adf modularization
- **WHEN** `atlassy-adf` modularization is complete
- **THEN** all three quality gate commands pass
