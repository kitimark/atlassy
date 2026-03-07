## Context

The live sandbox investigation (`qa/investigations/2026-03-07-live-confluence-failure.md`) identified two independent failure modes: (1) process panic during live client startup due runtime model mismatch, and (2) Confluence publish contract rejection (`400` with missing/incremented version message) after upstream states succeeded. The goal is to restore deterministic live execution semantics already expected by existing runtime and readiness specs.

Constraints:

- Keep scope minimal and focused on live runtime reliability.
- Preserve deterministic error taxonomy and existing summary/report formats.
- Avoid introducing broad architectural churn unrelated to the two validated failures.

## Goals / Non-Goals

**Goals:**

- Eliminate runtime panic path in live mode startup and ensure failures become deterministic run errors.
- Align live publish payload with Confluence contract so scoped prose publish can succeed when verify passes.
- Add regression coverage that catches both failure classes before future live QA runs.

**Non-Goals:**

- Reworking full pipeline state design or adding new pipeline states.
- Expanding scope to table-route behavior beyond existing negative safety checks.
- Achieving production sign-off in this change; this change only restores live QA viability.

## Decisions

### 1) Use a runtime model that avoids async/blocking lifecycle conflicts

Decision: keep the live Confluence client path compatible with a non-async orchestration entrypoint for `run`/`run-batch`/`run-readiness` execution flow.

Rationale:

- Investigation backtrace tied panic to `reqwest::blocking` runtime drop inside async context.
- This is the smallest-scope fix that restores deterministic behavior without refactoring all runtime-facing interfaces to async.

Alternatives considered:

- Migrate live client and orchestration stack to async reqwest end-to-end: cleaner long-term, but materially larger change surface and higher regression risk for this bug-fix scope.
- Wrap blocking init in ad hoc async boundary (`spawn_blocking` only around construction): partial mitigation but fragile and easy to regress.

### 2) Make publish payload contract explicit and testable

Decision: codify an explicit publish payload builder contract for live Confluence update calls, including required top-level fields, version increment handling, and `atlas_doc_format` representation/value shape expected by Confluence.

Rationale:

- Upstream states produced valid candidate data while publish failed, isolating defect to request contract assembly.
- Explicit payload contract tests provide deterministic protection against silent request-shape drift.

Alternatives considered:

- Keep inline payload assembly with no contract tests: fastest short-term, but likely to regress.
- Add post-hoc retries for HTTP 400 payload errors: masks root cause and weakens diagnostics.

### 3) Preserve deterministic hard-error mapping in live failures

Decision: treat live startup and publish contract failures as mapped hard errors (no process panic), preserving existing readiness/report expectations.

Rationale:

- Existing specs already require deterministic taxonomy for live failures.
- Investigation evidence and operator workflows depend on structured failure state and error code outputs.

Alternatives considered:

- Catch panics and continue with opaque fallback errors: reduces crash frequency but loses precise causality and may hide defects.

### 4) Re-validate through QA smoke evidence workflow

Decision: execute the documented QA flow after implementation and capture a follow-up evidence bundle under `qa/evidence/` to confirm remediation.

Rationale:

- This repo now treats investigation evidence as committed collaboration artifacts.
- Fix completion should be proven by reproducible run summaries, not ad hoc command output.

## Risks / Trade-offs

- **[Risk]** Runtime entrypoint adjustment could affect CLI behavior assumptions in existing tests. → **Mitigation:** keep interface/flags unchanged and run full crate test suite plus targeted live smoke sequence.
- **[Risk]** Confluence payload behavior may differ by tenant/editor edge cases. → **Mitigation:** validate against sandbox page types used in investigation and keep request-shape tests as regression guard.
- **[Risk]** Fixing publish contract may expose previously hidden live mapping gaps. → **Mitigation:** assert deterministic error mapping in failing-path tests and keep failure-state assertions in QA checks.

## Migration Plan

1. Implement runtime model compatibility and publish contract fixes behind existing CLI/runtime flags.
2. Run local tests for affected crates and deterministic error mapping checks.
3. Execute sandbox smoke flow (`preflight`, `prose publish`, `negative safety`) using `qa/confluence-sandbox-test-plan.md`.
4. Record post-fix evidence bundle and investigation follow-up entry.
5. Rollback strategy: revert change commit(s), restoring previous behavior and investigation baseline.

## Open Questions

- Do we lock payload contract assertions to Confluence v1 endpoint semantics only, or add compatibility notes/tests for future endpoint migration?
- Should we add a dedicated operator-facing diagnostic field for startup-failure classification distinct from publish/fetch failures?
