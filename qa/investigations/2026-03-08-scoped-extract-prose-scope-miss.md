# 2026-03-08 Scoped Extract Prose ERR_SCOPE_MISS

## Summary

All scoped operations using heading selectors fail at the `extract_prose` pipeline state with `ERR_SCOPE_MISS`. The state iterates all `editable_prose` nodes from the full-page node manifest and fails when `canonicalize_mapped_path` encounters paths outside `allowed_scope_paths`. This blocks KPI batch execution for all optimized (scoped) runs.

Scope resolution itself works correctly. The failure is purely in how `extract_prose` handles out-of-scope nodes — it fails instead of skipping them.

## Provenance

- `git_commit_sha`: `e05f1935e7df1c152b689b77ec053b4c80c731eb`
- `git_dirty`: `true`
- Runtime mode: `live`

## Affected Runs

| Run ID | Page | Scope Selector | Failure |
| --- | --- | --- | --- |
| `spike-fetch-p1-scoped` | P1 (65934) | `heading:Introduction` | `ERR_SCOPE_MISS` at `extract_prose`: path `/content/10` outside allowed scope |
| `spike-fetch-p2-scoped` | P2 (98323) | `heading:Data` | `ERR_SCOPE_MISS` at `extract_prose`: path `/content/0` outside allowed scope |
| `spike-fetch-p3-scoped` | P3 (131227) | `heading:Notes` | `ERR_SCOPE_MISS` at `extract_prose`: path `/content/0` outside allowed scope |
| `spike-autodiscover-p1` | P1 (65934) | `heading:Introduction` | Same as above |
| `spike-autodiscover-p2-table` | P2 (98323) | `heading:Data` | Same as above |
| `spike-autodiscover-p3` | P3 (131227) | `heading:Notes` | Same as above |

## Scope Resolution Is Correct

Despite the `extract_prose` failure, scope resolution passes on all 3 pages:

| Page | `scope_resolution_failed` | `full_page_fetch` | `context_reduction_ratio` | `scoped_adf_bytes` | `full_page_adf_bytes` |
| --- | --- | --- | --- | --- | --- |
| P1 (65934) | `false` | `false` | 72.0% | 577 | 2061 |
| P2 (98323) | `false` | `false` | 17.3% | 1841 | 2226 |
| P3 (131227) | `false` | `false` | 67.6% | 604 | 1867 |

`allowed_scope_paths` correctly identifies heading section boundaries. For P1 with `heading:Introduction`:

```json
["/content/0", "/content/1", "/content/2", "/content/3"]
```

This covers the Introduction heading and its 3 paragraphs — correct.

## Root Cause

Location: `crates/atlassy-pipeline/src/lib.rs:608-616`, function `run_extract_prose_state()`.

```rust
for node in classify
    .payload
    .node_manifest
    .iter()
    .filter(|node| node.route == "editable_prose")
{
    let canonical_path =
        canonicalize_mapped_path(&node.path, &fetch.payload.allowed_scope_paths)
            .map_err(|error| to_hard_error(PipelineState::ExtractProse, error))?;
```

The `node_manifest` from the `classify` state contains nodes from the full page ADF, not just the scoped section. When the loop encounters an `editable_prose` node at a path like `/content/10` (outside the Introduction section scope of `/content/0` through `/content/3`), `canonicalize_mapped_path` returns an error because the path is not within `allowed_scope_paths`. The `map_err` converts this to a hard `ERR_SCOPE_MISS` error via `to_hard_error()`, which terminates the pipeline.

The state should **skip** out-of-scope `editable_prose` nodes, not **fail** on them. The scope boundary is meant to restrict edits, not block the pipeline from reading non-targeted sections.

## Proposed Fix

Add a scope filter before the `canonicalize_mapped_path` call. Use the existing `is_within_allowed_scope()` function from `atlassy-adf` to skip nodes whose paths fall outside `allowed_scope_paths`:

```rust
for node in classify
    .payload
    .node_manifest
    .iter()
    .filter(|node| node.route == "editable_prose")
    .filter(|node| is_within_allowed_scope(&node.path, &fetch.payload.allowed_scope_paths))
{
```

This preserves the existing behavior for full-page scope (where all paths are in scope) and correctly skips out-of-scope nodes for heading-scoped runs.

## Impact

- **Blocks**: All KPI batch optimized (scoped) runs. All auto-discovery validation with heading selectors.
- **Does not affect**: Full-page scope runs (baseline), single-page smoke tests (Steps 1-7), auto-discovery with full-page scope.
- **Pre-existing**: This is not a regression from roadmap/14 (target path auto-discovery). It affects all scoped operations regardless of auto-discovery.

## QA Execution Status

