## Purpose

Define the module boundaries, facade shape, dependency graph, and visibility rules for the `atlassy-cli` crate after modularization. This spec parallels `pipeline-module-structure` and `leaf-crate-module-structure`, establishing the same structural pattern for the CLI layer.

## ADDED Requirements

### Requirement: CLI lib.rs SHALL be a facade

`crates/atlassy-cli/src/lib.rs` SHALL contain only module declarations (`mod <name>;`), and re-exports (`pub use <module>::*;`). It SHALL NOT contain function bodies, type definitions, or domain logic.

#### Scenario: lib.rs contains only declarations and re-exports
- **WHEN** `crates/atlassy-cli/src/lib.rs` is inspected
- **THEN** it contains `mod types;`, `mod fixtures;`, `mod provenance;`, `mod manifest;`, `mod io;`, `mod batch;`, `mod readiness;`, `mod commands;` declarations
- **THEN** it contains `pub use` re-exports for the crate's public API
- **THEN** it contains no `fn` definitions (excluding `#[cfg(test)]` blocks)

#### Scenario: No logic functions in lib.rs
- **WHEN** `grep -n 'fn ' crates/atlassy-cli/src/lib.rs` is run
- **THEN** zero matches are returned

### Requirement: CLI main.rs SHALL be thin dispatch only

`crates/atlassy-cli/src/main.rs` SHALL contain only the `Cli` struct (clap derive), `Commands` enum (clap subcommands), the `main()` function, and dispatch logic. It SHALL NOT contain domain logic, type definitions beyond clap structs, or helper functions.

#### Scenario: main.rs is under 150 lines
- **WHEN** `wc -l crates/atlassy-cli/src/main.rs` is run
- **THEN** the line count is 150 or fewer

#### Scenario: main.rs contains only CLI dispatch
- **WHEN** `crates/atlassy-cli/src/main.rs` is inspected
- **THEN** it contains the `Cli` struct with `#[derive(Parser)]`
- **THEN** it contains the `Commands` enum with `#[derive(Subcommand)]`
- **THEN** it contains `fn main()` that parses args and matches on `Commands` variants
- **THEN** it contains `use atlassy_cli::*;` to import from the library crate
- **THEN** it contains no other `fn` definitions besides `main`

#### Scenario: No test module declarations in main.rs
- **WHEN** `crates/atlassy-cli/src/main.rs` is inspected
- **THEN** it does NOT contain `#[cfg(test)] mod tests;` or `#[cfg(test)] mod test_helpers;`

### Requirement: Each module SHALL have a single primary responsibility

Each extracted module SHALL correspond to one concern. The module map SHALL be:

| Module | Responsibility |
|---|---|
| `types` | Shared struct/enum definitions used across CLI modules (~33 types: `BatchReport`, `RunManifest`, `KpiReport`, `ReadinessChecklist`, `DecisionPacket`, etc.) |
| `fixtures` | Demo page body and empty page body constants |
| `provenance` | `collect_provenance`, provenance stamp construction |
| `manifest` | `normalize_manifest`, `validate_manifest`, `run_mode_from_manifest` |
| `io` | `load_required_json`, `load_run_summary` |
| `batch/safety` | `assess_drift`, `assess_scenario_coverage`, `assess_safety` |
| `batch/kpi` | KPI flow groups, rollups, stats, checks, recommendations, outlier detection, regression summaries |
| `batch/report` | `rebuild_batch_report_from_artifacts`, `build_artifact_index`, `classify_run_from_summary`, `summarize_failure_classes` |
| `readiness/evidence` | `load_readiness_evidence`, `validate_readiness_evidence` |
| `readiness/gates` | `evaluate_readiness_gates` (7 gate evaluations) |
| `readiness/runbooks` | `build_operator_runbooks`, `known_runbook_section` |
| `readiness/decision_packet` | `assemble_decision_packet`, `build_risk_status_deltas`, `persist_decision_packet`, `replay_decision_packet`, `verify_decision_packet_replay` |
| `commands/run` | `run_single_request`, `map_live_startup_error` |
| `commands/run_batch` | `execute_batch_from_manifest_file`, `execute_manifest_runs` |
| `commands/run_readiness` | `generate_readiness_outputs_from_artifacts`, `ensure_readiness_unblocked` |
| `commands/create_subpage` | `create_subpage` handler |

