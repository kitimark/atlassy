## 1. Workspace and Foundations

- [x] 1.1 Create the Rust workspace layout and crate manifests for `atlassy-cli`, `atlassy-pipeline`, `atlassy-adf`, `atlassy-confluence`, and `atlassy-contracts`
- [x] 1.2 Add baseline dependencies (`clap`, `tokio`, `reqwest`, `serde`, `serde_json`, `tracing`, `thiserror`) with pinned versions
- [x] 1.3 Define shared error taxonomy constants aligned to v1 contract error codes
- [x] 1.4 Add root cargo commands/scripts for build, test, and lint consistency

## 2. Contract and State Envelope Layer

- [x] 2.1 Implement typed request/response envelope structs with required metadata fields (`request_id`, `page_id`, `state`, `timestamp`)
- [x] 2.2 Implement envelope validation that fails on missing or invalid required fields
- [x] 2.3 Model per-state input/output types for all v1 states in the canonical order
- [x] 2.4 Add contract tests to verify enum values, path sorting/uniqueness expectations, and serialization stability

## 3. Orchestrator State Machine

- [x] 3.1 Implement orchestrator transition graph enforcing exact order: `fetch -> classify -> extract_prose -> md_assist_edit -> adf_table_edit -> merge_candidates -> patch -> verify -> publish`
- [x] 3.2 Implement fail-fast halt behavior on first hard error and include origin state in run summary
- [x] 3.3 Add guard for out-of-order transition attempts with deterministic transition error reporting
- [x] 3.4 Implement run summary output with success/fail status, applied/blocked paths, error codes, and token metrics placeholders

## 4. Scoped Fetch and Node Indexing

- [x] 4.1 Implement scope selector resolution for heading/block ID selectors
- [x] 4.2 Implement bounded full-page fallback with explicit `scope_resolution_failed` and fallback reason capture
- [x] 4.3 Implement deterministic `node_path_index` generation using JSON Pointer paths
- [x] 4.4 Add integrity checks to reject duplicate path entries during index construction

## 5. Patch Guardrails and Artifact Persistence

- [x] 5.1 Implement path-targeted patch op builder that rejects whole-body rewrite attempts
- [x] 5.2 Enforce `changed_paths` uniqueness and lexicographic sort before verify handoff
- [x] 5.3 Enforce allowed-scope boundary checks and wire verify failure mapping to `ERR_OUT_OF_SCOPE_MUTATION`
- [x] 5.4 Implement per-state artifact persistence (`state_input.json`, `state_output.json`, `diagnostics.json`) and run-level `summary.json`

## 6. CLI Wiring and End-to-End Validation

- [x] 6.1 Add CLI command entrypoint to execute the orchestrator for no-op and simple scoped update flows
- [x] 6.2 Add fixture-backed integration tests for happy path, contract validation failure, and verify-before-publish blocking
- [x] 6.3 Add tests proving no publish call occurs after hard errors and no out-of-order state execution is possible
- [x] 6.4 Validate replay artifact directory structure and required file presence for successful and failed runs
