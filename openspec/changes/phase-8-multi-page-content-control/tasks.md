## 1. New Types (atlassy-contracts)

- [x] 1.1 Add `MultiPageRequest { plan_id, pages: Vec<PageTarget>, rollback_on_failure, provenance, timestamp }` to types.rs
- [x] 1.2 Add `PageTarget { page_id: Option<String>, create: Option<CreatePageTarget>, edit_intent, scope_selectors, run_mode, block_ops, bootstrap_empty_page, depends_on: Vec<String> }` to types.rs
- [x] 1.3 Add `CreatePageTarget { title, parent_page_id, space_key }` to types.rs
- [x] 1.4 Add `PageSnapshot { page_id, version_before, adf_before, version_after: Option<u64> }` to types.rs
- [x] 1.5 Add `MultiPageSummary { plan_id, success, page_results, rollback_results, total_pages, succeeded_pages, failed_pages, rolled_back_pages, latency_ms }` to types.rs
- [x] 1.6 Add `PageResult { page_id, created, summary: RunSummary }` to types.rs
- [x] 1.7 Add `RollbackResult { page_id, success, conflict, error: Option<String> }` to types.rs
- [x] 1.8 Verify all new types derive Serialize, Deserialize, Debug, Clone

## 2. Error Codes (atlassy-contracts)

- [x] 2.1 Add `MultiPagePartialFailure` variant to `ErrorCode` with `as_str` returning `"ERR_MULTI_PAGE_PARTIAL_FAILURE"`
- [x] 2.2 Add `RollbackConflict` variant with `as_str` returning `"ERR_ROLLBACK_CONFLICT"`
- [x] 2.3 Add `DependencyCycle` variant with `as_str` returning `"ERR_DEPENDENCY_CYCLE"`
- [x] 2.4 Add `PageCreationFailed` variant with `as_str` returning `"ERR_PAGE_CREATION_FAILED"`
- [x] 2.5 Update `ErrorCode::ALL` array and test coverage for all 4 new variants

## 3. Dependency Ordering Module (atlassy-pipeline)

- [x] 3.1 Create topological sort function: `sort_page_targets(pages: &[PageTarget]) -> Result<Vec<usize>, PipelineError>` returning indices in execution order
- [x] 3.2 Implement cycle detection — return `ERR_DEPENDENCY_CYCLE` if graph has cycles
- [x] 3.3 Implement unresolvable dependency detection — error if `depends_on` references a page_id not in the plan
- [x] 3.4 Handle pages with `page_id: None` (creating) — use plan-local index or stable identifier for dependency references

## 4. PageSnapshot Module (atlassy-pipeline)

- [x] 4.1 Implement `take_snapshot(client, page_id) -> Result<PageSnapshot, PipelineError>` — calls `fetch_page`, saves ADF + version
- [x] 4.2 Implement `rollback_page(client, snapshot) -> RollbackResult` — fetches current version, checks for conflict, re-publishes original ADF if safe
- [x] 4.3 Handle rollback conflict: if current_version != snapshot.version_after, return `RollbackResult { conflict: true }`

## 5. MultiPageOrchestrator Core (atlassy-pipeline)

- [x] 5.1 Create `atlassy-pipeline/src/multi_page.rs` with `MultiPageOrchestrator<C: ConfluenceClient>` struct
- [x] 5.2 Implement `new(client, artifact_root)` constructor (mirrors Orchestrator pattern)
- [x] 5.3 Implement `run(multi_request: MultiPageRequest) -> Result<MultiPageSummary, PipelineError>` entry point
- [x] 5.4 Implement validation step: call topological sort, reject cycles and unresolvable deps
- [x] 5.5 Implement snapshot step: for each existing page (has page_id), call `take_snapshot`
- [x] 5.6 Implement execution loop: iterate pages in sorted order, build RunRequest from PageTarget, call `Orchestrator::run()`
- [x] 5.7 On page success: record `PageResult`, update `snapshot.version_after`
- [x] 5.8 On page failure: record failed `PageResult`, stop execution, proceed to rollback

