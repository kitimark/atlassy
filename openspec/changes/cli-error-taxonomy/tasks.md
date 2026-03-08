## 1. Define ErrorClass enum with serde impls

- [ ] 1.1 Add `ErrorClass` enum to `crates/atlassy-cli/src/types.rs` with 6 variants (`Io`, `TelemetryIncomplete`, `ProvenanceIncomplete`, `RetryPolicy`, `RuntimeUnmappedHard`, `PipelineHard`) and derive `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`
- [ ] 1.2 Implement custom `Serialize` for `ErrorClass` producing stable strings (`"io"`, `"telemetry_incomplete"`, `"provenance_incomplete"`, `"retry_policy"`, `"runtime_unmapped_hard"`, `"pipeline_hard"`)
- [ ] 1.3 Implement custom `Deserialize` for `ErrorClass` parsing the same 6 strings back to variants
- [ ] 1.4 Add inline round-trip tests in `types.rs`: serialize each variant to JSON and deserialize back, verifying equality and exact string output

## 2. Define DiagnosticCode enum with serde impls

- [ ] 2.1 Add `DiagnosticCode` enum to `crates/atlassy-cli/src/types.rs` with 4 variants (`Pipeline(ErrorCode)`, `SummaryMissing`, `TelemetryIncomplete`, `ProvenanceMismatch`) and derive `Debug`, `Clone`, `PartialEq`, `Eq`
- [ ] 2.2 Implement custom `Serialize` for `DiagnosticCode` that flattens `Pipeline(ErrorCode)` to the inner `ErrorCode`'s `as_str()` output and serializes CLI-only variants to their `ERR_*` strings
- [ ] 2.3 Implement custom `Deserialize` for `DiagnosticCode` that checks CLI-only code strings first, then looks up pipeline codes via `ErrorCode::ALL.iter()` comparing `as_str()`, failing on unknown strings
- [ ] 2.4 Add inline round-trip tests in `types.rs`: serialize/deserialize each CLI-only variant, serialize/deserialize `Pipeline(ErrorCode)` for all 12 `ErrorCode` variants, verify `"ERR_SCOPE_MISS"` deserializes to `Pipeline(ErrorCode::ScopeMiss)` not a tagged struct, verify unknown string fails

## 3. Migrate BatchRunDiagnostic field types

- [ ] 3.1 Change `BatchRunDiagnostic.error_class` from `Option<String>` to `Option<ErrorClass>` and `error_code` from `Option<String>` to `Option<DiagnosticCode>` in `types.rs`
- [ ] 3.2 Re-export `ErrorClass` and `DiagnosticCode` from `lib.rs` so integration tests can access them

## 4. Migrate classify_run_from_summary

- [ ] 4.1 Replace the 7 string-literal `error_class` assignments in `batch/report.rs` `classify_run_from_summary` with `ErrorClass` variant construction
- [ ] 4.2 Replace the 3 CLI-only string-literal `error_code` assignments (`"ERR_SUMMARY_MISSING"`, `"ERR_TELEMETRY_INCOMPLETE"`, `"ERR_PROVENANCE_MISMATCH"`) with `DiagnosticCode` CLI-only variant construction
- [ ] 4.3 Replace the `"ERR_CONFLICT_RETRY_EXHAUSTED"` string literal (retry_count > 1 branch) with `DiagnosticCode::Pipeline(ErrorCode::ConflictRetryExhausted)`
- [ ] 4.4 Replace `ErrorCode::RuntimeUnmappedHard.to_string()` assignment with `DiagnosticCode::Pipeline(ErrorCode::RuntimeUnmappedHard)`
- [ ] 4.5 Replace `summary.error_codes.first().cloned()` passthrough (pipeline_hard fallthrough) with `ErrorCode::ALL` lookup that wraps the matched code in `DiagnosticCode::Pipeline`, falling back to `None` if no match

## 5. Migrate downstream consumers

- [ ] 5.1 Replace `.as_deref() == Some("pipeline_hard")` in `readiness/runbooks.rs:12` with `ErrorClass::PipelineHard` pattern match
- [ ] 5.2 Replace `.as_deref() == Some("retry_policy")` in `readiness/runbooks.rs:39` with `ErrorClass::RetryPolicy` pattern match
- [ ] 5.3 Replace `.as_deref() == Some("telemetry_incomplete")` in `readiness/runbooks.rs:97` with `ErrorClass::TelemetryIncomplete` pattern match
- [ ] 5.4 Remove the `matches!()` known-class guard (runbooks.rs:122-158) and the `unknown:{class}` fallback runbook emission — replace with exhaustive `match` on `ErrorClass` with explicit no-op arms for `Io`, `ProvenanceIncomplete`, `RuntimeUnmappedHard`
- [ ] 5.5 Replace `.as_deref() == Some("ERR_SCHEMA_INVALID")` in `readiness/decision_packet.rs:64` with `DiagnosticCode::Pipeline(ErrorCode::SchemaInvalid)` pattern match
- [ ] 5.6 Update `batch/report.rs` `summarize_failure_classes` to work with `ErrorClass` instead of `String` for the `error_class` field

## 6. Migrate tests

- [ ] 6.1 Update `tests/batch_report.rs` — replace 6 `.as_deref() == Some("...")` assertions with typed enum comparisons
- [ ] 6.2 Update `tests/readiness.rs` — replace any string-based error class/code assertions with typed comparisons
- [ ] 6.3 Update inline test in `commands/run.rs` if it references error class or code strings

## 7. Quality gates

- [ ] 7.1 Run `cargo fmt --all -- --check` and fix any formatting issues
- [ ] 7.2 Run `cargo clippy --workspace --all-targets -- -D warnings` and fix any warnings
- [ ] 7.3 Run `cargo test --workspace` and verify all tests pass including `verify_decision_packet_replay` roundtrip parity
- [ ] 7.4 Verify no `"ERR_*"` string literals remain in CLI production code outside serde impls (grep check)
- [ ] 7.5 Verify no `.as_deref() == Some("...")` patterns remain in CLI production code (grep check)
