## Why

Atlassy is currently validated in stub mode, but pilot readiness requires live Confluence execution with decision-grade provenance and trustworthy patch/telemetry behavior. This change is needed now to close the gap between archived phase outputs and real runtime evidence without expanding v1 editing scope.

## What Changes

- Add provenance stamping to run, batch, and readiness outputs (`git_commit_sha`, `git_dirty`, `pipeline_version`) so decisions are traceable.
- Ensure patch-stage correctness by applying generated `patch_ops` into candidate ADF before verify/publish, with regression coverage for prose and table-cell paths.
- Add live runtime backend support (`stub|live`) with deterministic error mapping and one-scoped-retry conflict policy alignment.
- Add non-placeholder telemetry quality improvements needed for valid KPI analysis in live/stub reruns.
- Add gated rerun workflow outputs so smoke, scoped positive runs, negative safety run, batch, and readiness replay remain deterministic.

## Capabilities

### New Capabilities
- `runtime-provenance-stamping`: Defines required provenance fields and enforcement in decision-grade outputs.
- `patch-stage-candidate-application`: Defines patch application invariants from `patch_ops` into candidate ADF before verify/publish.
- `live-confluence-runtime-selection`: Defines backend selection (`stub|live`), live fetch/publish behavior, and deterministic error mapping.

### Modified Capabilities
- `kpi-telemetry-and-reporting`: Require non-placeholder timing/token/scope telemetry quality constraints for KPI-valid outputs.
- `pipeline-state-orchestration`: Update state behavior expectations for patch-stage mutation correctness and live runtime execution parity.

## Impact

- Affected code: `crates/atlassy-cli`, `crates/atlassy-pipeline`, `crates/atlassy-confluence`, `crates/atlassy-contracts`.
- Affected outputs: run summaries, batch reports, readiness checklist/runbook/decision packet artifacts.
- Runtime impact: introduces explicit live backend execution path while preserving stub compatibility and v1 safety constraints.
