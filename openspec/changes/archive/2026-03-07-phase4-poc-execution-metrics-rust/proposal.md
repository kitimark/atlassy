## Why

Phases 1-3 implemented the v1 pipeline behavior, but we do not yet have reproducible PoC evidence that the system meets token, scope-retrieval, fidelity, and latency targets. Phase 4 is needed now to run structured experiments and produce decision-grade metrics for `go | iterate | stop`.

## What Changes

- Define and implement a PoC execution harness for paired baseline vs optimized runs across Pattern A/B/C and the baseline page set.
- Add required per-run telemetry fields, artifact indexing, and aggregation logic aligned to KPI protocol.
- Add reporting outputs for median/p90 comparisons, delta calculations, pass/fail rules, and recommendation synthesis.
- Add live-vs-stub drift detection workflow and scenario coverage checks for required v1 error/safety paths.
- Add runbook-oriented validation gates so incomplete telemetry, safety violations, or retry policy breaches fail the PoC workflow deterministically.

## Capabilities

### New Capabilities
- `poc-run-matrix-execution`: Executes deterministic paired experiment runs (baseline and optimized) over defined patterns/pages with manifest tracking.
- `kpi-telemetry-and-reporting`: Collects required KPI telemetry and generates aggregate comparison reports with pass/fail outcomes.
- `drift-validation-and-scenario-coverage`: Validates stub/live behavior parity and required scenario coverage before PoC sign-off.

### Modified Capabilities
- None.

## Impact

- Affected code: `crates/atlassy-cli` (batch/run-manifest entrypoints), `crates/atlassy-pipeline` (telemetry completeness and summary emission), and test/fixture orchestration under `crates/atlassy-pipeline/tests`.
- Data/reporting outputs: artifact index generation, per-run summaries, and aggregate KPI reports for decision review.
- Operational process: codifies PoC runbook, gate checks, and drift handling expected by `roadmap/04-kpi-and-experiments.md` and `roadmap/10-testing-strategy-and-simulation.md`.
- Dependencies/systems: continued use of stub simulation plus controlled live Confluence sandbox probes; no v1 scope expansion to deferred features.
