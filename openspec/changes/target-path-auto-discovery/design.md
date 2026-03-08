## Context

The pipeline currently requires an explicit `target_path` (a JSON pointer into the ADF tree) for every edit run. This path identifies the exact text node to modify. Operators discover these paths manually via `jq` inspection of fetched ADF state, then hardcode them into batch manifests. Paths are tied to a specific page version and break whenever page structure changes.

All the data needed for automatic discovery already exists in the pipeline before any edit state runs:
- `node_path_index` (path-to-type map from `build_node_path_index()`)
- `allowed_scope_paths` (scope boundary from `resolve_scope()`)
- Route classification logic (prose vs table ancestry checks via `path_has_ancestor_type()`)

The change makes `target_path` optional in the two route-specific `RunMode` variants. When omitted, the pipeline resolves a valid target at runtime using the existing index and scope data.

## Goals / Non-Goals

**Goals:**

- Eliminate the manual `jq` path discovery cycle from the KPI experiment workflow.
- Make `target_path` optional for `SimpleScopedProseUpdate` and `SimpleScopedTableCellUpdate` run modes.
- Provide deterministic, reproducible target selection (same page state + same index = same target).
- Preserve full backward compatibility with explicit `target_path` manifests.
- Surface the discovered path in telemetry for operator visibility.

**Non-Goals:**

- Auto-discovery for `SimpleScopedUpdate` (ambiguous route — sends path to both prose and table projections).
- Auto-discovery for synthetic/test variants (`ForbiddenTableOperation`, `SyntheticRouteConflict`, `SyntheticTableShapeDrift`).
- Changing the discovery algorithm to prefer "better" targets (e.g., longer text, more central position). First-match-by-document-order is sufficient.
- Multi-target discovery in a single run (selecting multiple targets simultaneously).

## Decisions

### 1. Discovery function lives in `atlassy-adf`, not `atlassy-pipeline`

**Choice**: Place `discover_target_path()` in `crates/atlassy-adf/src/lib.rs`.

**Rationale**: The function operates exclusively on ADF data structures (`BTreeMap<String, String>` node path index, scope path slices) and uses existing `atlassy-adf` helpers (`is_within_allowed_scope`, `path_has_ancestor_type`). Placing it in `atlassy-pipeline` would require either re-exporting those helpers or duplicating logic. The `atlassy-adf` crate is the natural home for ADF structural queries.

**Alternative considered**: Adding to `atlassy-pipeline` near the call sites. Rejected because it would create an upward dependency or require duplicating scope/path logic.

### 2. Discovery resolves in the edit state match arms, not inside projection functions

**Choice**: Discovery runs inside the `match &request.run_mode` arms in `run_md_assist_edit_state()` (line 710) and `run_adf_table_edit_state()` (line 817), resolving `Option<String>` to `String` before passing to `project_prose_candidate()` / `project_table_candidate()`.

**Rationale**: The projection functions (`project_prose_candidate` at line 1252, `project_table_candidate` at line 1291) take `target_path: &str`. Changing their signature to `Option<&str>` would spread optionality deeper into the pipeline and require every caller to handle `None`. Resolving early keeps the change surface minimal — only the two match arms change, and the projection functions remain unchanged.

```
run_md_assist_edit_state()
  └─ match &request.run_mode
       └─ SimpleScopedProseUpdate { target_path: None, .. }
            ├─ discover_target_path(..., Prose, target_index)  ← HERE
            └─ project_prose_candidate(&resolved_path, ...)    ← unchanged
```

### 3. Algorithm operates on node paths, appends `/text` at the end

**Choice**: The 7-step algorithm filters and selects node paths (e.g., `/content/1/content/0`), then appends `/text` only after selection to produce the property path (e.g., `/content/1/content/0/text`).

**Rationale**: `node_path_index` maps node paths to types — its keys don't include `/text` suffixes. The existing `path_has_ancestor_type()` function works on node paths. Converting to property paths earlier would break ancestor lookups. The `/text` suffix is a property path concern that only matters for the final output.

**Note**: This is why the algorithm uses `path_has_ancestor_type()` directly rather than `is_table_cell_text_path()` — the latter requires property paths (ending in `/text`).

### 4. Error routing via explicit `map_err`, not `to_hard_error()`

**Choice**: Discovery errors are mapped to `ERR_TARGET_DISCOVERY_FAILED` via explicit `map_err` at each call site, bypassing `to_hard_error()`.

**Rationale**: Two concrete problems with `to_hard_error()`:

