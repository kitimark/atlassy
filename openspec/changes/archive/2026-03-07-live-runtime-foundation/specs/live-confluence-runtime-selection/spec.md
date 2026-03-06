## ADDED Requirements

### Requirement: Runtime backend selection is explicit and deterministic
The runtime MUST support backend selection between `stub` and `live` modes using explicit operator configuration.

#### Scenario: Stub backend execution
- **WHEN** runtime backend is configured as `stub`
- **THEN** execution uses deterministic stub Confluence behavior and preserves existing fixture reproducibility

#### Scenario: Live backend execution
- **WHEN** runtime backend is configured as `live`
- **THEN** execution uses live Confluence fetch and publish paths with validated runtime configuration

### Requirement: Live runtime errors map to deterministic taxonomy
Live backend failures MUST map to deterministic error codes compatible with existing verifier, report, and runbook workflows.

#### Scenario: Live publish conflict follows mapped retry behavior
- **WHEN** live publish returns a version conflict
- **THEN** execution applies at most one scoped retry and emits deterministic retry diagnostics on failure

#### Scenario: Live hard error is classifiable
- **WHEN** live fetch or publish returns a hard failure
- **THEN** the failure is mapped to a deterministic error code used by readiness and triage outputs

### Requirement: Runtime mode is included in artifacts
Run, batch, and readiness artifacts MUST include the selected runtime backend mode.

#### Scenario: Runtime mode is auditable in outputs
- **WHEN** artifacts are generated for a run sequence
- **THEN** each decision-grade output records whether execution used `stub` or `live`
