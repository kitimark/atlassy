## ADDED Requirements

### Requirement: Prose formatting fidelity is non-regressive on fixtures
Phase 2 prose route validation MUST run fixture-backed checks that confirm prose formatting is non-regressive for in-scope prose edits.

#### Scenario: Prose fixture regression check passes
- **WHEN** prose-route fixtures are executed for representative prose-only and mixed-content pages
- **THEN** resulting prose formatting matches expected non-regressive outputs for mapped prose paths

### Requirement: Out-of-scope formatting changes are blocked
Any formatting mutation outside mapped prose paths SHALL be treated as an invalid candidate and MUST fail verification.

#### Scenario: Formatting drift outside prose map
- **WHEN** candidate formatting changes appear on paths outside mapped prose scope
- **THEN** verification fails and publish is blocked

### Requirement: Fixture suite covers route isolation and path safety
The prose fidelity suite MUST include cases that prove no table or locked-structural formatting changes are introduced by prose assist.

#### Scenario: Mixed-route fidelity fixture
- **WHEN** a fixture contains prose, table, and locked-structural nodes
- **THEN** prose formatting assertions are evaluated only for mapped prose paths
- **AND** non-prose route paths remain byte-stable or semantically unchanged per fixture expectations
