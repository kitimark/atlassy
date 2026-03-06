## 1. Provenance Stamping Foundation

- [x] 1.1 Add shared provenance model (`git_commit_sha`, `git_dirty`, `pipeline_version`, runtime mode) for decision-grade outputs
- [x] 1.2 Implement deterministic provenance collection from git/runtime context with clear failure handling for missing or malformed values
- [x] 1.3 Wire provenance stamp into per-run summary output generation
- [x] 1.4 Wire provenance stamp into batch report output generation
- [x] 1.5 Wire provenance stamp into readiness checklist/runbook/decision packet outputs
- [x] 1.6 Add validation that blocks KPI/readiness claims when required provenance fields are absent or invalid

## 2. Patch-Stage Candidate Application Correctness

- [x] 2.1 Ensure patch stage applies `patch_ops` into `candidate_page_adf` before verify
- [x] 2.2 Ensure publish uses the same verified candidate payload produced by patch stage
- [x] 2.3 Persist patch-stage evidence proving candidate mutation corresponds to `patch_ops`
- [x] 2.4 Add regression tests for prose path patch application correctness
- [x] 2.5 Add regression tests for table-cell path patch application correctness
- [x] 2.6 Add regression tests confirming non-targeted paths remain unchanged

## 3. Live Confluence Runtime Selection and Error Mapping

- [x] 3.1 Add explicit runtime backend selector (`stub|live`) to CLI/runtime configuration
- [x] 3.2 Implement live fetch/publish runtime path using environment-driven configuration
- [x] 3.3 Keep stub backend behavior unchanged and covered by existing fixture tests
- [x] 3.4 Map live runtime failures into deterministic error taxonomy used by verifier/reporting/runbooks
- [x] 3.5 Verify one-scoped-retry conflict behavior is enforced consistently in live runtime path
- [x] 3.6 Include selected runtime mode in run, batch, and readiness artifacts

## 4. Telemetry Quality for KPI Validity

- [x] 4.1 Replace placeholder/static timing telemetry with measured operational timing values
- [x] 4.2 Ensure token accounting fields are populated with real per-state and aggregate values
- [x] 4.3 Ensure scope retrieval telemetry (`scope_resolution_failed`, `full_page_fetch`, related diagnostics) is consistently populated
- [x] 4.4 Enforce non-evaluable marking for runs with incomplete telemetry or invalid provenance
- [x] 4.5 Ensure deterministic KPI aggregation excludes non-decision-grade runs with explicit diagnostics

## 5. Gated Validation Rerun and Deterministic Evidence

- [x] 5.1 Add/refresh command path and fixture support for smoke no-op rerun checkpoint
- [x] 5.2 Add/refresh command path and fixture support for scoped prose update checkpoint
- [x] 5.3 Add/refresh command path and fixture support for scoped table-cell update checkpoint
- [x] 5.4 Add/refresh command path and fixture support for negative safety checkpoint (expected hard fail)
- [x] 5.5 Ensure batch + readiness replay verification path emits deterministic, provenance-stamped outputs
- [x] 5.6 Add tests for stop conditions: missing provenance, replay mismatch, retry-policy breach, unmapped hard live errors
- [x] 5.7 Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` and fix issues
