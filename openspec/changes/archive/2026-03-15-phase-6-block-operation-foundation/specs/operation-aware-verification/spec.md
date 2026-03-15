## ADDED Requirements

### Requirement: Verify checks operation legality per operation type
The verify stage MUST include a `check_operation_legality()` function that validates each operation based on its type.

#### Scenario: Replace operation with path in scope passes
- **WHEN** an `Operation::Replace` has its path within `allowed_scope_paths`
- **THEN** the operation legality check MUST pass for this operation

#### Scenario: Insert operation with valid parent, type, and index passes
- **WHEN** an `Operation::Insert` has parent_path within scope, block type in `EDITABLE_PROSE_TYPES`, and index within bounds
- **THEN** the operation legality check MUST pass

#### Scenario: Insert operation with disallowed block type fails
- **WHEN** an `Operation::Insert` has block type not in `EDITABLE_PROSE_TYPES`
- **THEN** the verify stage MUST fail with `ERR_INSERT_POSITION_INVALID`

#### Scenario: Remove operation targeting scope anchor fails
- **WHEN** an `Operation::Remove` targets a heading that matches a scope selector
- **THEN** the verify stage MUST fail with `ERR_REMOVE_ANCHOR_MISSING`

#### Scenario: Remove operation targeting valid path in scope passes
- **WHEN** an `Operation::Remove` has target_path within scope and targets a non-anchor editable_prose block
- **THEN** the operation legality check MUST pass

### Requirement: Verify chain includes structural validity after operation legality
The verify check chain MUST execute in order: `check_forced_fail` → `check_table_shape_integrity` → `check_operation_legality` → `check_scope_containment` → `check_structural_validity`. First failure halts the chain.

#### Scenario: Operation legality failure blocks structural validity check
- **WHEN** `check_operation_legality` finds an invalid operation
- **THEN** `check_structural_validity` MUST NOT execute
- **AND** verify MUST return Fail with the operation legality error

#### Scenario: All checks pass
- **WHEN** no check function finds a violation
- **THEN** verify MUST return Pass
