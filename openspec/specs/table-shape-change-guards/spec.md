## Purpose

Define v1 guardrails that block forbidden table topology and attribute mutations.

## Requirements

### Requirement: Forbidden table shape operations are rejected
The table route MUST reject candidate operations that change table topology, including row/column add-remove and merge-split behavior.

#### Scenario: Row add operation requested
- **WHEN** an edit request implies insertion of a table row
- **THEN** the route rejects the request with `ERR_TABLE_SHAPE_CHANGE`
- **AND** no publish attempt is allowed

### Requirement: Table attribute changes are rejected in v1
The table route MUST reject candidate operations that modify table-level or structural attributes in v1 scope.

#### Scenario: Table attribute mutation requested
- **WHEN** an edit request implies updating table layout or table-level attrs
- **THEN** the route rejects the request with `ERR_TABLE_SHAPE_CHANGE`

### Requirement: Shape guard checks are enforced before publish
Verifier integration SHALL treat detected table shape or attribute drift as hard failure and block publish.

#### Scenario: Drift detected at verify
- **WHEN** candidate page ADF contains unauthorized table topology or attribute changes
- **THEN** verify fails with `ERR_TABLE_SHAPE_CHANGE`
- **AND** publish is blocked
