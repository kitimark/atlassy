## Purpose

Define table column insert/remove behavior, translation semantics, guardrails, and post-mutation consistency rules.

## Requirements

### Requirement: InsertColumn adds a cell to every row at the specified column index
The system SHALL support `BlockOp::InsertColumn { table_path, index }` which composes into N `Operation::Insert` commands - one per row - each inserting a tableCell (or tableHeader for header rows) at the specified column index.

#### Scenario: Insert column in table with header row
- **GIVEN** a table with 1 header row and 2 data rows, 3 columns
- **WHEN** `BlockOp::InsertColumn { table_path: "/content/0", index: 1 }` is processed
- **THEN** 3 `Operation::Insert` commands MUST be produced: tableHeader cell for header row, tableCell for each data row, all at column index 1

#### Scenario: Insert column at end (append)
- **WHEN** `InsertColumn` targets index equal to the current column count
- **THEN** cells MUST be appended to every row

#### Scenario: Insert column at out-of-bounds index
- **WHEN** `InsertColumn` targets index greater than the current column count
- **THEN** it MUST fail with `ERR_TABLE_COLUMN_INVALID`

### Requirement: RemoveColumn removes a cell from every row at the specified column index
The system SHALL support `BlockOp::RemoveColumn { table_path, index }` which composes into N `Operation::Remove` commands - one per row - each removing the cell at the specified column index.

#### Scenario: Remove column from table
- **GIVEN** a table with 3 rows, 4 columns
- **WHEN** `BlockOp::RemoveColumn { table_path: "/content/0", index: 2 }` is processed
- **THEN** 3 `Operation::Remove` commands MUST be produced, each targeting the cell at index 2 in their respective row

#### Scenario: Remove last remaining column
- **WHEN** `RemoveColumn` would leave rows with zero cells
- **THEN** it MUST fail with `ERR_TABLE_COLUMN_INVALID` (rows must have at least one cell)

#### Scenario: Remove column at out-of-bounds index
- **WHEN** `RemoveColumn` targets an index beyond the current column count
- **THEN** it MUST fail with `ERR_TABLE_COLUMN_INVALID`

### Requirement: Column operations maintain consistent column counts
After any column insert or remove, all rows in the table MUST have the same number of cells. The structural validity check MUST validate this.

#### Scenario: Post-column-insert consistency
- **WHEN** a column is inserted
- **THEN** every row MUST have exactly (original_cols + 1) cells after application
