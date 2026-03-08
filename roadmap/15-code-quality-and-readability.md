# Code Quality and Readability

## Objective

Improve code quality through behavior-preserving refactor, with explicit focus on:

- human-readability (maintainability, onboarding speed, safer edits)
- AI-readability (deterministic structure, low context bloat, clear boundaries)

The program targets source structure and test architecture, not feature expansion.

## Current State (2026-03-09)

All phases complete (1-4): leaf crate modularization, ErrorCode enum, pipeline modularization, CLI modularization, error taxonomy.

- No remaining monolithic hotspots.
- Modularized crates (Phases 1-4):
  - `crates/atlassy-adf` — 6 modules (scope, path, index, patch, table_guard, bootstrap); `lib.rs` is 94-line facade; inline tests in scope/path/index
  - `crates/atlassy-contracts` — 3 modules (constants, types, validation); `lib.rs` is 10-line barrel; inline tests in constants/validation; `ErrorCode` is a 12-variant enum with `as_str()`/`Display`/`Serialize` impls
  - `crates/atlassy-confluence` — 3 modules (live, stub, types); `lib.rs` is 7-line barrel; inline tests in live; `src/tests.rs` removed
  - `crates/atlassy-pipeline` — 14 modules (6 infrastructure + 9 states via `states/`); `lib.rs` is 58-line facade with `RunMode`/`RunRequest` defined inline; inline tests in `state_tracker.rs`, `util.rs`
  - `crates/atlassy-cli` — 22 source files across 4 module groups (commands/4, batch/3, readiness/4, top-level/5 + 3 `mod.rs` + `cli_args.rs`); `main.rs` is 110-line entrypoint (Clap dispatch only); `lib.rs` is 19-line facade with selective re-exports; inline tests in `commands/run.rs`, `types.rs`
- Test placement:
  - Inline `#[cfg(test)] mod tests` in 10 files: `adf/scope.rs`, `adf/path.rs`, `adf/index.rs`, `contracts/constants.rs`, `contracts/validation.rs`, `confluence/live.rs`, `pipeline/state_tracker.rs`, `pipeline/util.rs`, `cli/commands/run.rs`, `cli/types.rs`
  - No external `src/tests.rs` files remain in any crate
  - Integration tests in `tests/` across 4 crates; CLI integration tests (`batch_report.rs`, `readiness.rs`) import via `use atlassy_cli::*` (enabled by `lib.rs` extraction)
- Error classification — fully typed across both layers:
  - Pipeline layer (TYPED): `to_hard_error()` pattern-matches on `AdfError` variants → `ErrorCode` enum; `PipelineError::Hard.code` is `ErrorCode`, not `String`
  - CLI layer (TYPED): `ErrorClass` enum (6 variants) and `DiagnosticCode` enum (4 variants) in `types.rs`; `classify_run_from_summary()` in `batch/report.rs` uses typed enum values at all 7 assignment sites; zero string-literal `error_class`/`error_code` assignments in production code
  - CLI downstream: zero `.as_deref() == Some("...")` comparisons for error classification; test assertions use direct enum equality (`diag.error_class == Some(ErrorClass::RetryPolicy)`)
  - Wire format: `RunSummary.error_codes: Vec<String>` — pipeline writes `ErrorCode` variants via `.to_string()`, CLI reads via `.as_str()` comparisons; custom `Serialize`/`Deserialize` impls on `ErrorClass` and `DiagnosticCode` preserve flat-string wire format
- Open items (non-blocking): 10 modules with private helpers lack inline `#[cfg(test)]` blocks — see consolidated list at end of Phase 4

## Scope

### In Scope

- Refactor for module clarity and smaller responsibility boundaries.
- Enforce single-responsibility modules where inline unit tests are naturally right-sized.
- Preserve behavior and output contracts while restructuring.
- Strengthen deterministic error taxonomy and verification consistency.
- Keep existing quality gates (`fmt`, `clippy`, `test`) as mandatory pass conditions.

### Out of Scope

- New product features.
- Behavior changes unrelated to quality/refactor targets.
- Output schema changes unless explicitly versioned and approved.
- Infrastructure changes outside current CI/testing model.

## Readability Standards

### Human-Readability

- One module should have one primary responsibility.
- Entry files should be thin (bootstrap/facade), not logic-dense.
- Domain logic, policy, and I/O concerns should be separated.
- Error handling should be typed and explicit where possible.

### AI-Readability

