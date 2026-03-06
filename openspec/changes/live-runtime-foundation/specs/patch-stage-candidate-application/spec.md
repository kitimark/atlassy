## ADDED Requirements

### Requirement: Patch stage applies patch operations to candidate ADF
The patch stage MUST apply generated `patch_ops` into `candidate_page_adf` before `verify` and `publish` are executed.

#### Scenario: Prose patch mutates candidate payload
- **WHEN** patch operations target an allowed prose path
- **THEN** `candidate_page_adf` reflects the patch result before `verify`

#### Scenario: Table-cell patch mutates candidate payload
- **WHEN** patch operations target an allowed table-cell path
- **THEN** `candidate_page_adf` reflects the table-cell update before `publish`

### Requirement: Patch application preserves untouched paths
Patch application SHALL only mutate paths targeted by valid `patch_ops` and MUST preserve unchanged paths.

#### Scenario: Unchanged paths remain unchanged
- **WHEN** a run applies patch operations to a subset of paths
- **THEN** all non-targeted paths remain byte-equivalent in candidate output

### Requirement: Patch evidence is replayable
The system MUST persist patch-stage evidence sufficient to verify that candidate payload mutation matches `patch_ops`.

#### Scenario: Replay confirms patch application
- **WHEN** patch replay artifacts are inspected
- **THEN** `state_input`, `state_output`, and diagnostics prove candidate payload changes are explained by `patch_ops`
