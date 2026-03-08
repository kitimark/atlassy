## Context

The pipeline layer classifies errors through a typed `ErrorCode` enum (12 variants in `atlassy-contracts`). The CLI layer classifies errors through `Option<String>` fields on `BatchRunDiagnostic`, using two independent vocabularies:

- `error_class` — 6 operational categories (why the run failed)
- `error_code` — specific error identifiers mixing pipeline codes (`ERR_SCOPE_MISS`) and CLI-only codes (`ERR_SUMMARY_MISSING`)

These two vocabularies serve different purposes and cross a serialization boundary (`RunSummary.error_codes: Vec<String>` on disk). The typed enums introduced here live entirely within the CLI crate — they don't cross the pipeline/CLI wire boundary.

Current data flow:

```
Pipeline                          Wire                              CLI
ErrorCode enum ──.to_string()──▶ Vec<String> ──.first().cloned()──▶ Option<String>
(12 variants)                    (JSON on disk)                     error_code
                                                                    error_class
```

After this change:

```
Pipeline                          Wire                              CLI
ErrorCode enum ──.to_string()──▶ Vec<String> ──.first()──────────▶ Option<DiagnosticCode>
(12 variants)                    (JSON on disk)  + ErrorCode        Option<ErrorClass>
                                                   lookup
```

## Goals / Non-Goals

**Goals:**
- Replace all string-literal error classification in CLI production code with typed enums
- Make the compiler enforce exhaustive handling when new error classes are added
- Eliminate the `matches!()` known-class guard and `unknown:{class}` fallback pattern in runbooks
- Preserve identical JSON serialization output (wire-compatible)
- Preserve `verify_decision_packet_replay` roundtrip parity

**Non-Goals:**
- Changing `RunSummary.error_codes: Vec<String>` to `Vec<ErrorCode>` (deferred — requires `Deserialize` on `ErrorCode` with unknown-variant tolerance)
- Adding `Deserialize` to `ErrorCode` in `atlassy-contracts`
- Changing pipeline error handling or adding new error classifications
- Modifying CLI behavior — this is a structure-only refactor

## Decisions

### Decision 1: Closed enums without Unknown variant

**Choice**: Both `ErrorClass` (6 variants) and `DiagnosticCode` (4 variants) are closed enums with no `Unknown(String)` catch-all.

**Rationale**: `BatchRunDiagnostic` is produced and consumed within the same CLI binary during a single batch run. `classify_run_from_summary` produces values, `runbooks.rs` and `decision_packet.rs` consume them. There is no cross-version deserialization path — `verify_decision_packet_replay` rebuilds the packet from the same code that created it.