- Deterministic module map and naming conventions.
- Short trace paths for core flows (`run -> pipeline -> publish`).
- Minimize hidden coupling and cross-cutting helper sprawl.
- Avoid string-matching control flow for error code decisions.

## Refactoring Vocabulary

Each refactoring step in this program is grounded in the [refactoring.guru catalog](https://refactoring.guru/refactoring/catalog). This section maps the code smells found in the codebase to the techniques applied and the phases that address them.

### Code Smells Identified

| Smell | Category | Where | Addressed By |
|---|---|---|---|
| [Large Class](https://refactoring.guru/smells/large-class) | Bloaters | `main.rs` (2,780 lines), `lib.rs` (1,534 lines) | [Extract Class](https://refactoring.guru/extract-class) — Phases 2, 3 |
| [Long Method](https://refactoring.guru/smells/long-method) | Bloaters | `run_internal()` (229 lines), `classify_run_from_summary()` (115 lines), `run_adf_table_edit_state()` (148 lines) | [Extract Method](https://refactoring.guru/extract-method) — Phases 2, 3 |
| [Primitive Obsession](https://refactoring.guru/smells/primitive-obsession) | Bloaters | `error_class: Option<String>`, `error_code: Option<String>`, `error_codes: Vec<String>` used where typed enums should exist | [Replace Type Code with Class](https://refactoring.guru/replace-type-code-with-class) — Phase 2 prereq (done), Phase 4 |
| [Feature Envy](https://refactoring.guru/smells/feature-envy) | Couplers | 9 state methods on `Orchestrator` primarily operate on their parameters, not `self`; only 2 of 9 use `self.client` | [Move Method](https://refactoring.guru/move-method) — Phase 2 |
| [Shotgun Surgery](https://refactoring.guru/smells/shotgun-surgery) | Change Preventers | Adding a new error class requires changes in `classify_run_from_summary`, runbook routing, risk deltas, failure summaries, and the `matches!()` known-class guard | [Replace Type Code with Class](https://refactoring.guru/replace-type-code-with-class) — Phase 4 |

### Techniques Applied

| Technique | Category | Usage |
|---|---|---|
| [Extract Class](https://refactoring.guru/extract-class) | Moving Features | Split monolithic files into focused modules (all phases) |
| [Move Method](https://refactoring.guru/move-method) | Moving Features | State methods from `Orchestrator` to free functions (Phase 2) |
| [Extract Method](https://refactoring.guru/extract-method) | Composing Methods | Co-locate helpers with their single-caller modules (Phases 2, 3) |
| [Replace Type Code with Class](https://refactoring.guru/replace-type-code-with-class) | Organizing Data | `&str` constants → `ErrorCode` enum (done); `Option<String>` → `ErrorClass`/`DiagnosticCode` enums (Phase 4) |
| [Hide Delegate](https://refactoring.guru/hide-delegate) | Moving Features | `lib.rs` facades re-export module internals (all phases) |

### Anti-Patterns Avoided

- [Speculative Generality](https://refactoring.guru/smells/speculative-generality) — no parameter object/context struct for pipeline states; no state trait/macro abstraction
- [Divergent Change](https://refactoring.guru/smells/divergent-change) — bootstrap interlude stays in orchestrator, not in fetch (fetch would change for two reasons otherwise)

## Test Architecture Standard

Follows [Rust Book Ch. 11-3: Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html).

### Unit Tests (Rust Book Default)

- Inline `#[cfg(test)] mod tests { ... }` in each source file.
- Access private items via `use super::*`.
- If a test block grows too large, the fix is to split the production module (single responsibility), not to extract tests to a separate file.

### Integration Tests

- `crates/*/tests/*.rs` — each file is compiled as its own crate, public API only.
- No `#[cfg(test)]` annotation needed (Cargo handles it).
- Shared test helpers go in `tests/common/mod.rs` (not `tests/common.rs`) to avoid Cargo treating them as test crates.

### Fixtures

- `crates/*/tests/fixtures/` (or existing fixture locations).

### Binary Crate Constraint

- Binary crates (`main.rs` without `lib.rs`) cannot be integration-tested via `use` — the Rust compiler does not produce a linkable library.
- Solution: keep `main.rs` as thin CLI dispatch; push logic into `lib.rs` modules so integration tests can `use` the crate.

### API Surface Rule

- Do not widen public API visibility solely for tests.
- Inline `mod tests` gives private access via `use super::*` — no visibility widening needed.

## Refactor Program

### Phase 1: ADF / Contracts / Confluence Modularization — COMPLETE

Reduced mixed concerns in leaf crates via [Extract Class](https://refactoring.guru/extract-class). These crates have zero workspace dependencies, so refactoring them was self-contained and unblocked later phases.

#### Workspace dependency graph

```
adf, contracts, confluence   (leaf — zero workspace deps)
        ^
        |   (50+ imports)
    pipeline
        ^
        |   (4-symbol API)
      cli
```

Pipeline has 50+ deep imports from all three leaf crates. Restructuring leaf crates first, then adapting pipeline, avoided dual-breakage.

#### Completion summary

- `atlassy-adf`: 6 modules (scope, path, index, patch, table_guard, bootstrap); `lib.rs` is 94-line facade with shared types and re-exports; inline `#[cfg(test)] mod tests` in scope, path, index
- `atlassy-contracts`: 3 modules (constants, types, validation); `lib.rs` is 10-line barrel; inline `#[cfg(test)] mod tests` in constants, validation
- `atlassy-confluence`: 3 modules (live, stub, types); `lib.rs` is 7-line barrel; inline `#[cfg(test)] mod tests` in live; `src/tests.rs` removed (was public-API-only, covered by integration tests)
- Integration tests in `tests/` unchanged across all three crates

### Phase 2: Pipeline Modularization — COMPLETE

Refactor `atlassy-pipeline` into focused modules while keeping `lib.rs` as facade/API surface. Coupling is star-shaped: the orchestrator is the hub; individual states never call each other. Data flows linearly through `StateEnvelope<*Output>` structs.

State functions are extracted as free functions with explicit dependency parameters, not as methods on `Orchestrator`. Each state function receives only the dependencies it needs (`&ArtifactStore`, `&RunRequest`, `&mut StateTracker`, and for fetch/publish only, `&mut C`). The orchestrator's `run_internal()` calls these functions and threads data between them.

#### Prerequisite: ErrorCode enum in atlassy-contracts — COMPLETE

Converted the 12 `&str` error code constants to an `ErrorCode` enum ([Replace Type Code with Class](https://refactoring.guru/replace-type-code-with-class)) with `as_str()`, `Display`, and `Serialize` impls producing identical string representations. `PipelineError::Hard.code` is now `ErrorCode`, not `String`. `to_hard_error()` pattern-matches on typed `AdfError` variants. CLI uses `.to_string()` on the enum as a temporary bridge (resolved in Phase 4). Round-trip coverage tests ensure `ALL` array covers every variant and `as_str()` output matches `Display`.

#### Target module shape (indicative)

| Module | ~Lines | Complexity | Technique | Notes |
|---|---|---|---|---|
| `state_tracker` | 30 | Easy | [Extract Class](https://refactoring.guru/extract-class) | Zero-risk proof of concept; extract first to validate pattern |
| `error_map` | 80 | Easy | [Extract Class](https://refactoring.guru/extract-class) | `PipelineError` + `to_hard_error` + `confluence_error_to_hard_error` + `From` impls; foundational — unlocks `artifact_store` and all states |
| `artifact_store` | 45 | Easy | [Extract Class](https://refactoring.guru/extract-class) | Depends only on `error_map` |
| `states/fetch` | 55 | Easy | [Move Method](https://refactoring.guru/move-method) | Does NOT include bootstrap interlude (stays in orchestrator; see design decisions) |
| `states/classify` | 78 | Easy | [Move Method](https://refactoring.guru/move-method), [Extract Method](https://refactoring.guru/extract-method) | Co-extract `route_for_node`/`has_table_ancestor`/`parent_path` helpers (37 lines) |
| `states/extract_prose` | 75 | Easy | [Move Method](https://refactoring.guru/move-method) | Delegates to `atlassy-adf` |
| `states/md_assist_edit` | 167 | Medium | [Move Method](https://refactoring.guru/move-method), [Extract Method](https://refactoring.guru/extract-method) | Co-extract `project_prose_candidate` (38 lines, single caller); mutates `RunSummary.discovered_target_path` |
| `states/adf_table_edit` | 201 | Medium | [Move Method](https://refactoring.guru/move-method), [Extract Method](https://refactoring.guru/extract-method) | Co-extract `project_table_candidate` (53 lines, single caller); mutates `RunSummary.discovered_target_path` |
| `states/merge_candidates` | 96 | Easy | [Move Method](https://refactoring.guru/move-method), [Extract Method](https://refactoring.guru/extract-method) | Co-extract `paths_overlap` helper |
| `states/patch` | 73 | Easy | [Move Method](https://refactoring.guru/move-method) | Many inputs but read-only; delegates to `atlassy-adf` |
| `states/verify` | 73 | Easy | [Move Method](https://refactoring.guru/move-method) | Self-contained verification |
| `states/publish` | 90 | Easy-Medium | [Move Method](https://refactoring.guru/move-method) | Only state besides fetch needing `&mut client` for retry logic |
| `orchestrator` | 330 | Medium-Hard | [Extract Class](https://refactoring.guru/extract-class) | Hub: `run()` (82 lines) + `run_internal()` (229 lines) + `hard_fail()` (13 lines) + bootstrap interlude; extract last after all states are moved out |
| `util` | 40 | Easy | [Extract Method](https://refactoring.guru/extract-method) | Shared helpers only: `meta()`, `estimate_tokens()`, `compute_section_bytes()`, `add_duration_suffix()`; projection helpers move to their state modules |

#### Recommended extraction order

1. `state_tracker` (zero-risk proof of concept — validates `mod` declarations, re-exports, CI pass; depends only on `atlassy_contracts`, zero coupling to anything else in `lib.rs`)
2. `error_map` (foundational — `PipelineError` + error converters + `From` impls; unlocks `artifact_store` and all states; depends on `atlassy_contracts`, `atlassy_adf`, `atlassy_confluence`)
3. `artifact_store` (depends only on `error_map`)
4. Individual `states/*` modules (each depends on `error_map` + external crates; order within states is flexible)
5. `util` (shared helpers used by orchestrator — only 4 functions after projection helpers move to states)
6. `orchestrator` (last — imports everything else; `run()` + `run_internal()` + `hard_fail()` + bootstrap interlude)

#### Design decisions

- **Free functions over split impl blocks.** State methods on `Orchestrator` are [Feature Envious](https://refactoring.guru/smells/feature-envy) — they primarily operate on their input parameters, not on `self`. Only 2 of 9 states use `self.client`. Extracting as free functions via [Move Method](https://refactoring.guru/move-method) makes dependencies explicit and each state independently testable.
- **No parameter object / context struct.** Shared parameters (`artifact_store`, `request`, `tracker`) are passed individually. `client` is only needed by fetch and publish states, so bundling all four into a shared context would widen every state's dependency surface unnecessarily (avoid [Speculative Generality](https://refactoring.guru/smells/speculative-generality)).
- **State skeleton as convention, not abstraction.** All state functions follow a consistent transition/persist/return pattern (~5 lines of boilerplate). This is kept as readable convention rather than formalized via trait or macro. The boilerplate is small, the types vary per state, and the consistency is self-documenting. Noted as optional future cleanup if state count grows.
- **Bootstrap interlude stays in orchestrator.** The empty-page check and scaffold injection (~50 lines between fetch and classify in `run_internal`) is a transition concern, not a fetch responsibility. Moving it into fetch would cause [Divergent Change](https://refactoring.guru/smells/divergent-change) — fetch would need modification for both fetching logic changes and bootstrap policy changes.
- **Projection helpers co-locate with their state modules.** `project_prose_candidate` (38 lines, 2 call sites in `md_assist_edit` only) and `project_table_candidate` (53 lines, 4 call sites in `adf_table_edit` only) are private single-caller helpers. Placing them in `util` would create [Feature Envy](https://refactoring.guru/smells/feature-envy) in reverse — a function that exists solely to serve one module but lives elsewhere.
- **`RunSummary` mutation scatter is documented as convention.** Five functions write summary fields: `run()`, `run_internal()`, `run_md_assist_edit_state()`, `run_adf_table_edit_state()`, `hard_fail()`. State functions are limited to `discovered_target_path` writes only; the orchestrator owns bulk population. This is kept as readable convention rather than enforced by types, because formalizing it (e.g., a builder pattern) would add complexity disproportionate to the risk.

#### Test distribution

Current `src/tests.rs` (3 tests) distributes into new modules:

- `StateTracker` out-of-order test → inline in `state_tracker.rs` (or move to `tests/` since `StateTracker` is public API)
- `compute_section_bytes` tests (2) → inline in `util.rs` (private function, must stay in `src/`)
- `src/tests.rs` is removed after distribution.

#### Acceptance criteria

- `src/lib.rs` is a thin facade re-exporting `Orchestrator`, `PipelineError`, `RunMode`, `RunRequest`, `StateTracker`.
- `src/tests.rs` no longer exists; tests live inline in their respective modules.
- Each new module with private logic includes inline `#[cfg(test)] mod tests { ... }`.
- State order and semantics remain unchanged.
- Integration parity holds in `crates/atlassy-pipeline/tests/pipeline_integration.rs`.

#### Completion summary

- `lib.rs` reduced from 1,534 to 58 lines; facade re-exports `ArtifactStore`, `PipelineError`, `Orchestrator`, `StateTracker`; `RunMode` and `RunRequest` defined inline
- 14 modules created: `state_tracker`, `error_map`, `artifact_store`, `orchestrator`, `util`, `states/mod` + 9 state modules (`fetch`, `classify`, `extract_prose`, `md_assist_edit`, `adf_table_edit`, `merge_candidates`, `patch`, `verify`, `publish`)
- `src/tests.rs` removed; tests inlined in `state_tracker.rs` (1 test) and `util.rs` (2 tests)
- Integration parity held: `tests/pipeline_integration.rs` unchanged and green
- Open item: 4 state modules (`classify`, `md_assist_edit`, `adf_table_edit`, `merge_candidates`) have private helpers without inline `#[cfg(test)]` blocks; consolidated in Phase 4 completion summary

### Phase 3: CLI Modularization — COMPLETE

Refactor `atlassy-cli` so `main.rs` is thin dispatch only. No shared mutable state exists — all functions are pure or perform I/O through parameters.

Extract `src/lib.rs` alongside `src/main.rs` to resolve the binary crate testing constraint (see Test Architecture Standard). Without `lib.rs`, Rust cannot produce a linkable library, so integration tests in `tests/` cannot `use atlassy_cli::*`. Current `src/tests.rs` (13 tests) and `src/test_helpers.rs` access ~15 private functions/structs via `use super::*` — these must distribute into new modules as inline `#[cfg(test)] mod tests { ... }` blocks.

#### Target module shape (indicative)

| Module | ~Lines | Complexity | Technique | Notes |
|---|---|---|---|---|
| `commands/run` | 80 | Easy | [Extract Class](https://refactoring.guru/extract-class) | Self-contained; delegates to `Orchestrator` |
| `commands/run_batch` | 120 | Medium | [Extract Class](https://refactoring.guru/extract-class) | Wide dependency fan-out across batch/report, safety, provenance |
| `commands/run_readiness` | 90 | Easy | [Extract Class](https://refactoring.guru/extract-class) | Thin orchestration over readiness sub-modules |
| `commands/create_subpage` | 40 | Easy | [Extract Class](https://refactoring.guru/extract-class) | Fully self-contained |
| `batch/report` | 460 | Hard | [Extract Class](https://refactoring.guru/extract-class), [Extract Method](https://refactoring.guru/extract-method) | Largest block; central integration point for KPI, safety, provenance |
| `batch/kpi` | 260 | Easy | [Extract Method](https://refactoring.guru/extract-method) | Clean functional code, no side effects |
| `batch/safety` | 85 | Easy | [Extract Method](https://refactoring.guru/extract-method) | Pure assessment functions |
| `readiness/evidence` | 180 | Medium | [Extract Class](https://refactoring.guru/extract-class) | Cross-module type dependencies |
| `readiness/gates` | 180 | Easy | [Extract Method](https://refactoring.guru/extract-method) | Self-contained gate evaluation |
| `readiness/runbooks` | 205 | Easy | [Extract Method](https://refactoring.guru/extract-method) | Pure construction from report diagnostics |
| `readiness/decision_packet` | 250 | Medium | [Extract Class](https://refactoring.guru/extract-class) | Bridges batch report, gates, and runbooks types |
| `provenance` | 70 | Easy | [Extract Method](https://refactoring.guru/extract-method) | Pure utility functions |
| `fixtures` | 50 | Easy | [Extract Class](https://refactoring.guru/extract-class) | Zero coupling; pure data |
| `types` | 360 | Medium | [Extract Class](https://refactoring.guru/extract-class) | 30+ struct/enum definitions used across modules — must extract early; Phase 4 adds `ErrorClass` and `DiagnosticCode` here |
| `manifest` | 130 | Easy | [Extract Method](https://refactoring.guru/extract-method) | `validate_manifest`, `normalize_manifest`, `run_mode_from_manifest` |
| `io` | 30 | Easy | [Extract Method](https://refactoring.guru/extract-method) | `load_required_json`, `load_run_summary` |
| `lib` | ~30 | Easy | [Hide Delegate](https://refactoring.guru/hide-delegate) | Facade: re-exports all modules so `tests/` can `use atlassy_cli::*` |

#### Recommended extraction order

1. `fixtures` (zero dependencies)
2. `provenance` (near-zero dependencies, widely consumed)
3. `types` (shared struct/enum definitions — unblocks other modules)
4. `manifest`, `io` (shared utilities)
5. `batch/safety`, `batch/kpi` (clean pure functions)
6. `readiness/gates`, `readiness/runbooks`
7. `readiness/decision_packet`, `readiness/evidence`
8. `batch/report` (depends on KPI, safety, provenance being extracted first)
9. `commands/*` (glue layer — extract last)
10. Slim `main.rs` to CLI parsing + dispatch (~120 lines)

#### Test distribution

Current `src/tests.rs` (13 tests) and `src/test_helpers.rs` distribute into new modules:

- Each module gets inline `#[cfg(test)] mod tests { ... }` for its private logic.
- `src/tests.rs` and `src/test_helpers.rs` are removed after distribution.
- Integration tests in `tests/` can now `use atlassy_cli::*` to test public API (enabled by `lib.rs` extraction).

#### Acceptance criteria

- `src/main.rs` becomes entrypoint + CLI arg parsing only.
- `src/lib.rs` exists and re-exports module API.
- `src/tests.rs` and `src/test_helpers.rs` no longer exist; tests are inline in their modules.
- Batch/readiness outputs remain schema-compatible.
- Existing CLI integration tests remain green.

#### Completion summary

- `main.rs` reduced from 2,780 to 110 lines; entrypoint + Clap dispatch only; `cli_args.rs` (63 lines) holds subcommand arg structs
- `lib.rs` created (19 lines); declares 8 modules (`commands`, `batch`, `readiness`, `provenance`, `fixtures`, `types`, `manifest`, `io`) with selective re-exports
- 22 source files across 4 module groups: `commands/` (4: `run`, `run_batch`, `run_readiness`, `create_subpage`), `batch/` (3: `report`, `kpi`, `safety`), `readiness/` (4: `evidence`, `gates`, `runbooks`, `decision_packet`), top-level (5: `types`, `fixtures`, `provenance`, `manifest`, `io`) + 3 `mod.rs` files + `cli_args.rs`
- `src/tests.rs` (573 lines, 13 tests) and `src/test_helpers.rs` (19 lines) removed; tests distributed to inline blocks and integration tests
- Integration tests (`batch_report.rs`, `readiness.rs`) import via `use atlassy_cli::*` (binary crate constraint resolved by `lib.rs` extraction); `live_runtime_startup.rs` unchanged (binary-level test)
- Open item: `commands/run.rs` and `types.rs` have inline `#[cfg(test)]` blocks (`types.rs` added during Phase 4); remaining modules with private logic lack inline unit tests

### Phase 4: Error Taxonomy — COMPLETE

Replaced remaining string-based error classification in the CLI with typed enums. The pipeline layer was already fully typed (Phase 2 prerequisite). This phase addressed the 15 string-based classification points at the CLI diagnostic layer.

This phase depended on Phase 3 (CLI modularization) because `ErrorClass` and `DiagnosticCode` live in the CLI `types` module created during Phase 3.

#### Two-layer error taxonomy

The codebase has two distinct error classification layers that must not be conflated:

| Layer | Location | Purpose | Status |
|---|---|---|---|
| Pipeline `ErrorCode` | `atlassy-contracts` | What went wrong during pipeline execution (12 variants) | TYPED (Phase 2 prereq) |
| CLI diagnostic classification | `atlassy-cli` | What went wrong with the run from an operational perspective (6 classes + 3 CLI-only codes) | TYPED |

A run can succeed at the pipeline level but fail at the CLI level (e.g., telemetry incomplete, provenance mismatch). The two layers describe different concerns and should remain separate types.

#### Target types

**`ErrorClass` enum** — resolves [Primitive Obsession](https://refactoring.guru/smells/primitive-obsession) and [Shotgun Surgery](https://refactoring.guru/smells/shotgun-surgery) via [Replace Type Code with Class](https://refactoring.guru/replace-type-code-with-class). Lives in CLI `types` module. 6 variants matching the current closed string vocabulary:

| Variant | Current string | Meaning |
|---|---|---|
| `Io` | `"io"` | Missing summary artifact |
| `TelemetryIncomplete` | `"telemetry_incomplete"` | Telemetry validation failure |
| `ProvenanceIncomplete` | `"provenance_incomplete"` | Provenance mismatch |
| `RetryPolicy` | `"retry_policy"` | Retry count exceeded |
| `RuntimeUnmappedHard` | `"runtime_unmapped_hard"` | Unmapped runtime error |
| `PipelineHard` | `"pipeline_hard"` | Generic pipeline failure (fallback) |

**`DiagnosticCode` enum** — resolves the polymorphic `error_code: Option<String>` field on `BatchRunDiagnostic`, which currently holds either pipeline `ErrorCode` strings or CLI-only strings. Lives in CLI `types` module:

| Variant | Current string | Origin |
|---|---|---|
| `Pipeline(ErrorCode)` | varies | Passthrough from pipeline `RunSummary.error_codes` |
| `SummaryMissing` | `"ERR_SUMMARY_MISSING"` | CLI-only: no summary artifact on disk |
| `TelemetryIncomplete` | `"ERR_TELEMETRY_INCOMPLETE"` | CLI-only: telemetry validation failed |
| `ProvenanceMismatch` | `"ERR_PROVENANCE_MISMATCH"` | CLI-only: provenance does not match batch |

Both enums require `Serialize` impls that produce the same string output as the current literals to preserve wire compatibility.

#### `RunSummary.error_codes: Vec<String>` — no change

This field stays `Vec<String>`. Rationale:

1. The two-layer type bridge at `classify_run_from_summary` (where `summary.error_codes.first()` is assigned to `BatchRunDiagnostic.error_code`) must be designed first — that is this phase's `DiagnosticCode::Pipeline(ErrorCode)` wrapper.
2. Changing to `Vec<ErrorCode>` requires a `Deserialize` impl for `ErrorCode`. This introduces a new failure mode: if the CLI reads a summary from a newer pipeline version with unknown error codes, deserialization fails. With `Vec<String>`, unknown codes pass through silently — better behavior for a batch system that must be resilient to version skew.
3. The current `.as_str()` comparisons (10 sites) are type-safe at the enum end — they compare `String` against `ErrorCode::*.as_str()`, so typos are caught at compile time on the enum side.

Optional future cleanup after Phase 4 core: add `Deserialize` impl with unknown-variant tolerance, migrate `Vec<String>` → `Vec<ErrorCode>`, simplify all comparison sites.

#### Priority targets

1. **`ErrorClass` enum** in CLI `types` module ([Replace Type Code with Class](https://refactoring.guru/replace-type-code-with-class))
2. **`DiagnosticCode` enum** in CLI `types` module
3. **Migrate `classify_run_from_summary`** to return `BatchRunDiagnostic` with typed `error_class: Option<ErrorClass>` and `error_code: Option<DiagnosticCode>` — eliminates 7 string-literal `error_class` assignments and 5 string-literal `error_code` assignments
4. **Migrate downstream consumers** — eliminates all `.as_deref() == Some("...")` patterns (4 sites in runbooks, 1 in risk deltas) and the `matches!()` known-class guard (1 site). With an `ErrorClass` enum, adding a new class without handling it in runbooks becomes a compile error, resolving the [Shotgun Surgery](https://refactoring.guru/smells/shotgun-surgery) smell.

#### Acceptance criteria

- Error classification is deterministic and explicit.
- No regression in existing hard-error test coverage.
- CLI matches on `ErrorClass` and `DiagnosticCode` enums directly, not via `.as_deref()` or string literals.
- No raw `"ERR_*"` string literals in CLI production code outside serde impls.
- `BatchRunDiagnostic` serialization produces identical JSON output (wire-compatible).

#### Completion summary

- `ErrorClass` enum (6 variants: `Io`, `TelemetryIncomplete`, `ProvenanceIncomplete`, `RetryPolicy`, `RuntimeUnmappedHard`, `PipelineHard`) in `types.rs` with `as_str()`, `from_str()`, custom `Serialize`/`Deserialize` impls; const `ALL` array; rejects unknown variants at deserialization
- `DiagnosticCode` enum (4 variants: `Pipeline(ErrorCode)`, `SummaryMissing`, `TelemetryIncomplete`, `ProvenanceMismatch`) in `types.rs` with `as_str()`, `from_str()`, custom `Serialize`/`Deserialize` impls; `Pipeline(ErrorCode)` wrapper bridges the two-layer taxonomy; `from_str()` falls back to `ErrorCode::ALL` for pipeline codes
- `classify_run_from_summary` in `batch/report.rs` uses typed enum values at all 7 assignment sites; zero string-literal `error_class`/`error_code` assignments remain in production code
- All `.as_deref() == Some("...")` patterns for error classification eliminated; test assertions use direct enum equality (e.g., `diag.error_class == Some(ErrorClass::RetryPolicy)`)
- `BatchRunDiagnostic` fields changed from `Option<String>` to `Option<ErrorClass>` and `Option<DiagnosticCode>`; wire format preserved via custom serde impls
- `types.rs` gained inline `#[cfg(test)] mod tests` block for serde round-trip and unknown-variant rejection tests
- Open items (non-blocking): 10 modules with private helpers lack inline `#[cfg(test)]` blocks:
  - Pipeline (4): `states/classify.rs` (3 helpers), `states/md_assist_edit.rs` (1), `states/adf_table_edit.rs` (1), `states/merge_candidates.rs` (1)
  - CLI (6): `batch/kpi.rs` (7 helpers), `readiness/gates.rs` (3), `readiness/decision_packet.rs` (2), `readiness/evidence.rs` (2), `readiness/runbooks.rs` (1), `provenance.rs` (2)

## Phase Sequencing Constraints

| Phase | Status | Depends On |
|---|---|---|
| Phase 1 (leaf crates) | **COMPLETE** | — |
| Phase 2 prereq (ErrorCode enum) | **COMPLETE** | Phase 1 |
| Phase 2 (pipeline) | **COMPLETE** | Phase 1, Phase 2 prereq |
| Phase 3 (CLI) | **COMPLETE** | Can overlap with Phase 2 |
| Phase 4 (error taxonomy) | **COMPLETE** | Phase 3 (ErrorClass/DiagnosticCode live in CLI `types` module) |

All phases complete. No remaining execution order.

## Quality Gates

Mandatory on each phase:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

Mandatory on each phase (test architecture):

- Each new module includes inline `#[cfg(test)] mod tests { ... }` for unit tests.
- Integration tests in `tests/` use public API only — no `pub(crate)` backdoors for test access.

## Done Criteria

- Test placement follows Rust Book Ch. 11-3: inline `#[cfg(test)] mod tests` for unit tests, `tests/` for integration tests.
- Entrypoints/facades are thin and responsibility boundaries are clear.
- Monolithic files are broken into focused modules.
- Error mapping is typed/deterministic where feasible.
- No regression in:
  - `crates/atlassy-pipeline/tests/pipeline_integration.rs`
  - `crates/atlassy-cli/tests/live_runtime_startup.rs`
- All workspace quality gates pass.

## Risks and Mitigations

- Risk: accidental behavior drift during file moves/refactor.
  - Mitigation: phase-by-phase migration with parity checks after every phase.
- Risk: API visibility expansion for test access.
  - Mitigation: inline `mod tests` gives private access via `use super::*`; no visibility widening needed.
- Risk: binary crate constraint blocks CLI integration testing.
  - Mitigation: Phase 3 extracts `lib.rs` alongside `main.rs`, enabling `tests/` to `use atlassy_cli::*`.
- Risk: schema drift in batch/readiness outputs.
  - Mitigation: keep compatibility checks in test fixtures and rebuild/parity tests.
- Risk: phase ordering violation causing cross-crate breakage.
  - Mitigation: respect dependency graph (leaf crates before pipeline before CLI). See Phase Sequencing Constraints.
- Risk: silent error reclassification during Phase 4 typed-error migration.
  - Mitigation: preserve existing test assertions as regression guards; add typed-error round-trip tests before removing string-based paths.
- Risk: `RunSummary` mutation scatter makes state functions harder to test independently.
  - Mitigation: state functions are limited to `discovered_target_path` writes only; the orchestrator owns bulk summary population. Convention is documented in Phase 2 design decisions.
- Risk: `RunSummary.error_codes: Vec<String>` creates a permanent `.as_str()` bridge at every comparison site.
  - Mitigation: defer to optional post-Phase-4 step. Current comparisons use typed enum on one side (`ErrorCode::*.as_str()`), limiting fragility to one direction. A future `Deserialize` impl must handle unknown variants gracefully for version-skew resilience.

## Cross-References

- `10-testing-strategy-and-simulation.md` (test layering and scenario intent)
- `13-ci-and-automation.md` (automation gates)
- `06-decisions-and-defaults.md` (safety and verification defaults)
- `09-ai-contract-spec.md` (error and contract expectations)
