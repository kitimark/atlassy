## Why

Phase 4 delivered PoC execution and reporting, but we still need explicit hardening and readiness controls before a defensible v1 `go | iterate | stop` decision. Phase 5 is needed now to convert PoC outputs into operator-ready runbooks, gated readiness evidence, and decision governance artifacts.

## What Changes

- Add a deterministic readiness-gate checklist workflow that validates entry/exit conditions from the v1 execution-readiness plan.
- Add operator-facing triage runbook outputs for high-priority failure classes, including verify failures, scoped-retry exhaustion, and safety-gate blockers.
- Add risk re-scoring and decision-packet assembly that links KPI outcomes, failure summaries, and recommendation rationale.
- Add reproducibility checks ensuring readiness outputs can be regenerated from stored batch artifacts.

## Capabilities

### New Capabilities
- `readiness-gate-checklist`: Defines required readiness gates, pass/fail recording, and blocking behavior for incomplete sign-off conditions.
- `operator-triage-runbooks`: Defines deterministic runbook outputs for priority error classes and escalation ownership.
- `decision-packet-governance`: Defines final decision packet contents, risk/KPI linkage, and recommendation traceability requirements.

### Modified Capabilities
- None.

## Impact

- Affected code: `crates/atlassy-cli` (readiness/decision packet commands and outputs), with possible supporting updates in `crates/atlassy-pipeline` and `crates/atlassy-contracts` for structured evidence fields.
- Affected artifacts: batch-level readiness reports, runbook bundles, and decision packet files under `artifacts/`.
- Operational impact: adds explicit ownership-ready outputs for release review and phase-exit governance without expanding v1 editing scope.
