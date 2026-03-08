## MODIFIED Requirements

### Requirement: Scoped ADF byte size captured after scoping
The pipeline orchestrator SHALL measure the total serialized byte size of the ADF nodes within `allowed_scope_paths` and MUST record this value as `scoped_adf_bytes` in the run summary. This value represents the section bytes — the size of only the nodes the pipeline is scoped to edit — not the size of the `scoped_adf` payload.

#### Scenario: Section byte size recorded after successful scope resolution
- **WHEN** scope resolution produces a non-empty `allowed_scope_paths`
- **THEN** the run summary includes `scoped_adf_bytes` equal to the sum of compact-JSON-serialized byte lengths of each node at the paths in `allowed_scope_paths`

#### Scenario: Scoped byte size equals full-page size on scope fallback
- **WHEN** scope resolution fails and the pipeline falls back to the full page
- **THEN** `scoped_adf_bytes` equals `full_page_adf_bytes` in the run summary

### Requirement: Context reduction ratio computed per run
The pipeline orchestrator SHALL compute `context_reduction_ratio` as `1.0 - (scoped_adf_bytes / full_page_adf_bytes)` where `scoped_adf_bytes` is the section byte size (sum of nodes within `allowed_scope_paths`), and MUST store the result as an `f64` on the run summary.

#### Scenario: Ratio computed for non-zero full-page size
- **WHEN** `full_page_adf_bytes` is greater than zero
- **THEN** `context_reduction_ratio` equals `1.0 - (scoped_adf_bytes as f64 / full_page_adf_bytes as f64)` where `scoped_adf_bytes` is computed from section path nodes

#### Scenario: Ratio is zero when full-page size is zero
- **WHEN** `full_page_adf_bytes` is zero (e.g., fetch failure or bootstrapped empty page)
- **THEN** `context_reduction_ratio` equals `0.0`
