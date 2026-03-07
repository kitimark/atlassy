# Raw ADF Page Seeding

## Status

Promoted to roadmap (Phase 4 experiment tooling)

## Roadmap Linkage (current source of truth)

- `roadmap/03-phased-roadmap.md` (Phase 4 Blocking Prerequisites: `seed-page` CLI command)
- `roadmap/04-kpi-and-experiments.md` (Blocking Prerequisites for Re-Run)
- `roadmap/06-decisions-and-defaults.md` (D-016: page content seeding policy)
- `roadmap/08-poc-scope.md` (In Scope: programmatic page content seeding)

## Plain Problem Points

- Atlassy's CLI pipeline can only perform scoped text-level mutations on existing ADF structure (`replace` ops at leaf paths). It cannot add new headings, tables, macros, or other structural blocks.
- The bootstrap scaffold is hardcoded to one empty heading + one empty paragraph (`bootstrap_scaffold()` in `crates/atlassy-adf/src/lib.rs`, lines 294-310). It accepts no parameters and cannot inject rich content.
- There is no CLI command to publish arbitrary ADF JSON to a page.
- Setting up sandbox pages with structural variety (headings, tables, expand macros, media) for KPI experiments requires manual editing in the Confluence UI or direct REST API calls.
- The underlying `LiveConfluenceClient.publish_page()` (`crates/atlassy-confluence/src/lib.rs`, lines 367-442) does a full-body PUT of any `serde_json::Value` and has no structural restrictions. The capability exists but is unexposed.

## Proposed Direction

Add a new CLI command (e.g., `seed-page`) that accepts an ADF JSON file and publishes it to a specified page, bypassing the pipeline's safety envelope.

```
cargo run -p atlassy-cli -- seed-page \
  --page-id "$PAGE_ID" \
  --adf-file "path/to/content.json" \
  --runtime-backend live
```

Design constraints:
- Explicit opt-in command (not part of the `run` pipeline).
- Requires the page to already exist (use `create-subpage` first).
- Full-body replacement of page ADF content (same as `LiveConfluenceClient.publish_page()`).
- No verify gates, no route classification, no scope enforcement — this is a setup tool, not an editing tool.
- Validate that the input is syntactically valid ADF JSON before publishing.
- Require `--runtime-backend live` explicitly (no stub mode needed for setup).

Alternative: parameterize `bootstrap_scaffold()` to accept a file path or richer content specification. This is more constrained but keeps the pipeline safety model intact.

## Historical Why Not Now (pre-promotion)

- v1 focus is on validating the scoped editing pipeline, not on page authoring.
- Manual Confluence UI editing or curl workaround is sufficient for small-scale QA setup (3-5 pages).
- Adding a bypass command requires careful safety messaging to prevent misuse as a general editing tool.

## Risks

- A raw publish command bypasses all v1 safety guarantees (verify gates, locked-node protection, scope enforcement). Users could inadvertently use it for editing instead of the safe pipeline.
- ADF schema validation is complex; minimal validation may allow publishing invalid content.
- Command could be confused with the safe `run` pipeline, leading to incorrect mental models.

## Signals To Revisit

- KPI experiment page setup becomes a recurring bottleneck (more than 5 pages needed).
- CI/CD automation needs programmatic page setup for test fixtures.
- MCP server integration (`ideas/2026-03-mcp-server-integration.md`) needs a page seeding capability for agent workflows.

## Promotion Outcome

This idea has been promoted into v1 Phase 4 experiment tooling scope.

- `seed-page` is a command-first setup capability, not an editing tool.
- Bypasses pipeline safety envelope by design (no verify, no route classification, no scope enforcement).
- Policy recorded as D-016 in `roadmap/06-decisions-and-defaults.md`.
- Promotion triggered by: pages reaching version 6 after baseline publishes, making manual re-seeding a recurring bottleneck that blocks KPI batch re-runs.
