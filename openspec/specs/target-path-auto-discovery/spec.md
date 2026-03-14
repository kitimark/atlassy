## Purpose

Define deterministic runtime target-path discovery for route-specific scoped edits so operators can omit brittle manifest `target_path` values while preserving backward compatibility for explicit paths.

## Requirements

### Requirement: Discovery function resolves a valid text node path within scope

The `discover_target_path()` function SHALL accept a node path index, allowed scope paths, a target route (`Prose` or `TableCell`), and a target index, and MUST return a property path (ending in `/text`) pointing to a valid text node within the allowed scope boundary. For `Prose` route, the resolved text node MUST NOT be under a `SCOPE_ANCHOR_TYPES` ancestor.

#### Scenario: Discovers first prose text node in a section with heading

- **WHEN** the node path index contains a heading text node and paragraph text nodes under a heading section, and allowed scope paths include that section
- **THEN** `discover_target_path()` with route `Prose` and index `0` returns the property path of the first paragraph (or other non-heading editable prose) text node within scope, skipping the heading text node

#### Scenario: Discovers Nth prose text node with target index

- **WHEN** the node path index contains a heading and multiple paragraph text nodes within scope and target index is `1`
- **THEN** `discover_target_path()` returns the property path of the second non-heading text node (by document-order path sort), not the first

#### Scenario: Discovers first table cell text node

- **WHEN** the node path index contains text nodes under table cell ancestry within allowed scope
- **THEN** `discover_target_path()` with route `TableCell` and index `0` returns the property path of the first text node that has a `table`/`tableRow`/`tableCell` ancestor

#### Scenario: Heading-only section produces discovery failure

- **WHEN** `allowed_scope_paths` covers a section that contains only a heading with no paragraph, list, or other non-heading editable prose nodes
- **THEN** `discover_target_path()` with route `Prose` returns `TargetDiscoveryFailed` with `found: 0`

### Requirement: Discovery filters by route using ancestor type checks

The discovery function SHALL filter candidate text nodes by route: `Prose` candidates MUST have at least one ancestor in the editable prose types list, MUST NOT have any ancestor in `SCOPE_ANCHOR_TYPES`, and MUST NOT have any `table`, `tableRow`, or `tableCell` ancestors. `TableCell` candidates MUST have at least one `table`, `tableRow`, or `tableCell` ancestor. Filtering MUST use `path_has_ancestor_type()` on node paths (not property paths).

#### Scenario: Prose route excludes table-nested text nodes

- **WHEN** a text node exists under a `tableCell` ancestor within scope
- **THEN** `discover_target_path()` with route `Prose` does not select it, even if it also has a `paragraph` ancestor

#### Scenario: Table route excludes non-table text nodes

- **WHEN** a text node exists under a `paragraph` ancestor (not inside a table) within scope
- **THEN** `discover_target_path()` with route `TableCell` does not select it

#### Scenario: Prose route excludes heading text nodes

- **WHEN** a text node exists under a `heading` ancestor within scope
- **THEN** `discover_target_path()` with route `Prose` does not select it, because `heading` is in `SCOPE_ANCHOR_TYPES`

#### Scenario: First prose candidate is paragraph not heading

- **WHEN** a section contains a heading at `/content/0` followed by a paragraph at `/content/1`, both within scope
- **THEN** `discover_target_path()` with route `Prose` and index `0` returns `/content/1/content/0/text` (the paragraph text), not `/content/0/content/0/text` (the heading text)

#### Scenario: Heading text is still reachable via explicit target path

- **WHEN** a manifest or CLI invocation provides `target_path: "/content/0/content/0/text"` pointing to a heading text node
- **THEN** the pipeline uses the explicit path directly without invoking discovery, and the heading text node is editable as before

### Requirement: Discovery respects scope boundary

The discovery function SHALL only consider text nodes within `allowed_scope_paths`. Text nodes outside the scope boundary MUST be excluded regardless of their type or route classification.

#### Scenario: Text nodes outside scope are excluded

- **WHEN** the node path index contains text nodes both inside and outside `allowed_scope_paths`
- **THEN** `discover_target_path()` only considers nodes within scope and ignores out-of-scope nodes