#### Scenario: Batch modules contain only batch-related functions
- **WHEN** `crates/atlassy-cli/src/batch/report.rs` is inspected
- **THEN** it contains `rebuild_batch_report_from_artifacts`, `build_artifact_index`, `classify_run_from_summary`, and `summarize_failure_classes`
- **THEN** it contains no readiness, manifest, or command-dispatch logic

#### Scenario: Readiness modules contain only readiness-related functions
- **WHEN** `crates/atlassy-cli/src/readiness/gates.rs` is inspected
- **THEN** it contains `evaluate_readiness_gates` and its gate evaluation helpers
- **THEN** it contains no batch, KPI, or command-dispatch logic

#### Scenario: Command modules are thin orchestration
- **WHEN** `crates/atlassy-cli/src/commands/run_batch.rs` is inspected
- **THEN** it contains `execute_batch_from_manifest_file` which orchestrates calls to manifest, batch, and IO modules
- **THEN** it does not duplicate logic that belongs in batch or readiness modules

### Requirement: Types SHALL reside in a single types module

All shared struct and enum definitions (excluding clap-derive structs in `main.rs`) SHALL reside in `src/types.rs`. Types SHALL NOT be domain-split across multiple sub-modules.

#### Scenario: types.rs contains all shared definitions
- **WHEN** `crates/atlassy-cli/src/types.rs` is inspected
- **THEN** it contains `BatchReport`, `BatchRunDiagnostic`, `RunManifest`, `KpiReport`, `ReadinessChecklist`, `DecisionPacket`, and all other shared struct/enum definitions
- **THEN** no other module file under `src/` defines shared struct or enum types (private helper structs within a module are permitted)

#### Scenario: Clap-derive types stay in main.rs
- **WHEN** `crates/atlassy-cli/src/main.rs` is inspected
- **THEN** it contains the `Cli` struct and `Commands` enum with clap derive attributes
- **THEN** `src/types.rs` does NOT contain `Cli` or `Commands`

### Requirement: Batch and readiness modules SHALL use submodule directories

Batch modules SHALL be organized under `src/batch/` with a `mod.rs` barrel file. Readiness modules SHALL be organized under `src/readiness/` with a `mod.rs` barrel file. Command modules SHALL be organized under `src/commands/` with a `mod.rs` barrel file.

#### Scenario: Batch directory structure
- **WHEN** `crates/atlassy-cli/src/batch/` is listed
- **THEN** it contains `mod.rs`, `report.rs`, `kpi.rs`, `safety.rs`

#### Scenario: Readiness directory structure
- **WHEN** `crates/atlassy-cli/src/readiness/` is listed
- **THEN** it contains `mod.rs`, `evidence.rs`, `gates.rs`, `runbooks.rs`, `decision_packet.rs`

#### Scenario: Commands directory structure
- **WHEN** `crates/atlassy-cli/src/commands/` is listed
- **THEN** it contains `mod.rs`, `run.rs`, `run_batch.rs`, `run_readiness.rs`, `create_subpage.rs`

#### Scenario: Barrel files re-export submodule contents
- **WHEN** `crates/atlassy-cli/src/batch/mod.rs` is inspected
- **THEN** it contains `mod report;`, `mod kpi;`, `mod safety;` declarations
- **THEN** it contains `pub use` or `pub(crate) use` re-exports for each submodule's public items

### Requirement: Module dependency graph SHALL be acyclic

No module within the crate SHALL depend on another module that depends back on it. Dependencies SHALL flow in one direction.

