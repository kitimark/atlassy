## ADDED Requirements

### Requirement: Patch operations byte size captured in patch output
The patch stage SHALL measure the serialized byte size of the generated patch operations and MUST include this value as `patch_ops_bytes` in the patch output and the run summary.

#### Scenario: Patch ops byte size recorded on successful patch
- **WHEN** the patch stage generates patch operations
- **THEN** the patch output includes `patch_ops_bytes` equal to the byte length of the compact-JSON-serialized `patch_ops` vector
- **AND** the run summary includes the same `patch_ops_bytes` value

#### Scenario: Patch ops byte size is zero when patch stage is not reached
- **WHEN** the pipeline fails before the patch stage executes
- **THEN** the run summary includes `patch_ops_bytes` equal to `0`
