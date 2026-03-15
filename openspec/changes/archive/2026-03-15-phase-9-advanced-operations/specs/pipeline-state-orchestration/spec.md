## MODIFIED Requirements

### Requirement: Workspace includes atlassy-mcp crate
The Cargo workspace MUST include `atlassy-mcp` as a member. The MCP crate depends on `atlassy-pipeline`, `atlassy-contracts`, and `atlassy-confluence` but no existing crate depends on `atlassy-mcp`.

#### Scenario: atlassy-mcp builds as part of workspace
- **WHEN** `cargo build --workspace` is run
- **THEN** `atlassy-mcp` MUST compile without errors

#### Scenario: Existing crates unaffected
- **WHEN** `atlassy-mcp` is added to the workspace
- **THEN** no existing crate's Cargo.toml MUST change (atlassy-mcp is a leaf dependency)
