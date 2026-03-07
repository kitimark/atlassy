## Context

Atlassy's v1 pipeline assumes pages already exist. All current flows start with `fetch_page` on a known `page_id`. Two lifecycle gaps remain before v1 release readiness:

1. No way to create pages. QA and end-to-end testing require creating sandbox pages programmatically.
2. No handling for empty pages. A freshly created blank page has no `editable_prose` nodes, so the pipeline's classify/extract/edit stages have nothing to operate on.

The codebase currently has:
- A `ConfluenceClient` trait with `fetch_page`, `publish_page`, and `publish_attempts` (no create).
- A 9-state linear pipeline in `Orchestrator::run_internal` with no early-exit or pre-classify detection.
- No empty-page detection logic anywhere (confirmed by code search).
- Readiness evaluation with 6 gates; Gate 7 (lifecycle) exists only in roadmap docs.
- Error constants following the `ERR_SCREAMING_SNAKE_CASE` pattern in `atlassy-contracts`.

## Goals / Non-Goals

**Goals:**

- Enable programmatic blank sub-page creation under a parent page via CLI, with both stub and live backends.
- Enable first-edit bootstrap on empty pages with explicit operator intent, deterministic safety behavior, and minimal ADF scaffold injection.
- Add Gate 7 to readiness evaluation so lifecycle evidence is required for v1 sign-off.
- Preserve all existing v1 safety constraints, route policies, and pipeline behavior for non-empty pages.

**Non-Goals:**

- Implicit page creation during `run` — creation remains a separate, explicit command.
- Implicit bootstrap — the `--bootstrap-empty-page` flag is always required; no auto-detection.
- Structural-node bootstrap (macros, media, layouts) — scaffold is prose-only.
- Multi-page orchestration or batch-level page creation.
- Changes to the 9-state pipeline ordering or `PipelineState` enum.

## Decisions

### D1: `create-subpage` bypasses the pipeline entirely

**Decision:** `create-subpage` is a standalone CLI command that calls `ConfluenceClient::create_page` directly and prints the result. It does not instantiate an `Orchestrator` or run any pipeline states.

**Rationale:** Page creation is not an edit operation. It has no ADF to scope, classify, or patch. Routing it through the pipeline would require either no-op passthrough for all 9 states or a separate pipeline mode, both of which add complexity without value.

**Alternative considered:** Adding a `RunMode::CreatePage` variant. Rejected because it would require every pipeline state to handle a mode that produces no meaningful output, and the `StateTracker` enforces strict linear progression that doesn't fit a create-only operation.

### D2: `create_page` requires `space_key` as an explicit parameter

**Decision:** The `create_page` method takes `title`, `parent_page_id`, and `space_key` as required parameters.

**Rationale:** The Confluence v1 REST API (`POST /wiki/rest/api/content`) requires `space.key` in the request body even when `ancestors` is provided. The API does not infer the space from the parent page. Making it explicit avoids a hidden metadata-fetch round trip.

**Alternative considered:** Fetching the parent page to extract its space key automatically. Rejected because it adds latency, an extra API call, and a new failure mode — all to save one CLI argument.

### D3: Bootstrap detection happens after fetch, before classify, without a new pipeline state

**Decision:** Empty-page detection and scaffold injection are implemented as a block inside `run_internal`, between the `run_fetch_state` call (line 262) and the `run_classify_state` call (line 270). This block inspects `FetchOutput.scoped_adf`, evaluates the bootstrap matrix, and either hard-fails or mutates the `FetchOutput` in place before classify sees it. No new `PipelineState` variant is added.

**Rationale:** Adding a new state (e.g., `PipelineState::Bootstrap`) would break the strict 9-state ordering, require changes to `StateTracker::expected_next`, invalidate all existing artifact directory structures, and require every state-enumeration site to handle the new variant. The bootstrap check is a conditional guard, not a processing stage — it either rejects the run or adjusts the fetch output. Inserting it as inline logic between fetch and classify keeps the state machine unchanged.

**Alternative considered:** A `PipelineState::Bootstrap` variant. Rejected for the reasons above.

### D4: Empty-page detection uses content-level analysis, not byte-length

