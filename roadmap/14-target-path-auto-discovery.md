# Target Path Auto-Discovery

## Objective

Eliminate manual `target_path` discovery from the KPI experiment workflow. The pipeline should auto-select a valid edit target at runtime based on the page's ADF structure and scope, removing the need for operators to run `jq` path inspection and hardcode paths into manifests.

## Current Baseline

The KPI batch manifest (`qa/manifests/kpi-revalidation-batch.json`) requires an explicit `target_path` for every run entry. This path is a JSON pointer into the page's ADF tree (e.g., `/content/1/content/0/text`) identifying the exact text node to edit.

### Discovery workflow (current)

1. Run a full-page preflight fetch with `--mode no-op --force-verify-fail`.
2. Inspect the fetch state output with `jq` to list all text leaf paths.
3. Manually pick prose and table cell text paths per page.
4. Record paths in `qa/manifests/sandbox-page-inventory.md`.
5. Copy paths into the batch manifest's `target_path` fields.

### Problems

- **Fragile**: paths are valid only for a specific page version. Any structural edit (adding/removing paragraphs, reordering content) invalidates them.
- **Manual overhead**: every KPI experiment requires a multi-step discovery cycle before the batch can run.
- **Stale data**: the 2026-03-08 KPI revalidation found that baseline runs advanced pages to version 6, requiring re-discovery before the next batch.
- **Coupling**: the manifest encodes runtime state (page structure) that should be resolved dynamically.

## Design

### Core change

Make `target_path` optional in `RunMode` variants. When omitted, the pipeline auto-discovers the first valid text node within the allowed scope after fetch and classify.

### Discovery function

New public function in `crates/atlassy-adf/src/lib.rs`:

```rust
pub fn discover_target_path(
    node_path_index: &BTreeMap<String, String>,
    allowed_scope_paths: &[String],
    route: TargetRoute,
    target_index: usize,
) -> Result<String, AdfError>
```

**Parameters:**

| Parameter | Type | Purpose |
|-----------|------|---------|
| `node_path_index` | `&BTreeMap<String, String>` | Path-to-type index from fetch state (already available) |
| `allowed_scope_paths` | `&[String]` | Section-expanded scope boundary from fetch state (already available) |
| `route` | `TargetRoute` | Whether to find a prose text node or a table cell text node |
| `target_index` | `usize` | Which match to select (0 = first, 1 = second, etc.) |

**`TargetRoute` enum** (new, in `crates/atlassy-adf/src/lib.rs`):

```rust
pub enum TargetRoute {
    Prose,
    TableCell,
}
```

**Algorithm:**

1. Collect all paths in `node_path_index` where node type is `"text"`. These are node paths (e.g., `/content/1/content/0`), not property paths.
2. Filter to paths within `allowed_scope_paths` (reuse existing `is_within_allowed_scope`).
3. Filter by route using `path_has_ancestor_type()` on node paths:
   - `Prose`: exclude paths where `path_has_ancestor_type(path, index, &["table", "tableRow", "tableCell"])` is true. Include only paths where `path_has_ancestor_type(path, index, EDITABLE_PROSE_TYPES)` is true.
   - `TableCell`: include only paths where `path_has_ancestor_type(path, index, &["table", "tableRow", "tableCell"])` is true.
4. Sort paths lexicographically (deterministic ordering by document position).
5. Index into the sorted list at `target_index`.
6. Construct the property path by appending `/text` to the selected node path.
7. If index is out of bounds, return `AdfError::TargetDiscoveryFailed`.

> **Note**: Step 3 uses `path_has_ancestor_type()` directly instead of `is_table_cell_text_path()`. The latter requires paths ending in `/text` (property paths), but at this point in the algorithm we are working with node paths from `node_path_index` which do not have the `/text` suffix. The property path is constructed after selection in step 6.

### Data availability

All inputs to the discovery function are already materialized by the pipeline before any edit state runs:

| Input | Produced by | Pipeline state | Location |
|-------|------------|----------------|----------|
| `node_path_index` | `build_node_path_index()` | fetch | `atlassy-adf/src/lib.rs:99` |
| `allowed_scope_paths` | `resolve_scope()` / `expand_heading_to_section()` | fetch | `atlassy-adf/src/lib.rs:48` |
| Route classification | `route_for_node()` | classify | `atlassy-pipeline/src/lib.rs:1355` |
| `is_table_cell_text_path()` | existing function | available anytime | `atlassy-adf/src/lib.rs:241` |

No new pipeline state is needed. Discovery runs inside the existing edit states when `target_path` is `None`.

### Shared prose whitelist constant

The discovery function's route filter (algorithm step 3) checks ancestors against the 7-type prose whitelist. This list currently exists only as a match pattern inside `route_for_node()` in `crates/atlassy-pipeline/src/lib.rs`. Since `discover_target_path()` lives in `crates/atlassy-adf/src/lib.rs`, the whitelist must be accessible from both crates.

