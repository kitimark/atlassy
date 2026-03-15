## 1. New Types (atlassy-contracts)

- [ ] 1.1 Add `MultiPageRequest { plan_id, pages: Vec<PageTarget>, rollback_on_failure, provenance, timestamp }` to types.rs
- [ ] 1.2 Add `PageTarget { page_id: Option<String>, create: Option<CreatePageTarget>, edit_intent, scope_selectors, run_mode, block_ops, bootstrap_empty_page, depends_on: Vec<String> }` to types.rs
- [ ] 1.3 Add `CreatePageTarget { title, parent_page_id, space_key }` to types.rs
- [ ] 1.4 Add `PageSnapshot { page_id, version_before, adf_before, version_after: Option<u64> }` to types.rs
- [ ] 1.5 Add `MultiPageSummary { plan_id, success, page_results, rollback_results, total_pages, succeeded_pages, failed_pages, rolled_back_pages, latency_ms }` to types.rs
- [ ] 1.6 Add `PageResult { page_id, created, summary: RunSummary }` to types.rs
- [ ] 1.7 Add `RollbackResult { page_id, success, conflict, error: Option<String> }` to types.rs
- [ ] 1.8 Verify all new types derive Serialize, Deserialize, Debug, Clone

## 2. Error Codes (atlassy-contracts)

- [ ] 2.1 Add `MultiPagePartialFailure` variant to `ErrorCode` with `as_str` returning `"ERR_MULTI_PAGE_PARTIAL_FAILURE"`
- [ ] 2.2 Add `RollbackConflict` variant with `as_str` returning `"ERR_ROLLBACK_CONFLICT"`
- [ ] 2.3 Add `DependencyCycle` variant with `as_str` returning `"ERR_DEPENDENCY_CYCLE"`
- [ ] 2.4 Add `PageCreationFailed` variant with `as_str` returning `"ERR_PAGE_CREATION_FAILED"`
- [ ] 2.5 Update `ErrorCode::ALL` array and test coverage for all 4 new variants

## 3. Dependency Ordering Module (atlassy-pipeline)

- [ ] 3.1 Create topological sort function: `sort_page_targets(pages: &[PageTarget]) -> Result<Vec<usize>, PipelineError>` returning indices in execution order
- [ ] 3.2 Implement cycle detection — return `ERR_DEPENDENCY_CYCLE` if graph has cycles
- [ ] 3.3 Implement unresolvable dependency detection — error if `depends_on` references a page_id not in the plan
- [ ] 3.4 Handle pages with `page_id: None` (creating) — use plan-local index or stable identifier for dependency references

## 4. PageSnapshot Module (atlassy-pipeline)

- [ ] 4.1 Implement `take_snapshot(client, page_id) -> Result<PageSnapshot, PipelineError>` — calls `fetch_page`, saves ADF + version
- [ ] 4.2 Implement `rollback_page(client, snapshot) -> RollbackResult` — fetches current version, checks for conflict, re-publishes original ADF if safe
- [ ] 4.3 Handle rollback conflict: if current_version != snapshot.version_after, return `RollbackResult { conflict: true }`

## 5. MultiPageOrchestrator Core (atlassy-pipeline)

- [ ] 5.1 Create `atlassy-pipeline/src/multi_page.rs` with `MultiPageOrchestrator<C: ConfluenceClient>` struct
- [ ] 5.2 Implement `new(client, artifact_root)` constructor (mirrors Orchestrator pattern)
- [ ] 5.3 Implement `run(multi_request: MultiPageRequest) -> Result<MultiPageSummary, PipelineError>` entry point
- [ ] 5.4 Implement validation step: call topological sort, reject cycles and unresolvable deps
- [ ] 5.5 Implement snapshot step: for each existing page (has page_id), call `take_snapshot`
- [ ] 5.6 Implement execution loop: iterate pages in sorted order, build RunRequest from PageTarget, call `Orchestrator::run()`
- [ ] 5.7 On page success: record `PageResult`, update `snapshot.version_after`
- [ ] 5.8 On page failure: record failed `PageResult`, stop execution, proceed to rollback

