## MODIFIED Requirements

### Requirement: RunMode variants accept optional target path

`RunMode::SimpleScopedProseUpdate` and `RunMode::SimpleScopedTableCellUpdate` SHALL have `target_path` typed as `Option<String>`. Synthetic and test variants SHALL keep their path fields as `String`. The `RunMode::SimpleScopedUpdate` variant SHALL be removed as dead code — it is superseded by the route-specific variants and cannot support auto-discovery due to route ambiguity.

#### Scenario: SimpleScopedProseUpdate accepts None

- **WHEN** a `RunMode::SimpleScopedProseUpdate` is constructed with `target_path: None`
- **THEN** the variant is valid and triggers auto-discovery at runtime

#### Scenario: SimpleScopedUpdate is removed

- **WHEN** the `RunMode` enum is inspected
- **THEN** no `SimpleScopedUpdate` variant exists
- **AND** no `ManifestMode::SimpleScopedUpdate` variant exists
- **AND** no CLI mode string `"simple-scoped-update"` is accepted

## REMOVED Requirements

### Requirement: SimpleScopedUpdate still requires explicit path

**Reason**: The `SimpleScopedUpdate` variant is dead code — unused in integration tests, QA manifests, or production workflows. It is fully superseded by `SimpleScopedProseUpdate` and `SimpleScopedTableCellUpdate`, which support auto-discovery. Removing it eliminates a duplicated magic string, simplifies match arms in both pipeline states, and addresses Dead Code/Speculative Generality smells per refactoring.guru.

**Migration**: Replace `RunMode::SimpleScopedUpdate { target_path, new_value }` with either `RunMode::SimpleScopedProseUpdate { target_path: Some(path), markdown }` or `RunMode::SimpleScopedTableCellUpdate { target_path: Some(path), text }` depending on the intended route. Replace `ManifestMode::SimpleScopedUpdate` with `simple_scoped_prose_update` or `simple_scoped_table_cell_update` in manifest JSON files.
