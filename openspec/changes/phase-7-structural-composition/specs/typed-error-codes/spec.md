## MODIFIED Requirements

### Requirement: Error codes cover section boundary and structural composition failures
The `ErrorCode` enum MUST include variants for section operation failures and builder validation failures.

#### Scenario: SectionBoundaryInvalid error code
- **WHEN** a RemoveSection targets a non-heading block or an unresolvable path
- **THEN** the error code MUST be `ERR_SECTION_BOUNDARY_INVALID`

#### Scenario: StructuralCompositionFailed error code
- **WHEN** a builder function fails to construct valid ADF (e.g., zero rows/cols for table, invalid heading level)
- **THEN** the error code MUST be `ERR_STRUCTURAL_COMPOSITION_FAILED`

#### Scenario: New error codes in ALL constant and tests
- **WHEN** `ErrorCode::ALL` is checked
- **THEN** it MUST include `SectionBoundaryInvalid` and `StructuralCompositionFailed`
- **AND** `as_str()` and `Display` tests MUST cover both new variants
