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
- Test placement is mixed (counts are `#[test]` functions, not files):
  - `#[test]` functions in `src/**`: 77 (across 5 files)
  - `#[test]` functions in `tests/**`: 30 (across 2 files)
- Embedded `mod tests { ... }` blocks exist in large production files:
  - `crates/atlassy-cli/src/main.rs`
  - `crates/atlassy-pipeline/src/lib.rs`
  - `crates/atlassy-adf/src/lib.rs`
  - `crates/atlassy-contracts/src/lib.rs`
  - `crates/atlassy-confluence/src/lib.rs`
- Error classification includes 17 brittle string-based mapping points in production code:
  - `to_hard_error()` in `atlassy-pipeline/src/lib.rs:1496-1514` — 5 `.contains()` checks on stringified error messages to assign error codes (highest fragility)
  - `classify_run_from_summary()` in `atlassy-cli/src/main.rs:1920-2034` — 7 string-literal error class assignments (`"io"`, `"pipeline_hard"`, `"telemetry_incomplete"`, etc.)
  - downstream string class matching in `atlassy-cli` (runbooks, risk deltas, failure summaries) — 5 points using `.as_deref() == Some("...")` or `matches!()` on string class values
  - error code constants defined as `&str` in `atlassy-contracts/src/lib.rs:9-20` (12 constants; `PipelineError::Hard.code` is `String`, not enum)

## Scope

### In Scope

- Refactor for module clarity and smaller responsibility boundaries.
- Extract automated tests out of production logic files.
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

### Placement Policy (default)

- Private/unit tests:
  - keep in dedicated test files under `src` (for private access when needed)
  - do not keep large test bodies inline in production logic files
- Integration/black-box tests:
  - keep under `crates/*/tests/`
- Fixtures:
  - keep under `crates/*/tests/fixtures/` (or existing fixture locations)

### Structural Rule

- Production files may contain at most thin test module declarations (for example `#[cfg(test)] mod tests;`).
- Production logic files should not contain long inline `mod tests { ... }` bodies.

### API Surface Rule

- Do not widen public API visibility solely for tests.
- Prefer private/unit tests in `src` test files when internal access is required.

## Refactor Program

### Phase 1: Test Extraction First (behavior-preserving) — size S

Move inline tests from production logic files into dedicated test files.

#### Per-crate target

- `atlassy-adf`:
  - split tests into domain files (scope resolution, target discovery, patch ops, emptiness/bootstrap)
- `atlassy-contracts`:
  - move contract/invariant tests into dedicated test files
- `atlassy-confluence`:
  - move payload/stub behavior tests into dedicated test files
- `atlassy-pipeline`:
  - move core invariant unit tests out of `src/lib.rs`; keep integration suite in `tests/pipeline_integration.rs`
- `atlassy-cli`:
  - move inline tests out of `src/main.rs`; keep process-level CLI tests in `tests/`

#### Acceptance criteria

- No long inline test bodies remain in production logic files.
- Existing test behavior is preserved.
- `cargo test --workspace` remains green.

### Phase 2: ADF / Contracts / Confluence Modularization — size S

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

#### Acceptance criteria

- Clear module boundaries with minimal cross-module coupling.
- Existing contract validations and runtime behavior preserved.

### Phase 3: Pipeline Modularization — size M

Refactor `atlassy-pipeline` into focused modules while keeping `lib.rs` as facade/API surface. Coupling is star-shaped: the orchestrator is the hub; individual states never call each other. Data flows linearly through `StateEnvelope<*Output>` structs.

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
| `util` | 20 | Easy | Shared helpers: `meta()`, `estimate_tokens()`, `compute_section_bytes()`, `add_duration_suffix()` |

#### Recommended extraction order

1. `error_map` (leaf dependency)
2. `state_tracker` (standalone)
3. `artifact_store` (depends only on `error_map`)
4. Individual `states/*` modules (each depends on `error_map` + external crates)
5. `util` (shared helpers used by orchestrator)
6. `orchestrator` (last — imports everything else)

