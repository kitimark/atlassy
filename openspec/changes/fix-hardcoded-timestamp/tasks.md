## 1. Add chrono dependency

- [x] 1.1 Add `chrono` to `[workspace.dependencies]` in workspace `Cargo.toml` with features `clock` and `serde`
- [x] 1.2 Add `chrono.workspace = true` to `[dependencies]` in `crates/atlassy-cli/Cargo.toml`

## 2. Replace hardcoded timestamps

- [x] 2.1 In `crates/atlassy-cli/src/commands/run.rs`, add `use chrono::Utc` and replace the hardcoded `"2026-03-06T10:00:00Z"` at line 113 with `Utc::now().to_rfc3339()`
- [x] 2.2 In `crates/atlassy-cli/src/types.rs`, update `default_manifest_timestamp()` at line 563 to return `Utc::now().to_rfc3339()` instead of `"1970-01-01T00:00:00Z"`

## 3. Verify test determinism

- [x] 3.1 Confirm hardcoded timestamps in `crates/atlassy-pipeline/tests/pipeline_integration.rs` and `crates/atlassy-contracts/tests/contract_validation.rs` are unchanged
- [x] 3.2 Run `make test` and verify all 154 tests pass

## 4. Lint and format

- [x] 4.1 Run `make fmt` to format changed files
- [x] 4.2 Run `make lint` and verify zero clippy warnings
