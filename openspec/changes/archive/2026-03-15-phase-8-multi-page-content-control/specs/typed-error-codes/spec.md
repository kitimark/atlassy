## MODIFIED Requirements

### Requirement: Error codes cover multi-page operation failures
The `ErrorCode` enum MUST include variants for multi-page orchestration failures.

#### Scenario: MultiPagePartialFailure error code
- **WHEN** at least one page in a multi-page operation fails
- **THEN** the error code MUST be `ERR_MULTI_PAGE_PARTIAL_FAILURE`

#### Scenario: RollbackConflict error code
- **WHEN** rollback is attempted but the page version changed due to a concurrent edit
- **THEN** the error code MUST be `ERR_ROLLBACK_CONFLICT`

#### Scenario: DependencyCycle error code
- **WHEN** the page dependency graph contains a cycle
- **THEN** the error code MUST be `ERR_DEPENDENCY_CYCLE`

#### Scenario: PageCreationFailed error code
- **WHEN** `create_page` fails for a `PageTarget` with a `create` config
- **THEN** the error code MUST be `ERR_PAGE_CREATION_FAILED`

#### Scenario: New error codes in ALL constant and tests
- **WHEN** `ErrorCode::ALL` is checked
- **THEN** it MUST include all 4 new variants
- **AND** `as_str()` and `Display` tests MUST cover all new variants