New public constant in `crates/atlassy-adf/src/lib.rs`:

```rust
pub const EDITABLE_PROSE_TYPES: &[&str] = &[
    "paragraph", "heading", "bulletList", "orderedList",
    "listItem", "blockquote", "codeBlock",
];
```

`route_for_node()` in `atlassy-pipeline` should reference `EDITABLE_PROSE_TYPES` instead of its inline match pattern, establishing a single source of truth.

### Pipeline integration

The discovery call is inserted into `project_prose_candidate()` and `project_table_candidate()` in `crates/atlassy-pipeline/src/lib.rs`, before canonicalization:

```
if target_path is None:
    target_path = discover_target_path(
        node_path_index,
        allowed_scope_paths,
        route,       // Prose or TableCell based on RunMode variant
        target_index // from RunRequest, default 0
    )
    .map_err(|e| PipelineError::Hard {
        state: current_pipeline_state,       // MdAssistEdit or AdfTableEdit
        code: ERR_TARGET_DISCOVERY_FAILED,
        message: e.to_string(),
    })?
```

**`target_index` placement**: Add `pub target_index: usize` to `RunRequest` (default `0`). It is set from `ManifestRunEntry.target_index` (batch mode) or `--target-index` (CLI mode). The field is only consulted when `target_path` is `None` in the active `RunMode` variant.

After discovery, the resolved path flows through the existing canonicalization, scope check, and patch logic unchanged.

### Error handling

**New error variant** in `crates/atlassy-adf/src/lib.rs`:

```rust
#[error("no valid {route} target found in scope (index {index}, candidates found: {found})")]
TargetDiscoveryFailed {
    route: String,
    index: usize,
    found: usize,
}
```

**Error routing**: Discovery errors are mapped to `ERR_TARGET_DISCOVERY_FAILED` via explicit `map_err` at the call site in each edit state function, not through the generic `to_hard_error()` string matcher. This is necessary because:

1. `to_hard_error()` uses substring matching on the error message. The `TargetDiscoveryFailed` message contains "scope", which would incorrectly match `ERR_SCOPE_MISS` before falling through to the intended code.
2. The `From<AdfError> for PipelineError` impl hardcodes `PipelineState::Patch`, but discovery errors originate in `MdAssistEdit` or `AdfTableEdit`. Explicit `map_err` attaches the correct state.

`ERR_TARGET_DISCOVERY_FAILED` is a new constant in `crates/atlassy-contracts/src/lib.rs`.

**Design decision**: no silent fallback. If auto-discovery finds no valid target in scope, the run fails with a clear error. This prevents masking scope misconfiguration or empty sections.

### `RunMode` changes

In `crates/atlassy-pipeline/src/lib.rs`, change `target_path` from `String` to `Option<String>` in the two route-specific variants:

```rust
pub enum RunMode {
    NoOp,
    SimpleScopedUpdate { target_path: String, new_value: Value },
    SimpleScopedProseUpdate { target_path: Option<String>, markdown: String },
    SimpleScopedTableCellUpdate { target_path: Option<String>, text: String },
    ForbiddenTableOperation { target_path: String, operation: TableOperation },
    SyntheticRouteConflict { prose_path: String, table_path: String },
    SyntheticTableShapeDrift { path: String },
}
```

When `Some(path)` is provided, current behavior is preserved (explicit path, no discovery). When `None`, auto-discovery is triggered.

> **Variant scope**: Only `SimpleScopedProseUpdate` and `SimpleScopedTableCellUpdate` support auto-discovery. `SimpleScopedUpdate` keeps `target_path: String` because it sends the path to both prose and table route projections — auto-discovery would be ambiguous without a known route. `ForbiddenTableOperation`, `SyntheticRouteConflict`, and `SyntheticTableShapeDrift` are test/synthetic variants with deliberately-crafted paths that should not be auto-discovered.

### Manifest format

**Backward compatible.** The `ManifestRunEntry` in `crates/atlassy-cli/src/main.rs` already has `target_path: Option<String>`. Changes:

1. Stop defaulting `target_path` to a hardcoded path in `run_mode_from_manifest()` when the field is absent.
2. Add `target_index: Option<u32>` field to `ManifestRunEntry` (default 0 when absent).

**Before** (current, fragile):

```json
{
  "scope_selectors": ["heading:Introduction"],
  "mode": "simple_scoped_prose_update",
  "target_path": "/content/1/content/0/text",
  "new_value": "Updated text"
}
```

**After** (auto-discover):

```json
{
  "scope_selectors": ["heading:Introduction"],
  "mode": "simple_scoped_prose_update",
  "new_value": "Updated text"
}
```

