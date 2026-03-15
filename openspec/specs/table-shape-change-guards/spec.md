## Purpose

Define v1 guardrails that block forbidden table topology and attribute mutations.

## Requirements

### Requirement: Forbidden table shape operations are rejected
The table route MUST reject candidate operations that change table topology, including row/column add-remove and merge-split behavior.

#### Scenario: Row add operation requested
- **WHEN** an edit request implies insertion of a table row
- **THEN** the route rejects the request with `ERR_TABLE_SHAPE_CHANGE`
- **AND** no publish attempt is allowed

### Requirement: Table attribute changes are rejected in v1
The table route MUST reject candidate operations that modify table-level or structural attributes in v1 scope.

#### Scenario: Table attribute mutation requested
- **WHEN** an edit request implies updating table layout or table-level attrs
- **THEN** the route rejects the request with `ERR_TABLE_SHAPE_CHANGE`

### Requirement: Shape guard checks are enforced before publish
Verifier integration SHALL treat detected table shape or attribute drift as hard failure and block publish. The check MUST be implemented as an extracted function `check_table_shape_integrity()` that returns `Option<VerifyResult>`, returning `Some(Fail)` with `ERR_TABLE_SHAPE_CHANGE` diagnostic when a violation is found, or `None` to continue to the next check.

#### Scenario: Drift detected at verify
- **WHEN** candidate page ADF contains unauthorized table topology or attribute changes
- **THEN** verify fails with `ERR_TABLE_SHAPE_CHANGE`
- **AND** publish is blocked

#### Scenario: Table shape check is an independent function
- **WHEN** the verify stage runs
- **THEN** table shape integrity MUST be checked by calling `check_table_shape_integrity()` as a separate function
- **AND** the function MUST be callable and testable independently of the full verify pipeline

### Requirement: Verify check functions are independently extracted
The verify stage MUST extract its checks into focused functions: `check_forced_fail()`, `check_table_shape_integrity()`, and `check_scope_containment()`. Each function SHALL accept the relevant inputs and return `Option<VerifyResult>` - `Some(Fail)` to halt with a specific error, or `None` to continue.

#### Scenario: Forced fail check extracted
- **WHEN** `request.force_verify_fail` is `true`
- **THEN** `check_forced_fail()` MUST return `Some(VerifyResult::Fail)` with `ERR_SCHEMA_INVALID` diagnostic

#### Scenario: Scope containment check extracted
- **WHEN** a changed path falls outside `allowed_scope_paths`
- **THEN** `check_scope_containment()` MUST return `Some(VerifyResult::Fail)` with `ERR_OUT_OF_SCOPE_MUTATION` diagnostic

#### Scenario: All checks pass returns None
- **WHEN** no verification failures are detected
- **THEN** all three check functions MUST return `None`
- **AND** the verify stage MUST return `VerifyResult::Pass`

#### Scenario: Check order matches previous behavior
- **WHEN** the verify stage executes
- **THEN** checks MUST be called in order: `check_forced_fail` then `check_table_shape_integrity` then `check_scope_containment`
- **AND** the first check returning `Some(Fail)` MUST halt further checks (same as previous if/else if/else behavior)
