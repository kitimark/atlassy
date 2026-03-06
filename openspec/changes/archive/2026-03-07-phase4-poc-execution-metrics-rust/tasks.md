## 1. Run Manifest and Matrix Execution

- [x] 1.1 Define run manifest schema for `(run_id, page_id, pattern, flow, edit_intent_hash)` with strict required fields
- [x] 1.2 Implement manifest loader/validator that rejects duplicate `run_id` values and missing identity fields
- [x] 1.3 Implement paired-key validation ensuring each `(page_id, pattern, edit_intent_hash)` has both baseline and optimized runs
- [x] 1.4 Implement deterministic execution ordering for manifest entries and persist batch manifest metadata in artifacts

## 2. Batch Runner and Retry Invariant Enforcement

- [x] 2.1 Add CLI batch entrypoint for PoC run-matrix execution using manifest input
- [x] 2.2 Implement batch runner orchestration that executes baseline/optimized pairs while preserving existing pipeline safety gates
- [x] 2.3 Enforce one-scoped-retry maximum as a batch invariant and fail runs that exceed policy
- [x] 2.4 Emit run-level diagnostics for manifest/runner failures with deterministic error classification

## 3. KPI Telemetry Completeness and Artifact Indexing

- [x] 3.1 Extend run summary schema to include all KPI-required telemetry fields from roadmap definitions
- [x] 3.2 Implement telemetry completeness validator that marks incomplete runs invalid for aggregation
- [x] 3.3 Add artifact index generation linking run summaries, per-state artifacts, and batch metadata
- [x] 3.4 Add tests proving incomplete telemetry blocks KPI report generation

## 4. KPI Aggregation and Reporting

- [x] 4.1 Implement deterministic KPI aggregation for baseline vs optimized (median, p90, min, max, deltas)
- [x] 4.2 Implement per-pattern (A/B/C) and global rollup reporting outputs
- [x] 4.3 Implement target/pass-fail evaluation against v1 KPI thresholds and safety gate rules
- [x] 4.4 Generate recommendation-ready report sections (`go | iterate | stop`) with outlier and regression summaries

## 5. Drift Validation and Scenario Coverage Gates

- [x] 5.1 Implement live-vs-stub drift status input model and comparison checks for key behavior parity
- [x] 5.2 Gate final PoC sign-off when unresolved material drift is present
- [x] 5.3 Implement required scenario ID coverage validation for positive and negative v1 paths
- [x] 5.4 Ensure safety violations (locked-node, out-of-scope, table-shape) are surfaced as hard blockers in final reports

## 6. End-to-End Verification and PoC Runbook Fit

- [x] 6.1 Add fixture-backed integration tests for complete paired matrix execution and aggregate report generation
- [x] 6.2 Add integration tests for unmatched pair rejection, retry-limit breach, and drift/coverage gate failures
- [x] 6.3 Verify artifact replay index and report outputs are reproducible from stored run data
- [x] 6.4 Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` and fix any issues
