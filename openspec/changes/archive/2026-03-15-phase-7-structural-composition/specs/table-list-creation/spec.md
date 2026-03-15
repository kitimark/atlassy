## ADDED Requirements

### Requirement: InsertTable creates a new table with specified dimensions
The system SHALL support `BlockOp::InsertTable` which constructs a valid table ADF node using builders and produces a single `Operation::Insert` command.

#### Scenario: Insert table with header row
- **WHEN** `BlockOp::InsertTable { parent_path: "/content", index: 3, rows: 3, cols: 2, header_row: true }` is processed
- **THEN** `adf_block_ops` MUST produce 1 Operation::Insert containing a table with 1 header row (tableHeader cells) and 2 data rows (tableCell cells)

#### Scenario: Insert table without header row
- **WHEN** `BlockOp::InsertTable { parent_path: "/content", index: 1, rows: 2, cols: 3, header_row: false }` is processed
- **THEN** `adf_block_ops` MUST produce 1 Operation::Insert containing a table with 2 data rows only

#### Scenario: Insert table with zero rows or columns rejected
- **WHEN** `BlockOp::InsertTable` has `rows: 0` or `cols: 0`
- **THEN** `adf_block_ops` MUST fail with an appropriate error

#### Scenario: Insert table respects scope
- **WHEN** `BlockOp::InsertTable` has `parent_path` outside `allowed_scope_paths`
- **THEN** `adf_block_ops` MUST fail with `ERR_OUT_OF_SCOPE_MUTATION`

#### Scenario: Inserted table passes structural validity
- **WHEN** a table from InsertTable is applied to the document
- **THEN** `check_structural_validity` MUST pass on the resulting ADF

### Requirement: InsertList creates a new list with specified items
The system SHALL support `BlockOp::InsertList` which constructs a valid list ADF node using builders and produces a single `Operation::Insert` command.

#### Scenario: Insert unordered list
- **WHEN** `BlockOp::InsertList { parent_path: "/content", index: 2, ordered: false, items: vec!["Item A", "Item B", "Item C"] }` is processed
- **THEN** `adf_block_ops` MUST produce 1 Operation::Insert containing a bulletList with 3 listItem children

#### Scenario: Insert ordered list
- **WHEN** `BlockOp::InsertList { ... ordered: true, items: vec!["First", "Second"] }` is processed
- **THEN** `adf_block_ops` MUST produce 1 Operation::Insert containing an orderedList

#### Scenario: Insert list with empty items rejected
- **WHEN** `BlockOp::InsertList` has empty `items` vector
- **THEN** `adf_block_ops` MUST fail with an appropriate error

#### Scenario: Insert list respects scope
- **WHEN** `BlockOp::InsertList` has `parent_path` outside `allowed_scope_paths`
- **THEN** `adf_block_ops` MUST fail with `ERR_OUT_OF_SCOPE_MUTATION`
