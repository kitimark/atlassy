## MODIFIED Requirements

### Requirement: CI checks formatting
The CI workflow SHALL reject code that does not pass the format check by calling `make fmt-check`.

#### Scenario: Unformatted code fails CI
- **WHEN** the workspace contains unformatted Rust code
- **THEN** the format check step fails and the workflow reports failure

#### Scenario: Formatted code passes CI
- **WHEN** all Rust code in the workspace passes `cargo fmt` check
- **THEN** the format check step succeeds

### Requirement: CI checks lint warnings
The CI workflow SHALL reject code that produces clippy warnings by calling `make lint`.

#### Scenario: Clippy warning fails CI
- **WHEN** the workspace contains code producing clippy warnings
- **THEN** the lint step fails and the workflow reports failure

#### Scenario: Clean clippy passes CI
- **WHEN** the workspace produces zero clippy warnings
- **THEN** the lint step succeeds

### Requirement: CI runs full test suite
The CI workflow SHALL run the full workspace test suite by calling `make test` and reject code with any test failure.

#### Scenario: Test failure fails CI
- **WHEN** any test in the workspace fails
- **THEN** the test step fails and the workflow reports failure

#### Scenario: All tests pass
- **WHEN** all workspace tests pass
- **THEN** the test step succeeds