## 6. Content-Bearing Page Creation (atlassy-pipeline)

- [x] 6.1 In execution loop: if `PageTarget.create` is present, call `client.create_page(title, parent_page_id, space_key)` before pipeline run
- [x] 6.2 Assign returned page_id to the `RunRequest` for pipeline execution
- [x] 6.3 Set `bootstrap_empty_page: true` on the `RunRequest` for created pages
- [x] 6.4 On `create_page` failure: report `ERR_PAGE_CREATION_FAILED`, stop execution, proceed to rollback
- [x] 6.5 Mark `PageResult.created = true` for pages that were created

## 7. Rollback Logic (atlassy-pipeline)

- [x] 7.1 Implement rollback step: if `rollback_on_failure` is true and a page failed, iterate successfully published pages in reverse order
- [x] 7.2 For each page: call `rollback_page(client, snapshot)` and collect `RollbackResult`
- [x] 7.3 Skip created pages during rollback (no prior state to restore)
- [x] 7.4 Populate `MultiPageSummary.rollback_results`
- [x] 7.5 If `rollback_on_failure` is false: skip rollback, leave `rollback_results` empty

## 8. Summary Construction (atlassy-pipeline)

- [x] 8.1 Build `MultiPageSummary` with aggregate stats: `total_pages`, `succeeded_pages`, `failed_pages`, `rolled_back_pages`
- [x] 8.2 Set `success = true` only if all pages succeeded (no failures, no rollback needed)
- [x] 8.3 Compute `latency_ms` from plan start to completion
- [x] 8.4 Persist summary via artifact store

## 9. Pipeline Exports

- [x] 9.1 Add `pub mod multi_page` to `atlassy-pipeline/src/lib.rs`
- [x] 9.2 Export `MultiPageOrchestrator` from the pipeline crate

## 10. CLI Command (atlassy-cli)

- [x] 10.1 Add `RunMultiPage(RunMultiPageArgs)` variant to CLI `Commands` enum
- [x] 10.2 Add `RunMultiPageArgs { manifest: PathBuf, artifacts_dir: PathBuf, runtime_backend: String }`
- [x] 10.3 Implement manifest parsing: read JSON file, deserialize as `MultiPageRequest`
- [x] 10.4 Implement command execution: construct `MultiPageOrchestrator`, call `.run()`, print `MultiPageSummary` as JSON
- [x] 10.5 Handle manifest file not found and parse errors with clear error messages

## 11. Unit Tests

- [x] 11.1 Test topological sort: linear chain, diamond dependencies, independent pages, empty list
- [x] 11.2 Test cycle detection: direct cycle, indirect cycle, self-dependency
- [x] 11.3 Test unresolvable dependency detection
- [x] 11.4 Test `take_snapshot`: successful fetch, fetch failure
- [x] 11.5 Test `rollback_page`: successful rollback, version conflict, fetch failure during rollback
- [x] 11.6 Test all new types serde round-trip

## 12. Integration Tests (atlassy-pipeline)

- [x] 12.1 Test multi-page: 2 pages succeed, both published
- [x] 12.2 Test multi-page: page B fails, page A rolled back successfully
- [x] 12.3 Test multi-page: page B fails, page A rollback conflicts (concurrent edit simulated)
- [x] 12.4 Test multi-page: create sub-page with content, verify page created and content applied
- [x] 12.5 Test multi-page: dependency ordering respected (A before B)
- [x] 12.6 Test multi-page: cycle rejected before execution
- [x] 12.7 Test multi-page: rollback_on_failure=false, no rollback attempted
- [x] 12.8 Test backward compatibility: existing single-page `Orchestrator::run()` unchanged

## 13. Final Validation

- [x] 13.1 Run `cargo test --workspace` — all tests pass
- [x] 13.2 Run `cargo clippy --workspace` — zero warnings
- [x] 13.3 Verify `Orchestrator::run()` source is unchanged (no per-page pipeline modifications)
- [x] 13.4 Verify no pipeline state files (`states/*.rs`) were modified