| Step | Status | Notes |
| --- | --- | --- |
| Step 1 (preflight) | PASS | |
| Step 2b (auto-discovery, full-page scope) | PASS | Page 131207, panel inner text |
| Step 3 (live prose update, full-page scope) | PASS | Published to version 7 |
| Step 5 (negative safety) | PASS | `ERR_SCHEMA_INVALID` at `md_assist_edit` |
| Step 6 (create subpage) | PASS | Page 66004 created |
| Step 7a/7b/7c (bootstrap matrix) | PASS | All 3 detection paths correct |
| Step 2b (scoped fetch spike, metrics) | PASS | Scope resolution correct on P1/P2/P3 |
| Step 2b (scoped fetch spike, downstream) | BLOCKED | `ERR_SCOPE_MISS` at `extract_prose` |
| Step 2b (auto-discovery with heading scope) | BLOCKED | Same |
| Step 8 (KPI batch) | BLOCKED | Optimized runs will fail |

## QA Execution Round 1 — Final Status

**Verdict**: Steps 1-7 PASS. Steps 2b (scoped) and 8 BLOCKED. Full re-test required after fix.

### All Runs

| Step | Run ID | Result |
| --- | --- | --- |
| Step 1 (preflight) | `live-preflight-001` | PASS — `failure_state: "verify"`, no publish |
| Step 2b (auto-discovery, full scope) | `investigate-autodiscover-fullpage` | PASS — `discovered_target_path: "/content/0/content/0/content/0/text"` |
| Step 3 (live prose, full scope) | `investigate-live-prose-panel` | PASS — `publish_result: "published"`, `new_version: 7` |
| Step 5 (negative safety) | `live-negative-prose-boundary-001` | PASS — `ERR_SCHEMA_INVALID` at `md_assist_edit` |
| Step 6 (create subpage) | page 66004 created | PASS — `page_version: 1` |
| Step 7a (empty, no flag) | `live-bootstrap-required-001` | PASS — `ERR_BOOTSTRAP_REQUIRED`, `empty_page_detected: true` |
| Step 7b (empty, with flag) | `live-bootstrap-success-001` | PASS — `bootstrap_applied: true`, `publish_result: "published"` |
| Step 7c (non-empty, with flag) | `live-bootstrap-invalid-001` | PASS — `ERR_BOOTSTRAP_INVALID_STATE`, `empty_page_detected: false` |
| Step 2b (P1 scoped fetch) | `spike-fetch-p1-scoped` | PARTIAL — scope resolution correct (72% reduction), blocked at `extract_prose` |
| Step 2b (P2 scoped fetch) | `spike-fetch-p2-scoped` | PARTIAL — scope resolution correct (17% reduction), blocked at `extract_prose` |
| Step 2b (P3 scoped fetch) | `spike-fetch-p3-scoped` | PARTIAL — scope resolution correct (68% reduction), blocked at `extract_prose` |
| Step 2b (P1 auto-discovery) | `spike-autodiscover-p1` | BLOCKED — `ERR_SCOPE_MISS` at `extract_prose` |
| Step 2b (P2 table auto-discovery) | `spike-autodiscover-p2-table` | BLOCKED — `ERR_SCOPE_MISS` at `extract_prose` |
| Step 2b (P3 auto-discovery) | `spike-autodiscover-p3` | BLOCKED — `ERR_SCOPE_MISS` at `extract_prose` |
| Step 8 (KPI batch) | not executed | BLOCKED — optimized runs require scoped operations |

### Pass/Fail Checklist (against test plan)

**Single-page smoke (Steps 1-7)**:

- [x] Preflight fails at verify and does not publish.
- [ ] Scoped-selector preflight run reports `scope_resolution_failed: false` and `full_page_fetch: false`. *(scope metrics correct, downstream blocked)*
- [x] Prose smoke run publishes successfully in `live` mode.
- [ ] Optional table smoke run publishes successfully. *(skipped — no table on primary page)*
- [x] Negative safety run fails with deterministic safety error code.
- [x] Create-subpage returns JSON with `page_id` and `page_version: 1`.
- [x] Bootstrap on empty page without flag fails with `ERR_BOOTSTRAP_REQUIRED`.
- [x] Bootstrap on empty page with flag succeeds with `bootstrap_applied: true`.
- [x] Bootstrap on non-empty page with flag fails with `ERR_BOOTSTRAP_INVALID_STATE`.
- [x] All run summaries include `empty_page_detected` and `bootstrap_applied` fields.
- [x] Artifacts exist for each run under `artifacts/<run_id>/`.

**KPI experiment (Step 8)**:

- [ ] Not executed — blocked by `extract_prose` scope filtering bug.

### Experiment Page Inventory

Pages exist with structural content matching the test plan specification:

| Page | ID | Headings |
| --- | --- | --- |
| P1 (Prose Rich) | 65934 | Introduction, Details, Summary |
| P2 (Mixed Prose Table) | 98323 | Overview, Data |
| P3 (Locked Adjacent) | 131227 | Context, Notes, References |

### Additional Investigation

Auto-discovery on minimal page structure (page 131207 with panel + empty paragraph) documented separately in `qa/investigations/2026-03-08-auto-discovery-minimal-page.md`. Auto-discovery correctly identifies panel inner paragraph text as a valid prose target with full-page scope.

## Next Steps

1. Fix `extract_prose` scope filtering in `crates/atlassy-pipeline/src/lib.rs`.
2. Verify `cargo test --workspace` and `cargo clippy` pass after fix.
3. Re-run full QA execution (Steps 1-8) from clean state.
