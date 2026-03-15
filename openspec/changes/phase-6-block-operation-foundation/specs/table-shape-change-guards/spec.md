## MODIFIED Requirements

### Requirement: Shape guard checks are enforced before publish
Verifier integration SHALL treat detected table shape or attribute drift as hard failure and block publish. The check MUST operate on paths extracted from the operation list and MUST be positioned in the verify chain before `check_operation_legality` and `check_structural_validity`.

#### Scenario: Drift detected at verify
- **WHEN** candidate page ADF contains unauthorized table topology or attribute changes
- **THEN** verify fails with `ERR_TABLE_SHAPE_CHANGE`
- **AND** publish is blocked

#### Scenario: Table shape check receives paths from operations
- **WHEN** the verify stage runs with `Vec<Operation>` in the input
- **THEN** `check_table_shape_integrity` MUST extract paths from the operations and check them
- **AND** the behavior MUST be identical to the previous path-based check for Replace-only runs
