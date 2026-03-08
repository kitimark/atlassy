## Context

Error codes in the workspace are 12 `&str` constants in `atlassy-contracts/src/constants.rs` (lines 4-15). `PipelineError::Hard` carries error codes as `String`:

```rust
PipelineError::Hard {
    state: PipelineState,
    code: String,        // ← this becomes ErrorCode
    message: String,
}
```

Every construction site calls `ERR_*.to_string()` to populate `code`. The CLI reads `RunSummary.error_codes: Vec<String>` and compares against the same `&str` constants.

The pipeline also has `to_hard_error()` which accepts `impl Display`, stringifies `AdfError`, and sniffs substrings to assign error codes. Every caller passes `AdfError` — the type is known but erased.

## Goals / Non-Goals

**Goals:**

- Replace 12 `&str` error code constants with an `ErrorCode` enum that produces identical string representations.
- Change `PipelineError::Hard.code` from `String` to `ErrorCode`.
- Narrow `to_hard_error` from `impl Display` to `AdfError`, eliminating string-sniffing.
- Keep all JSON output (particularly `RunSummary`) byte-identical.

**Non-Goals:**

- Typing the CLI's own error classifications (`ERR_SUMMARY_MISSING`, `ERR_TELEMETRY_INCOMPLETE`, `ERR_PROVENANCE_MISMATCH`) — these are CLI-level string literals, not pipeline error codes. Deferred to Phase 4 `ErrorClass` enum.
- Changing `ErrorInfo.code` (in `Diagnostics`) from `String` — it's a serialized diagnostics type with different lifecycle.
- Changing `RunSummary.error_codes` from `Vec<String>` — it's a serialized output contract.

## Decisions

### 1. ErrorCode enum location and naming

**Decision**: Define `ErrorCode` in `atlassy-contracts/src/constants.rs`, replacing the 12 `&str` constants. Variant names strip the `ERR_` prefix and use PascalCase.

```
ERR_SCOPE_MISS              → ErrorCode::ScopeMiss
ERR_ROUTE_VIOLATION         → ErrorCode::RouteViolation
ERR_SCHEMA_INVALID          → ErrorCode::SchemaInvalid
ERR_OUT_OF_SCOPE_MUTATION   → ErrorCode::OutOfScopeMutation
ERR_LOCKED_NODE_MUTATION    → ErrorCode::LockedNodeMutation
ERR_TABLE_SHAPE_CHANGE      → ErrorCode::TableShapeChange
ERR_CONFLICT_RETRY_EXHAUSTED → ErrorCode::ConflictRetryExhausted
ERR_RUNTIME_BACKEND         → ErrorCode::RuntimeBackend
ERR_RUNTIME_UNMAPPED_HARD   → ErrorCode::RuntimeUnmappedHard
ERR_BOOTSTRAP_REQUIRED      → ErrorCode::BootstrapRequired
ERR_BOOTSTRAP_INVALID_STATE → ErrorCode::BootstrapInvalidState
ERR_TARGET_DISCOVERY_FAILED → ErrorCode::TargetDiscoveryFailed
```

**Rationale**: Keeping the enum in `constants.rs` means import paths don't change for consumers. `&str` constants for non-error values (`FLOW_*`, `PATTERN_*`, `RUNTIME_*`) stay as-is — they're not error codes.

**Alternative considered**: Separate `error_codes.rs` module in contracts. Rejected — adds a file for one enum. Constants module already holds these values.

### 2. Display and Serialize produce original strings

**Decision**: `Display` impl returns the `ERR_*` string (e.g., `ErrorCode::ScopeMiss.to_string()` returns `"ERR_SCOPE_MISS"`). `Serialize` delegates to `Display` so JSON output is identical.

**Rationale**: `RunSummary.error_codes` is `Vec<String>` populated via `.to_string()`. CLI tests and batch report consumers compare against these strings. Changing the string representation would break output contracts.

**Implementation**: Manual `Display` impl with match arms (same pattern as `PipelineState::as_str()`). Add `as_str(&self) -> &'static str` method. Serialize via `#[serde(serialize_with = ...)]` or a custom impl that calls `as_str()`.

