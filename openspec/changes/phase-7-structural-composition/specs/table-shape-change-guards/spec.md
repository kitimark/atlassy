## MODIFIED Requirements

### Requirement: Locked boundary check is extracted into a reusable function
The locked structural boundary check SHALL be extracted into a `check_locked_boundary(operation, locked_paths)` function that can be reused by Phase 8 (per-page) and extended by Phase 9 (table-specific rules).

#### Scenario: Replace operation blocked by locked boundary
- **WHEN** `check_locked_boundary` receives an `Operation::Replace` with path overlapping a locked structural node
- **THEN** it MUST return a route violation error

#### Scenario: Insert and Remove blocked at locked node level
- **WHEN** `check_locked_boundary` receives `Operation::Insert` or `Operation::Remove` with path directly overlapping a locked structural node
- **THEN** it MUST return a route violation error

#### Scenario: Function is callable from both merge and verify contexts
- **WHEN** `check_locked_boundary` is called
- **THEN** it MUST operate on a single operation and a list of locked paths
- **AND** it MUST NOT depend on pipeline state or merge/verify context

### Requirement: Verify uses type policy functions for operation legality
The `check_operation_legality` function MUST use `is_insertable_type()` and `is_removable_type()` from the type policy module instead of directly checking `EDITABLE_PROSE_TYPES`.

#### Scenario: Table insert passes operation legality
- **WHEN** verify checks an `Operation::Insert` with block type `"table"`
- **THEN** `is_insertable_type("table")` returns `true` and legality check MUST pass

#### Scenario: Panel insert fails operation legality
- **WHEN** verify checks an `Operation::Insert` with block type `"panel"`
- **THEN** `is_insertable_type("panel")` returns `false` and legality check MUST fail
