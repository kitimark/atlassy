## Purpose

Define merge safety behavior for combining table and non-table route candidates in v1.

## Requirements

### Requirement: Merge enforces unique changed paths across routes
The `merge_candidates` state SHALL enforce uniqueness of changed paths across prose and table route candidates and MUST fail on duplicate path collisions.

#### Scenario: Duplicate changed path collision
- **WHEN** prose and table candidates include the same changed path
- **THEN** merge fails with a deterministic hard error
- **AND** downstream `patch`, `verify`, and `publish` are not executed

### Requirement: Cross-route boundary conflicts fail fast
Merge logic MUST reject cross-route candidate sets when table candidates overlap prohibited prose or locked-structural boundaries.

#### Scenario: Table candidate overlaps locked route
- **WHEN** a table candidate path intersects a locked-structural path boundary
- **THEN** merge fails fast and records conflict diagnostics

### Requirement: Conflict-safe merge output remains verifiable
When no conflicts are present, merged changed paths SHALL remain lexicographically sorted and fully compatible with verifier scope checks.

#### Scenario: Valid mixed-route merge
- **WHEN** prose and table candidates are disjoint and in scope
- **THEN** merge emits a unique sorted `changed_paths` set accepted by verify