### 3. Derive traits

**Decision**: `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]` plus manual `Display` and `Serialize`.

**Rationale**: `Copy` because it's a fieldless enum. `Hash` enables use as map key if needed. `PartialEq` for direct comparison (replacing `code == ERR_*` string comparisons). No `Deserialize` — error codes flow outward (construction → serialization), never inward from JSON.

### 4. to_hard_error narrows to AdfError

**Decision**: Change `to_hard_error` signature from `fn to_hard_error(state: PipelineState, error: impl Display) -> PipelineError` to `fn to_hard_error(state: PipelineState, error: AdfError) -> PipelineError`. Replace substring matching with variant matching:

```
AdfError::OutOfScope(_)              → ErrorCode::OutOfScopeMutation
AdfError::WholeBodyRewriteDisallowed → ErrorCode::RouteViolation
AdfError::ScopeResolutionFailed      → ErrorCode::ScopeMiss
AdfError::InvalidSelector(_)         → ErrorCode::SchemaInvalid
AdfError::InvalidPath(_)             → ErrorCode::SchemaInvalid
AdfError::DuplicatePath(_)           → ErrorCode::SchemaInvalid
AdfError::MappingIntegrity(_)        → ErrorCode::SchemaInvalid
AdfError::TargetDiscoveryFailed{..}  → ErrorCode::SchemaInvalid
```

**Rationale**: Every caller already passes `AdfError`. The `impl Display` signature erases type information that's needed for correct classification. The current substring matching is fragile — `"out of scope"` must be checked before `"scope"`, and any upstream wording change silently reclassifies errors. Variant matching is exhaustive and compiler-verified.

**Note**: `TargetDiscoveryFailed` maps to `SchemaInvalid` here, not `TargetDiscoveryFailed`, because direct `discover_target_path` call sites already construct `PipelineError::Hard` with `ErrorCode::TargetDiscoveryFailed` explicitly. The `to_hard_error` path for `TargetDiscoveryFailed` is only reachable if the error is propagated generically (not currently the case), so `SchemaInvalid` is the safe fallback matching current behavior.

### 5. From impls stay but become typed

**Decision**: Keep `From<AdfError> for PipelineError` and `From<ConfluenceError> for PipelineError`. Both already delegate to `to_hard_error` / `confluence_error_to_hard_error` respectively.

**Rationale**: They're compatible with the narrowed signatures. They hardcode a default state (`Patch` for AdfError, `Fetch` for ConfluenceError) which is imprecise, but no call site uses them via `?` — all use explicit `.map_err()` with the correct state. Removing them is optional cleanup, not required for this change.

### 6. CLI bridge: .to_string() on ErrorCode

**Decision**: Where the CLI constructs `PipelineError::Hard` (one site, `main.rs:611`), change `code: ERR_RUNTIME_BACKEND.to_string()` to `code: ErrorCode::RuntimeBackend`. Where CLI compares against `summary.error_codes` entries, no change needed — those are already `Vec<String>` comparisons against `ERR_*` constants, and the constants continue to exist as `ErrorCode::*.to_string()`.

CLI comparison sites like `summary.error_codes.iter().any(|code| code == ERR_RUNTIME_UNMAPPED_HARD)` currently compare `String` against `&str`. After the change, these compare `String` against... nothing, because the `ERR_*` constants no longer exist. These sites need to change to `code == ErrorCode::RuntimeUnmappedHard.as_str()`.

## Risks / Trade-offs

- **[Risk] Missed comparison site** → Compiler catches: removing `&str` constants causes compile errors at every use site. No silent drift possible.
- **[Risk] Serde output changes** → Mitigated by `as_str()` returning identical strings. Integration tests in `pipeline_integration.rs` verify serialized output.
- **[Trade-off] No Deserialize** → Error codes are never deserialized from JSON in application code. If needed later, add `Deserialize` with `#[serde(rename_all = "SCREAMING_SNAKE_CASE")]` or a custom impl. Not adding it now avoids defining behavior that isn't tested.
- **[Trade-off] Manual Display impl** → `strum` crate could derive this, but adding a dependency for one enum is not justified. Manual match has 12 arms and is trivially correct.
