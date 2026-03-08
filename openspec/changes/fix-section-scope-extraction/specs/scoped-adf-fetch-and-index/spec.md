## MODIFIED Requirements

### Requirement: Scoped fetch by selector with bounded fallback
The fetch state SHALL resolve provided scope selectors (heading or block identifier) to determine `allowed_scope_paths` and MUST return the full page ADF as `scoped_adf`. When a heading selector matches, `allowed_scope_paths` SHALL include the heading path and all subsequent sibling content paths until the next heading at equal or higher level (or end of content array). The fetch state MUST avoid marking the request as failed unless scope resolution itself errors.

#### Scenario: Scope selector resolves successfully
- **WHEN** a valid scope selector matches content on the page
- **THEN** fetch returns the full page ADF as `scoped_adf` and sets `scope_resolution_failed` to `false`
- **AND** `allowed_scope_paths` contains the heading path plus all section body paths

#### Scenario: Heading section includes trailing content
- **WHEN** a heading selector matches a heading at `/content/N` followed by non-heading siblings at `/content/N+1` through `/content/M`
- **THEN** `allowed_scope_paths` contains paths `/content/N` through `/content/M`

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

## ADDED Requirements

### Requirement: Section expansion for heading selectors
The scope resolver SHALL expand heading selector matches to include the full heading section using heading-level-aware sibling walking. Section expansion MUST only operate on headings that are direct children of the document's top-level content array.

#### Scenario: Non-top-level heading falls back to full page
- **WHEN** a heading selector matches a heading nested inside a container node (panel, layout, expand, or other non-document parent)
- **THEN** scope resolution falls back to full-page scope with `fallback_reason` set to `"nested_heading_scope_unsupported"`

#### Scenario: Multiple selectors produce union of section paths
- **WHEN** multiple heading selectors each match different headings
- **THEN** `allowed_scope_paths` contains the union of all expanded section paths, deduplicated and sorted

#### Scenario: Heading level defaults to 6 when missing
- **WHEN** a heading node lacks an `attrs.level` attribute
- **THEN** section expansion treats it as level 6 and stops at the next heading of any level
