# Empty-Page Bootstrap Editing

## Status

Promoted to roadmap (v1 release enablement)

## Roadmap Linkage (current source of truth)

- `roadmap/03-phased-roadmap.md`
- `roadmap/06-decisions-and-defaults.md`
- `roadmap/07-execution-readiness.md`
- `roadmap/08-poc-scope.md`
- `roadmap/11-live-runtime-execution-plan.md`
- `roadmap/12-page-lifecycle-expansion-plan.md`

## Plain Problem Points

- Confluence supports blank pages, but current Atlassy prose editing expects mapped editable prose paths.
- Empty or near-empty pages can fail route/mapping preconditions for targeted first edits.
- Operators currently need manual seed content before AI-driven edits can proceed reliably.

## Proposed Direction

Add explicit first-edit bootstrap support for empty pages:

- Introduce `--bootstrap-empty-page` as an explicit opt-in for first prose edit operations.
- Enforce deterministic behavior matrix:
  - empty page + no bootstrap flag -> hard fail (`ERR_BOOTSTRAP_REQUIRED`)
  - empty page + bootstrap flag -> bootstrap minimal prose scaffold, then apply requested edit
  - non-empty page + bootstrap flag -> hard fail (`ERR_BOOTSTRAP_INVALID_STATE`)
  - non-empty page + no bootstrap flag -> existing behavior unchanged
- Keep bootstrap minimal and constrained to safe prose initialization only.
- Emit explicit telemetry/provenance markers when bootstrap is evaluated or applied.

## Historical Why Not Now (pre-promotion)

- Existing v1 flow prioritizes strict route safety and deterministic taxonomy before adding lifecycle exceptions.
- Bootstrap logic needs careful boundaries to avoid structural drift.
- Requires new tests to ensure bootstrap behavior does not weaken existing safety checks.

## Risks

- Incorrect empty/non-empty detection could trigger wrong behavior.
- Overly broad bootstrap could mutate structure outside intended prose scope.
- Imprecise telemetry could make operator triage harder.
- Inconsistent bootstrap behavior across page shapes could reduce trust.

## Signals To Revisit

- Recurring failures on first-edit workflows for blank pages.
- Repeated operator requests for zero-touch first-write experiences.
- Increased usage of automated sub-page creation with blank defaults.

## Promotion Outcome

This idea has been promoted into v1 release-enablement scope.

- First edit on empty pages requires explicit `--bootstrap-empty-page`.
- Bootstrap on non-empty pages must hard-fail deterministically.
- Non-empty pages without bootstrap flag retain unchanged behavior.
- Release readiness requires full lifecycle matrix evidence before `go` sign-off.
