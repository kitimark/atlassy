## ADDED Requirements

### Requirement: Error classes are a closed typed enum
The CLI SHALL represent operational error classifications as variants of an `ErrorClass` enum defined in the CLI `types` module. The enum MUST contain exactly 6 variants: `Io`, `TelemetryIncomplete`, `ProvenanceIncomplete`, `RetryPolicy`, `RuntimeUnmappedHard`, `PipelineHard`. The enum MUST NOT include a catch-all or unknown variant.

#### Scenario: Exhaustive match on error classes
- **WHEN** code matches on an `ErrorClass` value
- **THEN** the compiler SHALL require all 6 variants to be handled, preventing silent omission of error classes

#### Scenario: Construction requires a valid variant
- **WHEN** a `BatchRunDiagnostic` is constructed with an error class
- **THEN** the `error_class` field MUST be `Option<ErrorClass>`, and the compiler SHALL reject arbitrary strings

### Requirement: Diagnostic codes are a closed typed enum
The CLI SHALL represent diagnostic error codes as variants of a `DiagnosticCode` enum defined in the CLI `types` module. The enum MUST contain exactly 4 variants: `Pipeline(ErrorCode)`, `SummaryMissing`, `TelemetryIncomplete`, `ProvenanceMismatch`. The `Pipeline` variant wraps any `ErrorCode` variant from `atlassy-contracts`, denoting namespace membership in the pipeline error vocabulary.

#### Scenario: Pipeline error code passthrough
- **WHEN** `classify_run_from_summary` assigns an error code that exists in the pipeline `ErrorCode` vocabulary
- **THEN** the diagnostic code MUST be `DiagnosticCode::Pipeline(ErrorCode::*)` regardless of whether the code was read from `summary.error_codes` or assigned by CLI policy

#### Scenario: CLI-only error code assignment
- **WHEN** `classify_run_from_summary` assigns an error code that has no `ErrorCode` counterpart
- **THEN** the diagnostic code MUST be one of the 3 CLI-only variants (`SummaryMissing`, `TelemetryIncomplete`, `ProvenanceMismatch`)

#### Scenario: Retry exhaustion uses pipeline namespace
- **WHEN** `classify_run_from_summary` classifies a run with `retry_count > 1`
- **THEN** the diagnostic code MUST be `DiagnosticCode::Pipeline(ErrorCode::ConflictRetryExhausted)`, not a CLI-only variant

### Requirement: ErrorClass serialization produces stable strings
Each `ErrorClass` variant SHALL serialize to its current string representation via a custom `Serialize` impl. The serialized output MUST match the string literals currently used in `classify_run_from_summary`.

#### Scenario: Serialize Io variant
- **WHEN** `ErrorClass::Io` is serialized to JSON
- **THEN** the JSON string value MUST be `"io"`

#### Scenario: Serialize PipelineHard variant
- **WHEN** `ErrorClass::PipelineHard` is serialized to JSON
- **THEN** the JSON string value MUST be `"pipeline_hard"`

#### Scenario: Deserialize roundtrip
- **WHEN** an `ErrorClass` variant is serialized to JSON and deserialized back
- **THEN** the result MUST equal the original variant

### Requirement: DiagnosticCode serialization flattens Pipeline variant
Each `DiagnosticCode` variant SHALL serialize to a flat `ERR_*` string via a custom `Serialize` impl. The `Pipeline(ErrorCode)` variant MUST serialize as the inner `ErrorCode`'s string representation, not as a tagged enum structure.

#### Scenario: Serialize Pipeline variant
- **WHEN** `DiagnosticCode::Pipeline(ErrorCode::ScopeMiss)` is serialized to JSON
- **THEN** the JSON string value MUST be `"ERR_SCOPE_MISS"`, not `{"Pipeline":"ERR_SCOPE_MISS"}`

#### Scenario: Serialize CLI-only variant
- **WHEN** `DiagnosticCode::SummaryMissing` is serialized to JSON
- **THEN** the JSON string value MUST be `"ERR_SUMMARY_MISSING"`

#### Scenario: Deserialize roundtrip
- **WHEN** a `DiagnosticCode` variant is serialized to JSON and deserialized back
- **THEN** the result MUST equal the original variant

#### Scenario: Deserialize pipeline code from string
- **WHEN** the string `"ERR_SCOPE_MISS"` is deserialized as a `DiagnosticCode`
- **THEN** the result MUST be `DiagnosticCode::Pipeline(ErrorCode::ScopeMiss)`

### Requirement: DiagnosticCode deserialization uses ErrorCode::ALL lookup
The `DiagnosticCode` `Deserialize` impl SHALL check CLI-only code strings first, then look up pipeline codes by iterating `ErrorCode::ALL` and comparing `as_str()` output. The `Deserialize` impl MUST NOT add a `Deserialize` or `FromStr` impl to `ErrorCode` in `atlassy-contracts`.

#### Scenario: CLI-only code takes precedence
- **WHEN** the string `"ERR_SUMMARY_MISSING"` is deserialized as a `DiagnosticCode`
- **THEN** the result MUST be `DiagnosticCode::SummaryMissing`, not a pipeline lookup failure

#### Scenario: Pipeline code matched via ALL array
- **WHEN** the string `"ERR_CONFLICT_RETRY_EXHAUSTED"` is deserialized as a `DiagnosticCode`
- **THEN** the result MUST be `DiagnosticCode::Pipeline(ErrorCode::ConflictRetryExhausted)` via `ErrorCode::ALL` iteration

#### Scenario: Unknown string fails deserialization
- **WHEN** a string that matches neither CLI-only codes nor any `ErrorCode::ALL` entry is deserialized as a `DiagnosticCode`
- **THEN** deserialization MUST fail with an error

### Requirement: BatchRunDiagnostic uses typed fields
The `BatchRunDiagnostic` struct SHALL use `Option<ErrorClass>` for its `error_class` field and `Option<DiagnosticCode>` for its `error_code` field. The struct MUST retain `Serialize`, `Deserialize`, `PartialEq`, `Debug`, and `Clone` derives or impls.

#### Scenario: JSON output is wire-compatible
- **WHEN** a `BatchRunDiagnostic` with typed fields is serialized to JSON
- **THEN** the JSON output MUST be byte-identical to the output produced by the previous `Option<String>` fields for the same logical values

#### Scenario: Decision packet replay parity
- **WHEN** `verify_decision_packet_replay` rebuilds a decision packet from stored artifacts
- **THEN** the rebuilt packet MUST equal the stored packet via `PartialEq`

### Requirement: No string-literal error classification in production code
After migration, CLI production code (excluding serde impls and tests) MUST NOT contain string literals used for error class or error code assignment or comparison. All classification MUST use `ErrorClass` and `DiagnosticCode` enum variants directly.

#### Scenario: No .as_deref() error class comparisons
- **WHEN** production code checks an error class value
- **THEN** the check MUST use pattern matching on `ErrorClass` variants, not `.as_deref() == Some("...")`

#### Scenario: No string-literal error code assignments
- **WHEN** `classify_run_from_summary` assigns an error code
- **THEN** the assignment MUST use a `DiagnosticCode` variant, not a string literal
