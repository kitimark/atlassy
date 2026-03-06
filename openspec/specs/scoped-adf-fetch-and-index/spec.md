## Purpose

Define scoped ADF retrieval, fallback behavior, and deterministic node-path indexing for downstream patch and verification boundaries.

## Requirements

### Requirement: Scoped fetch by selector with bounded fallback
The fetch state SHALL retrieve only the ADF subtree required by provided scope selectors (for example heading or block identifier) and MUST avoid full-page retrieval unless scope resolution fails.

#### Scenario: Scope selector resolves successfully
- **WHEN** a valid scope selector matches content on the page
- **THEN** fetch returns the scoped subtree and sets `scope_resolution_failed` to `false`

#### Scenario: Scope selector does not resolve
- **WHEN** no provided scope selector can be resolved
- **THEN** fetch sets `scope_resolution_failed` to `true` and records an explicit fallback reason for full-page retrieval

### Requirement: Fetch output includes version and allowed scope paths
The fetch output MUST include the source `page_version` and an `allowed_scope_paths` list that defines the mutation boundary for downstream patch and verify checks.

#### Scenario: Fetch prepares mutation boundary
- **WHEN** fetch completes for a scoped request
- **THEN** the output contains `page_version` and a non-empty `allowed_scope_paths` list aligned to the retrieved scope

### Requirement: Node-path index generation
The fetch state SHALL produce a deterministic `node_path_index` keyed by JSON Pointer paths, with unique entries for each indexed node in scope.

#### Scenario: Duplicate path candidate detected
- **WHEN** index construction encounters duplicate JSON Pointer paths
- **THEN** fetch fails with an index integrity error and does not emit a partial index
