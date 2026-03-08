## Why

`atlassy-pipeline/src/lib.rs` is a 1,534-line monolithic file containing the orchestrator, 9 state methods, error mapping, artifact persistence, and 12 helper functions. This makes it the largest remaining single-responsibility violation after Phase 1 leaf crate modularization. Extracting focused modules applies the same patterns proven in Phase 1 (facade entry files, single-responsibility modules, inline unit tests) to the pipeline crate, improving both human navigability and AI-readability.

## What Changes

- Extract `lib.rs` into ~14 focused modules following the leaf-crate-module-structure pattern.
- Convert 9 state methods on `Orchestrator` to free functions with explicit dependency parameters (resolves Feature Envy smell — only 2 of 9 methods use `self.client`).
- Co-locate single-caller helper functions with their state modules (`project_prose_candidate` → `states/md_assist_edit`, `project_table_candidate` → `states/adf_table_edit`, `route_for_node`/`has_table_ancestor`/`parent_path` → `states/classify`, `paths_overlap` → `states/merge_candidates`).
- Move shared utility helpers (`meta`, `estimate_tokens`, `compute_section_bytes`, `add_duration_suffix`) to a `util` module.
- Extract `StateTracker`, `ArtifactStore`, and error mapping (`PipelineError` + `to_hard_error` + `confluence_error_to_hard_error` + `From` impls) into their own modules.
- Redistribute 3 existing tests from `src/tests.rs` into inline `#[cfg(test)] mod tests` blocks in their destination modules; remove `src/tests.rs`.
- Reduce `lib.rs` to a thin facade: `mod` declarations, re-exports of `Orchestrator`, `PipelineError`, `RunMode`, `RunRequest`, `StateTracker`.
- No behavior changes. No public API changes. No new features.

## Capabilities

### New Capabilities
- `pipeline-module-structure`: Defines module boundaries, state-function extraction pattern (free functions over methods), facade shape, dependency graph, and visibility rules specific to the pipeline crate.

### Modified Capabilities
- `test-placement-policy`: Update pipeline-specific scenarios to reflect test distribution from central `src/tests.rs` into inline `mod tests` blocks within destination modules (`state_tracker.rs`, `util.rs`). The requirements themselves are unchanged — only the scenarios describing where pipeline tests live need updating.

## Impact

- **Code**: `crates/atlassy-pipeline/src/lib.rs` splits into ~14 files under `crates/atlassy-pipeline/src/`. `src/tests.rs` is removed.
- **Public API**: Unchanged. The 4-symbol public API (`Orchestrator`, `PipelineError`, `RunMode`, `RunRequest`) plus `StateTracker` remains importable at the crate root via facade re-exports.
- **Downstream crates**: `atlassy-cli` uses exactly these 4 symbols (line 15 of `main.rs`). No import changes needed.
- **Integration tests**: `crates/atlassy-pipeline/tests/pipeline_integration.rs` remains unchanged and must continue to pass.
- **Quality gates**: `cargo fmt`, `cargo clippy`, `cargo test` must all pass after each extraction step.