1. **Substring collision**: `to_hard_error()` uses `message.contains("scope")` to match `ERR_SCOPE_MISS`. The `TargetDiscoveryFailed` message ("no valid {route} target found in scope...") contains "scope", which would trigger the wrong error code.

2. **Wrong pipeline state**: The `From<AdfError> for PipelineError` impl hardcodes `PipelineState::Patch`. Discovery errors originate in `MdAssistEdit` or `AdfTableEdit`. Explicit `map_err` attaches the correct state.

**Alternative considered**: Fixing `to_hard_error()` to check for discovery errors first. Rejected because adding more substring checks to a fragile matcher increases coupling. Explicit mapping at the call site is clearer and self-documenting.

### 5. Lexicographic sort for deterministic ordering

**Choice**: Sort candidate paths lexicographically before indexing.

**Rationale**: ADF content paths use numeric segment indices (`/content/0`, `/content/1`, etc.). Lexicographic sort of these paths approximates document order for single-digit indices. For ADF documents with fewer than 10 top-level blocks per level (which covers all current sandbox pages and typical Confluence content), lexicographic and numeric order are identical.

**Known limitation**: Lexicographic sort would misordering paths once indices reach 10+ (e.g., `/content/9` sorts after `/content/10`). This is acceptable for the current use case (KPI experiments on structured sandbox pages). A numeric-aware path comparator could be added later if needed.

### 6. No silent fallback on discovery failure

**Choice**: If discovery finds no valid target in scope, the run fails with `AdfError::TargetDiscoveryFailed` (hard error). No fallback to a default path or full-page scope.

**Rationale**: Silent fallback would mask scope misconfiguration (wrong heading selector) or empty sections (heading with no content). Failing explicitly gives the operator a clear signal with diagnostic data (route, requested index, candidates found) to fix the manifest.

### 7. `target_index` on `RunRequest`, not on `RunMode`

**Choice**: Add `target_index: usize` to `RunRequest` (default 0), not as a field inside `RunMode` variants.

**Rationale**: `target_index` is only meaningful when `target_path` is `None`. Embedding it in `RunMode` variants would either require it on every variant (noise) or only on the two discovery-capable variants (asymmetric). Placing it on `RunRequest` keeps `RunMode` focused on describing the edit operation, while `RunRequest` handles runtime configuration like `force_verify_fail` and `bootstrap_empty_page`.

### 8. Shared `EDITABLE_PROSE_TYPES` constant

**Choice**: Define `EDITABLE_PROSE_TYPES` as a public `&[&str]` constant in `atlassy-adf`. Refactor `route_for_node()` in `atlassy-pipeline` to reference it instead of its inline match pattern.

**Rationale**: The 7-type whitelist (`paragraph`, `heading`, `bulletList`, `orderedList`, `listItem`, `blockquote`, `codeBlock`) currently exists only as a match pattern inside `route_for_node()`. The discovery function needs the same list for its prose route filter. Duplicating it would create a maintenance risk. A shared constant establishes a single source of truth across both crates.

### 9. `discovered_target_path` telemetry field

**Choice**: Add `discovered_target_path: Option<String>` to `RunSummary` in `atlassy-contracts`. Set it to the resolved path when auto-discovery is used; leave it `None` when an explicit `target_path` is provided.

**Rationale**: Operators need visibility into what the pipeline auto-selected, both for debugging and for validating that auto-discovery matches expectations against the known page inventory. The field is `Option` to distinguish "discovery was used" from "explicit path was given" — when `Some`, the value is what was discovered; when `None`, no discovery occurred.

## Risks / Trade-offs

**[Lexicographic sort vs numeric path order]** Lexicographic sort produces incorrect ordering for paths with segment indices >= 10. **Mitigation**: Current sandbox pages have < 10 top-level blocks. Document the limitation. Add numeric-aware sorting if page complexity grows.

**[Breaking `RunMode` type change]** Changing `target_path` from `String` to `Option<String>` in two variants breaks all existing construction sites. **Mitigation**: This is a compile-time break — the compiler will find every site. Wrap existing values in `Some(...)`. No runtime behavior changes for explicit paths.

**[Baseline run behavioral difference]** Baseline runs (empty `scope_selectors` = full-page scope) with auto-discovery will all target the same first text node on the page, unlike legacy behavior where each baseline targeted different manually-chosen sections. **Mitigation**: This is acceptable for KPI measurement (pipeline performance, not content targeting) and is explicitly documented in the roadmap. Operators can use `target_index` to vary targets if needed.

**[`is_within_allowed_scope()` is private]** The discovery function needs this helper. **Mitigation**: Both live in `crates/atlassy-adf/src/lib.rs`, so module-level visibility is sufficient. No change needed.
