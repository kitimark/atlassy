## Purpose

Define the module boundaries, facade shape, state-function extraction pattern, dependency graph, and visibility rules for the `atlassy-pipeline` crate after modularization.

## ADDED Requirements

### Requirement: Pipeline lib.rs SHALL be a facade

`crates/atlassy-pipeline/src/lib.rs` SHALL contain only module declarations (`mod <name>;`), shared type definitions (`RunMode`, `RunRequest`), and re-exports. It SHALL NOT contain function bodies, `impl` blocks with logic, or helper functions.

#### Scenario: lib.rs contains only declarations and re-exports
- **WHEN** `crates/atlassy-pipeline/src/lib.rs` is inspected
- **THEN** it contains `mod state_tracker;`, `mod error_map;`, `mod artifact_store;`, `mod util;`, `mod states;`, `mod orchestrator;` declarations
- **THEN** it contains `pub use` re-exports for `Orchestrator`, `PipelineError`, `RunMode`, `RunRequest`, `StateTracker`
- **THEN** it contains the `RunMode` enum definition and `RunRequest` struct definition (shared types used across modules)
- **THEN** it contains no `fn` definitions (excluding `#[cfg(test)]` blocks)

#### Scenario: No logic functions remain in lib.rs
- **WHEN** `grep -n '^    fn \|^pub fn \|^pub(crate) fn \|^fn ' crates/atlassy-pipeline/src/lib.rs` is run
- **THEN** zero matches are returned

### Requirement: Each module SHALL have a single primary responsibility

Each extracted module SHALL correspond to one concern. The module map SHALL be:

| Module | Responsibility |
|---|---|
| `state_tracker` | `StateTracker` struct, `new()`, `transition_to()`, `Default` impl |
| `error_map` | `PipelineError` enum, `to_hard_error`, `confluence_error_to_hard_error`, `From<AdfError>`, `From<ConfluenceError>` |
| `artifact_store` | `ArtifactStore` struct, `new()`, `persist_state()`, `persist_summary()` |
| `util` | `meta()`, `estimate_tokens()`, `compute_section_bytes()`, `add_duration_suffix()` |
| `states/fetch` | `run_fetch_state` free function |
| `states/classify` | `run_classify_state` free function + `route_for_node`, `has_table_ancestor`, `parent_path` helpers |
| `states/extract_prose` | `run_extract_prose_state` free function |
| `states/md_assist_edit` | `run_md_assist_edit_state` free function + `project_prose_candidate` helper |
| `states/adf_table_edit` | `run_adf_table_edit_state` free function + `project_table_candidate` helper |
| `states/merge_candidates` | `run_merge_candidates_state` free function + `paths_overlap` helper |
| `states/patch` | `run_patch_state` free function |
| `states/verify` | `run_verify_state` free function |
| `states/publish` | `run_publish_state` free function |
| `orchestrator` | `Orchestrator<C>` struct, `new()`, `client()`, `client_mut()`, `run()`, `run_internal()`, `hard_fail()`, bootstrap interlude |

#### Scenario: State modules contain only their state function and co-located helpers
- **WHEN** `crates/atlassy-pipeline/src/states/classify.rs` is inspected
- **THEN** it contains `pub(crate) fn run_classify_state(...)`, `fn route_for_node(...)`, `fn has_table_ancestor(...)`, and `fn parent_path(...)`
- **THEN** it contains no other public or private functions unrelated to the classify state

#### Scenario: Util module contains only shared helpers
- **WHEN** `crates/atlassy-pipeline/src/util.rs` is inspected
- **THEN** it contains `pub(crate) fn meta(...)`, `pub(crate) fn estimate_tokens(...)`, `pub(crate) fn compute_section_bytes(...)`, and `pub(crate) fn add_duration_suffix(...)`
- **THEN** it contains no single-caller helpers that belong in a state module

#### Scenario: Error map module contains all error types and converters
- **WHEN** `crates/atlassy-pipeline/src/error_map.rs` is inspected
- **THEN** it contains the `PipelineError` enum definition, `pub(crate) fn to_hard_error(...)`, `pub(crate) fn confluence_error_to_hard_error(...)`, `impl From<AdfError> for PipelineError`, and `impl From<ConfluenceError> for PipelineError`

### Requirement: State functions SHALL be free functions with explicit parameters

State methods on `Orchestrator` SHALL be extracted as `pub(crate)` free functions. Each function SHALL receive only the dependencies it needs as explicit parameters, not the full `Orchestrator` struct.

#### Scenario: Non-client state function signature
- **WHEN** `run_classify_state` is extracted from `Orchestrator`
- **THEN** its signature is `pub(crate) fn run_classify_state(artifact_store: &ArtifactStore, request: &RunRequest, tracker: &mut StateTracker, fetch: &StateEnvelope<FetchOutput>) -> Result<StateEnvelope<ClassifyOutput>, PipelineError>`
- **THEN** it does NOT receive `&self` or `&mut self`

#### Scenario: Client-dependent state function signature
- **WHEN** `run_fetch_state` is extracted from `Orchestrator`
- **THEN** its signature includes a generic client parameter: `pub(crate) fn run_fetch_state<C: ConfluenceClient>(client: &mut C, artifact_store: &ArtifactStore, request: &RunRequest, tracker: &mut StateTracker) -> Result<...>`
- **THEN** only `run_fetch_state` and `run_publish_state` have the `client` parameter

#### Scenario: Orchestrator calls free functions in run_internal
- **WHEN** `crates/atlassy-pipeline/src/orchestrator.rs` is inspected
- **THEN** `run_internal` calls `states::run_fetch_state(&mut self.client, &self.artifact_store, ...)` instead of `self.run_fetch_state(...)`
- **THEN** `run_internal` calls `states::run_classify_state(&self.artifact_store, ...)` instead of `self.run_classify_state(...)`

