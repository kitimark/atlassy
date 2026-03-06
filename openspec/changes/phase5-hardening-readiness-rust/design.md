## Context

Phase 4 added PoC batch execution, KPI aggregation, and gating outputs, but release readiness is still spread across roadmap docs and operator tribal knowledge. The current state lacks a single deterministic workflow that turns batch artifacts into a signed readiness decision with explicit ownership, escalation steps, and reproducible evidence.

Constraints for Phase 5:
- Keep v1 editing scope unchanged (no new content-editing capabilities).
- Reuse existing artifact outputs from Phase 4 rather than introducing parallel telemetry channels.
- Preserve deterministic behavior and replayability expected by OpenSpec and roadmap quality gates.

Stakeholders include engineering, QA, data/metrics owner, and release reviewer roles defined in execution-readiness planning.

## Goals / Non-Goals

**Goals:**
- Convert readiness gates from roadmap prose into machine-readable pass/fail outputs.
- Generate deterministic operator runbooks for high-priority failure classes and escalation ownership.
- Produce a decision packet that ties KPI outcomes, safety/drift gates, and risk status into a traceable `go | iterate | stop` recommendation.
- Ensure all readiness outputs are reproducible from persisted artifacts.

**Non-Goals:**
- Expanding table, structural, or multi-page editing scope.
- Replacing Phase 4 KPI formulas or re-architecting core pipeline state flow.
- Building long-term analytics infrastructure beyond release-readiness needs.

## Decisions

1. Use a layered readiness pipeline in CLI outputs.
   - Decision: treat readiness generation as four ordered layers: evidence load -> gate evaluation -> runbook synthesis -> decision packet assembly.
   - Rationale: enforces deterministic ordering and keeps failure localization clear.
   - Alternatives considered:
     - Single monolithic report builder: rejected due to weak debuggability.
     - Ad hoc scripts per team role: rejected due to inconsistent decision inputs.

2. Represent gates as normalized records instead of free-form text.
   - Decision: readiness checks emit a normalized structure (`name`, `target`, `pass`, optional evidence references) aligned to Gate 1-6 in readiness docs.
   - Rationale: enables deterministic pass/fail rollups and auditable sign-off.
   - Alternatives considered:
     - Markdown-only checklist: rejected because it is hard to validate and replay.
     - Deriving status from logs at review time: rejected due to manual error risk.

3. Generate runbooks from error classes, not from individual runs.
   - Decision: map high-priority classes (`verify` hard fail, retry exhaustion, safety-gate violations, drift unresolved, telemetry gaps) to stable response playbooks and ownership fields.
   - Rationale: operators need predictable procedures that remain stable across batches.
   - Alternatives considered:
     - Per-run custom guidance: rejected due to noisy and inconsistent outputs.
     - No generated runbooks: rejected because readiness review requires explicit response actions.

4. Use a strict decision hierarchy for recommendation synthesis.
   - Decision: resolve recommendation using precedence: safety/drift blockers -> incomplete readiness gates -> KPI target misses -> pass.
   - Rationale: preserves safety-first policy and prevents KPI improvements from masking hard blockers.
   - Alternatives considered:
     - Weighted score across all signals: rejected because hard blockers must remain absolute.
     - Human-only recommendation text: rejected due to weak reproducibility.

5. Make reproducibility a first-class gate.
   - Decision: decision packet generation must support rebuild from stored normalized manifest, artifact index, summaries, and gate outputs; mismatches fail readiness.
   - Rationale: release sign-off must be independently verifiable.
   - Alternatives considered:
     - Best-effort replay checks: rejected due to governance ambiguity.

## Risks / Trade-offs

- [Readiness overfits current artifact schema] -> Mitigation: version readiness schema and include compatibility checks.
- [Runbook outputs become stale as error taxonomy evolves] -> Mitigation: centralize error-class mapping and require taxonomy coverage tests.
- [Decision hierarchy hides nuance for borderline KPI cases] -> Mitigation: include explicit rationale and failed-gate details in packet output.
- [Replay checks increase operational overhead] -> Mitigation: keep replay inputs minimal and reuse existing Phase 4 artifacts.
- [Cross-role ownership fields drift from org reality] -> Mitigation: keep role mappings configurable in batch metadata.

## Migration Plan

1. Add readiness gate models and deterministic evaluation pipeline.
2. Add runbook synthesis for high-priority failure classes and escalation owners.
3. Add decision packet builder that composes KPI, gate, and risk evidence.
4. Add reproducibility verification command/path and fixture-backed tests.
5. Validate with `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.

Rollback strategy:
- If readiness pipeline destabilizes current reporting commands, disable new readiness commands behind a feature flag while keeping Phase 4 report generation intact.

## Open Questions

- Should decision packet ownership fields be static role labels or externally supplied per run/batch?
- Do we require cryptographic hashes for readiness artifacts in v1, or is deterministic content replay sufficient?
- Should unresolved medium-priority risks force `iterate`, or only high-priority/open risks?
