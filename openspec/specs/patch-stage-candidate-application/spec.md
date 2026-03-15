## Purpose

Define patch-stage mutation guarantees so verified and published payloads reflect generated patch operations.

## Requirements

### Requirement: Patch stage applies patch operations to candidate ADF
The patch stage MUST build `Operation::Replace` instances directly from prose and table change candidates, validate them with `validate_operations()`, sort them with `sort_operations()`, and apply them with `apply_operations()` into `candidate_page_adf` before `verify` and `publish` are executed.

#### Scenario: Prose patch mutates candidate payload
- **WHEN** patch operations target an allowed prose path
- **THEN** `candidate_page_adf` reflects the patch result before `verify`

#### Scenario: Table-cell patch mutates candidate payload
- **WHEN** patch operations target an allowed table-cell path
- **THEN** `candidate_page_adf` reflects the table-cell update before `publish`

#### Scenario: Operations are built directly without intermediate types
- **WHEN** the patch stage processes prose and table change candidates
- **THEN** it MUST construct `Operation::Replace` directly from candidate path and value
- **AND** it MUST NOT use `PatchCandidate`, `build_patch_ops()`, or `PatchOperation` (removed types)

### Requirement: Patch application preserves untouched paths
Patch application SHALL only mutate paths targeted by valid operations and MUST preserve unchanged paths.

#### Scenario: Unchanged paths remain unchanged
- **WHEN** a run applies operations to a subset of paths
- **THEN** all non-targeted paths remain byte-equivalent in candidate output

### Requirement: Patch evidence is replayable
The system MUST persist patch-stage evidence sufficient to verify that candidate payload mutation matches the applied operations.

#### Scenario: Replay confirms patch application
- **WHEN** patch replay artifacts are inspected
- **THEN** `state_input`, `state_output`, and diagnostics prove candidate payload changes are explained by the `Operation` list in `patch_ops`

### Requirement: validate_operations checks scope before application
The `validate_operations()` function MUST verify that every `Operation::Replace` path is within `allowed_scope_paths` and is not a whole-body rewrite (path is not `/` or empty).

#### Scenario: Out-of-scope operation rejected
- **WHEN** an `Operation::Replace` targets a path outside `allowed_scope_paths`
- **THEN** `validate_operations()` MUST return `AdfError::OutOfScope`

#### Scenario: Whole-body rewrite rejected
- **WHEN** an `Operation::Replace` has path `/` or empty string
- **THEN** `validate_operations()` MUST return `AdfError::WholeBodyRewriteDisallowed`

### Requirement: apply_operations matches on Operation variant
The `apply_operations()` function MUST use `match` on the `Operation` enum to dispatch per-variant logic. In Phase 5.5, only the `Replace` arm exists.

#### Scenario: Replace operation applies via pointer_mut
- **WHEN** `apply_operations()` processes an `Operation::Replace { path, value }`
- **THEN** it MUST resolve the path via `pointer_mut` and replace the target value
- **AND** the result MUST be identical to the previous `apply_patch_ops()` behavior
