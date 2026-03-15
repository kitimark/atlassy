## MODIFIED Requirements

### Requirement: AdfBlockOps translates all BlockOp variants including table and attr operations
The `AdfBlockOps` state MUST handle all 11 BlockOp variants by dispatching to per-variant translate functions.

#### Scenario: InsertRow translated
- **WHEN** `BlockOp::InsertRow` is processed
- **THEN** `translate_insert_row()` MUST produce a single `Operation::Insert` containing a valid tableRow

#### Scenario: RemoveRow translated
- **WHEN** `BlockOp::RemoveRow` is processed
- **THEN** `translate_remove_row()` MUST produce a single `Operation::Remove` targeting the tableRow

#### Scenario: InsertColumn translated to N operations
- **WHEN** `BlockOp::InsertColumn` is processed on a table with 3 rows
- **THEN** `translate_insert_column()` MUST produce 3 `Operation::Insert` commands (one per row)

#### Scenario: RemoveColumn translated to N operations
- **WHEN** `BlockOp::RemoveColumn` is processed on a table with 3 rows
- **THEN** `translate_remove_column()` MUST produce 3 `Operation::Remove` commands (one per row)

#### Scenario: UpdateAttrs translated 1:1
- **WHEN** `BlockOp::UpdateAttrs` is processed
- **THEN** `translate_update_attrs()` MUST produce a single `Operation::UpdateAttrs`