**With explicit index** (optional):

```json
{
  "scope_selectors": ["heading:Introduction"],
  "mode": "simple_scoped_prose_update",
  "target_index": 1,
  "new_value": "Updated text"
}
```

**With explicit path** (backward compat, unchanged):

```json
{
  "scope_selectors": ["heading:Introduction"],
  "mode": "simple_scoped_prose_update",
  "target_path": "/content/1/content/0/text",
  "new_value": "Updated text"
}
```

**Baseline run behavior**: Runs with empty `scope_selectors` fall back to full-page scope. Auto-discovery against full-page scope returns the first (or Nth) text node on the entire page. For KPI experiments, this means all baseline runs on the same page discover the same target node (absent differing `target_index` values). This is acceptable for KPI measurement (pipeline performance metrics, not content targeting) but differs from the legacy behavior where each baseline run explicitly targeted a different section's path.

### CLI changes

The `--target-path` CLI argument is already optional. When omitted, pass `None` through to `RunMode` instead of defaulting. Add `--target-index <N>` optional argument for CLI single-run usage.

### Telemetry

Add `discovered_target_path: Option<String>` to the run summary output. When auto-discovery is used, this field contains the resolved path. When explicit `target_path` is provided, this field is `null`. This gives operators visibility into what the pipeline auto-selected.

## Test Plan

| Test | Crate | Validates |
|------|-------|-----------|
| `discovers_first_prose_text_in_section` | `atlassy-adf` | Heading + paragraphs: returns first paragraph's text path |
| `discovers_nth_prose_text_with_index` | `atlassy-adf` | `target_index: 1` picks second text node, not first |
| `discovers_table_cell_text` | `atlassy-adf` | ADF with table in scope: returns first cell text path |
| `discovery_respects_scope_boundary` | `atlassy-adf` | Text nodes outside `allowed_scope_paths` are excluded |
| `discovery_fails_on_empty_section` | `atlassy-adf` | Heading-only section (no content nodes): returns `TargetDiscoveryFailed` |
| `discovery_fails_on_out_of_bounds_index` | `atlassy-adf` | `target_index: 5` when only 2 candidates: returns `TargetDiscoveryFailed` with `found: 2` |
| `explicit_target_path_skips_discovery` | `atlassy-pipeline` | Provided `target_path` bypasses discovery, discovery function is not called |
| `pipeline_auto_discovers_and_patches` | `atlassy-pipeline` | End-to-end: omit `target_path`, auto-discover, patch succeeds with correct path |

## Done Criteria

- `discover_target_path()` function exists in `atlassy-adf` and is public.
- `TargetRoute` enum exists in `atlassy-adf` and is public.
- `AdfError::TargetDiscoveryFailed` variant exists with route, index, and found fields.
- `EDITABLE_PROSE_TYPES` constant exists in `atlassy-adf` and is public.
- `route_for_node()` in `atlassy-pipeline` references `EDITABLE_PROSE_TYPES` (no inline whitelist duplication).
- `ERR_TARGET_DISCOVERY_FAILED` constant exists in `atlassy-contracts`.
- `RunMode::SimpleScopedProseUpdate` and `RunMode::SimpleScopedTableCellUpdate` accept `Option<String>` for `target_path`.
- `run_md_assist_edit_state()` calls discovery when `target_path` is `None` with explicit `map_err` to `ERR_TARGET_DISCOVERY_FAILED`.
- `run_adf_table_edit_state()` calls discovery when `target_path` is `None` with explicit `map_err` to `ERR_TARGET_DISCOVERY_FAILED`.
- `RunRequest` has `target_index: usize` field (default `0`).
- `ManifestRunEntry` has `target_index: Option<u32>` field.
- `ManifestRunEntry.timestamp` has `#[serde(default)]` (optional in manifests).
- `run_mode_from_manifest()` passes `None` when `target_path` is absent (no defaults).
- `--target-index` CLI argument is available.
- `discovered_target_path` appears in run summary output when auto-discovery is used.
- All 8 tests pass.
- `cargo test --workspace` passes.
- `cargo clippy --workspace --all-targets -- -D warnings` passes.
- Existing manifests with explicit `target_path` produce identical behavior (backward compat).

## Cross-References

- `04-kpi-and-experiments.md`: KPI batch workflow that this feature simplifies.
- `08-poc-scope.md`: PoC scope and edit patterns (A, B, C).
- `10-testing-strategy-and-simulation.md`: test strategy alignment.
- `qa/confluence-sandbox-test-plan.md`: live validation steps updated to include auto-discovery checks.
- `qa/manifests/sandbox-page-inventory.md`: target path inventory becomes informational.
- `qa/manifests/kpi-revalidation-auto-discovery.example.json`: example manifest using auto-discovery format.