**Decision:** A new function `is_page_effectively_empty(adf: &Value) -> bool` in `atlassy-adf` checks whether the ADF has no meaningful content. "Effectively empty" means the top-level `content` array is either absent, empty, or contains only paragraphs whose own content is absent, empty, or contains only empty-string text nodes.

**Rationale:** A newly created page via the API may have `content: []`, a single empty paragraph, or a paragraph with an empty text node — all are functionally blank. Byte-length checks would be fragile against Confluence adding metadata attributes (like `localId`) to empty nodes.

### D5: Minimal prose scaffold is a single heading plus paragraph

**Decision:** When bootstrap is triggered, the scaffold injected into `FetchOutput.scoped_adf` is:

```json
{
  "type": "doc",
  "version": 1,
  "content": [
    {
      "type": "heading",
      "attrs": {"level": 2},
      "content": [{"type": "text", "text": ""}]
    },
    {
      "type": "paragraph",
      "content": [{"type": "text", "text": ""}]
    }
  ]
}
```

**Rationale:** A heading is needed because many real edit intents use scope selectors like `heading:X`. Without a heading, scope resolution falls back to full-page mode. The heading text is empty so the edit intent controls the final content. The paragraph provides an `editable_prose` node for the edit to target. Both node types are in the `editable_prose` route, so they pass classification without creating locked-structural or table nodes.

**Alternative considered:** Bare paragraph only. This works for basic edits but forces full-page scope fallback for any heading-based scope selector, which contradicts the project's scoped-fetch thesis.

### D6: `RunRequest` gains a `bootstrap_empty_page: bool` field

**Decision:** The bootstrap flag is carried as a boolean on `RunRequest`, defaulting to `false`. The CLI maps `--bootstrap-empty-page` to this field. Batch manifests gain a matching optional field.

**Rationale:** Follows the existing pattern of `force_verify_fail: bool` — a per-run behavioral flag that the pipeline checks at the appropriate point.

### D7: `StubConfluenceClient` implements `create_page` with in-memory page insertion

**Decision:** The stub implementation inserts a new entry into its `pages: HashMap<String, StubPage>` with `version: 1` and an empty ADF doc. It returns a synthetic `page_id` derived from the title. Duplicate title detection checks existing entries.

**Rationale:** Follows the existing stub pattern where `publish_page` mutates the in-memory map. Keeps stub tests deterministic without network access.

### D8: Gate 7 evaluates lifecycle evidence markers from batch summaries

**Decision:** Gate 7 in `evaluate_readiness_gates` checks that the batch contains at least one run matching each lifecycle matrix outcome: bootstrap-required failure, bootstrap success, and bootstrap-on-non-empty failure. The create-subpage evidence is validated by the presence of a lifecycle metadata marker in the batch manifest.

**Rationale:** Follows the existing gate evaluation pattern — each gate inspects batch evidence, not live state. The gate passes only when all four lifecycle matrix paths have committed evidence.

## Risks / Trade-offs

**[Risk] Space key requirement adds friction to `create-subpage` CLI usage.**
Mitigation: Document the requirement clearly. In the future, an optional `--infer-space` flag could fetch the parent page to derive the space key, but this is out of scope for v1.

**[Risk] Empty-page detection may not cover all Confluence page variants.**
Mitigation: Test against real Confluence pages created via the API, the UI, and templates. The `is_page_effectively_empty` function should be conservative — when in doubt, treat the page as non-empty (which falls through to the existing safe v1 flow).

**[Risk] Bootstrap scaffold shape may not satisfy all edit intents.**
Mitigation: The scaffold is intentionally minimal. If the edit intent requires a specific heading text for scope resolution, the caller is expected to provide it via `--new-value` or the edit intent itself. The scaffold provides structure; the edit provides content.

**[Risk] Mutating `FetchOutput` in place before classify may surprise future contributors.**
Mitigation: Add clear inline comments explaining the bootstrap injection point. The mutation is visible in the fetch state's persisted artifacts (the diagnostics will record whether bootstrap was applied).

**[Risk] Gate 7 adds a mandatory pass condition that blocks v1 sign-off.**
Mitigation: This is intentional — lifecycle support is release-gating per the roadmap. The gate can only block sign-off if lifecycle tests were not run, which is the correct behavior.
