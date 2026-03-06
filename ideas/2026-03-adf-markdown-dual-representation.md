# ADF + Markdown Dual Representation

## Status

Promoted to roadmap (v1 baseline)

## Roadmap Linkage (current source of truth)

- `roadmap/02-solution-architecture.md`
- `roadmap/03-phased-roadmap.md`
- `roadmap/06-decisions-and-defaults.md`
- `roadmap/08-poc-scope.md`
- `roadmap/10-testing-strategy-and-simulation.md`

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

## Historical Why Not Now (pre-promotion)

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

## Promotion Outcome

This idea has been promoted into the v1 roadmap baseline.

- Route matrix is now defined in `roadmap/06-decisions-and-defaults.md`.
- KPI targets are now defined in `roadmap/04-kpi-and-experiments.md`.
- PoC and testing scope are now defined in `roadmap/08-poc-scope.md` and `roadmap/10-testing-strategy-and-simulation.md`.