### Requirement: Discovery constructs property paths from node paths

The discovery function SHALL operate on node paths from `node_path_index` (which do not include `/text` suffixes) during filtering and selection, and MUST append `/text` to the selected node path to produce the final property path.

#### Scenario: Output path has /text suffix

- **WHEN** `discover_target_path()` selects a node at path `/content/1/content/0`
- **THEN** the returned path is `/content/1/content/0/text`

### Requirement: Discovery uses deterministic document-order sorting

The discovery function SHALL sort candidate node paths in document order before applying the target index. Document order SHALL be determined by splitting paths on `/` and comparing each segment: segments parseable as unsigned integers SHALL be compared numerically, all other segments SHALL be compared lexicographically. Shorter paths SHALL sort before longer paths when all shared segments are equal. This ensures the same page state and scope always produce the same target selection in reading order.

#### Scenario: Deterministic ordering across invocations

- **WHEN** the same node path index and allowed scope paths are provided twice
- **THEN** `discover_target_path()` with the same route and index returns the same path both times

#### Scenario: Document order for double-digit content indices

- **WHEN** a page has top-level content blocks at indices 0 through 12, with paragraphs at `/content/2/content/0` and `/content/10/content/0`
- **THEN** `discover_target_path()` considers `/content/2/content/0` before `/content/10/content/0` (numeric comparison: 2 < 10), not after (string comparison: `"10"` < `"2"`)

#### Scenario: Single-digit indices retain correct order

- **WHEN** a page has top-level content blocks at indices 0 through 5
- **THEN** document-order sort produces the same order as lexicographic sort (no behavioral change for small pages)

### Requirement: Discovery fails explicitly when no valid target exists

The discovery function SHALL return `AdfError::TargetDiscoveryFailed` when no valid text node is found for the requested route and index. The error MUST include the route name, requested index, and number of candidates found. The pipeline MUST NOT silently fall back to a default path.

#### Scenario: Out-of-bounds index produces discovery failure

- **WHEN** `target_index` is `5` but only `2` candidates exist within scope for the requested route
- **THEN** `discover_target_path()` returns `TargetDiscoveryFailed` with `index: 5` and `found: 2`

### Requirement: Discovery errors map to ERR_TARGET_DISCOVERY_FAILED

Discovery errors MUST be mapped to `ERR_TARGET_DISCOVERY_FAILED` via explicit `map_err` at each pipeline call site (in `run_md_assist_edit_state()` and `run_adf_table_edit_state()`). Discovery errors MUST NOT pass through `to_hard_error()` to avoid substring matching collisions with `ERR_SCOPE_MISS`.

#### Scenario: Discovery error carries correct pipeline state

- **WHEN** prose discovery fails in `run_md_assist_edit_state()`
- **THEN** the resulting `PipelineError::Hard` has state `MdAssistEdit` and code `ERR_TARGET_DISCOVERY_FAILED`

#### Scenario: Table discovery error carries correct pipeline state

- **WHEN** table cell discovery fails in `run_adf_table_edit_state()`
- **THEN** the resulting `PipelineError::Hard` has state `AdfTableEdit` and code `ERR_TARGET_DISCOVERY_FAILED`

### Requirement: Explicit target path bypasses discovery

When `target_path` is `Some(path)` in a `RunMode` variant, the pipeline MUST use the provided path directly without invoking `discover_target_path()`. Existing behavior for explicit paths SHALL be preserved unchanged.

#### Scenario: Explicit path skips discovery

- **WHEN** `RunMode::SimpleScopedProseUpdate` has `target_path: Some("/content/1/content/0/text")`
- **THEN** the pipeline uses `/content/1/content/0/text` directly and does not call `discover_target_path()`

### Requirement: Pipeline auto-discovers and patches successfully

When `target_path` is `None` in a route-specific `RunMode` variant, the pipeline SHALL call `discover_target_path()` to resolve a target path, then proceed through canonicalization, scope check, and patch using the resolved path. The end-to-end flow MUST produce the same patch outcome as if the discovered path had been provided explicitly.

#### Scenario: End-to-end auto-discovery prose patch

