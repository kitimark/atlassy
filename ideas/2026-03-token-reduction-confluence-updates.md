# Reduce Token Usage for Confluence Wiki Updates

## Status

Promoted to roadmap (v1 baseline)

## Roadmap Linkage (current source of truth)

- `roadmap/02-solution-architecture.md`
- `roadmap/03-phased-roadmap.md`
- `roadmap/04-kpi-and-experiments.md`
- `roadmap/06-decisions-and-defaults.md`
- `roadmap/08-poc-scope.md`
- `roadmap/10-testing-strategy-and-simulation.md`

## Plain Problem Points

- Updating Confluence pages often sends too much content to the AI, especially full-page bodies.
- Repeating policy text, formatting rules, and prior context across runs wastes tokens.
- Full-body rewrite flows increase retries from version drift and conflict replays.
- ADF is required for feature coverage, but large unscoped ADF payloads are expensive.

## Proposed Direction

Use an ADF-first, token-budgeted, section-first update workflow:

- Retrieve only relevant sections by heading or block ID, not entire page bodies.
- Build context with a hard budget and deterministic truncation order.
- Generate path-targeted ADF patch operations for changed sections only.
- Keep a rolling memory summary for decisions/open questions instead of replaying full history.
- Reuse canonical references for repeated policy blocks instead of reinjecting raw text.
- Route prose edits through Markdown assist, but keep tables and structural features in ADF paths.

## Evidence Snapshot

- Case evidence is documented in `2026-03-confluence-adf-markdown-size-evidence.md`.
- In the public sample, aggregate uncompressed ADF payload is about 1.50x storage and 2.85x derived markdown.
- The sample includes many table/media/extension nodes, which are major verbosity and fidelity-risk drivers.

## V1 Defaults (Planning)

- Use controlled live Confluence sandbox probes for behavior research (including write paths), and use deterministic stub simulation as the default mode for CI/regression.
- Keep ADF as canonical source for fetch, patch, verify, and publish.
- Allow table edits via ADF-native path, but v1 scope is cell text only.
- Use conservative lock policy for unsupported fidelity-critical blocks: media, macros/extensions, layouts, mentions, panels, and status chips.
- Use a public benchmark seed dataset first (for example, a 5-page sample from `xilinx-wiki.atlassian.net`), then add private datasets later if needed.
- Track advanced table operations as deferred ideas (`2026-03-advanced-table-editing-modes.md`).
- Historical note: an earlier draft used read-only simulation in the first PoC step; this was superseded by the live-sandbox-plus-stub policy.

## Historical Why Not Now (pre-promotion)

- Project focus is still on defining the baseline roadmap and architecture.
- This idea needs instrumentation design first to prove token reduction objectively.
- Safe patch semantics and conflict policy must be agreed before implementation.

## Risks

- Section retrieval may miss dependencies and cause incomplete edits.
- Aggressive truncation can drop critical requirements.
- Token optimization can conflict with fidelity if patch scope is too narrow.
- Table-cell targeting bugs can mutate unintended cells.
- Extra caching/memory layers can become stale without version-aware invalidation.

## Signals To Revisit

- Frequent context-window overflow during Confluence update tasks.
- High `tokens_per_successful_update` compared to target.
- Repeated full-page fetches for single-section edits.
- Retry loops caused by version conflicts and repeated prompt regeneration.
- Demand for row/column operations beyond cell text updates.

## Promotion Outcome

This idea has been promoted into the v1 roadmap baseline.

- KPI targets are now defined in `roadmap/04-kpi-and-experiments.md`.
- Patch strategy and conflict policy are now defined in `roadmap/06-decisions-and-defaults.md` and `roadmap/09-ai-contract-spec.md`.
- Route policy and PoC scope are now defined in `roadmap/06-decisions-and-defaults.md` and `roadmap/08-poc-scope.md`.
