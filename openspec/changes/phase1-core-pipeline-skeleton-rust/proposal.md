## Why

Atlassy has a clear v1 architecture and safety policy, but there is no executable runtime skeleton yet to run end-to-end scoped updates with deterministic state contracts. We need a Phase 1 foundation now so later prose/table route work can build on a stable, observable pipeline instead of ad hoc implementation.

## What Changes

- Implement the Rust pipeline skeleton for v1 state order: `fetch -> classify -> extract_prose -> md_assist_edit -> adf_table_edit -> merge_candidates -> patch -> verify -> publish`.
- Define typed state envelopes and orchestrator transitions so each state has contract-valid input/output and hard errors halt execution deterministically.
- Add scoped ADF retrieval and node-path indexing primitives required by downstream route-specific logic.
- Add path-targeted patch planning guards that reject whole-body rewrite attempts.
- Persist replay artifacts per state (`state_input.json`, `state_output.json`, `diagnostics.json`) plus run summary for reproducibility.
- Establish baseline diagnostics/error code propagation for failure triage and KPI instrumentation readiness.

## Capabilities

### New Capabilities
- `pipeline-state-orchestration`: Deterministic execution of the v1 state machine with typed envelopes, ordered transitions, and fail-fast hard-error behavior.
- `scoped-adf-fetch-and-index`: Scoped ADF retrieval with page version capture, scope fallback signaling, and node-path indexing for downstream patch/verify operations.
- `patch-guard-and-replay-artifacts`: Path-targeted patch guardrails (including whole-body rewrite rejection) and per-state replay artifact persistence.

### Modified Capabilities
- None.

## Impact

- Affected code areas: planned new Rust workspace crates (`crates/atlassy-cli`, `crates/atlassy-pipeline`, `crates/atlassy-adf`, `crates/atlassy-confluence`, `crates/atlassy-contracts`).
- External systems: Confluence integration surface for scoped fetch/publish path and version metadata handling.
- APIs/contracts: introduces executable v1 state envelopes and orchestrator behavior aligned to `roadmap/09-ai-contract-spec.md`.
- Dependencies/tooling: Rust toolchain and core libraries (`clap`, `tokio`, `reqwest`, `serde`, `serde_json`, `tracing`, `thiserror`) plus fixture/replay artifact storage.
