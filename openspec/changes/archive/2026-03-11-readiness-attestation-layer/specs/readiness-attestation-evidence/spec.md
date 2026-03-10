## ADDED Requirements

### Requirement: Attestation file format and container structure
The attestation file (`artifacts/batch/attestations.json`) SHALL use a versioned container with an `entries` array. Each entry MUST include `attestation_id` (string identifier), `attested_by` (operator role), `provenance` (ProvenanceStamp), `evidence_refs` (list of evidence file paths), and `claims` (JSON value with attestation-kind-specific structure).

#### Scenario: Valid attestation file with lifecycle entry
- **WHEN** `artifacts/batch/attestations.json` contains a valid container with `schema_version: "v1"` and one entry with `attestation_id: "lifecycle_validation"`
- **THEN** the attestation is loaded successfully
- **AND** the entry is queryable by `attestation_id`

#### Scenario: Attestation entry with all required fields
- **WHEN** an attestation entry includes `attestation_id`, `attested_by`, `provenance`, `evidence_refs`, and `claims`
- **THEN** the entry passes validation
- **AND** the `provenance` stamp is validated as well-formed via `validate_provenance_stamp()`

### Requirement: Attestation file is optional with empty default
The readiness evidence loader MUST treat the attestation file as optional. When `artifacts/batch/attestations.json` does not exist, attestations SHALL default to an empty container with no entries. When the file exists but is malformed, evidence loading MUST fail with an operator-facing error.

#### Scenario: Missing attestation file defaults to empty
- **WHEN** `artifacts/batch/attestations.json` does not exist in the artifacts directory
- **THEN** `ReadinessEvidence.attestations` defaults to an empty `Attestations` with `schema_version: "v1"` and zero entries
- **AND** evidence loading succeeds without error

#### Scenario: Malformed attestation file fails evidence loading
- **WHEN** `artifacts/batch/attestations.json` exists but contains invalid JSON or does not match the expected schema
- **THEN** evidence loading fails with an operator-facing error message referencing the attestation file

### Requirement: Attestation provenance is self-contained
Each attestation entry MUST carry its own `ProvenanceStamp`. The evidence loader MUST validate that each attestation's provenance stamp is well-formed but SHALL NOT require it to match the batch provenance.

#### Scenario: Attestation provenance differs from batch provenance
- **WHEN** an attestation entry has a valid provenance stamp with a different `git_commit_sha` than the batch provenance
- **THEN** evidence loading succeeds
- **AND** the attestation is available for gate evaluation

#### Scenario: Attestation with malformed provenance fails validation
- **WHEN** an attestation entry has a provenance stamp with an invalid `git_commit_sha` (not 40-character hex)
- **THEN** evidence loading fails with an operator-facing error

### Requirement: Attestation claims use gate-specific typed deserialization
Gate evaluation code MUST deserialize the `claims` JSON value into a typed struct specific to the attestation kind. For `lifecycle_validation`, the typed struct is `LifecycleClaims` with four boolean fields: `bootstrap_required_failure`, `bootstrap_success`, `bootstrap_on_non_empty_failure`, `create_subpage_validated`. If deserialization fails, the attestation MUST be treated as absent.

#### Scenario: Valid lifecycle claims deserialize successfully
- **WHEN** a `lifecycle_validation` attestation has claims `{"bootstrap_required_failure": true, "bootstrap_success": true, "bootstrap_on_non_empty_failure": true, "create_subpage_validated": true}`
- **THEN** claims deserialize into a `LifecycleClaims` struct with all four fields set to `true`

#### Scenario: Malformed claims treated as absent attestation
- **WHEN** a `lifecycle_validation` attestation has claims that do not match the `LifecycleClaims` structure (e.g., missing fields, wrong types)
- **THEN** the attestation is treated as absent for gate evaluation purposes
- **AND** the gate falls back to its existing evidence source

### Requirement: Attestation file path included in source artifacts
When the attestation file exists and is loaded, its path MUST be included in `ReadinessEvidence.source_artifacts` for audit traceability. When absent, no attestation path SHALL appear in source artifacts.

#### Scenario: Present attestation file appears in source artifacts
- **WHEN** `artifacts/batch/attestations.json` exists and is loaded
- **THEN** `"artifacts/batch/attestations.json"` is included in `ReadinessEvidence.source_artifacts`

#### Scenario: Absent attestation file omitted from source artifacts
- **WHEN** `artifacts/batch/attestations.json` does not exist
- **THEN** no attestation file path appears in `ReadinessEvidence.source_artifacts`
