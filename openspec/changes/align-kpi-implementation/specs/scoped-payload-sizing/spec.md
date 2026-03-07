## ADDED Requirements

### Requirement: Full-page ADF byte size captured before scoping
The fetch stage SHALL measure the serialized byte size of the full-page ADF payload before scope resolution and MUST include this value as `full_page_adf_bytes` in the fetch output and the run summary.

#### Scenario: Full-page byte size recorded on successful fetch
- **WHEN** the fetch stage retrieves a page from Confluence (live or stub)
- **THEN** the fetch output includes `full_page_adf_bytes` equal to the byte length of the compact-JSON-serialized full-page ADF
- **AND** the run summary includes the same `full_page_adf_bytes` value

#### Scenario: Full-page byte size is zero on fetch failure
- **WHEN** the fetch stage fails before page retrieval completes
- **THEN** the run summary includes `full_page_adf_bytes` equal to `0`

### Requirement: Scoped ADF byte size captured after scoping
The pipeline orchestrator SHALL measure the serialized byte size of the scoped ADF payload after scope resolution and MUST record this value as `scoped_adf_bytes` in the run summary.

#### Scenario: Scoped byte size recorded after successful scope resolution
- **WHEN** scope resolution produces a scoped subtree
- **THEN** the run summary includes `scoped_adf_bytes` equal to the byte length of the compact-JSON-serialized scoped ADF

#### Scenario: Scoped byte size equals full-page size on scope fallback
- **WHEN** scope resolution fails and the pipeline falls back to the full page
- **THEN** `scoped_adf_bytes` equals `full_page_adf_bytes` in the run summary

### Requirement: Context reduction ratio computed per run
The pipeline orchestrator SHALL compute `context_reduction_ratio` as `1.0 - (scoped_adf_bytes / full_page_adf_bytes)` and MUST store the result as an `f64` on the run summary.

#### Scenario: Ratio computed for non-zero full-page size
- **WHEN** `full_page_adf_bytes` is greater than zero
- **THEN** `context_reduction_ratio` equals `1.0 - (scoped_adf_bytes as f64 / full_page_adf_bytes as f64)`

#### Scenario: Ratio is zero when full-page size is zero
- **WHEN** `full_page_adf_bytes` is zero (e.g., fetch failure or bootstrapped empty page)
- **THEN** `context_reduction_ratio` equals `0.0`
