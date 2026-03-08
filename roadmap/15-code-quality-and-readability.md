# Code Quality and Readability

## Objective

Improve code quality through behavior-preserving refactor, with explicit focus on:

- human-readability (maintainability, onboarding speed, safer edits)
- AI-readability (deterministic structure, low context bloat, clear boundaries)

The program targets source structure and test architecture, not feature expansion.

## Current Baseline (2026-03-08)

- Monolithic hotspots:
  - `crates/atlassy-cli/src/main.rs` (3380 lines)
  - `crates/atlassy-pipeline/src/lib.rs` (1587 lines)
  - `crates/atlassy-adf/src/lib.rs` (1408 lines)
- Test placement (counts are `#[test]` functions, not files):
  - `#[test]` functions in `src/**`: 20 (across 4 files in 3 crates)
  - `#[test]` functions in `tests/**`: 80 (across 8 files in 4 crates)
- Test module declarations in source files:
  - `crates/atlassy-cli/src/main.rs` — `#[cfg(test)] mod tests;` → external `src/tests.rs` (13 tests) + `src/test_helpers.rs`
  - `crates/atlassy-pipeline/src/lib.rs` — `#[cfg(test)] mod tests;` → external `src/tests.rs` (3 tests)
  - `crates/atlassy-confluence/src/lib.rs` — `#[cfg(test)] mod tests;` → external `src/tests.rs` (3 tests, public API only)
  - `crates/atlassy-confluence/src/live.rs` — inline `mod tests { ... }` (4 tests, tests private methods)
- Crates with no inline tests (all tests in `tests/`):
  - `crates/atlassy-adf` (43 tests across 5 files)
  - `crates/atlassy-contracts` (8 tests in 1 file)
- Error classification includes 17 brittle string-based mapping points in production code:
  - `to_hard_error()` in `atlassy-pipeline/src/lib.rs:1496-1514` — 5 `.contains()` checks on stringified error messages to assign error codes (highest fragility)
  - `classify_run_from_summary()` in `atlassy-cli/src/main.rs:1920-2034` — 7 string-literal error class assignments (`"io"`, `"pipeline_hard"`, `"telemetry_incomplete"`, etc.)
  - downstream string class matching in `atlassy-cli` (runbooks, risk deltas, failure summaries) — 5 points using `.as_deref() == Some("...")` or `matches!()` on string class values
  - error code constants defined as `&str` in `atlassy-contracts/src/lib.rs:9-20` (12 constants; `PipelineError::Hard.code` is `String`, not enum)

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

### Phase 1: ADF / Contracts / Confluence Modularization — size S

Reduce mixed concerns in leaf crates. These crates have zero workspace dependencies, so refactoring them is self-contained and unblocks later phases.

#### Rationale for ordering

The workspace dependency graph is:

```
adf, contracts, confluence   (leaf — zero workspace deps)
        ^
        |   (50+ imports)
    pipeline
        ^
        |   (4-symbol API)
      cli
```

Pipeline has 50+ deep imports from all three leaf crates. Restructuring leaf crates first, then adapting pipeline, avoids dual-breakage.

#### Focus

- `atlassy-adf`: split scope/path/index/patch/table-guard/bootstrap concerns
- `atlassy-contracts`: split constants/types/validation
- `atlassy-confluence`: split trait/live/stub/payload handling

#### Test expectations

- Each new module includes inline `#[cfg(test)] mod tests { ... }` for unit tests covering its private logic.
- Existing integration tests in `tests/` remain unchanged.
- `atlassy-confluence`: move `src/tests.rs` to `tests/` (it only tests public API via `StubConfluenceClient`); `live.rs` inline tests stay (they test private methods `build_publish_payload` and `build_create_payload`).

#### Acceptance criteria

- Clear module boundaries with minimal cross-module coupling.
- Existing contract validations and runtime behavior preserved.

### Phase 2: Pipeline Modularization — size M

Refactor `atlassy-pipeline` into focused modules while keeping `lib.rs` as facade/API surface. Coupling is star-shaped: the orchestrator is the hub; individual states never call each other. Data flows linearly through `StateEnvelope<*Output>` structs.

State functions are extracted as free functions with explicit dependency parameters, not as methods on `Orchestrator`. Each state function receives only the dependencies it needs (`&ArtifactStore`, `&RunRequest`, `&mut StateTracker`, and for fetch/publish only, `&mut C`). The orchestrator's `run_internal()` calls these functions and threads data between them.

