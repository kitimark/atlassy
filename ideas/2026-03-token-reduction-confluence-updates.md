# Reduce Token Usage for Confluence Wiki Updates

## Status

Incubating (not scheduled)

## Plain Problem Points

- Updating Confluence pages often sends too much content to the AI, especially full-page bodies.
- Repeating policy text, formatting rules, and prior context across runs wastes tokens.
- Full-body rewrite flows increase retries from version drift and conflict replays.
- Large ADF payloads are verbose, so prompts become expensive and harder to reason about.

## Proposed Direction

Use a token-budgeted, section-first update workflow:

- Retrieve only relevant sections by heading or block ID, not entire page bodies.
- Build context with a hard budget and deterministic truncation order.
- Generate minimal patch operations for changed sections only.
- Keep a rolling memory summary for decisions/open questions instead of replaying full history.
- Reuse canonical references for repeated policy blocks instead of reinjecting raw text.

## Evidence Snapshot

- Case evidence is documented in `2026-03-confluence-adf-markdown-size-evidence.md`.
- In the public sample, aggregate uncompressed ADF payload is about 1.50x storage and 2.85x derived markdown.
- The sample includes many table/media/extension nodes, which are major verbosity and fidelity-risk drivers.

## V1 Defaults (Planning)

- Start with read-only simulation (`read -> budget -> patch-plan -> verify`), no write publish in the first PoC step.
- Use conservative lock policy for fidelity-critical blocks: tables, media, macros/extensions, layouts, mentions, panels, and status chips.
- Use a public benchmark seed dataset first (for example, a 5-page sample from `xilinx-wiki.atlassian.net`), then add private datasets later if needed.

## Why Not Now

- Project focus is still on defining the baseline roadmap and architecture.
- This idea needs instrumentation design first to prove token reduction objectively.
- Safe patch semantics and conflict policy must be agreed before implementation.

## Risks

- Section retrieval may miss dependencies and cause incomplete edits.
- Aggressive truncation can drop critical requirements.
- Token optimization can conflict with fidelity if patch scope is too narrow.
- Extra caching/memory layers can become stale without version-aware invalidation.

## Signals To Revisit

- Frequent context-window overflow during Confluence update tasks.
- High `tokens_per_successful_update` compared to target.
- Repeated full-page fetches for single-section edits.
- Retry loops caused by version conflicts and repeated prompt regeneration.

## Promotion Path

Move this idea to `roadmap/` when all conditions are true:

- Token baseline is captured for current Confluence update flow.
- KPI targets are approved (token reduction, retrieval reduction, retry reduction).
- Section-level patch strategy and conflict policy are defined.
- A small PoC scope is selected (one large page type + 2-3 edit patterns).
