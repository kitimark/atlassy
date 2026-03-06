## ADDED Requirements

### Requirement: Prose edits are constrained to mapped prose paths
The `md_assist_edit` state SHALL emit `prose_changed_paths` only from paths present in `md_to_adf_map` and MUST reject candidate changes that target unmapped paths.

#### Scenario: Candidate includes unmapped path
- **WHEN** markdown assist output implies a change outside mapped prose paths
- **THEN** the pipeline marks the candidate as invalid and blocks publish through verifier failure

### Requirement: Top-level type and boundary safety
Markdown-assisted prose edits MUST preserve top-level node type and prose block boundary for each mapped path.

#### Scenario: Top-level type expansion attempt
- **WHEN** markdown assist output would change a mapped prose node into an incompatible top-level type
- **THEN** the pipeline rejects the candidate with a hard validation failure

### Requirement: Route isolation during prose assist
The prose assist route MUST NOT produce changes for `table_adf` or `locked_structural` routes.

#### Scenario: Mixed-content page prose update
- **WHEN** a scoped update includes prose, table, and locked structural content
- **THEN** `prose_changed_paths` includes only `editable_prose` paths
- **AND** table and locked structural paths remain unchanged
