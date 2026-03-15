## Purpose

Define patch-stage mutation guarantees so verified and published payloads reflect generated patch operations.

## Requirements

### Requirement: Patch stage applies patch operations to candidate ADF
The patch stage MUST receive `Vec<Operation>` from `MergeCandidatesOutput.operations`, validate them with `validate_operations()`, sort them with `sort_operations()`, and apply them with `apply_operations()`. The patch stage SHALL NOT build operations from prose or table candidates - that responsibility belongs to merge.

#### Scenario: Prose patch mutates candidate payload
- **WHEN** operations include a Replace targeting an allowed prose path
- **THEN** `candidate_page_adf` reflects the replace result before `verify`

#### Scenario: Insert operation mutates candidate payload
- **WHEN** operations include an Insert targeting an allowed parent path
- **THEN** `candidate_page_adf` contains the new block at the specified position before `verify`

#### Scenario: Remove operation mutates candidate payload
- **WHEN** operations include a Remove targeting an allowed block path
- **THEN** `candidate_page_adf` no longer contains the block at that path before `verify`

#### Scenario: Patch state does not receive md_edit or table_edit
- **WHEN** the patch state function is called
- **THEN** it SHALL NOT accept `MdAssistEditOutput` or `AdfTableEditOutput` as parameters
- **AND** it SHALL only receive `FetchOutput` and `MergeCandidatesOutput` (plus standard params)

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
The `apply_operations()` function MUST match on all `Operation` variants: Replace (pointer_mut + swap), Insert (parent array + Vec::insert), Remove (parent array + Vec::remove).

#### Scenario: Replace operation applies via pointer_mut
- **WHEN** `apply_operations()` processes an `Operation::Replace`
- **THEN** it MUST resolve the path via `pointer_mut` and replace the target value

#### Scenario: Insert operation applies via parent array insert
- **WHEN** `apply_operations()` processes an `Operation::Insert`
- **THEN** it MUST navigate to parent_path, get the array, and call insert at the specified index

#### Scenario: Remove operation applies via parent array remove
- **WHEN** `apply_operations()` processes an `Operation::Remove`
- **THEN** it MUST parse the target path into (parent_path, index), navigate to parent, and remove the element at that index
