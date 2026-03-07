# PoC Scope (v1)

## Objective

Validate that ADF-canonical routing with prose Markdown assist and ADF table-cell edits reduces token waste without fidelity regression.

## Dataset

- Primary benchmark set: author-created sandbox pages with planned structural variety (page size spread, prose-only sections, mixed prose/table sections, and locked-structural adjacency).
- Optional reference seed: 5-page sample from `xilinx-wiki.atlassian.net` (space `A`) for historical comparability.
- Evidence reference for baseline payload characteristics: `ideas/2026-03-confluence-adf-markdown-size-evidence.md`.

## Edit Patterns

- Pattern A: prose-only section rewrite (headings, paragraphs, list updates).
- Pattern B: mixed edit (prose rewrite + one table cell text update).
- Pattern C: constrained correction near locked structural blocks (no structural mutation allowed).

## In Scope

- Scoped ADF retrieval by heading or block ID.
- Route classification into `editable_prose`, `table_adf`, `locked_structural`.
- Markdown assist round-trip for prose nodes only.
- ADF-native table cell text patching.
- Path-targeted patch generation and strict verifier gates.
- Command-first sub-page creation for lifecycle E2E testing (`create-subpage`) with blank-page default.
- Explicit empty-page first-edit bootstrap via `--bootstrap-empty-page` with deterministic hard-fail preconditions.

## Out of Scope

- Table row/column add/remove.
- Table merge/split and table attribute changes.
- New support for macros/extensions/media/layout transformations.
- Multi-page orchestration and autonomous conflict resolution.

Deferred idea tracking:

- `ideas/2026-03-advanced-table-editing-modes.md`
- `ideas/2026-03-structural-block-editing-support.md`
- `ideas/2026-03-multi-page-orchestration-and-autonomous-conflict-resolution.md`

Lifecycle release-enablement reference:

- `roadmap/12-page-lifecycle-expansion-plan.md`

## Instrumentation

- `context_reduction_ratio`
- `scoped_section_tokens`
- `edit_success_rate`
- `structural_preservation`
- `conflict_rate`
- `publish_latency`

## Success Targets

- Context reduction: 70-90% on optimized in-scope runs.
- Scoped section size: report median and p90 by pattern (diagnostic target).
- Edit success rate: >95% on in-scope runs.
- Structural preservation: 100% non-target structure preserved.
- Conflict rate: <10% of runs with one scoped retry cap.
- Publish latency: median <3000 ms for scoped optimized runs; p90 non-regressive vs baseline.

## Exit Gates

- All in-scope edit patterns pass verifier checks.
- No locked-node mutation is observed in PoC logs.
- Conflict handling shows bounded retry behavior (one scoped retry max).
- Lifecycle matrix passes (blank subpage create success, empty-first-edit bootstrap required fail, bootstrap success, bootstrap-on-non-empty hard fail).
- Decision log is updated with measured outcomes and recommended next scope.
