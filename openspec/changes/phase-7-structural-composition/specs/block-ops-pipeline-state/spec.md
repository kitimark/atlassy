## MODIFIED Requirements

### Requirement: AdfBlockOps translates all BlockOp variants to Operations
The `AdfBlockOps` state MUST handle all 6 BlockOp variants (Insert, Remove, InsertSection, RemoveSection, InsertTable, InsertList) by dispatching to per-variant translate functions.

#### Scenario: InsertSection translated to multiple Operation::Insert
- **WHEN** `RunRequest.block_ops` contains `BlockOp::InsertSection`
- **THEN** `AdfBlockOps` MUST produce N `Operation::Insert` commands (1 heading + N body blocks)

#### Scenario: RemoveSection translated to multiple Operation::Remove
- **WHEN** `RunRequest.block_ops` contains `BlockOp::RemoveSection`
- **THEN** `AdfBlockOps` MUST use `find_section_range()` to detect the section boundary
- **AND** produce N `Operation::Remove` commands in reverse document order

#### Scenario: InsertTable translated to single Operation::Insert
- **WHEN** `RunRequest.block_ops` contains `BlockOp::InsertTable`
- **THEN** `AdfBlockOps` MUST use `build_table()` to construct the ADF and produce 1 `Operation::Insert`

#### Scenario: InsertList translated to single Operation::Insert
- **WHEN** `RunRequest.block_ops` contains `BlockOp::InsertList`
- **THEN** `AdfBlockOps` MUST use `build_list()` to construct the ADF and produce 1 `Operation::Insert`

### Requirement: AdfBlockOps has access to scoped ADF for section operations
The `run_adf_block_ops_state` function MUST receive `scoped_adf` from `FetchOutput` so that `translate_remove_section` can perform section boundary detection.

#### Scenario: RemoveSection accesses page ADF
- **WHEN** `BlockOp::RemoveSection` is processed
- **THEN** the translate function MUST receive the fetched `scoped_adf` to call `find_section_range()`

### Requirement: AdfBlockOps validates inserted block types via type policy
The state MUST use `is_insertable_type()` from the type policy module instead of directly checking `EDITABLE_PROSE_TYPES`.

#### Scenario: Table insertion passes type validation
- **WHEN** `BlockOp::InsertTable` is processed
- **THEN** `is_insertable_type("table")` MUST return `true` and the operation MUST proceed
