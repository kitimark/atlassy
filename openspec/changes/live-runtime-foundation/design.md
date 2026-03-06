## Context

Atlassy currently proves behavior through deterministic stub execution and fixture-backed tests, but pilot-readiness evidence must be produced against a live Confluence sandbox with traceable provenance and replayable artifacts. The current recommendation remains `iterate`, so this change focuses on runtime foundation quality rather than scope expansion.

Constraints:
- Preserve v1 edit boundaries (no new editing capabilities).
- Keep one-scoped-retry conflict policy unchanged.
- Preserve deterministic artifact/replay model already established in prior phases.
- Ensure outputs used for decisions always carry implementation provenance.

Stakeholders are the same readiness roles from roadmap docs: engineering owner, QA owner, metrics owner, and release reviewer.

## Goals / Non-Goals

**Goals:**
- Add provenance stamping (`git_commit_sha`, `git_dirty`, `pipeline_version`) across run, batch, and readiness outputs.
- Guarantee patch-stage correctness by applying `patch_ops` to candidate ADF before verify/publish.
- Add a runtime backend selector (`stub|live`) that preserves stub behavior and enables live sandbox fetch/publish with deterministic error mapping.
- Improve telemetry quality so KPI calculations are based on meaningful timing/token/scope data.
- Support deterministic gated rerun outputs across smoke, scoped positive, negative safety, batch, and readiness replay checks.

**Non-Goals:**
- Expanding v1 route matrix or enabling deferred capabilities from `ideas/`.
- Multi-page orchestration or autonomous conflict resolution.
- Long-term analytics/data platform redesign.

## Decisions

1. Add a single shared provenance stamp model used by run, batch, and readiness artifacts.
   - Rationale: avoids drift between artifact types and enforces a consistent decision-evidence contract.
   - Alternatives considered:
     - Independent provenance fields per artifact type: rejected due to schema divergence risk.
     - Batch-only provenance: rejected because per-run traceability is required for incident triage.

2. Keep patch application inside orchestration path before verify and publish, not as a post-processing utility.
   - Rationale: verify must inspect the true candidate payload that publish will attempt.
   - Alternatives considered:
     - Verify-only synthetic patch simulation: rejected because publish could diverge from verified payload.
     - External patch preprocessor: rejected due to state envelope contract fragmentation.

3. Use explicit runtime backend configuration (`stub|live`) with a stable client trait boundary.
   - Rationale: allows live execution enablement without forking orchestration logic and keeps tests stable in stub mode.
   - Alternatives considered:
     - Compile-time feature flags only: rejected because operator runtime selection is needed.
     - Separate binaries for live and stub: rejected due to duplicated command surface and drift risk.

4. Preserve deterministic error taxonomy at backend boundary.
   - Rationale: readiness and runbook logic depends on stable, classifiable error codes.
   - Alternatives considered:
     - Pass through raw upstream errors: rejected because it weakens deterministic triage.

5. Treat telemetry quality as contract-validity requirement for KPI outputs.
   - Rationale: placeholder/static values produce false KPI conclusions and invalid decision outcomes.
   - Alternatives considered:
     - Best-effort telemetry with warnings only: rejected due to governance risk.

6. Keep gated rerun order fixed and stop on defined triage conditions.
   - Rationale: repeatable checkpoint ordering is required for comparing outcomes across iterations.
   - Alternatives considered:
     - Flexible rerun ordering: rejected because order variance can confound KPI and failure interpretation.

## Risks / Trade-offs

- [Live backend introduces nondeterministic service variance] -> Mitigation: keep stub as baseline mode and classify live incident markers separately.
- [Provenance collection can fail in shallow/detached git contexts] -> Mitigation: explicit fallback behavior that blocks decision-grade output when provenance is missing.
- [Patch-application fixes may reveal hidden assumptions in existing tests] -> Mitigation: add focused regression fixtures for prose and table-cell patch semantics.
- [Telemetry strictness may fail more runs initially] -> Mitigation: treat as intentional quality gate and rerun after instrumentation repair.
- [Runtime selector complexity increases operator error risk] -> Mitigation: clear CLI defaults, explicit mode echo in outputs, and environment validation checks.

## Migration Plan

1. Introduce provenance stamp model and wire into run/batch/readiness serializers.
2. Align patch state output to ensure candidate ADF reflects applied `patch_ops` before verify/publish.
3. Add backend selector and live Confluence client path while preserving existing stub integration tests.
4. Replace placeholder telemetry fields with measured values and contract checks.
5. Execute gated rerun sequence and verify replay determinism.

Rollback strategy:
- If live mode destabilizes reliability, default back to `stub` while preserving provenance and patch-correctness fixes.
- If telemetry strictness blocks all reports, retain artifacts and fix instrumentation before re-enabling decision output.

## Open Questions

- Should provenance include repository origin/branch in addition to commit SHA and dirty flag for release audit clarity?
- Should live mode require an explicit sandbox identifier in output artifacts to prevent environment ambiguity?
- Do we require separate incident-filtered KPI view in this change, or defer to a follow-up if external variance dominates live runs?
