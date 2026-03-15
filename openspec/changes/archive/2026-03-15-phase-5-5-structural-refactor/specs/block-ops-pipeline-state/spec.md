## ADDED Requirements

### Requirement: AdfBlockOps pipeline state exists as no-op
The pipeline MUST include an `AdfBlockOps` state positioned between `AdfTableEdit` and `MergeCandidates` in the execution order. In Phase 5.5, this state SHALL be a no-op pass-through that produces empty output without affecting the pipeline data flow.

#### Scenario: AdfBlockOps executes without error
- **WHEN** a pipeline run executes with any RunMode
- **THEN** the `AdfBlockOps` state MUST execute between `AdfTableEdit` and `MergeCandidates` without error
- **AND** its output MUST NOT affect the merge, patch, verify, or publish stages

#### Scenario: AdfBlockOps persists artifacts
- **WHEN** the `AdfBlockOps` state executes
- **THEN** it MUST persist state input and output artifacts to the artifact store following the same pattern as other pipeline states

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
