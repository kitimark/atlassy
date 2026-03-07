## Why

The KPI framework was revised in D-014 to replace internal pipeline metrics with MCP-predictive metrics that measure real-world scoped-editing value. The roadmap, README, QA, and decision docs now reference six new KPIs (`context_reduction_ratio`, `scoped_section_tokens`, `edit_success_rate`, `structural_preservation`, `conflict_rate`, `publish_latency`) and four new run-summary fields (`full_page_adf_bytes`, `scoped_adf_bytes`, `context_reduction_ratio`, `patch_ops_bytes`). The Rust implementation still uses the old KPI names and does not capture or compute the new fields. The code must be aligned before the revised PoC experiment can produce meaningful results.

## What Changes

- **BREAKING**: Add `full_page_adf_bytes: u64` to `FetchOutput` struct.
- **BREAKING**: Add `patch_ops_bytes: u64` to `PatchOutput` struct.
- **BREAKING**: Add `full_page_adf_bytes`, `scoped_adf_bytes`, `context_reduction_ratio`, `patch_ops_bytes`, and `out_of_scope_mutation` fields to `RunSummary` struct.
- Measure full-page ADF byte size in fetch stage before scoping, and scoped ADF byte size after scoping.
- Measure serialized patch operations byte size in patch stage.
- Compute `context_reduction_ratio` and `out_of_scope_mutation` in pipeline run finalization.
- Replace four retired KPIs (`tokens_per_successful_update`, `full_page_retrieval_rate`, `retry_conflict_token_waste`, `formatting_fidelity_pass_rate`) with five new KPIs (`context_reduction_ratio`, `scoped_section_tokens`, `edit_success_rate`, `structural_preservation`, `conflict_rate`) in batch aggregation and gate logic. Keep `publish_latency` with tightened threshold.
- Update all gate checks to use revised pass/fail rules from `roadmap/04-kpi-and-experiments.md`.
- Update outlier detection and regression tagging to reference new KPI names.

## Capabilities

### New Capabilities

- `scoped-payload-sizing`: Capture full-page and scoped ADF byte sizes in fetch output and run summary, and compute context reduction ratio per run.
- `patch-ops-sizing`: Capture serialized patch operations byte size in patch output and run summary.

### Modified Capabilities

- `kpi-telemetry-and-reporting`: Replace retired KPI names and formulas with revised metrics; update gate thresholds and pass/fail rules to match D-014.
- `scoped-adf-fetch-and-index`: Fetch output adds `full_page_adf_bytes` field to carry pre-scoping payload size.

## Impact

- `crates/atlassy-contracts`: `FetchOutput`, `PatchOutput`, and `RunSummary` structs gain new fields (breaking for all callers that construct these structs).
- `crates/atlassy-pipeline`: Fetch and patch stages measure byte sizes; run finalization computes ratio and OOS mutation flag.
- `crates/atlassy-cli`: KPI aggregation, gate evaluation, outlier detection, and regression tagging are rewritten for new metric names and thresholds.
- All existing tests that construct `RunSummary`, `FetchOutput`, or `PatchOutput` will fail to compile until updated with the new fields.
