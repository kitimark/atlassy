# PoC Scope

## Objective

Define validation scope for Atlassy capabilities across Foundation (text replacement) and Structural (insert/edit/delete operations) phases.

## Foundation Objective (Phases 0-5)

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
- Scope resolution section extraction: heading selectors return the heading plus subsequent sibling content until the next heading or end of parent array (blocking prerequisite for KPI experiments).
- Route classification into `editable_prose`, `table_adf`, `locked_structural`.
- Markdown assist round-trip for prose nodes only.
- ADF-native table cell text patching.
- Path-targeted patch generation and strict verifier gates.
- Command-first sub-page creation for lifecycle E2E testing (`create-subpage`) with blank-page default.
- Explicit empty-page first-edit bootstrap via `--bootstrap-empty-page` with deterministic hard-fail preconditions.
- Programmatic page content seeding via `seed-page` command for reproducible experiment page setup.

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

## Structural Validation Scope (Phases 6-9)

### Structural Objective

Validate that insert, edit, and delete operations on ADF blocks produce correct, schema-valid results across single pages and page hierarchies.

### Phase 6 Validation: Block Operation Foundation

- Insert prose blocks (paragraph, heading) at valid positions within scope.
- Delete prose blocks within scope.
- Mixed operations: insert + delete + replace in same run.
- Backward compatibility: Foundation replace-only operations produce identical results.
- Post-mutation schema validation catches invalid ADF before publish.
- Error handling: invalid insert positions, missing removal targets, schema violations.

### Phase 7 Validation: Structural Composition

- Section operations: insert and delete full sections (heading + body).
- Table creation: insert new tables with specified dimensions.
- List creation: insert new lists with specified items.
- Container insert/delete: insert child blocks into panels, expands, layouts.
- Template blocks: predefined structural patterns produce consistent output.

### Phase 8 Validation: Multi-Page Content Control

- Content-bearing sub-page creation with specified ADF structure.
- Coordinated multi-page edits with dependency ordering.
- Rollback on partial failure: completed pages revert cleanly.
- Provenance tracking across multi-page operations.

### Phase 9 Validation: Advanced Operations

- Table topology changes: row/column add/remove.
- Structural block attribute editing: media metadata, macro parameters.
- MCP server integration: all operations available via MCP with same safety guarantees.

### Structural Edit Patterns

- Pattern D: insert-only (add paragraphs, headings within scope).
- Pattern E: delete-only (remove paragraphs, headings within scope).
- Pattern F: mixed insert/edit/delete (structural modification + text replacement in same run).
- Pattern G: multi-page (coordinated operations across parent + child pages, Phase 8+).
- Patterns A/B/C from Foundation retained for backward-compatibility regression testing.

### Structural Instrumentation

- `operation_success_rate`
- `schema_validity_rate`
- `operation_precision`
- `structural_integrity`
- `conflict_rate`
- `publish_latency`
- `context_reduction_ratio` (diagnostic only)
- `scoped_section_tokens` (diagnostic only)

### Structural Exit Gates

- All Structural edit patterns pass verifier checks including post-mutation schema validation.
- No unintended side effects on non-target blocks (`operation_precision` = 100%).
- Backward compatibility: Foundation test suite passes unchanged.
- Phase-specific readiness gate (8/9/10) passes.
- Decision log is updated with measured outcomes.
