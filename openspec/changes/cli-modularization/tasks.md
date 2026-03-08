## 1. Baseline and Scaffold

- [ ] 1.1 Record test count baseline: `grep -r '#\[test\]' crates/ --include='*.rs' | wc -l` (expect 107)
- [ ] 1.2 Run quality gates to confirm green starting state: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
- [ ] 1.3 Create `src/lib.rs` as empty facade; create `src/types.rs` with all 33 shared struct/enum definitions moved from `main.rs` (lines 25-470, excluding `Cli`/`Commands`/`CliMode`/`RuntimeBackend`); add `mod types; pub use types::*;` to `lib.rs`; add `use atlassy_cli::*;` to `main.rs`; run quality gates
- [ ] 1.4 Extract `src/fixtures.rs` (demo page body, empty page body constants); add `mod fixtures; pub use fixtures::*;` to `lib.rs`; run quality gates

## 2. Foundation Modules

- [ ] 2.1 Extract `src/provenance.rs` (`collect_provenance` and related functions); add to `lib.rs`; run quality gates
- [ ] 2.2 Extract `src/manifest.rs` (`normalize_manifest`, `validate_manifest`, `run_mode_from_manifest`); add to `lib.rs`; run quality gates
- [ ] 2.3 Extract `src/io.rs` (`load_required_json`, `load_run_summary`); add to `lib.rs`; run quality gates

## 3. Batch Domain Modules

- [ ] 3.1 Create `src/batch/` directory with `mod.rs` barrel file
- [ ] 3.2 Extract `src/batch/safety.rs` (`assess_drift`, `assess_scenario_coverage`, `assess_safety`); add to batch barrel; run quality gates
- [ ] 3.3 Extract `src/batch/kpi.rs` (flow groups, rollups, stats, checks, recommendations, outlier detection, regression summaries); add to batch barrel; run quality gates
- [ ] 3.4 Extract `src/batch/report.rs` (`rebuild_batch_report_from_artifacts`, `build_artifact_index`, `classify_run_from_summary`, `summarize_failure_classes`); add to batch barrel; run quality gates

## 4. Readiness Domain Modules

- [ ] 4.1 Create `src/readiness/` directory with `mod.rs` barrel file
- [ ] 4.2 Extract `src/readiness/gates.rs` (`evaluate_readiness_gates` and 7 gate evaluation helpers); add to readiness barrel; run quality gates
- [ ] 4.3 Extract `src/readiness/runbooks.rs` (`build_operator_runbooks`, `known_runbook_section`); add to readiness barrel; run quality gates
- [ ] 4.4 Extract `src/readiness/evidence.rs` (`load_readiness_evidence`, `validate_readiness_evidence`); add to readiness barrel; run quality gates
- [ ] 4.5 Extract `src/readiness/decision_packet.rs` (`assemble_decision_packet`, `build_risk_status_deltas`, `persist_decision_packet`, `replay_decision_packet`, `verify_decision_packet_replay`); add to readiness barrel; run quality gates

## 5. Command Modules

- [ ] 5.1 Create `src/commands/` directory with `mod.rs` barrel file
- [ ] 5.2 Extract `src/commands/run.rs` (`run_single_request`, `map_live_startup_error`); add to commands barrel; run quality gates
- [ ] 5.3 Extract `src/commands/create_subpage.rs` (`create_subpage` handler); add to commands barrel; run quality gates
- [ ] 5.4 Extract `src/commands/run_batch.rs` (`execute_batch_from_manifest_file`, `execute_manifest_runs`); add to commands barrel; run quality gates
- [ ] 5.5 Extract `src/commands/run_readiness.rs` (`generate_readiness_outputs_from_artifacts`, `ensure_readiness_unblocked`); add to commands barrel; run quality gates

## 6. Slim main.rs

- [ ] 6.1 Slim `main.rs` to dispatch only: `Cli` struct, `Commands` enum, `main()` function with match on subcommands; remove all remaining function bodies and type definitions; verify under 150 lines; run quality gates
- [ ] 6.2 Remove `#[cfg(test)] mod tests;` and `#[cfg(test)] mod test_helpers;` declarations from `main.rs` (tests will temporarily not compile — proceed directly to step 7)

## 7. Test Migration

- [ ] 7.1 Create `tests/common/mod.rs` with `fixture_path` helper function (moved from `src/test_helpers.rs`)
- [ ] 7.2 Create `tests/batch_report.rs` with tests 2-9 (batch execution, report generation, artifact rebuild, error classification); use `use atlassy_cli::*;` imports and `mod common;` for fixture helper
- [ ] 7.3 Create `tests/readiness.rs` with tests 10-15 (gate evaluation, runbook generation, decision packet assembly, replay verification); use `use atlassy_cli::*;` imports and `mod common;` for fixture helper
- [ ] 7.4 Move test 1 (`live_startup_errors_map_to_runtime_backend_hard_error`) to inline `#[cfg(test)] mod tests` block in `src/commands/run.rs`
- [ ] 7.5 Delete `src/tests.rs` and `src/test_helpers.rs`
- [ ] 7.6 Run quality gates; verify test count matches baseline (107)

## 8. Finalize

- [ ] 8.1 Verify `lib.rs` is a clean facade: only `mod` declarations and `pub use` re-exports, no function bodies
- [ ] 8.2 Verify `main.rs` is under 150 lines with only clap types and dispatch
- [ ] 8.3 Verify no `src/tests.rs` or `src/test_helpers.rs` exist
- [ ] 8.4 Verify module dependency graph is acyclic (no circular `use` between modules)
- [ ] 8.5 Run full quality gates one final time: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
- [ ] 8.6 Verify test count is preserved: `grep -r '#\[test\]' crates/ --include='*.rs' | wc -l` matches baseline (107)
