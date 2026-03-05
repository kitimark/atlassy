# ADF + Markdown Dual Representation

## Status

Incubating (not scheduled)

## Plain Problem Points

- Atlassy needs Confluence-native feature coverage that Markdown cannot represent fully.
- ADF must stay canonical to avoid fidelity drift for tables, macros, media, layouts, and extensions.
- Human editing quality is higher in Markdown for prose-only edits.
- Full-page ADF workflows are token-heavy without strict scope and patch controls.

## Proposed Direction

Use an ADF-canonical dual-representation model with block routing:

- `editable_prose`: use temporary Markdown assist for prose and simple structures.
- `table_adf`: edit tables in ADF-native mode (v1 allows cell text changes only).
- `locked_structural`: keep unsupported Confluence-native blocks locked in ADF.
- Use minimal path-targeted ADF patch updates and preserve all untouched locked blocks.

## Why Not Now

- Current project phase is focused on foundation and roadmap definition.
- This idea needs robust path mapping between Markdown assist blocks and ADF nodes.
- It introduces additional complexity in classifier, patch planner, and fidelity checks.

## Risks

- Incorrect block classification can route unsafe content to Markdown assist.
- Table path targeting bugs can mutate the wrong cells.
- Over-locking can reduce usability; under-locking can regress fidelity.
- Confluence API behavior changes can affect conversion and validation stability.

## Signals To Revisit

- Repeated fidelity regressions in mixed prose/table editing flows.
- High token costs from ADF-heavy prompts.
- Frequent manual rework for unsupported structural blocks.
- Clear owner and capacity available for a dedicated PoC.

## Promotion Path

Move this idea to `roadmap/` when all conditions are true:

- A v1 support matrix is approved (`editable_prose`, `table_adf`, `locked_structural`).
- KPI targets are defined and accepted.
- Test corpus for prose round-trip and table-cell patch fidelity is prepared.
- Phase 0 implementation capacity is committed.
