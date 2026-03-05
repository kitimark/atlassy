# Testing Strategy and Simulation Plan (v1)

## Objective

Define a test strategy that combines live Confluence behavior research with deterministic simulation to validate v1 safety, fidelity, and KPI outcomes.

## Principles

- Treat live Confluence as the behavior truth source.
- Treat stubbed simulation as the default execution path for CI and regression testing.
- Keep all tests aligned to v1 route constraints and error contracts.
- Prefer deterministic artifacts and repeatable results over ad hoc manual verification.

## Scope

- Applies to v1 single-page pipeline states and publish flow.
- Covers positive and negative paths for all hard errors in `09-ai-contract-spec.md`.
- Covers measurement and reporting needs in `04-kpi-and-experiments.md`.

## Environment Model

### Live Research Environment

- Use a dedicated Confluence sandbox space with controlled write permissions.
- Run probe scenarios for:
  - scoped fetch behavior and scope resolution misses
  - publish success and version conflict behavior
  - error payload shapes for auth, permission, invalid payload, and server failures
- Store sanitized request and response artifacts for fixture generation.

### Stub Simulation Environment

- Implement a Confluence client interface with two adapters:
  - `LiveConfluenceClient`
  - `StubConfluenceClient`
- Drive tests through `StubConfluenceClient` by default.
- Make stub behavior deterministic and scenario-addressable.

## Test Layers

### Contract Tests

- Validate state envelopes and required fields.
- Enforce sorted and unique `changed_paths`.
- Validate enum and error code usage against contract taxonomy.

### Unit and Invariant Tests

- Route classification and lock behavior.
- Scope and path mutation constraints.
- Table shape constraints for v1 (`cell_text_update` only).

### Fixture and Snapshot Tests

- Golden ADF fixtures for route outputs, patch ops, verify diagnostics, and publish decisions.
- Error fixtures for each hard error path.

### Integration Tests (Stubbed)

- End-to-end state sequence through a simulated Confluence backend.
- Conflict retry behavior with one-retry maximum.
- Deterministic artifact persistence and run summaries.

### Live Smoke Tests

- Scheduled live probes against sandbox for drift detection.
- Minimal set focused on API behavior parity and publish conflict handling.

## Scenario Catalog (minimum v1)

- S-001 happy path scoped prose update.
- S-002 mixed prose + one table cell text update.
- S-003 constrained edit near locked structural nodes.
- S-004 out-of-scope mutation rejection (`ERR_OUT_OF_SCOPE_MUTATION`).
- S-005 locked-node mutation rejection (`ERR_LOCKED_NODE_MUTATION`).
- S-006 table shape change rejection (`ERR_TABLE_SHAPE_CHANGE`).
- S-007 schema-invalid candidate rejection (`ERR_SCHEMA_INVALID`).
- S-008 publish conflict resolved on first scoped retry.
- S-009 publish conflict retry exhausted (`ERR_CONFLICT_RETRY_EXHAUSTED`).
- S-010 telemetry completeness and artifact integrity check.

## Drift Management

- Compare live smoke outputs with stub scenario expectations.
- If drift is detected:
  - update fixtures and stub scenarios first
  - re-run affected regression suites
  - record behavior update in decisions or risk docs when material

## CI and Cadence

- PR CI: contract + unit + fixture + stub integration tests.
- Nightly: full scenario pack plus live smoke subset.
- Phase exit: KPI report and safety gate review before `go | iterate | stop`.

## Entry and Exit Gates

### Entry Gates

- Rust toolchain and test harness are available.
- Sandbox credentials and write access are provisioned.
- Stub scenario runner and artifact schema are in place.

### Exit Gates

- All required v1 scenarios pass in stubbed regression suite.
- Live smoke results show no unresolved behavior drift.
- No unresolved high-priority safety failures.
- KPI evidence is complete for decision review.

## OpenSpec Execution Mapping

- Planning and tracking should run under:
  - `phase4-poc-execution-metrics-rust`
  - `phase5-hardening-readiness-rust`
- Required artifacts: `proposal`, `specs`, `design`, `tasks`.
- Test scenario IDs should be referenced directly in `tasks.md` checklists.
