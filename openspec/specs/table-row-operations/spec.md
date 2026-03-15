## Purpose

Define table row insert/remove behavior, validation constraints, and translation to patch operations.

## Requirements

### Requirement: InsertRow adds a new row to an existing table
The system SHALL support `BlockOp::InsertRow { table_path, index, cells }` which composes into a single `Operation::Insert` that adds a tableRow at the specified index within the table's content array.

#### Scenario: Insert row with matching cell count
- **WHEN** `BlockOp::InsertRow` targets a table with 3 columns and provides 3 cell values
- **THEN** a single `Operation::Insert` MUST be produced containing a tableRow with 3 tableCell children

#### Scenario: Insert row at end of table (append)
- **WHEN** `BlockOp::InsertRow` targets index equal to the table's row count
- **THEN** the row MUST be appended as the last row

#### Scenario: Insert row with mismatched cell count
- **WHEN** `BlockOp::InsertRow` provides a different number of cells than the table's column count
- **THEN** it MUST fail with `ERR_TABLE_ROW_INVALID`

#### Scenario: Insert row at out-of-bounds index
- **WHEN** `BlockOp::InsertRow` targets an index greater than the table's row count
- **THEN** it MUST fail with `ERR_TABLE_ROW_INVALID`

### Requirement: RemoveRow removes an existing row from a table
The system SHALL support `BlockOp::RemoveRow { table_path, index }` which composes into a single `Operation::Remove` that deletes the tableRow at the specified index.

#### Scenario: Remove data row
- **WHEN** `BlockOp::RemoveRow` targets a data row (non-header)
- **THEN** a single `Operation::Remove` MUST be produced targeting the tableRow path

#### Scenario: Remove last remaining row
- **WHEN** `BlockOp::RemoveRow` would leave the table with zero rows
- **THEN** it MUST fail with `ERR_TABLE_ROW_INVALID` (tables must have at least one row)

#### Scenario: Remove row at out-of-bounds index
- **WHEN** `BlockOp::RemoveRow` targets an index beyond the table's row count
- **THEN** it MUST fail with `ERR_TABLE_ROW_INVALID`
