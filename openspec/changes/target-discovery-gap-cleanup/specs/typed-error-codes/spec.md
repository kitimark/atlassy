## MODIFIED Requirements

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
