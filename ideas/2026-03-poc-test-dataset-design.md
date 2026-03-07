# PoC Test Dataset Design for Scoped KPI Validation

## Status

Incubating

## Plain Problem Points

- Historical PoC runs relied on limited public pages and often omitted scope selectors.
- KPI goals now depend on measuring scoped-vs-full context reduction, which needs controlled page structures.
- Without known structural variety, fallback analysis and outlier diagnosis are noisy.

## Proposed Direction

Create an author-controlled sandbox dataset specifically for revised KPI reruns.

- Build a page matrix with intentional variety:
  - small prose-only pages,
  - medium mixed prose/table pages,
  - large pages with repeated sections,
  - pages with locked structural adjacency (macros/media/extensions).
- Define known selector targets for each page (`heading:*` and optional `block:*`).
- Pair each edit intent as baseline (empty selectors) vs optimized (explicit selectors).
- Record page profile metadata alongside manifests so result interpretation stays deterministic.

## Why Not Now

- KPI and telemetry doc changes are being finalized before implementation updates.
- Dataset construction should align with the exact run-summary fields emitted by the pipeline (`full_page_adf_bytes`, `scoped_adf_bytes`, `context_reduction_ratio`).
- Immediate priority is documentation and experiment protocol alignment.

## Risks

- Synthetic sandbox pages may not represent all enterprise authoring patterns.
- Overfitting selectors to curated pages can inflate observed context-reduction outcomes.
- Dataset maintenance overhead grows as route policy and lifecycle behavior evolve.

## Signals To Revisit

- Pipeline emits revised KPI telemetry fields consistently.
- QA manifests are ready for paired scoped reruns.
- Decision review requires stronger per-page variance explanation.

## Promotion Path

- Promote into `roadmap/08-poc-scope.md` and `qa/manifests/` when telemetry implementation is ready.
- Capture final dataset profile in a committed QA investigation with provenance.
