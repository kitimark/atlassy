## MODIFIED Requirements

### Requirement: Scoped fetch by selector with bounded fallback
The fetch state SHALL resolve provided scope selectors (heading or block identifier) to determine `allowed_scope_paths` and MUST return the full page ADF as `scoped_adf`. Heading selectors SHALL use exact text equality to match heading content. When a heading selector matches, `allowed_scope_paths` SHALL include the heading path and all subsequent sibling content paths until the next heading at equal or higher level (or end of content array). The fetch state MUST avoid marking the request as failed unless scope resolution itself errors.

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
- **THEN** `allowed_scope_paths` contains the union of all matched heading sections, deduplicated and sorted

## ADDED Requirements

### Requirement: Block selector matches by attribute ID
The scope resolver SHALL support `block:` selectors that match nodes by `attrs.id` or `attrs.localId` attribute values. Block selectors SHALL use exact string equality for matching.

#### Scenario: Block selector matches attrs.id
- **WHEN** a `block:X` selector is provided and the document contains a node with `attrs.id` equal to `X`
- **THEN** the node path is included in `allowed_scope_paths`

#### Scenario: Block selector matches attrs.localId
- **WHEN** a `block:X` selector is provided and the document contains a node with `attrs.localId` equal to `X`
- **THEN** the node path is included in `allowed_scope_paths`

#### Scenario: Block selector does not match
- **WHEN** a `block:X` selector is provided and no node in the document has `attrs.id` or `attrs.localId` equal to `X`
- **THEN** scope resolution falls back to full-page scope with a fallback reason

### Requirement: Scope utility function is publicly accessible
The `atlassy-adf` crate SHALL expose `is_within_allowed_scope` as a public function so that downstream crates can perform scope filtering on node paths against `allowed_scope_paths`.

#### Scenario: Pipeline crate uses scope check
- **WHEN** `atlassy-pipeline` needs to filter nodes by scope before processing
- **THEN** it references `is_within_allowed_scope` from `atlassy-adf` instead of reimplementing the logic