### Requirement: hard_fail SHALL remain a method on Orchestrator

The `hard_fail` function SHALL stay as a private method on `Orchestrator<C>`, not extracted as a free function. It is called only from `run_internal()` closures where it mutates the summary and returns the error.

#### Scenario: hard_fail is a private method on Orchestrator
- **WHEN** `crates/atlassy-pipeline/src/orchestrator.rs` is inspected
- **THEN** it contains `fn hard_fail(&self, summary: &mut RunSummary, state: PipelineState, error: PipelineError) -> PipelineError` as a method in the `impl<C: ConfluenceClient> Orchestrator<C>` block

### Requirement: Bootstrap interlude SHALL reside in orchestrator

The empty-page detection and scaffold injection logic (~50 lines between fetch and classify in `run_internal`) SHALL remain in `orchestrator.rs`, not in `states/fetch.rs` or a separate module.

#### Scenario: Bootstrap logic is in orchestrator run_internal
- **WHEN** `crates/atlassy-pipeline/src/orchestrator.rs` is inspected
- **THEN** `run_internal` contains the `is_page_effectively_empty` check, the four-branch `match (page_empty, request.bootstrap_empty_page)`, and the `bootstrap_scaffold` injection
- **THEN** `crates/atlassy-pipeline/src/states/fetch.rs` does NOT contain `is_page_effectively_empty` or `bootstrap_scaffold` calls

### Requirement: State functions SHALL live under a states/ submodule directory

The 9 state functions SHALL be organized under `src/states/` with a `mod.rs` barrel file re-exporting all state functions.

#### Scenario: States directory structure
- **WHEN** `crates/atlassy-pipeline/src/states/` is listed
- **THEN** it contains `mod.rs`, `fetch.rs`, `classify.rs`, `extract_prose.rs`, `md_assist_edit.rs`, `adf_table_edit.rs`, `merge_candidates.rs`, `patch.rs`, `verify.rs`, `publish.rs`

#### Scenario: States mod.rs re-exports all state functions
- **WHEN** `crates/atlassy-pipeline/src/states/mod.rs` is inspected
- **THEN** it contains `mod` declarations for each state module
- **THEN** it contains `pub(crate) use` re-exports for each `run_*_state` function

### Requirement: Public API surface SHALL be preserved via facade re-exports

All items that are `pub` before modularization SHALL remain importable at the crate root path after modularization. Downstream crates SHALL NOT require any import path changes.

#### Scenario: CLI imports unchanged after pipeline modularization
- **WHEN** `atlassy-cli` uses `use atlassy_pipeline::{Orchestrator, PipelineError, RunMode, RunRequest};`
- **THEN** those imports compile without changes after modularization

#### Scenario: StateTracker remains importable at crate root
- **WHEN** `atlassy-cli` or integration tests use `use atlassy_pipeline::StateTracker;`
- **THEN** that import compiles without changes after modularization

### Requirement: Module-internal visibility SHALL follow minimum-necessary principle

Functions that were private and are only used within their new module SHALL remain private. Functions called from other modules within the crate SHALL become `pub(crate)`. No item SHALL become `pub` or `pub(crate)` solely for test access.

#### Scenario: Single-caller helpers stay private
- **WHEN** `project_prose_candidate` is moved to `states/md_assist_edit.rs`
- **THEN** it remains `fn` (private), not `pub(crate)`

#### Scenario: Cross-module helpers become pub(crate)
- **WHEN** `meta` is moved to `util.rs`
- **THEN** it becomes `pub(crate) fn meta(...)` because it is called from both state modules and `orchestrator.rs`

#### Scenario: Error converters become pub(crate)
- **WHEN** `to_hard_error` and `confluence_error_to_hard_error` are moved to `error_map.rs`
- **THEN** they become `pub(crate) fn` because they are called from state modules

### Requirement: Module dependency graph SHALL be acyclic

No module within the crate SHALL depend on another module that depends back on it. Dependencies SHALL flow in one direction.

#### Scenario: Pipeline module dependencies are acyclic
- **WHEN** the `atlassy-pipeline` module dependency graph is traced
- **THEN** `state_tracker` has zero intra-crate module dependencies
- **THEN** `error_map` has zero intra-crate module dependencies
- **THEN** `artifact_store` depends only on `error_map`
- **THEN** `util` has zero intra-crate module dependencies
- **THEN** each `states/*` module depends on `error_map`, `artifact_store`, and `util`
- **THEN** `orchestrator` depends on `state_tracker`, `error_map`, `artifact_store`, `util`, and `states`
- **THEN** no circular dependency exists

### Requirement: State order and semantics SHALL be preserved

The pipeline state execution order, data flow, and error handling behavior SHALL be identical before and after modularization. No state function SHALL change its inputs, outputs, or side effects.

#### Scenario: Integration tests pass without modification
- **WHEN** `cargo test --test pipeline_integration` is run after modularization
- **THEN** all tests pass without any changes to the test file

#### Scenario: State execution order is unchanged
- **WHEN** a pipeline run executes
- **THEN** states execute in the order: fetch, classify, extract_prose, md_assist_edit, adf_table_edit, merge_candidates, patch, verify, publish

### Requirement: Quality gates SHALL pass after modularization

`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` SHALL all pass after modularization.

#### Scenario: All quality gates pass
- **WHEN** all modules have been extracted
- **THEN** `cargo fmt --all -- --check` reports zero issues
- **THEN** `cargo clippy --workspace --all-targets -- -D warnings` reports zero warnings
- **THEN** `cargo test --workspace` passes all tests
