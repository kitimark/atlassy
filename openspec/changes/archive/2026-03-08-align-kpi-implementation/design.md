## Context

D-014 revised the KPI framework from internal pipeline metrics to MCP-predictive metrics. The roadmap docs, README, QA plans, and decision log now use the new metric names and targets. The Rust codebase still uses the old names and lacks the data-capture fields needed by the new KPIs. Three crates are affected: `atlassy-contracts` (struct definitions), `atlassy-pipeline` (measurement and population), and `atlassy-cli` (aggregation, gating, outlier/regression tagging). All 76 existing tests pass; struct changes will cause compilation failures that must be resolved as part of this change.

## Goals / Non-Goals

**Goals:**

- `RunSummary`, `FetchOutput`, and `PatchOutput` carry all fields required by the revised instrumentation contract.
- Pipeline fetch and patch stages measure and record byte sizes.
- Pipeline run finalization computes `context_reduction_ratio` and `out_of_scope_mutation`.
- CLI batch aggregation computes six revised KPIs instead of five legacy KPIs.
- CLI gate logic enforces revised pass/fail thresholds from `roadmap/04-kpi-and-experiments.md`.
- All tests compile and pass after the change.

**Non-Goals:**

- Changing pipeline behavior, route policy, or safety gates.
- Adding new pipeline stages or CLI commands.
- Implementing MCP server surface.
- Creating sandbox test dataset or running experiments.
- Modifying `start_ts` placeholder (tracked separately).

## Decisions

### D1: Byte size measurement method

Use `serde_json::to_vec(&value).map(|v| v.len() as u64).unwrap_or(0)` to measure ADF and patch-ops byte sizes.

**Rationale:** Consistent with how `total_tokens` is already derived from serialized payload size. `to_vec` produces compact JSON without formatting, giving a stable byte count. The `unwrap_or(0)` fallback handles the (extremely unlikely) case where serialization fails on a `serde_json::Value` that was just deserialized.

**Alternatives considered:**
- `to_string().len()` — equivalent output but allocates a `String` instead of `Vec<u8>`; marginally less efficient.
- Track byte count at the HTTP response level — more accurate for network cost but unavailable for stub runs.

### D2: `full_page_adf_bytes` lives on `FetchOutput`, not just `RunSummary`

The full-page byte count is measured in `run_fetch_state` and carried out via `FetchOutput`. The orchestrator (`run_internal`) copies it to `RunSummary`.

**Rationale:** Keeps measurement close to the data source. Matches the pattern where `scope_resolution_failed` and `full_page_fetch` are set in fetch output and then copied to the summary. Downstream stages or diagnostics can access the value from the fetch envelope without needing the summary.

### D3: `scoped_adf_bytes` is measured in `run_internal`, not in `FetchOutput`

The scoped byte count is computed from `fetch.payload.scoped_adf` in the orchestrator after the fetch stage returns, rather than inside `run_fetch_state` itself.

**Rationale:** `FetchOutput` already carries `scoped_adf` as a `serde_json::Value`. The orchestrator serializes it to measure bytes and sets `summary.scoped_adf_bytes`. This avoids adding a redundant field to `FetchOutput` (the scoped ADF value is already there — the byte count is derived from it).

### D4: `patch_ops_bytes` lives on `PatchOutput`

The patch-ops byte count is measured in `run_patch_state` after `PatchOp` vec construction and carried out via `PatchOutput`. The orchestrator copies it to `RunSummary`.

**Rationale:** Mirrors D2 for `FetchOutput`. Keeps measurement at the source. The `PatchOutput` struct is the right place because it already carries `patch_ops`.

### D5: `out_of_scope_mutation` derived from `error_codes`, same pattern as `locked_node_mutation`

Add `out_of_scope_mutation: bool` to `RunSummary`. Derive it in run finalization by checking `error_codes.contains("ERR_OUT_OF_SCOPE_MUTATION")`, matching the existing `locked_node_mutation` derivation.

**Rationale:** The `structural_preservation` KPI requires checking three conditions: `verify_result == "pass"`, `!locked_node_mutation`, `!out_of_scope_mutation`. The first two are already first-class fields. Adding the third as a first-class field keeps the KPI formula readable and consistent.

### D6: `context_reduction_ratio` computed in `run_internal`, stored as `f64`

Formula: `if full_page_adf_bytes > 0 { 1.0 - (scoped_adf_bytes as f64 / full_page_adf_bytes as f64) } else { 0.0 }`.

**Rationale:** Computing in the orchestrator (after fetch populates both values) is the earliest safe point. Storing as `f64` preserves precision. The CLI multiplies by 100 for percentage display in KPI reports.

### D7: Old KPI names fully replaced, not aliased

The four retired KPI names are removed from `kpi_values`, `build_kpi_rollup`, `evaluate_kpi_checks`, `build_outliers`, and `build_regressions`. No backward-compatible aliases are maintained.

**Rationale:** The old names are not referenced in any external contract or persisted artifact format. Run summaries are ephemeral (`artifacts/` is unversioned). The doc revision is the authority; dual-naming adds complexity with no consumer.

### D8: Gate check approach — absolute thresholds on optimized flow, not relative deltas

The old gates compared `delta_relative` between baseline and optimized (e.g., 40-60% reduction). The new gates use absolute thresholds on the optimized flow's statistics (e.g., `optimized.median >= 70.0` for `context_reduction_ratio`).

**Rationale:** The revised KPIs measure properties of the optimized flow itself (success rate, preservation rate, conflict rate) rather than deltas. `context_reduction_ratio` is inherently a comparison metric (full vs scoped) so an absolute threshold on it is equivalent to a delta check. `publish_latency` retains a relative check (`p90 non-regressive vs baseline`) alongside the new absolute median threshold.

## Risks / Trade-offs

- **Test churn** — Struct field additions break every call site that constructs `RunSummary`, `FetchOutput`, or `PatchOutput`. Mitigation: update all construction sites in a single pass using compiler errors as a checklist.
- **Serialization cost** — `serde_json::to_vec` on full-page ADF adds one extra serialization per run. Mitigation: this happens once per run, the page ADF is already in memory, and the cost is negligible relative to network I/O.
- **Ratio precision for very small pages** — If `full_page_adf_bytes` is very small (e.g., <100 bytes for a near-empty page), the ratio may be noisy. Mitigation: the KPI doc already notes this is a diagnostic metric for small pages; bootstrap runs produce `context_reduction_ratio: 0.0` by design.
- **`out_of_scope_mutation` detection depends on error code presence** — If the verifier fails to emit `ERR_OUT_OF_SCOPE_MUTATION` for a real OOS violation, the flag will be false. Mitigation: verifier already deterministically emits this error code; no change to verifier logic is in scope.
