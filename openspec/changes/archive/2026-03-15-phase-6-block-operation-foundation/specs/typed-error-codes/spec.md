## MODIFIED Requirements

### Requirement: Error code enum covers all pipeline failure modes
The `ErrorCode` enum MUST include variants for all pipeline failure modes including Insert/Remove operation failures. Three new variants MUST be added: `InsertPositionInvalid`, `RemoveAnchorMissing`, `PostMutationSchemaInvalid`.

#### Scenario: InsertPositionInvalid error code
- **WHEN** an insert operation fails due to out-of-bounds index, invalid parent path, or disallowed block type
- **THEN** the error code MUST be `ERR_INSERT_POSITION_INVALID`

#### Scenario: RemoveAnchorMissing error code
- **WHEN** a remove operation fails due to non-existent target, scope anchor protection, or disallowed block type
- **THEN** the error code MUST be `ERR_REMOVE_ANCHOR_MISSING`

#### Scenario: PostMutationSchemaInvalid error code
- **WHEN** post-mutation ADF fails structural validity checks
- **THEN** the error code MUST be `ERR_POST_MUTATION_SCHEMA_INVALID`

#### Scenario: New error codes appear in ALL constant and tests
- **WHEN** the `ErrorCode::ALL` array is checked
- **THEN** it MUST include `InsertPositionInvalid`, `RemoveAnchorMissing`, and `PostMutationSchemaInvalid`
- **AND** `as_str()` and `Display` tests MUST cover all new variants