#### Prerequisite: ErrorCode enum in atlassy-contracts

Before pipeline extraction begins, convert the 12 `&str` error code constants in `atlassy-contracts/src/constants.rs` to an `ErrorCode` enum with `Display`/`Serialize` impls that produce identical string representations. Update `PipelineError::Hard.code` from `String` to `ErrorCode`. CLI temporarily uses `.to_string()` on the enum where it previously consumed the `String` directly. This ensures `error_map.rs` is born with typed error codes rather than cementing string-based codes into a new module.

This is a backwards-compatible leaf change in an already-modularized crate (Phase 1 complete). Pulled forward from Phase 4 item 2 to avoid moving code known to be structurally wrong.

#### Target module shape (indicative)

| Module | ~Lines | Complexity | Notes |
|---|---|---|---|
| `error_map` | 75 | Easy | Foundational — extract first; everything depends on `PipelineError` |
| `state_tracker` | 30 | Easy | Standalone, zero internal coupling |
| `artifact_store` | 45 | Easy | Depends only on `error_map` |
| `states/fetch` | 105 | Easy | Includes bootstrap interlude |
| `states/classify` | 80 | Easy | Co-extract `route_for_node`/`has_table_ancestor`/`parent_path` helpers |
| `states/extract_prose` | 75 | Easy | Delegates to `atlassy-adf` |
| `states/md_assist_edit` | 165 | Medium | Mutates `RunSummary`, matches on `RunMode`; co-extract `project_prose_candidate` |
| `states/adf_table_edit` | 200 | Medium | Mutates `RunSummary`, matches on `RunMode`; co-extract `project_table_candidate` |
| `states/merge_candidates` | 95 | Easy | Co-extract `paths_overlap` helper |
| `states/patch` | 70 | Easy | Many inputs but read-only; delegates to `atlassy-adf` |
| `states/verify` | 70 | Easy | Self-contained verification |
| `states/publish` | 90 | Easy-Medium | Needs `&mut client` for retry logic |
| `orchestrator` | 130 | Medium-Hard | Hub; extract last after all states are moved out |
| `util` | ~40 | Easy | Shared helpers: `meta()`, `estimate_tokens()`, `compute_section_bytes()`, `add_duration_suffix()` |

#### Recommended extraction order

1. `error_map` (leaf dependency)
2. `state_tracker` (standalone)
3. `artifact_store` (depends only on `error_map`)
4. Individual `states/*` modules (each depends on `error_map` + external crates)
5. `util` (shared helpers used by orchestrator)
6. `orchestrator` (last — imports everything else)

#### Design decisions

- **Free functions over split impl blocks.** State methods on `Orchestrator` are Feature Envious (refactoring.guru: Move Method) — they primarily operate on their input parameters, not on `self`. Only 2 of 9 states use `self.client`. Extracting as free functions makes dependencies explicit and each state independently testable.
- **No parameter object / context struct.** Shared parameters (`artifact_store`, `request`, `tracker`) are passed individually. `client` is only needed by fetch and publish states, so bundling all four into a shared context would widen every state's dependency surface unnecessarily (refactoring.guru: avoid Speculative Generality).
- **State skeleton as convention, not abstraction.** All state functions follow a consistent transition/persist/return pattern (~5 lines of boilerplate). This is kept as readable convention rather than formalized via trait or macro. The boilerplate is small, the types vary per state, and the consistency is self-documenting. Noted as optional future cleanup if state count grows.

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

### Phase 3: CLI Modularization — size L

Refactor `atlassy-cli` so `main.rs` is thin dispatch only. No shared mutable state exists — all functions are pure or perform I/O through parameters.

Extract `src/lib.rs` alongside `src/main.rs` to resolve the binary crate testing constraint (see Test Architecture Standard). Without `lib.rs`, Rust cannot produce a linkable library, so integration tests in `tests/` cannot `use atlassy_cli::*`. Current `src/tests.rs` (13 tests) and `src/test_helpers.rs` access ~15 private functions/structs via `use super::*` — these must distribute into new modules as inline `#[cfg(test)] mod tests { ... }` blocks.

#### Target module shape (indicative)

