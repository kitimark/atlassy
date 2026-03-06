## Purpose

Define deterministic operator runbook generation for priority readiness failure classes with explicit severity and escalation ownership metadata.

## Requirements

### Requirement: Deterministic runbook generation for priority failure classes
The system MUST generate operator runbooks using deterministic mappings for priority failure classes, including verify hard failures, scoped-retry exhaustion, safety-gate violations, unresolved drift, and telemetry-completeness failures.

#### Scenario: Generate runbook for verify hard failure
- **WHEN** batch diagnostics include a verify hard failure class
- **THEN** the system emits the mapped verify-failure runbook section
- **AND** the section includes required triage steps and expected evidence checks

### Requirement: Runbooks include escalation ownership and severity
Each generated runbook section SHALL include severity classification, primary owner role, escalation owner role, and escalation trigger conditions.

#### Scenario: Include ownership metadata in runbook output
- **WHEN** runbook synthesis runs for any supported failure class
- **THEN** output includes severity, primary owner role, escalation owner role, and escalation trigger criteria

### Requirement: Unknown failure classes are handled safely
If diagnostics include an unmapped failure class, the system MUST emit a fallback runbook section that blocks automatic sign-off and routes the issue to manual review.

#### Scenario: Fallback runbook for unknown class
- **WHEN** a failure class has no deterministic mapping
- **THEN** output includes a fallback runbook entry with manual-review instructions
- **AND** readiness status is marked non-passing until triage mapping is defined
