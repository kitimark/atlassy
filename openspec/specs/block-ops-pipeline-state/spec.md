## Purpose

Define Phase 5.5 block-ops pipeline scaffolding, including no-op execution and state ordering.

## Requirements

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
