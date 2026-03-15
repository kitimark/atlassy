## Purpose

Define a typed, closed-set error code taxonomy for pipeline hard errors while preserving stable serialized `ERR_*` outputs.

## Requirements

### Requirement: Error codes are a closed typed enum
The `ErrorCode` enum MUST include variants for all pipeline failure modes including Insert/Remove operation failures, section operation failures, and builder validation failures. Five new variants MUST be added: `InsertPositionInvalid`, `RemoveAnchorMissing`, `PostMutationSchemaInvalid`, `SectionBoundaryInvalid`, `StructuralCompositionFailed`.

#### Scenario: InsertPositionInvalid error code
- **WHEN** an insert operation fails due to out-of-bounds index, invalid parent path, or disallowed block type
- **THEN** the error code MUST be `ERR_INSERT_POSITION_INVALID`

#### Scenario: RemoveAnchorMissing error code
- **WHEN** a remove operation fails due to non-existent target, scope anchor protection, or disallowed block type
- **THEN** the error code MUST be `ERR_REMOVE_ANCHOR_MISSING`

#### Scenario: PostMutationSchemaInvalid error code
- **WHEN** post-mutation ADF fails structural validity checks
- **THEN** the error code MUST be `ERR_POST_MUTATION_SCHEMA_INVALID`

#### Scenario: SectionBoundaryInvalid error code
- **WHEN** a RemoveSection targets a non-heading block or an unresolvable path
- **THEN** the error code MUST be `ERR_SECTION_BOUNDARY_INVALID`

#### Scenario: StructuralCompositionFailed error code
- **WHEN** a builder function fails to construct valid ADF (e.g., zero rows/cols for table, invalid heading level)
- **THEN** the error code MUST be `ERR_STRUCTURAL_COMPOSITION_FAILED`

#### Scenario: New error codes appear in ALL constant and tests
- **WHEN** the `ErrorCode::ALL` array is checked
- **THEN** it MUST include `InsertPositionInvalid`, `RemoveAnchorMissing`, `PostMutationSchemaInvalid`, `SectionBoundaryInvalid`, and `StructuralCompositionFailed`
- **AND** `as_str()` and `Display` tests MUST cover all new variants

### Requirement: Stable string representation
Each `ErrorCode` variant SHALL produce a stable `ERR_*` string via `Display` and `as_str()`. The string representation MUST match the original `&str` constant values exactly (e.g., `ErrorCode::ScopeMiss` displays as `"ERR_SCOPE_MISS"`).

#### Scenario: Display produces original constant string
- **WHEN** `.to_string()` is called on any `ErrorCode` variant
- **THEN** the result MUST equal the corresponding original `&str` constant value

#### Scenario: Serialization matches display
- **WHEN** an `ErrorCode` variant is serialized to JSON
- **THEN** the JSON string value MUST equal the `Display` output

### Requirement: ADF errors map to error codes by variant
The pipeline's `to_hard_error` function SHALL accept `AdfError` directly (not `impl Display`) and SHALL map each `AdfError` variant to an `ErrorCode` variant via exhaustive match. Substring matching on error messages MUST NOT be used for error code classification. `AdfError::TargetDiscoveryFailed` SHALL map to `ErrorCode::TargetDiscoveryFailed` in `to_hard_error()`, even though this arm is unreachable at runtime (discovery errors use explicit `map_err` at call sites).

#### Scenario: OutOfScope maps to OutOfScopeMutation
- **WHEN** an `AdfError::OutOfScope` is converted to a pipeline hard error
- **THEN** the error code MUST be `ErrorCode::OutOfScopeMutation`

#### Scenario: WholeBodyRewriteDisallowed maps to RouteViolation
- **WHEN** an `AdfError::WholeBodyRewriteDisallowed` is converted to a pipeline hard error
- **THEN** the error code MUST be `ErrorCode::RouteViolation`

#### Scenario: ScopeResolutionFailed maps to ScopeMiss
- **WHEN** an `AdfError::ScopeResolutionFailed` is converted to a pipeline hard error
- **THEN** the error code MUST be `ErrorCode::ScopeMiss`

#### Scenario: TargetDiscoveryFailed maps to TargetDiscoveryFailed
- **WHEN** an `AdfError::TargetDiscoveryFailed` is converted to a pipeline hard error via `to_hard_error()`
- **THEN** the error code MUST be `ErrorCode::TargetDiscoveryFailed`

#### Scenario: Remaining ADF variants map to SchemaInvalid
- **WHEN** an `AdfError` variant other than `OutOfScope`, `WholeBodyRewriteDisallowed`, `ScopeResolutionFailed`, or `TargetDiscoveryFailed` is converted to a pipeline hard error
- **THEN** the error code MUST be `ErrorCode::SchemaInvalid`

### Requirement: Confluence errors map to error codes by variant
The pipeline's `confluence_error_to_hard_error` function SHALL map each `ConfluenceError` variant to an `ErrorCode` variant. The mapping MUST be: `Conflict` to `ConflictRetryExhausted`, `NotFound` and `Transport` to `RuntimeBackend`, `NotImplemented` to `RuntimeUnmappedHard`.

#### Scenario: Conflict maps to ConflictRetryExhausted
- **WHEN** a `ConfluenceError::Conflict` is converted to a pipeline hard error
- **THEN** the error code MUST be `ErrorCode::ConflictRetryExhausted`

#### Scenario: NotFound maps to RuntimeBackend
- **WHEN** a `ConfluenceError::NotFound` is converted to a pipeline hard error
- **THEN** the error code MUST be `ErrorCode::RuntimeBackend`

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

### Requirement: Error codes cover table topology and attribute editing failures
The `ErrorCode` enum MUST include variants for Phase 9 operation failures.

#### Scenario: TableRowInvalid error code
- **WHEN** a row insert/remove fails (mismatched cell count, out of bounds, last row removal)
- **THEN** the error code MUST be `ERR_TABLE_ROW_INVALID`

#### Scenario: TableColumnInvalid error code
- **WHEN** a column insert/remove fails (out of bounds, last column removal, inconsistent table)
- **THEN** the error code MUST be `ERR_TABLE_COLUMN_INVALID`

#### Scenario: AttrUpdateBlocked error code
- **WHEN** UpdateAttrs targets a non-attr-editable node type or unresolvable path
- **THEN** the error code MUST be `ERR_ATTR_UPDATE_BLOCKED`

#### Scenario: AttrSchemaViolation error code
- **WHEN** UpdateAttrs provides disallowed attr keys for the target node type
- **THEN** the error code MUST be `ERR_ATTR_SCHEMA_VIOLATION`

#### Scenario: New error codes in ALL constant and tests
- **WHEN** `ErrorCode::ALL` is checked
- **THEN** it MUST include all 4 new variants
