## MODIFIED Requirements

### Requirement: Decision packet is reproducible from stored artifacts
The decision workflow MUST support regenerating an equivalent decision packet from persisted manifest, artifact-index, run-summary, attestation, and gate-result outputs. The attestation file is an optional input — when present during the original generation, it MUST also be present during replay for the packet to match.

#### Scenario: Rebuild packet from persisted evidence
- **WHEN** packet regeneration is executed against stored artifacts
- **THEN** regenerated packet content matches the original decision packet
- **AND** any mismatch is surfaced as a readiness failure

#### Scenario: Rebuild packet with attestation evidence
- **WHEN** packet regeneration is executed against stored artifacts that include `artifacts/batch/attestations.json`
- **THEN** attestation evidence is loaded identically to the original generation
- **AND** gate outcomes that depend on attestation evidence produce the same results

#### Scenario: Attestation file added between generation and replay causes mismatch
- **WHEN** `artifacts/batch/attestations.json` was absent during original generation but is present during replay
- **THEN** replay detects a mismatch and surfaces a readiness failure
