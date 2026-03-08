## Why

The CLI diagnostic layer uses 15 string-based error classification points (`Option<String>` fields on `BatchRunDiagnostic`) while the pipeline layer is fully typed (`ErrorCode` enum, Phase 2 complete). This creates [Primitive Obsession](https://refactoring.guru/smells/primitive-obsession) and [Shotgun Surgery](https://refactoring.guru/smells/shotgun-surgery) — adding a new error class requires coordinated string-literal changes across `classify_run_from_summary`, runbook routing, risk deltas, the `matches!()` known-class guard, and tests, with no compiler enforcement. A redundant dead-code condition in the runbooks guard (`pipeline_hard` checked twice) and 3 error classes silently falling through to `unknown:{class}` fallback runbooks are symptoms of this fragility.

## What Changes

- Introduce `ErrorClass` enum (6 variants: `Io`, `TelemetryIncomplete`, `ProvenanceIncomplete`, `RetryPolicy`, `RuntimeUnmappedHard`, `PipelineHard`) in CLI `types` module via [Replace Type Code with Class](https://refactoring.guru/replace-type-code-with-class). Custom `Serialize` producing current string values for wire compatibility.
- Introduce `DiagnosticCode` enum (4 variants: `Pipeline(ErrorCode)`, `SummaryMissing`, `TelemetryIncomplete`, `ProvenanceMismatch`) in CLI `types` module. `Pipeline(ErrorCode)` denotes namespace membership (code defined in pipeline vocabulary), not provenance (where the value was read from). Custom `Serialize` that flattens `Pipeline(ErrorCode)` to the inner `ERR_*` string.
- Migrate `BatchRunDiagnostic.error_class` from `Option<String>` to `Option<ErrorClass>` and `error_code` from `Option<String>` to `Option<DiagnosticCode>`.
- Migrate `classify_run_from_summary` to construct typed enum values — eliminates 7 string-literal `error_class` assignments and 5 string-literal `error_code` assignments.
- Migrate downstream consumers in `runbooks.rs` and `decision_packet.rs` — eliminates 4 `.as_deref() == Some("...")` comparisons in production code and the `matches!()` known-class guard. Exhaustive `match` on `ErrorClass` replaces the manual fallback routing.
- `RunSummary.error_codes: Vec<String>` is unchanged (version-skew resilience at the pipeline/CLI boundary).
- Both enums are closed (no `Unknown` variant) — the same CLI binary that produces values also consumes them, so version-skew tolerance is unnecessary and would undermine exhaustive match enforcement.

## Capabilities

### New Capabilities
- `cli-diagnostic-classification`: Typed enum classification for CLI-layer error diagnostics (`ErrorClass`, `DiagnosticCode`), including serialization contracts and the `BatchRunDiagnostic` field migration.

### Modified Capabilities
- `operator-triage-runbooks`: The `matches!()` known-class guard and `unknown:{class}` fallback pattern are replaced by exhaustive `match` on `ErrorClass`. Three currently-unmapped classes (`Io`, `ProvenanceIncomplete`, `RuntimeUnmappedHard`) require explicit runbook decisions instead of silently falling through to generic fallback sections. The redundant dead-code condition on `pipeline_hard` is eliminated by the restructure.

## Impact

- **Code**: `crates/atlassy-cli/src/types.rs` (enum definitions, `BatchRunDiagnostic` field types), `batch/report.rs` (classify function), `readiness/runbooks.rs` (match restructure, fallback elimination), `readiness/decision_packet.rs` (risk delta comparison). Test files: `tests/batch_report.rs` (6 string comparison updates).
- **Wire format**: `BatchRunDiagnostic` JSON serialization must produce identical output — `Serialize` impls produce the same strings as current literals. `Deserialize` impls must roundtrip for `verify_decision_packet_replay` parity.
- **Dependencies**: No crate dependency changes. `ErrorCode` in `atlassy-contracts` is unchanged. No new `Deserialize` impl on `ErrorCode`.
- **Risk**: Silent error reclassification during migration. Mitigated by preserving existing test assertions as regression guards before changing types.
