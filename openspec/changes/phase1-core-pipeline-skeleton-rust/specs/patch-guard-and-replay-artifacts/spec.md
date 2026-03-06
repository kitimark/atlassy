## ADDED Requirements

### Requirement: Path-targeted patch operations only
The patch state MUST generate path-targeted operations only and SHALL reject any candidate that implies whole-body replacement.

#### Scenario: Whole-body rewrite attempt
- **WHEN** candidate generation proposes replacing the entire scoped document body
- **THEN** patch returns a hard error and no publish attempt is allowed

### Requirement: Scope-safe changed path set
The merged changed path set MUST be unique, lexicographically sorted, and fully contained within `allowed_scope_paths`.

#### Scenario: Out-of-scope changed path
- **WHEN** any changed path is outside `allowed_scope_paths`
- **THEN** verify fails with `ERR_OUT_OF_SCOPE_MUTATION` and publish is blocked

### Requirement: Per-state replay artifacts
For every executed state, the runtime SHALL persist `state_input.json`, `state_output.json`, and `diagnostics.json` under `artifacts/<run_id>/<state>/` and SHALL write `artifacts/<run_id>/summary.json` at run end.

#### Scenario: Failed run still writes diagnostics
- **WHEN** a run fails at an intermediate state
- **THEN** artifacts for all executed states exist on disk and the run summary includes failure state and error codes
