# Atlassy

Atlassy is a token-efficient Confluence workflow toolkit for AI and MCP-driven editing.

## Problem

Large Confluence pages increase token usage and make updates fragile.

- Context windows overflow when full pages are injected into prompts.
- Large payload updates can fail or time out.
- Full-body updates increase version drift and conflict retries.
- Formatting fidelity can regress (tables, macros, media) after repeated edits.

## Goals

- Reduce `tokens_per_successful_update` by 40-60%.
- Reduce full-page retrievals by 60-80%.
- Maintain or improve publish success and formatting fidelity.

## Non-goals (v1)

- Full multi-space orchestration on day one.
- Fully autonomous conflict resolution without human review.

## Approach

Use a minimal-change pipeline:

`read -> plan -> patch -> verify -> publish`

## Planned Capabilities

- `fetch`
- `index`
- `plan`
- `patch`
- `verify`
- `publish`
- `cache`

## Roadmap

Detailed planning docs are under `roadmap/`.

Start here: `roadmap/README.md`.

## Ideas

Incubating concepts that are not scheduled yet are stored under `ideas/`.

Start here: `ideas/README.md`.

## Current Status

Phase 0 (design baseline): in planning.

## Success Metrics

- `tokens_per_successful_update`
- `full_page_retrieval_rate`
- `retry_conflict_token_waste`
- `formatting_fidelity_pass_rate`
- `publish_latency`

## Contributing

Use a design-first flow.

- Add or update roadmap docs before implementation changes.
- Record major tradeoffs and defaults in decision notes.
- Keep proposals testable with measurable success criteria.
