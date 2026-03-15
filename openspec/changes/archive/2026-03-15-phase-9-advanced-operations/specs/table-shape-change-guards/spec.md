## MODIFIED Requirements

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
