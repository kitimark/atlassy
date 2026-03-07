# PoC Test Dataset Design for Scoped KPI Validation

## Status

Incubating (signals met, execution in progress)

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

- ~~KPI and telemetry doc changes are being finalized before implementation updates.~~ Resolved: KPI telemetry aligned at commit `e29065f`.
- ~~Dataset construction should align with the exact run-summary fields emitted by the pipeline (`full_page_adf_bytes`, `scoped_adf_bytes`, `context_reduction_ratio`).~~ Resolved: all six KPI fields are wired up end-to-end.
- ~~Immediate priority is documentation and experiment protocol alignment.~~ Resolved: QA test plan updated with full KPI batch protocol.

All "Why Not Now" blockers are resolved. Execution is in progress.

## Risks

- Synthetic sandbox pages may not represent all enterprise authoring patterns.
- Overfitting selectors to curated pages can inflate observed context-reduction outcomes.
- Dataset maintenance overhead grows as route policy and lifecycle behavior evolve.

## Signals To Revisit

- ~~Pipeline emits revised KPI telemetry fields consistently.~~ Met: `align-kpi-implementation` change archived at `e29065f`.
- ~~QA manifests are ready for paired scoped reruns.~~ Met: `qa/confluence-sandbox-test-plan.md` updated with Step 8 KPI batch protocol.
- Decision review requires stronger per-page variance explanation.

All signals met. Dataset construction is underway.

## Promotion Path

- Promote into `roadmap/08-poc-scope.md` and `qa/manifests/` when telemetry implementation is ready.
- Capture final dataset profile in a committed QA investigation with provenance.

## Execution Progress

- Page inventory: `qa/manifests/sandbox-page-inventory.md`.
- KPI batch protocol: `qa/confluence-sandbox-test-plan.md`, Step 8.
- Experiment pages created and bootstrapped (commit `dce393b`):
  - P1 (prose-rich): page `65934`.
  - P2 (mixed prose+table): page `98323`.
  - P3 (locked-adjacent): page `131227`.
- Next: seed pages with structural content, run scoped fetch spike, execute KPI batch.
