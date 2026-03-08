## 1. Tier 1: state_tracker extraction (proof of concept)

- [ ] 1.1 Create `src/state_tracker.rs` with `StateTracker` struct, `new()`, `transition_to()`, `Default` impl (lines 87-116 of current `lib.rs`)
- [ ] 1.2 Move the `state_tracker_blocks_out_of_order_transitions` test from `src/tests.rs` into an inline `#[cfg(test)] mod tests` block in `state_tracker.rs`
- [ ] 1.3 Add `mod state_tracker;` declaration and `pub use state_tracker::StateTracker;` re-export to `lib.rs`; remove `StateTracker` code from `lib.rs`
- [ ] 1.4 Run `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` — verify all pass

## 2. Tier 2: error_map extraction

- [ ] 2.1 Create `src/error_map.rs` with `PipelineError` enum (lines 71-85), `to_hard_error` (lines 1501-1519), `confluence_error_to_hard_error` (lines 1473-1499), `From<AdfError>` impl (lines 1521-1525), `From<ConfluenceError>` impl (lines 1527-1531); make `to_hard_error` and `confluence_error_to_hard_error` `pub(crate)`
- [ ] 2.2 Add `mod error_map;` declaration and `pub use error_map::PipelineError;` re-export to `lib.rs`; remove error code from `lib.rs`
- [ ] 2.3 Run `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` — verify all pass

## 3. Tier 3: artifact_store and util extraction

- [ ] 3.1 Create `src/artifact_store.rs` with `ArtifactStore` struct, `new()`, `persist_state()`, `persist_summary()` (lines 118-163); import `PipelineError` from `error_map`
- [ ] 3.2 Add `mod artifact_store;` declaration and `pub use artifact_store::ArtifactStore;` re-export to `lib.rs`; remove `ArtifactStore` code from `lib.rs`
- [ ] 3.3 Create `src/util.rs` with `pub(crate) fn meta()` (lines 1291-1298), `pub(crate) fn estimate_tokens()` (lines 1445-1449), `pub(crate) fn compute_section_bytes()` (lines 1451-1467), `pub(crate) fn add_duration_suffix()` (lines 1469-1471)
- [ ] 3.4 Move the 2 `compute_section_bytes` tests from `src/tests.rs` into an inline `#[cfg(test)] mod tests` block in `util.rs`
- [ ] 3.5 Add `mod util;` to `lib.rs`; remove utility function code from `lib.rs`
- [ ] 3.6 Run `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` — verify all pass

## 4. Tier 4: states extraction

- [ ] 4.1 Create `src/states/` directory and `src/states/mod.rs` barrel file with `mod` declarations and `pub(crate) use` re-exports for all 9 state functions
- [ ] 4.2 Create `src/states/fetch.rs` — extract `run_fetch_state` (lines 499-551); convert to free function with `client: &mut C`, `artifact_store: &ArtifactStore`, `request: &RunRequest`, `tracker: &mut StateTracker` parameters; import `meta` from `util`, error converters from `error_map`
- [ ] 4.3 Create `src/states/classify.rs` — extract `run_classify_state` (lines 553-593) + co-locate `route_for_node` (lines 1403-1419), `has_table_ancestor` (lines 1421-1432), `parent_path` (lines 1434-1443) as private helpers
- [ ] 4.4 Create `src/states/extract_prose.rs` — extract `run_extract_prose_state` (lines 595-669)
- [ ] 4.5 Create `src/states/md_assist_edit.rs` — extract `run_md_assist_edit_state` (lines 671-799) + co-locate `project_prose_candidate` (lines 1300-1337) as private helper
- [ ] 4.6 Create `src/states/adf_table_edit.rs` — extract `run_adf_table_edit_state` (lines 801-948) + co-locate `project_table_candidate` (lines 1339-1391) as private helper
- [ ] 4.7 Create `src/states/merge_candidates.rs` — extract `run_merge_candidates_state` (lines 950-1036) + co-locate `paths_overlap` (lines 1393-1401) as private helper
- [ ] 4.8 Create `src/states/patch.rs` — extract `run_patch_state` (lines 1038-1110)
- [ ] 4.9 Create `src/states/verify.rs` — extract `run_verify_state` (lines 1112-1184)
- [ ] 4.10 Create `src/states/publish.rs` — extract `run_publish_state` (lines 1186-1275); convert to free function with `client: &mut C` parameter
- [ ] 4.11 Add `mod states;` to `lib.rs`; remove all state methods and co-located helpers from `lib.rs`
- [ ] 4.12 Run `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` — verify all pass

## 5. Tier 5: orchestrator extraction and facade finalization

- [ ] 5.1 Create `src/orchestrator.rs` — extract `Orchestrator<C>` struct (lines 165-168), `impl<C: ConfluenceClient> Orchestrator<C>` block containing `new()`, `client()`, `client_mut()`, `run()`, `run_internal()`, `hard_fail()` with bootstrap interlude; update `run_internal` to call `states::run_*_state(...)` free functions instead of `self.run_*_state(...)`
- [ ] 5.2 Add `mod orchestrator;` and `pub use orchestrator::Orchestrator;` to `lib.rs`
- [ ] 5.3 Finalize `lib.rs` as facade — verify it contains only: `use` imports, `mod` declarations (6 modules), `pub use` re-exports, `RunMode` enum, `RunRequest` struct; no `fn` definitions, no `impl` blocks with logic
- [ ] 5.4 Delete `src/tests.rs` — all 3 tests have been redistributed (1 in `state_tracker.rs`, 2 in `util.rs`)
- [ ] 5.5 Run `cargo fmt --all -- --check` — verify zero formatting issues
- [ ] 5.6 Run `cargo clippy --workspace --all-targets -- -D warnings` — verify zero warnings
- [ ] 5.7 Run `cargo test --workspace` — verify all tests pass and test count is preserved