## 6. Content-Bearing Page Creation (atlassy-pipeline)

- [ ] 6.1 In execution loop: if `PageTarget.create` is present, call `client.create_page(title, parent_page_id, space_key)` before pipeline run
- [ ] 6.2 Assign returned page_id to the `RunRequest` for pipeline execution
- [ ] 6.3 Set `bootstrap_empty_page: true` on the `RunRequest` for created pages
- [ ] 6.4 On `create_page` failure: report `ERR_PAGE_CREATION_FAILED`, stop execution, proceed to rollback
- [ ] 6.5 Mark `PageResult.created = true` for pages that were created

## 7. Rollback Logic (atlassy-pipeline)

- [ ] 7.1 Implement rollback step: if `rollback_on_failure` is true and a page failed, iterate successfully published pages in reverse order
- [ ] 7.2 For each page: call `rollback_page(client, snapshot)` and collect `RollbackResult`
- [ ] 7.3 Skip created pages during rollback (no prior state to restore)
- [ ] 7.4 Populate `MultiPageSummary.rollback_results`
- [ ] 7.5 If `rollback_on_failure` is false: skip rollback, leave `rollback_results` empty

## 8. Summary Construction (atlassy-pipeline)

- [ ] 8.1 Build `MultiPageSummary` with aggregate stats: `total_pages`, `succeeded_pages`, `failed_pages`, `rolled_back_pages`
- [ ] 8.2 Set `success = true` only if all pages succeeded (no failures, no rollback needed)
- [ ] 8.3 Compute `latency_ms` from plan start to completion
- [ ] 8.4 Persist summary via artifact store

## 9. Pipeline Exports

- [ ] 9.1 Add `pub mod multi_page` to `atlassy-pipeline/src/lib.rs`
- [ ] 9.2 Export `MultiPageOrchestrator` from the pipeline crate

## 10. CLI Command (atlassy-cli)

- [ ] 10.1 Add `RunMultiPage(RunMultiPageArgs)` variant to CLI `Commands` enum
- [ ] 10.2 Add `RunMultiPageArgs { manifest: PathBuf, artifacts_dir: PathBuf, runtime_backend: String }`
- [ ] 10.3 Implement manifest parsing: read JSON file, deserialize as `MultiPageRequest`
- [ ] 10.4 Implement command execution: construct `MultiPageOrchestrator`, call `.run()`, print `MultiPageSummary` as JSON
- [ ] 10.5 Handle manifest file not found and parse errors with clear error messages

## 11. Unit Tests

- [ ] 11.1 Test topological sort: linear chain, diamond dependencies, independent pages, empty list
- [ ] 11.2 Test cycle detection: direct cycle, indirect cycle, self-dependency
- [ ] 11.3 Test unresolvable dependency detection
- [ ] 11.4 Test `take_snapshot`: successful fetch, fetch failure
- [ ] 11.5 Test `rollback_page`: successful rollback, version conflict, fetch failure during rollback
- [ ] 11.6 Test all new types serde round-trip

## 12. Integration Tests (atlassy-pipeline)

- [ ] 12.1 Test multi-page: 2 pages succeed, both published
- [ ] 12.2 Test multi-page: page B fails, page A rolled back successfully
- [ ] 12.3 Test multi-page: page B fails, page A rollback conflicts (concurrent edit simulated)
- [ ] 12.4 Test multi-page: create sub-page with content, verify page created and content applied
- [ ] 12.5 Test multi-page: dependency ordering respected (A before B)
- [ ] 12.6 Test multi-page: cycle rejected before execution
- [ ] 12.7 Test multi-page: rollback_on_failure=false, no rollback attempted
- [ ] 12.8 Test backward compatibility: existing single-page `Orchestrator::run()` unchanged

## 13. Final Validation

- [ ] 13.1 Run `cargo test --workspace` — all tests pass
- [ ] 13.2 Run `cargo clippy --workspace` — zero warnings
- [ ] 13.3 Verify `Orchestrator::run()` source is unchanged (no per-page pipeline modifications)
- [ ] 13.4 Verify no pipeline state files (`states/*.rs`) were modified
