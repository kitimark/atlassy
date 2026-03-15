## Context

The pipeline is entirely single-page. `Orchestrator.run()` takes one `RunRequest` with one `page_id` and runs the 10-state pipeline (fetch → ... → publish). `RunSummary` is flat with a single `page_id`. `RunBatch` runs multiple independent pages from a manifest without coordination, rollback, or dependency ordering.

Phases 6-7 added block insert/remove/section/table operations within a single page. Phase 8 adds a layer above the per-page pipeline that coordinates operations across multiple pages.

The existing per-page pipeline, all 10 states, and the ADF crate are completely unchanged by Phase 8. The Confluence client trait is also unchanged — `create_page`, `fetch_page`, and `publish_page` already exist.

## Goals / Non-Goals

**Goals:**

- Coordinate N page pipelines with dependency ordering (topological sort).
- Save page state before modification and restore on failure (Memento-based rollback).
- Create new sub-pages with content in one coordinated step.
- Provide a CLI command for multi-page operations.
- Leave the per-page pipeline completely untouched (Open/Closed Principle).

**Non-Goals:**

- Parallel execution of independent pages — sequential only in Phase 8 (Speculative Generality prevention).
- `list_child_pages` or page tree auto-discovery — caller provides page IDs explicitly.
- Locked structural relaxation — deferred to Phase 9 (Rule of Three: first-time use was Phase 7, no third use yet).
- Cross-page scope resolution or content linking.
- Page deletion or moving pages between parents.

## Decisions

### D1: Facade + Mediator for MultiPageOrchestrator

`MultiPageOrchestrator` is a Facade (simple interface hiding complex coordination) and a Mediator (coordinates N per-page pipelines without them knowing about each other).

```
MultiPageOrchestrator.run(MultiPageRequest)
    → validates plan (dependency sort, cycle detection)
    → takes snapshots (Memento)
    → executes per-page pipelines in order
    → rollbacks on failure
    → returns MultiPageSummary
```

It wraps the existing `Orchestrator` — calls `Orchestrator::run()` for each page. The per-page Orchestrator is unaware it's being coordinated. This follows Open/Closed: the existing pipeline is unchanged.

Alternative considered: extending `Orchestrator` to handle multiple pages internally. Rejected because it would violate Single Responsibility (single-page pipeline vs multi-page coordination are different concerns) and would modify a well-tested, stable component.

### D2: Memento Pattern for PageSnapshot rollback

Before modifying a page, save its state as a `PageSnapshot { page_id, version_before, adf_before }`. The snapshot IS the memento.

Rollback flow on failure:
1. For each successfully published page (reverse order):
2. Fetch current version from Confluence.
3. If current_version == snapshot.version_after: safe to roll back → publish original ADF.
4. If current_version != snapshot.version_after: concurrent edit detected → report `RollbackConflict`, don't overwrite.

This reuses the existing `fetch_page` + `publish_page` Confluence API — no new methods needed.

For newly created pages: rollback does NOT delete the page (destructive). It leaves the created page as-is. The user can clean up manually. This is the safe default.

Alternative considered: optimistic rollback (overwrite regardless). Rejected because it could destroy concurrent edits by other users. Versioned rollback respects Confluence's OCC model.

### D3: Single Responsibility for page creation vs content operations

Page creation (`PageTarget.create: Option<CreatePageTarget>`) is separate from content operations (`block_ops: Vec<BlockOp>`). The `MultiPageOrchestrator` handles creation as a pre-condition step:

1. If `create` is present: `client.create_page(...)` → assigns `page_id`.
2. Build `RunRequest` with resolved `page_id` + `bootstrap_empty_page: true`.
3. Run per-page pipeline normally.

Alternative considered: `BlockOp::CreatePage` variant. Rejected because page lifecycle (creation) is a different concern from page content (block operations). Following Single Responsibility.

### D4: No list_child_pages (Speculative Generality prevention)

The caller provides page IDs explicitly in `MultiPageRequest.pages`. New pages get IDs from `create_page` return values. No page tree discovery needed.

If auto-discovery is needed later, `list_child_pages()` can be added to `ConfluenceClient` without affecting the orchestration architecture.

### D5: Sequential execution in dependency order

Pages execute one at a time in topological sort order based on `depends_on`. No parallel execution in Phase 8.

Topological sort algorithm:
1. Build adjacency graph from `depends_on` references.
2. Kahn's algorithm: process pages with no dependencies first.
3. Cycle detection: if all remaining pages have dependencies, reject with `ERR_DEPENDENCY_CYCLE`.

Alternative considered: parallel execution of independent pages. Deferred as Speculative Generality — sequential is simpler and sufficient for initial multi-page use cases.

### D6: No preparatory refactoring needed

Rule of Three check: `RunBatch` runs multiple pages independently (1st use), `MultiPageOrchestrator` coordinates multiple pages (2nd use). Not the 3rd — no refactoring mandate.

Phase 8 is almost entirely additive (new files, new types). No existing code needs restructuring.

## Risks / Trade-offs

[Rollback can fail due to concurrent edits] → If someone else edits a page between our publish and our rollback attempt, we can't safely roll back. Mitigation: report `RollbackConflict` with details. The user decides what to do with the conflicting page.

[Partial failure leaves mixed state] → If page B fails and page A rollback also fails (conflict), the system is in a partially modified state. Mitigation: `MultiPageSummary` reports per-page status so the user can see exactly which pages succeeded, failed, or have rollback conflicts.

[Created pages are not rolled back] → New pages created during the operation are left in place even on failure. Mitigation: explicitly documented behavior. Deletion requires a different API flow and is destructive.

[Sequential execution may be slow for large page sets] → N pages run one at a time. Mitigation: parallel execution can be added later without changing the architecture (each page pipeline is independent once dependencies are resolved).
