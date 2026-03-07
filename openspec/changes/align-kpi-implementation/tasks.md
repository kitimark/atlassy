## 1. Contract Struct Changes (atlassy-contracts)

- [ ] 1.1 Add `full_page_adf_bytes: u64` field to `FetchOutput` struct
- [ ] 1.2 Add `patch_ops_bytes: u64` field to `PatchOutput` struct
- [ ] 1.3 Add `full_page_adf_bytes: u64`, `scoped_adf_bytes: u64`, `context_reduction_ratio: f64`, `patch_ops_bytes: u64`, and `out_of_scope_mutation: bool` fields to `RunSummary` struct
- [ ] 1.4 Update `validate_run_summary_telemetry` if needed for new fields
- [ ] 1.5 Update the `RunSummary` test construction at line 896 with all new fields

## 2. Pipeline Measurement and Population (atlassy-pipeline)

- [ ] 2.1 In `run_fetch_state`: measure `full_page_adf_bytes` via `serde_json::to_vec(&page.adf)` before scoping and include in `FetchOutput`
- [ ] 2.2 In `run_patch_state`: measure `patch_ops_bytes` via `serde_json::to_vec(&patch_ops)` and include in `PatchOutput`
- [ ] 2.3 In `run` summary initialization: add default values for all 5 new `RunSummary` fields
- [ ] 2.4 In `run_internal`: after fetch, copy `full_page_adf_bytes` from fetch output to summary and measure `scoped_adf_bytes` from `fetch.payload.scoped_adf`
- [ ] 2.5 In `run_internal`: compute `context_reduction_ratio` from `full_page_adf_bytes` and `scoped_adf_bytes`
- [ ] 2.6 In `run_internal`: after patch, copy `patch_ops_bytes` from patch output to summary
- [ ] 2.7 In `run` finalization: derive `out_of_scope_mutation` from `error_codes` containing `ERR_OUT_OF_SCOPE_MUTATION`
- [ ] 2.8 Update all `FetchOutput` and `PatchOutput` construction sites in pipeline tests with new fields

## 3. CLI KPI Aggregation Replacement (atlassy-cli)

- [ ] 3.1 Replace KPI name list in `build_kpi_rollup` with the six revised names: `context_reduction_ratio`, `scoped_section_tokens`, `edit_success_rate`, `structural_preservation`, `conflict_rate`, `publish_latency`
- [ ] 3.2 Rewrite `kpi_values` match arms with revised formulas for all six KPIs
- [ ] 3.3 Rewrite `evaluate_kpi_checks` with absolute-threshold gates for all five checked KPIs
- [ ] 3.4 Update `build_outliers` to use `context_reduction_ratio` (lowest) instead of `tokens_per_successful_update` (highest)
- [ ] 3.5 Update `build_regressions` to use `context_reduction_ratio` and `edit_success_rate` instead of retired KPI names
- [ ] 3.6 Update risk delta reference at line 1406 from `full_page_retrieval_rate` to `context_reduction_ratio`
- [ ] 3.7 Update all `RunSummary` construction sites in CLI tests with new fields

## 4. Verification

- [ ] 4.1 Run `cargo test --workspace` and fix any remaining compilation errors from struct changes
- [ ] 4.2 Verify all 76+ tests pass with no regressions
- [ ] 4.3 Run a stub-mode `run` and confirm new fields appear in `summary.json` output
