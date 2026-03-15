## Purpose

Define Phase 5.5 block-ops pipeline scaffolding, including no-op execution and state ordering.

## Requirements

### Requirement: AdfBlockOps translates all BlockOp variants to Operations
The `AdfBlockOps` state MUST handle all 11 BlockOp variants (Insert, Remove, InsertSection, RemoveSection, InsertTable, InsertList, InsertRow, RemoveRow, InsertColumn, RemoveColumn, UpdateAttrs) by dispatching to per-variant translate functions.

#### Scenario: BlockOp::Insert translated to Operation::Insert
- **WHEN** `RunRequest.block_ops` contains `BlockOp::Insert { parent_path, index, block }`
- **THEN** `AdfBlockOps` output MUST include `Operation::Insert { parent_path, index, block }`

#### Scenario: BlockOp::Remove translated to Operation::Remove
- **WHEN** `RunRequest.block_ops` contains `BlockOp::Remove { target_path }`
- **THEN** `AdfBlockOps` output MUST include `Operation::Remove { target_path }`

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

#### Scenario: Block type not in editable_prose scope
- **WHEN** a `BlockOp::Insert` specifies a block with type not in `EDITABLE_PROSE_TYPES`
- **THEN** `AdfBlockOps` MUST fail with `ERR_INSERT_POSITION_INVALID`

#### Scenario: Insert parent path outside allowed scope
- **WHEN** a `BlockOp::Insert` has parent_path outside `allowed_scope_paths`
- **THEN** `AdfBlockOps` MUST fail with `ERR_OUT_OF_SCOPE_MUTATION`

#### Scenario: Remove target path outside allowed scope
- **WHEN** a `BlockOp::Remove` has target_path outside `allowed_scope_paths`
- **THEN** `AdfBlockOps` MUST fail with `ERR_OUT_OF_SCOPE_MUTATION`

#### Scenario: Empty block_ops produces empty operations
- **WHEN** `RunRequest.block_ops` is empty
- **THEN** `AdfBlockOps` output MUST have an empty `operations` list (backward compatible)

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

### Requirement: AdfBlockOps output is wired to merge
The orchestrator MUST pass `AdfBlockOps` output to the `MergeCandidates` state so that block operations are included in the unified operation list.

#### Scenario: AdfBlockOps operations flow to merge
- **WHEN** `AdfBlockOps` produces operations
- **THEN** those operations MUST be received by `MergeCandidates` and included in its output

### Requirement: RunRequest accepts block_ops field
`RunRequest` MUST include a `block_ops: Vec<BlockOp>` field. In Phase 5.5, this field MUST always be an empty vector. The pipeline MUST NOT process block_ops in Phase 5.5.

#### Scenario: RunRequest with empty block_ops
- **WHEN** a `RunRequest` is constructed with `block_ops: vec![]`
- **THEN** the pipeline MUST execute identically to the pre-refactoring behavior

#### Scenario: RunRequest block_ops defaults to empty
- **WHEN** a CLI command constructs a `RunRequest`
- **THEN** `block_ops` MUST be set to an empty vector

### Requirement: PipelineState enum includes AdfBlockOps
The `PipelineState` enum MUST include an `AdfBlockOps` variant. The `expected_next()` ordering MUST place `AdfBlockOps` after `AdfTableEdit` and before `MergeCandidates`. The `ORDER` constant MUST include `AdfBlockOps` at the correct position.

#### Scenario: State transition from AdfTableEdit to AdfBlockOps
- **WHEN** the orchestrator completes `AdfTableEdit`
- **THEN** `expected_next()` MUST return `AdfBlockOps` as the valid next state

#### Scenario: State transition from AdfBlockOps to MergeCandidates
- **WHEN** the orchestrator completes `AdfBlockOps`
- **THEN** `expected_next()` MUST return `MergeCandidates` as the valid next state
