# Atlassy Roadmap

This folder contains design and delivery planning for Atlassy.

## Purpose

- Keep project decisions explicit and easy to review.
- Break large planning content into small, reusable documents.
- Reduce context bloat in AI conversations by referencing focused files.
- Use `roadmap/` for active committed planning; use `ideas/` for incubating concepts and historical source notes.

## Scope

This roadmap spans two major versions:

- **Foundation (Phases 0-5)**: token-efficient text-replacement pipeline. Complete. All 7 readiness gates pass, 159 tests pass.
- **Structural (Phases 6-9)**: structural operations (insert/edit/delete ADF blocks), structural composition, multi-page content control, and advanced operations. Planning active; Phase 6 is next.

Foundation sections within documents are preserved as historical record. Structural additions are clearly marked.

## Roadmap Document Set (current)

- `01-problem-points.md`
- `02-solution-architecture.md`
- `03-phased-roadmap.md`
- `04-kpi-and-experiments.md`
- `05-risks-and-mitigations.md`
- `06-decisions-and-defaults.md`
- `07-execution-readiness.md`
- `08-poc-scope.md`
- `09-ai-contract-spec.md`
- `10-testing-strategy-and-simulation.md`
- `11-live-runtime-execution-plan.md`
- `12-page-lifecycle-expansion-plan.md`
- `13-ci-and-automation.md`
- `14-target-path-auto-discovery.md`
- `15-code-quality-and-readability.md`

## Working Rules

- Keep each file scoped to one topic.
- Prefer short sections with concrete acceptance criteria.
- Add measurable targets where possible.
- Update decision docs when defaults change.
- Treat `artifacts/` outputs as temporary execution data; keep durable policy in docs, not generated JSON.
- Include commit provenance (`git_commit_sha`, `git_dirty`, `pipeline_version`) when recording KPI or readiness outcomes.
- Preserve Foundation sections as historical record; add Structural content under clearly labeled headings.
