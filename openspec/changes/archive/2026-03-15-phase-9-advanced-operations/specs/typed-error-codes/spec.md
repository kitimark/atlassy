## MODIFIED Requirements

### Requirement: Error codes cover table topology and attribute editing failures
The `ErrorCode` enum MUST include variants for Phase 9 operation failures.

#### Scenario: TableRowInvalid error code
- **WHEN** a row insert/remove fails (mismatched cell count, out of bounds, last row removal)
- **THEN** the error code MUST be `ERR_TABLE_ROW_INVALID`

#### Scenario: TableColumnInvalid error code
- **WHEN** a column insert/remove fails (out of bounds, last column removal, inconsistent table)
- **THEN** the error code MUST be `ERR_TABLE_COLUMN_INVALID`

#### Scenario: AttrUpdateBlocked error code
- **WHEN** UpdateAttrs targets a non-attr-editable node type or unresolvable path
- **THEN** the error code MUST be `ERR_ATTR_UPDATE_BLOCKED`

#### Scenario: AttrSchemaViolation error code
- **WHEN** UpdateAttrs provides disallowed attr keys for the target node type
- **THEN** the error code MUST be `ERR_ATTR_SCHEMA_VIOLATION`

#### Scenario: New error codes in ALL constant and tests
- **WHEN** `ErrorCode::ALL` is checked
- **THEN** it MUST include all 4 new variants
