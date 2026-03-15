## Purpose

Define v1 guardrails that block forbidden table topology and attribute mutations.

## Requirements

### Requirement: Table shape guard is op-type-aware for declared operations
The `check_table_shape_integrity` function MUST allow table structural changes that are declared via BlockOp row/column operations, while continuing to block undeclared table mutations.

#### Scenario: Declared row add passes table shape guard
- **WHEN** the operation manifest includes a `BlockOp::InsertRow` and the resulting operations modify table structure
- **THEN** the table shape guard MUST allow the changes

#### Scenario: Declared column remove passes table shape guard
- **WHEN** the operation manifest includes a `BlockOp::RemoveColumn` and the resulting operations modify table structure
- **THEN** the table shape guard MUST allow the changes

#### Scenario: Undeclared table structural change remains blocked
- **WHEN** a table structural change is detected that does not correspond to a declared row/column BlockOp
- **THEN** the table shape guard MUST fail with `ERR_TABLE_SHAPE_CHANGE`

#### Scenario: Cell merge/split remains blocked
- **WHEN** a table mutation corresponds to merge/split semantics
- **THEN** the table shape guard MUST fail with `ERR_TABLE_SHAPE_CHANGE` (Mode 3 not in scope)

### Requirement: Table attribute changes are rejected in v1
The table route MUST reject candidate operations that modify table-level or structural attributes in v1 scope.

#### Scenario: Table attribute mutation requested
- **WHEN** an edit request implies updating table layout or table-level attrs
- **THEN** the route rejects the request with `ERR_TABLE_SHAPE_CHANGE`

### Requirement: Shape guard checks are enforced before publish
Verifier integration SHALL treat detected table shape or attribute drift as hard failure and block publish. The check MUST operate on paths extracted from the operation list and MUST be positioned in the verify chain before `check_operation_legality` and `check_structural_validity`.

#### Scenario: Drift detected at verify
- **WHEN** candidate page ADF contains unauthorized table topology or attribute changes
- **THEN** verify fails with `ERR_TABLE_SHAPE_CHANGE`
- **AND** publish is blocked

#### Scenario: Table shape check receives paths from operations
- **WHEN** the verify stage runs with `Vec<Operation>` in the input
- **THEN** `check_table_shape_integrity` MUST extract paths from the operations and check them
- **AND** the behavior MUST be identical to the previous path-based check for Replace-only runs

### Requirement: Verify check functions are independently extracted
The verify stage MUST extract its checks into focused functions: `check_forced_fail()`, `check_table_shape_integrity()`, and `check_scope_containment()`. Each function SHALL accept the relevant inputs and return `Option<VerifyResult>` - `Some(Fail)` to halt with a specific error, or `None` to continue.

#### Scenario: Forced fail check extracted
- **WHEN** `request.force_verify_fail` is `true`
- **THEN** `check_forced_fail()` MUST return `Some(VerifyResult::Fail)` with `ERR_SCHEMA_INVALID` diagnostic

#### Scenario: Scope containment check extracted
- **WHEN** a changed path falls outside `allowed_scope_paths`
- **THEN** `check_scope_containment()` MUST return `Some(VerifyResult::Fail)` with `ERR_OUT_OF_SCOPE_MUTATION` diagnostic

#### Scenario: All checks pass returns None
- **WHEN** no verification failures are detected
- **THEN** all three check functions MUST return `None`
- **AND** the verify stage MUST return `VerifyResult::Pass`

#### Scenario: Check order matches previous behavior
- **WHEN** the verify stage executes
- **THEN** checks MUST be called in order: `check_forced_fail` then `check_table_shape_integrity` then `check_scope_containment`
- **AND** the first check returning `Some(Fail)` MUST halt further checks (same as previous if/else if/else behavior)

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
