## ADDED Requirements

### Requirement: Error codes are a closed typed enum
The system SHALL represent pipeline error codes as variants of an `ErrorCode` enum rather than string constants. The enum MUST be defined in `atlassy-contracts` and MUST contain exactly the 12 error classifications: `ScopeMiss`, `RouteViolation`, `SchemaInvalid`, `OutOfScopeMutation`, `LockedNodeMutation`, `TableShapeChange`, `ConflictRetryExhausted`, `RuntimeBackend`, `RuntimeUnmappedHard`, `BootstrapRequired`, `BootstrapInvalidState`, `TargetDiscoveryFailed`.

#### Scenario: Exhaustive match on error codes
- **WHEN** code matches on an `ErrorCode` value
- **THEN** the compiler SHALL require all 12 variants to be handled (or a wildcard arm), preventing silent omission of new error classifications

#### Scenario: Construction requires a valid variant
- **WHEN** a `PipelineError::Hard` is constructed
- **THEN** the `code` field MUST be an `ErrorCode` variant, and the compiler SHALL reject arbitrary strings

### Requirement: Stable string representation
Each `ErrorCode` variant SHALL produce a stable `ERR_*` string via `Display` and `as_str()`. The string representation MUST match the original `&str` constant values exactly (e.g., `ErrorCode::ScopeMiss` displays as `"ERR_SCOPE_MISS"`).

#### Scenario: Display produces original constant string
- **WHEN** `.to_string()` is called on any `ErrorCode` variant
- **THEN** the result MUST equal the corresponding original `&str` constant value

#### Scenario: Serialization matches display
- **WHEN** an `ErrorCode` variant is serialized to JSON
- **THEN** the JSON string value MUST equal the `Display` output

### Requirement: ADF errors map to error codes by variant
The pipeline's `to_hard_error` function SHALL accept `AdfError` directly (not `impl Display`) and SHALL map each `AdfError` variant to an `ErrorCode` variant via exhaustive match. Substring matching on error messages MUST NOT be used for error code classification.

#### Scenario: OutOfScope maps to OutOfScopeMutation
- **WHEN** an `AdfError::OutOfScope` is converted to a pipeline hard error
- **THEN** the error code MUST be `ErrorCode::OutOfScopeMutation`

#### Scenario: WholeBodyRewriteDisallowed maps to RouteViolation
- **WHEN** an `AdfError::WholeBodyRewriteDisallowed` is converted to a pipeline hard error
- **THEN** the error code MUST be `ErrorCode::RouteViolation`

#### Scenario: ScopeResolutionFailed maps to ScopeMiss
- **WHEN** an `AdfError::ScopeResolutionFailed` is converted to a pipeline hard error
- **THEN** the error code MUST be `ErrorCode::ScopeMiss`

#### Scenario: Remaining ADF variants map to SchemaInvalid
- **WHEN** an `AdfError` variant other than `OutOfScope`, `WholeBodyRewriteDisallowed`, or `ScopeResolutionFailed` is converted to a pipeline hard error
- **THEN** the error code MUST be `ErrorCode::SchemaInvalid`

### Requirement: Confluence errors map to error codes by variant
The pipeline's `confluence_error_to_hard_error` function SHALL map each `ConfluenceError` variant to an `ErrorCode` variant. The mapping MUST be: `Conflict` to `ConflictRetryExhausted`, `NotFound` and `Transport` to `RuntimeBackend`, `NotImplemented` to `RuntimeUnmappedHard`.

#### Scenario: Conflict maps to ConflictRetryExhausted
- **WHEN** a `ConfluenceError::Conflict` is converted to a pipeline hard error
- **THEN** the error code MUST be `ErrorCode::ConflictRetryExhausted`

#### Scenario: NotFound maps to RuntimeBackend
- **WHEN** a `ConfluenceError::NotFound` is converted to a pipeline hard error
- **THEN** the error code MUST be `ErrorCode::RuntimeBackend`
