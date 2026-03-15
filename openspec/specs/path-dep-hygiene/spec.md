### Requirement: Internal path dependencies omit version field
Internal workspace crate dependencies SHALL be declared with `path` only, without a `version` field. The `version` field is unnecessary for crates that are not published to a registry.

#### Scenario: Path dependency without version
- **WHEN** a crate in the workspace depends on another workspace crate
- **THEN** the dependency declaration SHALL use `{ path = "<relative-path>" }` without a `version` key

#### Scenario: External dependencies are unaffected
- **WHEN** a crate depends on an external crate from a registry
- **THEN** the dependency declaration continues to use `version` (or `.workspace = true`) as normal
