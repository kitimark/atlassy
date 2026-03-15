## MODIFIED Requirements

### Requirement: AdfBlockOps pipeline state translates BlockOp to Operation
The `AdfBlockOps` state MUST translate each `BlockOp` from `RunRequest.block_ops` into a corresponding `Operation` variant. `BlockOp::Insert` MUST produce `Operation::Insert`. `BlockOp::Remove` MUST produce `Operation::Remove`. The state MUST validate each operation before producing it.

#### Scenario: BlockOp::Insert translated to Operation::Insert
- **WHEN** `RunRequest.block_ops` contains `BlockOp::Insert { parent_path, index, block }`
- **THEN** `AdfBlockOps` output MUST include `Operation::Insert { parent_path, index, block }`

#### Scenario: BlockOp::Remove translated to Operation::Remove
- **WHEN** `RunRequest.block_ops` contains `BlockOp::Remove { target_path }`
- **THEN** `AdfBlockOps` output MUST include `Operation::Remove { target_path }`

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

### Requirement: AdfBlockOps output is wired to merge
The orchestrator MUST pass `AdfBlockOps` output to the `MergeCandidates` state so that block operations are included in the unified operation list.

#### Scenario: AdfBlockOps operations flow to merge
- **WHEN** `AdfBlockOps` produces operations
- **THEN** those operations MUST be received by `MergeCandidates` and included in its output
