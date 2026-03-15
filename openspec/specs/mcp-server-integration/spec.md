## Purpose

Define MCP server integration that exposes atlassy pipeline capabilities through stdio tools using shared contracts and existing safety guarantees.

## Requirements

### Requirement: MCP server exposes pipeline operations as tools
A new `atlassy-mcp` crate SHALL provide an MCP server that exposes pipeline capabilities as MCP tools via stdio transport.

#### Scenario: MCP server starts and lists tools
- **WHEN** the MCP server starts via stdio
- **THEN** it MUST respond to `tools/list` with available tools including `atlassy_run`, `atlassy_run_multi_page`, `atlassy_create_subpage`

#### Scenario: atlassy_run tool executes single-page pipeline
- **WHEN** the `atlassy_run` tool is called with page_id, edit_intent, scope_selectors, and block_ops
- **THEN** it MUST construct a `RunRequest`, call `Orchestrator::run()`, and return the `RunSummary` as the tool result

#### Scenario: atlassy_run_multi_page tool executes multi-page pipeline
- **WHEN** the `atlassy_run_multi_page` tool is called with a multi-page plan
- **THEN** it MUST construct a `MultiPageRequest`, call `MultiPageOrchestrator::run()`, and return the `MultiPageSummary`

#### Scenario: atlassy_create_subpage tool creates a page
- **WHEN** the `atlassy_create_subpage` tool is called with parent_page_id, space_key, title
- **THEN** it MUST call `ConfluenceClient::create_page()` and return the new page_id

### Requirement: MCP server uses shared contract types
Tool input/output schemas MUST be derived from types in `atlassy-contracts`. No parallel type definitions in the MCP crate.

#### Scenario: Tool input matches RunRequest
- **WHEN** the `atlassy_run` tool receives input
- **THEN** the input MUST be deserializable as fields of `RunRequest`

#### Scenario: Tool output matches RunSummary
- **WHEN** the `atlassy_run` tool returns a result
- **THEN** the result MUST be serializable from `RunSummary`

### Requirement: MCP server preserves safety guarantees
All pipeline safety checks (verify, scope enforcement, structural validity, locked boundary) MUST apply identically to MCP-invoked operations as to CLI-invoked operations.

#### Scenario: MCP insert outside scope is rejected
- **WHEN** an MCP tool call includes a block_op with a parent_path outside allowed scope
- **THEN** the pipeline MUST reject with the same error as a CLI invocation

### Requirement: MCP server reads Confluence credentials from environment
The MCP server MUST use the same env-var-based credential pattern as the CLI (`ATLASSY_CONFLUENCE_BASE_URL`, `ATLASSY_CONFLUENCE_EMAIL`, `ATLASSY_CONFLUENCE_API_TOKEN`).

#### Scenario: Missing credentials
- **WHEN** the MCP server starts without required env vars
- **THEN** it MUST report a clear error on the first tool call that needs Confluence access