#### Acceptance criteria

- `src/lib.rs` is a thin facade re-exporting `Orchestrator`, `PipelineError`, `RunMode`, `RunRequest`, `StateTracker`.
- State order and semantics remain unchanged.
- Integration parity holds in `crates/atlassy-pipeline/tests/pipeline_integration.rs`.

### Phase 4: CLI Modularization — size L

Refactor `atlassy-cli` so `main.rs` is thin dispatch only. No shared mutable state exists — all functions are pure or perform I/O through parameters.

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

#### Acceptance criteria

- `src/main.rs` becomes entrypoint + CLI arg parsing only.
- Batch/readiness outputs remain schema-compatible.
- Existing CLI integration tests remain green.

### Phase 5: Error Taxonomy + Verification Consistency — size S-M

Replace string-based hard-error mapping with typed mapping where practical. 17 production mapping points across 4 categories (see baseline).

#### Priority targets

1. **`to_hard_error()` in `atlassy-pipeline/src/lib.rs:1496-1514`** (highest priority).
   - Currently sniffs error messages via `.contains("out of scope")`, `.contains("table") && .contains("shape")`, etc. to assign error codes.
   - Ordering of checks matters: `"out of scope"` must be checked before bare `"scope"` or errors are misclassified.
   - Any upstream wording change silently reclassifies errors.
   - Replace with typed error variants that carry the error code at construction site, not at classification site.
2. **Error code constants in `atlassy-contracts/src/lib.rs:9-20`**: convert 12 `&str` constants to an enum with `Display`/`Serialize` impls. `PipelineError::Hard.code` becomes the enum type instead of `String`.
3. **`classify_run_from_summary()` in `atlassy-cli/src/main.rs:1920-2034`**: convert 7 string-literal error class assignments to a typed `ErrorClass` enum.
4. **Downstream consumers** (runbook routing, risk deltas, failure summaries): update to match on typed enums instead of string comparisons.

#### Acceptance criteria

- Error classification is deterministic and explicit.
- No regression in existing hard-error test coverage.
- `to_hard_error()` no longer uses substring matching on error messages.

## Phase Sequencing Constraints

| Phase pair | Parallel? | Risk | Reason |
|---|---|---|---|
| 2+3 (leaf crates + pipeline) | **No** | Critical | Pipeline has 50+ deep imports from all three leaf crates |
| 2+4 (leaf crates + CLI) | Yes, with coordination | Medium | CLI imports ~20 symbols from contracts + confluence |
| 3+4 (pipeline + CLI) | Yes, with coordination | Low | CLI uses only 4 pipeline public symbols |

Required execution order: **1 → 2 → 3 (can overlap with 4) → 4 → 5**.

Phase 3 (pipeline) and Phase 4 (CLI) can overlap if the pipeline's 4-symbol public API (`Orchestrator`, `PipelineError`, `RunMode`, `RunRequest`) is frozen before CLI work begins.

## Quality Gates

Mandatory on each phase:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

Mandatory after Phase 1:

- CI check that prevents large inline `mod tests { ... }` blocks in production files (locks in test extraction gains).

## Done Criteria

- Test placement policy is fully applied across crates.
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
  - Mitigation: prefer private/unit tests in dedicated `src` test files.
- Risk: schema drift in batch/readiness outputs.
  - Mitigation: keep compatibility checks in test fixtures and rebuild/parity tests.
- Risk: phase ordering violation causing cross-crate breakage.
  - Mitigation: respect dependency graph (leaf crates before pipeline before CLI). See Phase Sequencing Constraints.
- Risk: silent error reclassification during Phase 5 typed-error migration.
  - Mitigation: preserve existing test assertions as regression guards; add typed-error round-trip tests before removing string-based paths.

## Cross-References

- `10-testing-strategy-and-simulation.md` (test layering and scenario intent)
- `13-ci-and-automation.md` (automation gates)
- `06-decisions-and-defaults.md` (safety and verification defaults)
- `09-ai-contract-spec.md` (error and contract expectations)