#### Scenario: CLI module dependencies are acyclic
- **WHEN** the `atlassy-cli` module dependency graph is traced
- **THEN** `types` has zero intra-crate module dependencies
- **THEN** `fixtures` has zero intra-crate module dependencies
- **THEN** `provenance`, `manifest`, `io` depend only on `types`
- **THEN** `batch/safety` depends only on `types`
- **THEN** `batch/kpi` depends only on `types`
- **THEN** `batch/report` depends on `types`, `batch/kpi`, `batch/safety`, and `provenance`
- **THEN** `readiness/gates` depends only on `types`
- **THEN** `readiness/runbooks` depends only on `types`
- **THEN** `readiness/evidence` depends on `types`, `batch/report`, and `manifest`
- **THEN** `readiness/decision_packet` depends on `types`, `readiness/gates`, `readiness/runbooks`, and `batch/report`
- **THEN** `commands/*` modules depend on domain modules but domain modules do NOT depend on commands
- **THEN** no circular dependency exists

### Requirement: Public API surface SHALL be exposed via facade re-exports

Functions that `main()` dispatches to and types that appear in CLI output contracts SHALL be `pub` and re-exported from `lib.rs`. This is NOT visibility widening for test access — these functions ARE the CLI library's public interface.

#### Scenario: Entry-point functions are pub
- **WHEN** `crates/atlassy-cli/src/commands/run_batch.rs` is inspected
- **THEN** `execute_batch_from_manifest_file` is `pub fn`

#### Scenario: Output contract types are pub
- **WHEN** `crates/atlassy-cli/src/types.rs` is inspected
- **THEN** `BatchReport`, `DecisionPacket`, `ReadinessChecklist`, `KpiReport`, and other output types are `pub struct` or `pub enum`

#### Scenario: Internal helpers remain private
- **WHEN** `crates/atlassy-cli/src/commands/run.rs` is inspected
- **THEN** `map_live_startup_error` is `fn` (private), not `pub fn`

### Requirement: Module-internal visibility SHALL follow minimum-necessary principle

Functions that are only used within their module SHALL remain private. Functions called from other modules within the crate SHALL be `pub(crate)`. Functions that form the crate's public API (dispatched from `main()` or tested from `tests/`) SHALL be `pub`.

#### Scenario: Single-module helpers stay private
- **WHEN** `known_runbook_section` in `readiness/runbooks.rs` is only called within that module
- **THEN** it remains `fn` (private), not `pub(crate)` or `pub`

#### Scenario: Cross-module helpers become pub(crate)
- **WHEN** `classify_run_from_summary` in `batch/report.rs` is called from `commands/run_batch.rs`
- **THEN** it is `pub(crate) fn` (not `pub`, since it is not part of the external API)

### Requirement: Batch and readiness output schemas SHALL be preserved

Serialized JSON output from batch reporting and readiness assessment SHALL produce identical structure and field names before and after modularization.

#### Scenario: Batch report JSON is unchanged
- **WHEN** `rebuild_batch_report_from_artifacts` produces a `BatchReport`
- **THEN** serializing it to JSON produces the same schema as before modularization
- **THEN** all field names, nesting, and value types are identical

#### Scenario: Decision packet JSON is unchanged
- **WHEN** `assemble_decision_packet` produces a `DecisionPacket`
- **THEN** serializing it to JSON produces the same schema as before modularization

### Requirement: Quality gates SHALL pass after each extraction step

`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` SHALL all pass after each module extraction commit.

#### Scenario: All quality gates pass after each commit
- **WHEN** a module is extracted from `main.rs` into its own file
- **THEN** `cargo fmt --all -- --check` reports zero issues
- **THEN** `cargo clippy --workspace --all-targets -- -D warnings` reports zero warnings
- **THEN** `cargo test --workspace` passes all tests

#### Scenario: Extraction follows one-module-per-commit cadence
- **WHEN** the git log for the CLI modularization is inspected
- **THEN** each commit extracts at most one module (or closely related module group)
- **THEN** no commit leaves the workspace in a failing state
