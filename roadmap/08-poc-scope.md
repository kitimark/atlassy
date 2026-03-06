# PoC Scope (v1)

## Objective

Validate that ADF-canonical routing with prose Markdown assist and ADF table-cell edits reduces token waste without fidelity regression.

## Dataset

- Baseline public seed: 5-page sample from `xilinx-wiki.atlassian.net` (space `A`).
- Expand later with private pages only after baseline instrumentation is stable.
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

## Out of Scope

- Table row/column add/remove.
- Table merge/split and table attribute changes.
- New support for macros/extensions/media/layout transformations.
- Multi-page orchestration and autonomous conflict resolution.

Deferred idea tracking:

- `ideas/2026-03-advanced-table-editing-modes.md`
- `ideas/2026-03-structural-block-editing-support.md`
- `ideas/2026-03-multi-page-orchestration-and-autonomous-conflict-resolution.md`

## Instrumentation

- `tokens_per_successful_update`
- `full_page_retrieval_rate`
- `retry_conflict_token_waste`
- `formatting_fidelity_pass_rate`
- `publish_latency`

## Success Targets

- Token reduction: 40-60% vs baseline flow.
- Full-page retrieval reduction: 60-80% vs baseline flow.
- Fidelity pass rate: no regression from baseline.
- Publish success and latency: non-regressive.

## Exit Gates

- All in-scope edit patterns pass verifier checks.
- No locked-node mutation is observed in PoC logs.
- Conflict handling shows bounded retry behavior (one scoped retry max).
- Decision log is updated with measured outcomes and recommended next scope.