- **WHEN** `RunMode::SimpleScopedProseUpdate` has `target_path: None` and the page has prose text nodes in scope
- **THEN** the pipeline discovers a target, patches it with the provided markdown, and produces a valid patch output

#### Scenario: End-to-end auto-discovery table cell patch

- **WHEN** `RunMode::SimpleScopedTableCellUpdate` has `target_path: None` and the page has table cell text nodes in scope
- **THEN** the pipeline discovers a target, patches it with the provided text, and produces a valid patch output

### Requirement: RunMode variants accept optional target path

`RunMode::SimpleScopedProseUpdate` and `RunMode::SimpleScopedTableCellUpdate` SHALL have `target_path` typed as `Option<String>`. Synthetic and test variants SHALL keep their path fields as `String`. The `RunMode::SimpleScopedUpdate` variant SHALL be removed as dead code - it is superseded by the route-specific variants and cannot support auto-discovery due to route ambiguity.

#### Scenario: SimpleScopedProseUpdate accepts None

- **WHEN** a `RunMode::SimpleScopedProseUpdate` is constructed with `target_path: None`
- **THEN** the variant is valid and triggers auto-discovery at runtime

#### Scenario: SimpleScopedUpdate is removed

- **WHEN** the `RunMode` enum is inspected
- **THEN** no `SimpleScopedUpdate` variant exists
- **AND** no `ManifestMode::SimpleScopedUpdate` variant exists
- **AND** no CLI mode string `"simple-scoped-update"` is accepted

### Requirement: Target index selects among discovery candidates

`RunRequest` SHALL include a `target_index: usize` field (default `0`). When auto-discovery runs, it SHALL use `target_index` to select among the sorted candidate list. `target_index` MUST only be consulted when `target_path` is `None`.

#### Scenario: Default target index selects first candidate

- **WHEN** `target_index` is `0` (default) and auto-discovery finds 3 candidates
- **THEN** the first candidate by document order is selected

#### Scenario: Non-zero target index selects later candidate

- **WHEN** `target_index` is `2` and auto-discovery finds 3 candidates
- **THEN** the third candidate by document order is selected

### Requirement: Manifest supports optional target path and target index

`ManifestRunEntry` SHALL have `target_path: Option<String>` (existing) and `target_index: Option<u32>` (new, defaults to `0` when absent). `timestamp` SHALL have `#[serde(default)]` to allow omission in manifests. `run_mode_from_manifest()` SHALL pass `None` for `target_path` when the field is absent, without defaulting to any hardcoded path.

#### Scenario: Manifest entry without target_path triggers discovery

- **WHEN** a manifest entry omits `target_path`
- **THEN** `run_mode_from_manifest()` produces a `RunMode` variant with `target_path: None`

#### Scenario: Manifest entry with explicit target_path preserves it

- **WHEN** a manifest entry includes `"target_path": "/content/1/content/0/text"`
- **THEN** `run_mode_from_manifest()` produces a `RunMode` variant with `target_path: Some("/content/1/content/0/text")`

#### Scenario: Manifest entry with target_index

- **WHEN** a manifest entry includes `"target_index": 1` and omits `target_path`
- **THEN** auto-discovery uses index `1` to select the second candidate

### Requirement: CLI supports target index argument

The CLI SHALL accept an optional `--target-index <N>` argument. When provided, the value SHALL be passed to `RunRequest.target_index`. When omitted, `target_index` SHALL default to `0`.

#### Scenario: CLI with target index

- **WHEN** the user runs with `--target-index 2` and omits `--target-path`
- **THEN** the pipeline uses `target_index: 2` for auto-discovery

### Requirement: Discovered target path appears in telemetry

`RunSummary` SHALL include `discovered_target_path: Option<String>`. When auto-discovery resolves a target, this field MUST contain the resolved property path. When an explicit `target_path` is provided, this field MUST be `None`.

#### Scenario: Auto-discovery populates telemetry

- **WHEN** a run uses auto-discovery and succeeds
- **THEN** `summary.json` contains `discovered_target_path` set to the resolved path

#### Scenario: Explicit path leaves telemetry null

- **WHEN** a run uses an explicit `target_path`
- **THEN** `summary.json` contains `discovered_target_path` set to `null`
