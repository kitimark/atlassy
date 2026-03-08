## Context

`discover_target_path()` auto-selects a text node for editing when `target_path` is omitted from a manifest or CLI invocation. It filters the `node_path_index` for text nodes with editable prose ancestors, sorts candidates lexicographically, and returns the node at `target_index`.

Two defects were found during KPI revalidation v2:

1. **Heading text selected as edit target.** `EDITABLE_PROSE_TYPES` includes `"heading"`, so heading text nodes pass the prose ancestor filter. When a baseline run auto-discovers and overwrites heading text, subsequent optimized runs with `heading:X` scope selectors fail because the heading text no longer matches.

2. **Non-document-order sorting.** Candidates are string-sorted, so `/content/10/content/0` sorts before `/content/2/content/0` (character `'1'` < `'2'`). This means `target_index: 0` may not select the first node in document reading order for pages with 10+ top-level content blocks.

The system currently has three `.sort()` call sites on ADF paths (in `resolve_scope`, `discover_target_path`, and `normalize_changed_paths`) plus three `sort_by` calls on structs containing paths. All use default lexicographic comparison.

## Goals / Non-Goals

**Goals:**

- Heading text nodes are excluded from prose auto-discovery while remaining editable via explicit `target_path`.
- All ADF path sorting in the pipeline produces document order (the order a reader sees content on the page).
- The separation between "editable" and "discoverable" is explicit in the code and spec, not implicit.
- Existing tests that assert heading-first discovery are updated to reflect the corrected behavior.

**Non-Goals:**

- Changing route classification. Headings remain `editable_prose` in the classify state.
- Making the exclusion list configurable at runtime or via manifest fields.
- Fixing `normalize_changed_paths()` in `atlassy-contracts` — this sorts changed paths for dedup/validation, not for discovery ordering, and string sort is adequate there.
- Addressing `sort_by` on `md_block_id` or `table_candidates.path` in the pipeline — these sort by struct fields for determinism in mapping and merge, not for document-order selection.

## Decisions

### Decision 1: Exclusion constant derived from editability, not a parallel list

**Choice:** Add `SCOPE_ANCHOR_TYPES: &[&str] = &["heading"]` as an exclusion filter applied on top of `EDITABLE_PROSE_TYPES`.

**Rationale:** The discoverable set is `EDITABLE_PROSE_TYPES - SCOPE_ANCHOR_TYPES`. This avoids a parallel `DISCOVERABLE_PROSE_TYPES` list that could drift from the editable list. New types added to `EDITABLE_PROSE_TYPES` are discoverable by default (safe default). Only types explicitly added to `SCOPE_ANCHOR_TYPES` are excluded.

**Alternative considered:** A second positive list `DISCOVERABLE_PROSE_TYPES` without `"heading"`. Rejected because it creates two lists that must be kept in sync manually. A developer adding `"codeBlock"` to one list might forget the other.

**Alternative considered:** A runtime filter flag on the manifest (e.g., `exclude_heading_from_discovery: true`). Rejected because heading exclusion is a correctness requirement, not a preference. Heading text used as a scope anchor should never be auto-discovered.

### Decision 2: Name the constant `SCOPE_ANCHOR_TYPES`

**Choice:** The constant is named `SCOPE_ANCHOR_TYPES` rather than `DISCOVERY_EXCLUDED_TYPES` or `NON_DISCOVERABLE_PROSE_TYPES`.

**Rationale:** The name explains *why* these types are excluded: their text content serves as structural identifiers for heading-based scope selectors (`heading:X`). This is domain-specific and self-documenting. If another node type later becomes a scope anchor (hypothetically), the name still fits.

### Decision 3: Apply exclusion via `path_has_ancestor_type` negation

**Choice:** In `discover_target_path()`, add a negation filter:
```
path_has_ancestor_type(path, node_path_index, EDITABLE_PROSE_TYPES)
  && !path_has_ancestor_type(path, node_path_index, SCOPE_ANCHOR_TYPES)
  && !in_table
```

**Rationale:** Reuses the existing `path_has_ancestor_type` function. The filter reads naturally: "has an editable prose ancestor AND does NOT have a scope anchor ancestor AND is not in a table." No new helper functions needed for the exclusion logic itself.

### Decision 4: Document-order sort via segment-aware comparison

**Choice:** Add a `document_order_sort()` function that sorts ADF paths by splitting on `/` and comparing each segment — numeric segments compared as numbers, non-numeric segments compared as strings.

**Rationale:** ADF paths like `/content/2/content/0` have a natural numeric interpretation. The sort key for each path is a sequence of segments where array indices are parsed as `usize`. This produces document reading order: `/content/2` before `/content/10`.

**Implementation approach:** A comparison function that splits two paths on `/`, zips the segments, and compares segment-by-segment. Numeric segments (parseable as `usize`) compare numerically. Non-numeric segments compare lexicographically. Shorter paths sort before longer paths when all shared segments are equal.

**Alternative considered:** Pre-computing a sort key (e.g., `Vec<enum { Num(usize), Str(String) }>`) for each path. This would be faster for repeated sorts of the same collection but adds allocation overhead. The current use cases sort small collections (typically < 50 paths), so inline comparison is adequate.

### Decision 5: Apply document-order sort to `discover_target_path` and `resolve_scope` only

**Choice:** Replace `.sort()` with document-order sort at two call sites in `atlassy-adf`:
1. `discover_target_path()` line 297 — candidate selection
2. `resolve_scope()` line 117 — `matched_paths` ordering

Leave `normalize_changed_paths()` in `atlassy-contracts` unchanged (string sort for dedup is correct there). Leave `sort_by` calls in the pipeline unchanged (they sort by struct fields, not raw paths for document-order selection).

**Rationale:** The document-order sort matters where path ordering determines which node is selected or where order is observable in output. `discover_target_path` uses sort order for `target_index` selection. `resolve_scope` produces `allowed_scope_paths` which appears in telemetry and is used for scope boundary checks. `normalize_changed_paths` only needs sort+dedup for uniqueness validation — document order adds no value there.

## Risks / Trade-offs

**[Risk: Breaking change for target_index callers]** Manifests using `target_index: 0` with auto-discovery will now get paragraph text instead of heading text. → Mitigated by the fact that all existing auto-discovery manifests (e.g., `kpi-revalidation-auto-discovery.example.json`) intend to edit body content, not headings. The heading-first behavior was a bug, not a feature.

**[Risk: Sort order change affects scope path telemetry]** `allowed_scope_paths` order in `resolve_scope` output will change for pages with 10+ top-level blocks. → Low impact. The paths are used for `is_within_allowed_scope` boundary checks (order-independent) and telemetry (cosmetic). No functional behavior depends on their sort order beyond `discover_target_path`.

**[Risk: Future node types as scope anchors]** If a new node type becomes a scope anchor, a developer must remember to add it to `SCOPE_ANCHOR_TYPES`. → Mitigated by the constant's name and doc comment explaining the purpose. Additionally, scope anchors are inherently tied to `resolve_scope` selector parsing — a new selector kind (e.g., `panel:X`) would naturally prompt consideration of whether to exclude that type from discovery.

**[Trade-off: Heading text becomes undiscoverable]** After this change, there is no way to auto-discover a heading text node. You must use explicit `target_path` to edit headings. → Acceptable. Heading text is rarely the intended edit target for AI-assisted prose updates. The auto-discovery feature is designed for body content.