| Module | ~Lines | Complexity | Notes |
|---|---|---|---|
| `commands/run` | 80 | Easy | Self-contained; delegates to `Orchestrator` |
| `commands/run_batch` | 120 | Medium | Wide dependency fan-out across batch/report, safety, provenance |
| `commands/run_readiness` | 90 | Easy | Thin orchestration over readiness sub-modules |
| `commands/create_subpage` | 40 | Easy | Fully self-contained |
| `batch/report` | 460 | Hard | Largest block; central integration point for KPI, safety, provenance |
| `batch/kpi` | 260 | Easy | Clean functional code, no side effects |
| `batch/safety` | 85 | Easy | Pure assessment functions |
| `readiness/evidence` | 180 | Medium | Cross-module type dependencies |
| `readiness/gates` | 180 | Easy | Self-contained gate evaluation |
| `readiness/runbooks` | 205 | Easy | Pure construction from report diagnostics |
| `readiness/decision_packet` | 250 | Medium | Bridges batch report, gates, and runbooks types |
| `provenance` | 70 | Easy | Pure utility functions |
| `fixtures` | 50 | Easy | Zero coupling; pure data |
| `types` | 360 | Medium | 30+ struct/enum definitions used across modules — must extract early |
| `manifest` | 130 | Easy | `validate_manifest`, `normalize_manifest`, `run_mode_from_manifest` |
| `io` | 30 | Easy | `load_required_json`, `load_run_summary` |
| `lib` | ~30 | Easy | Facade: re-exports all modules so `tests/` can `use atlassy_cli::*` |

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

### Phase 4: Error Taxonomy + Verification Consistency — size S-M

Replace remaining string-based hard-error mapping with typed mapping. The `ErrorCode` enum in `atlassy-contracts` was already created as a Phase 2 prerequisite, reducing the remaining work to eliminating string-sniffing at error construction sites and typed classification in the CLI.

#### Priority targets

1. **`to_hard_error()` in `atlassy-pipeline/src/error_map.rs`** (highest priority).
   - Currently sniffs error messages via `.contains("out of scope")`, `.contains("table") && .contains("shape")`, etc. to select `ErrorCode` variants.
   - Ordering of checks matters: `"out of scope"` must be checked before bare `"scope"` or errors are misclassified.
   - Any upstream wording change silently reclassifies errors.
   - Replace with typed `AdfError` variants that carry the error code at construction site, not at classification site.
2. ~~**Error code constants in `atlassy-contracts`**~~ — completed as Phase 2 prerequisite (see Phase 2: Prerequisite: ErrorCode enum).
3. **`classify_run_from_summary()` in `atlassy-cli`**: convert 7 string-literal error class assignments to a typed `ErrorClass` enum. Replace `.to_string()` bridge introduced in Phase 2 prerequisite with direct enum matching.
4. **Downstream consumers** (runbook routing, risk deltas, failure summaries): update to match on typed enums instead of string comparisons.

#### Acceptance criteria

- Error classification is deterministic and explicit.
- No regression in existing hard-error test coverage.
- `to_hard_error()` no longer uses substring matching on error messages.
- CLI matches on `ErrorCode` and `ErrorClass` enums directly, not via `.to_string()` or `.as_deref()`.

## Phase Sequencing Constraints

| Phase pair | Parallel? | Risk | Reason |
|---|---|---|---|
| 1+2 (leaf crates + pipeline) | **No** | Critical | Pipeline has 50+ deep imports from all three leaf crates |
| 1+3 (leaf crates + CLI) | Yes, with coordination | Medium | CLI imports ~20 symbols from contracts + confluence |
| 2+3 (pipeline + CLI) | Yes, with coordination | Low | CLI uses only 4 pipeline public symbols |

Required execution order: **1 → 2 prereq (ErrorCode enum) → 2 (can overlap with 3) → 3 → 4**.

The Phase 2 prerequisite (ErrorCode enum in `atlassy-contracts`) must land before pipeline extraction begins. It touches only the already-modularized contracts crate and is backwards-compatible — CLI uses `.to_string()` as a temporary bridge. This does not conflict with Phase 1 completion.

Phase 2 (pipeline) and Phase 3 (CLI) can overlap if the pipeline's 4-symbol public API (`Orchestrator`, `PipelineError`, `RunMode`, `RunRequest`) is frozen before CLI work begins.

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

## Cross-References

- `10-testing-strategy-and-simulation.md` (test layering and scenario intent)
- `13-ci-and-automation.md` (automation gates)
- `06-decisions-and-defaults.md` (safety and verification defaults)
- `09-ai-contract-spec.md` (error and contract expectations)
