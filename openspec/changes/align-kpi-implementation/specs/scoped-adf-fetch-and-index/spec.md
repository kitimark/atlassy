## MODIFIED Requirements

### Requirement: Fetch output includes version and allowed scope paths
The fetch output MUST include the source `page_version`, an `allowed_scope_paths` list that defines the mutation boundary for downstream patch and verify checks, and `full_page_adf_bytes` recording the serialized byte size of the full-page ADF before scope resolution.

#### Scenario: Fetch prepares mutation boundary
- **WHEN** fetch completes for a scoped request
- **THEN** the output contains `page_version` and a non-empty `allowed_scope_paths` list aligned to the retrieved scope

#### Scenario: Fetch records full-page byte size
- **WHEN** fetch completes for any request (scoped or full-page fallback)
- **THEN** the output contains `full_page_adf_bytes` equal to the byte length of the compact-JSON-serialized full-page ADF before scope resolution
