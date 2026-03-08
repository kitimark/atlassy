## MODIFIED Requirements

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
