## Context

`atlassy-cli` is the only remaining monolithic crate in the workspace. Phases 1-2 of roadmap/15 decomposed the three leaf crates and `atlassy-pipeline` using [Extract Class](https://refactoring.guru/extract-class), producing thin facade `lib.rs` files with focused modules behind them. The CLI was left for last because it sits at the top of the workspace dependency graph — it depends on all other crates but nothing depends on it.

The CLI's `main.rs` (2,780 lines) is a binary crate with no `lib.rs`. This creates a structural constraint: Rust cannot produce a linkable library from a binary crate, so integration tests in `tests/` cannot `use atlassy_cli::*`. The 15 existing tests work around this via `#[cfg(test)] mod tests;` in `main.rs` with `use super::*`, but 14 of them test top-level entry-point functions — they are API tests accessing the API through a back door.

The workspace dependency graph is unchanged by this work:

```
adf, contracts, confluence   (leaf — zero workspace deps)
        ^
        |   (50+ imports)
    pipeline
        ^
        |   (4-symbol API)
      cli
```

## Goals / Non-Goals

**Goals:**
- Decompose `main.rs` into focused modules following the same Extract Class pattern proven in Phases 1-2.
- Extract `lib.rs` alongside `main.rs` to resolve the binary crate testing constraint.
- Redistribute tests to their correct locations per the test placement policy.
- Maintain behavior parity: identical CLI binary interface, batch/readiness JSON output schemas, and test coverage.

**Non-Goals:**
- No new features or behavior changes.
- No Phase 4 work (error taxonomy / `ErrorClass` / `DiagnosticCode` enums) — that depends on this phase and follows immediately after.
- No changes to other crates in the workspace.
- No output schema changes.

## Decisions

### Decision 1: One `types` module, not domain-split

**Choice:** All 33 shared struct/enum definitions go in a single `src/types.rs`.

**Alternatives considered:**
- Domain-split into `types/batch.rs`, `types/kpi.rs`, `types/readiness.rs`, etc. (5-6 sub-modules)

**Rationale:** The type dependency graph has shared references across domains: `GateCheck` is used by both `BatchReport` and `KpiReport`; `KpiReport` appears in both `BatchReport` and `DecisionPacket`; `ReadinessEvidence` contains `BatchReport`. Domain-splitting creates import chains between sub-modules and produces 3 modules with only 1-5 types each — [Lazy Class](https://refactoring.guru/smells/lazy-class) smell. These are pure data definitions with no logic; Extract Class motivation ("when one class does the work of two") doesn't apply to data-only modules.

### Decision 2: Bottom-up incremental extraction, one module per commit

**Choice:** Extract modules from `main.rs` one at a time, starting with zero-dependency modules, testing after each step.

**Alternatives considered:**
- One-pass extraction (move everything at once)
- Two-chunk approach (foundation modules first, then the rest)

**Rationale:** [Extract Class procedure](https://refactoring.guru/extract-class) explicitly prescribes: "Try to relocate just a little bit at a time and test the results after each move, in order to avoid a pileup of error-fixing at the very end." The [How to Refactor](https://refactoring.guru/refactoring/how-to) checklist warns against mixing "a whole bunch of refactorings into one big change." The intermediate state where `main.rs` imports from `lib.rs` while still containing some code is the prescribed Extract Class intermediate — not an anti-pattern.

**Extraction order** (dependency-respecting):

| Step | Module | ~Lines | Dependencies |
|------|--------|--------|--------------|
| 1 | `lib.rs` + `types` | 445 | none (keystone — everything depends on this) |
| 2 | `fixtures` | 50 | none |
| 3 | `provenance` | 70 | types |
| 4 | `manifest` | 130 | types |
| 5 | `io` | 30 | types |
| 6 | `batch/safety` | 85 | types |
| 7 | `batch/kpi` | 260 | types |
| 8 | `readiness/gates` | 180 | types |
| 9 | `readiness/runbooks` | 235 | types |
| 10 | `batch/report` | 460 | types, batch/kpi, batch/safety, provenance |
| 11 | `readiness/evidence` | 180 | types, batch/report, manifest |
| 12 | `readiness/decision_packet` | 250 | types, readiness/gates, readiness/runbooks, batch/report |
| 13 | `commands/run` | 60 | pipeline crate |
| 14 | `commands/create_subpage` | 40 | pipeline/confluence crates |
| 15 | `commands/run_batch` | 108 | manifest, batch/report, commands/run |
| 16 | `commands/run_readiness` | 90 | readiness/*, batch/report |
| 17 | Slim `main.rs` to dispatch | — | commands/* |
| 18 | Redistribute tests | — | all modules extracted |

Steps 1-5 are zero-risk foundation. Steps 6-9 are pure domain modules. Steps 10-12 are the integration-heavy modules. Steps 13-17 are the command layer. Step 18 is test migration.

### Decision 3: 14 tests become integration tests, 1 stays inline

**Choice:** Move 14 of 15 tests from `src/tests.rs` to `tests/` as integration tests. Keep 1 test inline.

**Alternatives considered:**
- Keep all tests as inline `mod tests` in their respective modules (would require `pub(crate)` visibility widening for cross-module calls)
- Move all 15 to integration tests (test 1 tests a private function — can't access from `tests/`)

**Rationale:** [Refactoring.guru's How to Refactor](https://refactoring.guru/refactoring/how-to) states that when tests break after refactoring because they were "testing private methods," then "the tests are to blame." 14 tests access top-level entry-point functions (`execute_batch_from_manifest_file`, `generate_readiness_outputs_from_artifacts`, etc.) that are the CLI's actual public API. Making these `pub` in `lib.rs` is not visibility widening for tests — these are the functions `main()` dispatches to. Test 1 (`map_live_startup_error`) tests a private 7-line adapter; it stays inline in `commands/run.rs`.

**Test file distribution:**

| Test file | Tests | What they exercise |
|-----------|-------|--------------------|
| `tests/batch_report.rs` | 2-9 | Batch execution, report generation, artifact rebuild, error classification |
| `tests/readiness.rs` | 10-15 | Gate evaluation, runbook generation, decision packet assembly, replay verification |
| `src/commands/run.rs` (inline) | 1 | Private `map_live_startup_error` adapter |
| `tests/common/mod.rs` | — | `fixture_path` helper |

### Decision 4: Clap types stay in `main.rs`, not in `types.rs`

**Choice:** `Cli` struct (Parser derive) and `Commands` enum (Subcommand derive) remain in `main.rs`.

**Rationale:** These types are CLI dispatch machinery — they configure argument parsing, not domain data. They're only used in `main()`. Moving them to `lib.rs` would make the library crate depend on clap's derive macros, which is unnecessary coupling. Other crates that might use `atlassy_cli` as a library don't need clap types.

### Decision 5: `lib.rs` as Facade, not re-exporter of everything

**Choice:** `lib.rs` re-exports the public API surface: entry-point functions and output contract types. Internal `pub(crate)` items are NOT re-exported.

**Rationale:** The [Facade pattern](https://refactoring.guru/design-patterns/facade) provides "a simplified interface to a complex subsystem." The CLI facade should expose what consumers need: the batch/readiness entry points and the types they return. Internal helpers like `classify_run_from_summary` are `pub(crate)` for cross-module access but not part of the external API. This follows the same pattern as `atlassy-pipeline`'s 58-line facade.

### Decision 6: Test migration is the final step

**Choice:** Keep tests in `src/tests.rs` (with `use super::*` accessing `main.rs` namespace via `use atlassy_cli::*` re-imports) throughout the extraction, then migrate them to `tests/` as the final step.

**Rationale:** During extraction, functions move from `main.rs` to lib.rs modules. `main.rs` imports them via `use atlassy_cli::*`. Tests access `main.rs` namespace via `use super::*`, which transitively reaches the re-imported symbols. This means tests continue to work unchanged during the entire extraction. Migrating them last avoids fixing tests mid-extraction and separates two kinds of risk: file reorganization vs. test redistribution.

## Risks / Trade-offs

**[Risk] Split-brain during extraction: half in `main.rs`, half in `lib.rs`.**
Mitigation: This is the [Extract Class prescribed intermediate state](https://refactoring.guru/extract-class). `main.rs` does `use atlassy_cli::*;` to reach extracted types/functions. Each commit is green-to-green. The split-brain lasts for ~17 commits, not indefinitely.

**[Risk] Phase 3 amplifies Shotgun Surgery smell that Phase 4 fixes.**
Mitigation: The 15 string-based error classification points in `classify_run_from_summary` and its downstream consumers currently live in one file. After modularization, they'll be spread across `batch/report.rs`, `readiness/runbooks.rs`, and `readiness/decision_packet.rs`. This makes the [Shotgun Surgery](https://refactoring.guru/smells/shotgun-surgery) smell more acute — but Phase 4 (error taxonomy) follows immediately and resolves it by replacing string literals with typed enums and exhaustive match. The two phases should not have a gap between them.

**[Risk] `batch/report.rs` at ~460 lines is the largest module.**
Mitigation: It contains 4 closely related functions (`rebuild_batch_report_from_artifacts`, `build_artifact_index`, `classify_run_from_summary`, `summarize_failure_classes`) that share data flow. Splitting further would create [Feature Envy](https://refactoring.guru/smells/feature-envy) — functions that exist separately but constantly reach into each other's data. If Phase 4 proves the module is still too large, `classify_run_from_summary` can be extracted then (it's the function that becomes simpler once error types exist).

**[Risk] Test count must be preserved exactly across 18 extraction steps.**
Mitigation: `cargo test --workspace` runs after every commit. Test count is verified at the start (baseline) and after test migration (final). No test is removed or renamed — only relocated.

**[Risk] JSON output schema drift.**
Mitigation: All struct definitions move verbatim — field names, serde attributes, and derive macros are unchanged. Existing integration tests in `tests/live_runtime_startup.rs` exercise the CLI binary end-to-end. Batch/readiness tests exercise the output types directly.

## Open Questions

None. The exploration phase resolved all design ambiguities:
- Types module structure: one flat module (decided, see Decision 1)
- Test placement: 14 integration + 1 inline (decided, see Decision 3)
- Extraction order: bottom-up incremental (decided, see Decision 2)
- Phase 3/4 coupling: sequential, no gap (decided, see Risks)