An `Unknown(String)` variant would be [Speculative Generality](https://refactoring.guru/smells/speculative-generality) — it would never be constructed in production and would undermine exhaustive `match` by requiring catch-all arms everywhere, reintroducing the exact fragility this change eliminates.

**Alternative rejected**: Open enum with `Unknown(String)` for version-skew resilience. Unnecessary because all producers and consumers compile together in the same workspace.

### Decision 2: Pipeline(ErrorCode) denotes namespace, not provenance

**Choice**: `DiagnosticCode::Pipeline(ErrorCode)` means "this code is defined in the pipeline `ErrorCode` vocabulary," not "this value was read from `summary.error_codes`."

**Rationale**: `classify_run_from_summary` assigns `ERR_CONFLICT_RETRY_EXHAUSTED` at two sites — once as a pipeline passthrough (fallthrough at line 335), once as a CLI policy decision (retry_count > 1 at line 280). Both reference the same `ErrorCode::ConflictRetryExhausted` variant. The wrapper type describes which namespace the code belongs to, not how it arrived.

The 3 CLI-only variants (`SummaryMissing`, `TelemetryIncomplete`, `ProvenanceMismatch`) are codes that have no `ErrorCode` counterpart — they exist only in the CLI vocabulary. The `Pipeline(ErrorCode)` wrapper is the complement: codes that exist in the pipeline vocabulary, regardless of how the CLI obtained the value.

**Alternative rejected**: A 4th CLI-only variant `RetryExhausted` that serializes to the same string as `ErrorCode::ConflictRetryExhausted`. Creates semantic overlap — two distinct enum values producing identical output — and obscures the fact that the code is already defined in the pipeline namespace.

### Decision 3: Custom Serialize/Deserialize, not derive

**Choice**: Both enums use hand-written `Serialize` and `Deserialize` impls.

**Rationale**: Default serde derive would produce tagged representations (`{"Pipeline": "ERR_SCOPE_MISS"}` instead of `"ERR_SCOPE_MISS"`). Wire compatibility requires flat string output matching current literals.

`ErrorClass` serialization is a direct variant-to-string map:

| Variant | Serialized string |
|---|---|
| `Io` | `"io"` |
| `TelemetryIncomplete` | `"telemetry_incomplete"` |
| `ProvenanceIncomplete` | `"provenance_incomplete"` |
| `RetryPolicy` | `"retry_policy"` |
| `RuntimeUnmappedHard` | `"runtime_unmapped_hard"` |
| `PipelineHard` | `"pipeline_hard"` |

`DiagnosticCode` serialization flattens `Pipeline(ErrorCode)` to the inner code's string:

| Variant | Serialized string |
|---|---|
| `Pipeline(ErrorCode::ScopeMiss)` | `"ERR_SCOPE_MISS"` |
| `Pipeline(ErrorCode::ConflictRetryExhausted)` | `"ERR_CONFLICT_RETRY_EXHAUSTED"` |
| `SummaryMissing` | `"ERR_SUMMARY_MISSING"` |
| `TelemetryIncomplete` | `"ERR_TELEMETRY_INCOMPLETE"` |
| `ProvenanceMismatch` | `"ERR_PROVENANCE_MISMATCH"` |

`DiagnosticCode` deserialization parses strings by checking CLI-only codes first, then looking up `ErrorCode::ALL` for pipeline codes. No `Deserialize` is added to `ErrorCode` itself — `DiagnosticCode` owns the parsing logic internally using `ErrorCode::ALL` and `as_str()`.

### Decision 4: Exhaustive match replaces matches!() guard in runbooks

**Choice**: The `matches!()` known-class guard (runbooks.rs:124-131) and the `unknown:{class}` fallback pattern are replaced by an exhaustive `match` on `ErrorClass` that handles each variant explicitly.

**Rationale**: The current guard has two defects: (1) a redundant dead-code condition (`pipeline_hard` is in the `matches!()` arm AND in the `||` condition), and (2) three classes (`Io`, `ProvenanceIncomplete`, `RuntimeUnmappedHard`) silently fall through to generic fallback runbooks. With a closed `ErrorClass` enum, the compiler forces every variant to be handled — no silent omission, no manual tracking of "known" classes.

For the 3 currently-unmapped classes, the explicit handling decisions are:

- `Io` — no specific runbook section (infrastructure issue; the generic "failed" status is sufficient)
- `ProvenanceIncomplete` — no specific runbook section (caught by the `provenance_complete` gate check; runbook coverage comes from the gate, not the error class)
- `RuntimeUnmappedHard` — no specific runbook section (by definition, the error is unmapped; routing to manual review is the correct response, but this is already handled by the existing safety and gate mechanisms)

These classes are explicitly skipped in the match (not ignored via wildcard), making the "no runbook" decision visible and deliberate.

### Decision 5: Migration order preserves test stability

**Choice**: Migrate in four steps — (1) define enums with serde impls and round-trip tests, (2) change `BatchRunDiagnostic` field types, (3) migrate `classify_run_from_summary`, (4) migrate downstream consumers.

**Rationale**: Step 1 establishes the type definitions and validates serialization before any production code changes. Step 2 changes the struct fields, which propagates compiler errors to every site that constructs or reads `BatchRunDiagnostic` — making the remaining migration mechanically guided by the compiler. Steps 3 and 4 fix those compiler errors. Existing test assertions serve as regression guards throughout.

## Risks / Trade-offs

- **[Risk] Silent reclassification during migration** → Existing test assertions (`tests/batch_report.rs`, `tests/readiness.rs`) are preserved unchanged before type migration begins. Any reclassification produces a test failure before the tests are updated.
- **[Risk] Serialization regression breaks `verify_decision_packet_replay`** → Round-trip tests for both enums are written before `BatchRunDiagnostic` fields change. The replay test itself serves as an integration-level guard.
- **[Risk] `DiagnosticCode` deserialization fails on unknown pipeline codes** → Cannot happen in practice: all crates compile together in the same workspace. Documented as a known limitation if crates are ever split into separate build units.
- **[Trade-off] Three error classes get no specific runbook section** → Explicit "no runbook" match arms make this deliberate rather than accidental. The current behavior is identical (fallback runbooks say "route to manual review"), but the decision is now visible in code.
- **[Trade-off] `ErrorCode` has no `Deserialize` or `FromStr`** → `DiagnosticCode` implements its own string-to-`ErrorCode` lookup via `ErrorCode::ALL.iter()`. This duplicates some knowledge but avoids adding a deserialization failure mode to the contracts crate.
