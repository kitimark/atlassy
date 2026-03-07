# Page Lifecycle Expansion Plan (v1 Release Enablement)

## Objective

Include page lifecycle support in v1 release scope to enable end-to-end testing before final `go | iterate | stop` sign-off.

Required lifecycle capabilities:

1. explicit Confluence sub-page creation (`create-subpage`) that creates truly blank pages, and
2. explicit first-edit bootstrap for empty pages (`--bootstrap-empty-page`) with deterministic safety behavior.

This track keeps v1 route safety defaults intact and avoids implicit side effects.

## Release-Gating Intent

This plan is release-gating for v1. A v1 release recommendation must not be `go` until lifecycle work packages and evidence criteria in this file are complete.

## Current Baseline

- Existing live runtime edits operate on an already known `page_id`.
- Existing v1 prose edits expect mapped `editable_prose` targets.
- Empty page first-write handling is not a first-class capability.

## Scope

### In Scope

1. Command-first sub-page creation with explicit parent-page targeting.
2. Blank-page default creation behavior (no automatic seed content).
3. Explicit empty-page bootstrap flag for first prose edit.
4. Deterministic error taxonomy and telemetry/provenance for lifecycle operations.
5. QA runbook and evidence updates for blank-page lifecycle flows.

### Out of Scope

- Implicit page creation inside `run`.
- Implicit/automatic bootstrap when flag is not provided.
- Structural-node bootstrap (macros/media/layouts).
- Multi-page orchestration policies.

## Product Defaults (Locked)

- `create-subpage` creates a truly blank child page.
- `--bootstrap-empty-page` is required for first prose edit on an empty page.
- If bootstrap is requested on a non-empty page, execution hard-fails.

## Work Packages (Strict Order)

### WP1: Sub-Page Creation Contract and Command Surface

**Goal**

- Add explicit page-creation capability without changing existing `run` semantics.

**Required Outcomes**

- CLI command: `create-subpage`.
- Contract extension for page creation in both stub and live runtime clients.
- Response payload includes created page ID and basic metadata.

**Done Criteria**

- Parent-page targeting is required and validated.
- Duplicate-title and permission failure paths are deterministic.
- Existing `run` behavior remains unchanged when command is not used.

### WP2: Blank-Page Creation Behavior

**Goal**

- Ensure newly created pages are blank by default and auditable.

**Required Outcomes**

- No automatic seed content at creation time.
- Provenance captures parent page ID, created page ID, and command context.

**Done Criteria**

- Created page body shape is consistently blank.
- QA evidence demonstrates reproducible create behavior in stub and live sandbox.

### WP3: Explicit Empty-Page Bootstrap Editing

**Goal**

- Enable first prose edit on empty pages through explicit operator intent.

**Required Outcomes**

- Add `--bootstrap-empty-page` support to edit execution.
- Enforce deterministic behavior matrix:
  - empty + no bootstrap -> hard fail (`ERR_BOOTSTRAP_REQUIRED`)
  - empty + bootstrap -> bootstrap minimal prose scaffold, then apply edit
  - non-empty + bootstrap -> hard fail (`ERR_BOOTSTRAP_INVALID_STATE`)
  - non-empty + no bootstrap -> unchanged v1 behavior

**Done Criteria**

- All matrix paths are covered by tests.
- Existing route safety guards remain non-regressive.

### WP4: Safety, Error Taxonomy, and Telemetry Hardening

**Goal**

- Keep lifecycle additions decision-grade and operator-traceable.

**Required Outcomes**

- Deterministic error mapping for create/bootstrap failures.
- Summary/report telemetry includes bootstrap evaluation/application markers.
- No degradation in existing safety error semantics.

**Done Criteria**

- New lifecycle errors are included in triage guidance.
- Decision artifacts remain provenance-complete and replay-stable.

### WP5: QA Validation and Readiness Evidence Update

**Goal**

- Validate lifecycle flows end-to-end in sandbox as part of v1 release readiness.

**Execution Sequence**

1. Create blank sub-page under sandbox parent.
2. Attempt first edit without bootstrap (expected deterministic fail).
3. Re-run first edit with bootstrap (expected success).
4. Re-run edit with bootstrap on now non-empty page (expected deterministic fail).
5. Run baseline/optimized paired validation including lifecycle-aware cases.

**Done Criteria**

- QA evidence bundle records all expected outcomes.
- Investigation summary documents behavior matrix with clean provenance.

## Stop Conditions (Immediate Triage)

- Any implicit page creation side effect during standard `run`.
- Any bootstrap path that mutates structural nodes outside scoped prose intent.
- Any non-deterministic failure mapping for create/bootstrap states.
- Any regression in existing v1 safety gate behavior.

## Exit Criteria (v1 Lifecycle Track Ready)

This plan is complete when all conditions hold:

1. `create-subpage` is stable in stub and live runtime modes.
2. Empty-page bootstrap matrix is fully deterministic and tested.
3. Existing v1 prose/table safety behavior remains non-regressive.
4. QA evidence is committed with provenance and replayable outcomes.
5. Readiness guidance includes lifecycle runbook updates.

## Source Idea References

- `ideas/2026-03-confluence-subpage-creation-support.md`
- `ideas/2026-03-empty-page-bootstrap-editing.md`
