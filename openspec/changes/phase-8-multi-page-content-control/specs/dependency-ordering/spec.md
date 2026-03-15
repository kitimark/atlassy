## ADDED Requirements

### Requirement: Pages execute in topological dependency order
The `MultiPageOrchestrator` MUST sort `PageTarget` entries by their `depends_on` relationships using topological sort before execution. Pages with no dependencies execute first.

#### Scenario: Linear dependency chain
- **GIVEN** pages A (no deps), B (depends on A), C (depends on B)
- **WHEN** the plan is executed
- **THEN** execution order MUST be A → B → C

#### Scenario: Independent pages at same level
- **GIVEN** pages A (no deps), B (depends on A), C (depends on A)
- **WHEN** the plan is executed
- **THEN** A MUST execute before both B and C
- **AND** B and C may execute in any order relative to each other

#### Scenario: No dependencies
- **GIVEN** pages A, B, C with no `depends_on` entries
- **WHEN** the plan is executed
- **THEN** pages MAY execute in any order

### Requirement: Dependency cycles are rejected before execution
The `MultiPageOrchestrator` MUST detect cycles in `depends_on` and reject the plan before executing any page pipeline.

#### Scenario: Direct cycle
- **GIVEN** page A depends on B, page B depends on A
- **WHEN** the plan is validated
- **THEN** it MUST fail with `ERR_DEPENDENCY_CYCLE` before any page is processed

#### Scenario: Indirect cycle
- **GIVEN** A depends on B, B depends on C, C depends on A
- **WHEN** the plan is validated
- **THEN** it MUST fail with `ERR_DEPENDENCY_CYCLE`

#### Scenario: Unresolvable dependency
- **GIVEN** page A depends on page_id "999" which is not in the plan
- **WHEN** the plan is validated
- **THEN** it MUST fail with an appropriate error (dependency target not found)
