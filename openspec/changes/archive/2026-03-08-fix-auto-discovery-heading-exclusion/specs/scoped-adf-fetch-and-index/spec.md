## ADDED Requirements

### Requirement: Scope anchor types constant

The `atlassy-adf` crate SHALL expose a public constant `SCOPE_ANCHOR_TYPES` containing node types whose text content serves as structural identifiers for scope selectors. The initial value SHALL be `["heading"]`. This constant SHALL be used as an exclusion filter in auto-discovery: text nodes under a `SCOPE_ANCHOR_TYPES` ancestor are editable via explicit `target_path` but MUST NOT be selected by `discover_target_path()`.

#### Scenario: Constant is accessible from other crates

- **WHEN** `atlassy-pipeline` or any downstream crate needs to check if a node type is a scope anchor
- **THEN** it references `SCOPE_ANCHOR_TYPES` from `atlassy-adf` instead of hardcoding `"heading"`

#### Scenario: Constant coexists with EDITABLE_PROSE_TYPES

- **WHEN** both constants are defined in `atlassy-adf`
- **THEN** `SCOPE_ANCHOR_TYPES` is a strict subset of `EDITABLE_PROSE_TYPES` (every scope anchor type is also an editable prose type)
- **AND** the discoverable prose set is logically `EDITABLE_PROSE_TYPES` minus `SCOPE_ANCHOR_TYPES`

#### Scenario: Heading type appears in both constants

- **WHEN** a node has type `"heading"`
- **THEN** it is in both `EDITABLE_PROSE_TYPES` (editable = true) and `SCOPE_ANCHOR_TYPES` (auto-discoverable = false)

### Requirement: Document-order sort function

The `atlassy-adf` crate SHALL expose a public function `document_order_sort()` that sorts a mutable slice of ADF path strings in document reading order. The sort SHALL split each path on `/`, compare segments pairwise: segments parseable as `usize` SHALL be compared numerically, all other segments SHALL be compared lexicographically. Shorter paths SHALL sort before longer paths when all shared segments are equal.

#### Scenario: Numeric segments compared numerically

- **WHEN** sorting paths `/content/2/content/0` and `/content/10/content/0`
- **THEN** `/content/2/content/0` sorts before `/content/10/content/0`

#### Scenario: Non-numeric segments compared lexicographically

- **WHEN** sorting paths `/attrs/level` and `/content/0`
- **THEN** `/attrs/level` sorts before `/content/0` (lexicographic: `"attrs"` < `"content"`)

#### Scenario: Shorter path sorts before longer path with shared prefix

- **WHEN** sorting paths `/content/0` and `/content/0/content/0`
- **THEN** `/content/0` sorts before `/content/0/content/0`

#### Scenario: Empty segments from leading slash are handled

- **WHEN** a path like `/content/0` is split on `/`
- **THEN** the leading empty segment is handled consistently across all paths and does not affect sort correctness

## MODIFIED Requirements

### Requirement: Scoped fetch by selector with bounded fallback

The fetch state SHALL resolve provided scope selectors (heading or block identifier) to determine `allowed_scope_paths` and MUST return the full page ADF as `scoped_adf`. Heading selectors SHALL use exact text equality to match heading content. When a heading selector matches, `allowed_scope_paths` SHALL include the heading path and all subsequent sibling content paths until the next heading at equal or higher level (or end of content array). The fetch state MUST avoid marking the request as failed unless scope resolution itself errors. `allowed_scope_paths` SHALL be sorted in document order using `document_order_sort()`.

#### Scenario: Scope selector resolves successfully
- **WHEN** a valid scope selector matches content on the page
- **THEN** fetch returns the full page ADF as `scoped_adf` and sets `scope_resolution_failed` to `false`
- **AND** `allowed_scope_paths` contains the heading path plus all section body paths, sorted in document order

#### Scenario: Heading section includes trailing content
- **WHEN** a heading selector matches a heading at `/content/N` followed by non-heading siblings at `/content/N+1` through `/content/M`
- **THEN** `allowed_scope_paths` contains paths `/content/N` through `/content/M` in document order

#### Scenario: Heading section stops at same-or-higher-level heading
- **WHEN** a heading selector matches a level-2 heading followed by a level-3 heading and then a level-2 heading
- **THEN** `allowed_scope_paths` includes the matched heading, the level-3 heading, and all content between them, but excludes the subsequent level-2 heading and its content

#### Scenario: Heading at end of content array
- **WHEN** a heading selector matches the last node in the content array
- **THEN** `allowed_scope_paths` contains only that heading path

#### Scenario: Adjacent same-level headings
- **WHEN** a heading selector matches a heading immediately followed by another heading at the same level
- **THEN** `allowed_scope_paths` contains only the matched heading path

#### Scenario: Scope selector does not resolve
- **WHEN** no provided scope selector can be resolved
- **THEN** fetch sets `scope_resolution_failed` to `true` and records an explicit fallback reason for full-page retrieval

#### Scenario: Heading selector uses exact text match
- **WHEN** a heading selector `heading:X` is provided
- **THEN** the selector matches only headings whose collected text content is exactly equal to `X`
- **AND** headings whose text contains `X` as a substring but is not equal to `X` SHALL NOT match

#### Scenario: Heading selector does not match substring
- **WHEN** a heading selector `heading:View` is provided and the document contains a heading titled "Overview"
- **THEN** the heading "Overview" SHALL NOT match
- **AND** scope resolution falls back to full-page scope with a fallback reason

#### Scenario: Duplicate headings match all instances
- **WHEN** a heading selector matches multiple headings with identical text at the same level
- **THEN** `allowed_scope_paths` contains the union of all matched heading sections, deduplicated and sorted in document order

#### Scenario: Multiple selectors produce union of section paths
- **WHEN** multiple heading selectors each match different headings
- **THEN** `allowed_scope_paths` contains the union of all expanded section paths, deduplicated and sorted in document order

#### Scenario: Document order for double-digit content indices in scope paths
- **WHEN** a scope resolution produces paths including `/content/2` and `/content/10`
- **THEN** `allowed_scope_paths` lists `/content/2` before `/content/10` (numeric comparison), not after (string comparison)
