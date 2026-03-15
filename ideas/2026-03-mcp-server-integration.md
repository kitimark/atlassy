# MCP Server Integration for Atlassy

## Status

Promoted to roadmap Phase 9 (Advanced Operations). See `roadmap/03-phased-roadmap.md`.

## Plain Problem Points

- The CLI-first workflow is effective for PoC validation but not ideal for agent-native consumption.
- AI agents need stable tool contracts, typed inputs, and predictable responses over a transport like MCP.
- Shell orchestration around `run` and `run-batch` adds operational friction for interactive assistants.

## Proposed Direction

Add an MCP server surface as a new crate (`crates/atlassy-mcp`) that reuses the existing core crates without changing v1 safety defaults.

- Expose MCP tools that map directly to existing CLI capabilities (`run`, `run-batch`, `run-readiness`, `create-subpage`).
- Keep shared payload contracts in `atlassy-contracts` to avoid CLI/MCP divergence.
- Preserve runtime backend controls (`stub|live`) and provenance requirements.
- Keep route policy and verifier behavior unchanged between CLI and MCP entry points.

## Why Not Now

- v1 PoC is currently focused on proving scoped-edit value and safety with the existing CLI.
- Adding an MCP surface now would mix interface expansion with KPI validation and make results harder to interpret.
- The highest-leverage immediate work is revised KPI revalidation using scoped-selector manifests.

## Risks

- Contract drift between CLI and MCP tool schemas.
- Runtime authentication handling complexity in long-lived MCP server processes.
- Increased support burden if MCP transport concerns are mixed with core pipeline bugs.

## Signals To Revisit

- Revised KPI PoC has decision-grade evidence under the new framework.
- Repeated operator demand for direct agent-tool integration instead of shell wrappers.
- Stable request/response contracts are validated across at least one full release cycle.

## Promotion Path

- Promote to roadmap after revised KPI reruns complete and v1 readiness recommendation is stable.
- Create a dedicated roadmap phase for MCP server design, tool catalog, and transport hardening.
