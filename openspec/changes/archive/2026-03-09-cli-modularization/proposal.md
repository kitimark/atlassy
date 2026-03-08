## Why

`crates/atlassy-cli/src/main.rs` is a 2,780-line monolith containing 57 functions, 37 type definitions, and 4 subcommand handlers in a single file. This is the last [Large Class](https://refactoring.guru/smells/large-class) bloater in the workspace. As a binary crate without `lib.rs`, it cannot be integration-tested via `use atlassy_cli::*`, forcing 15 tests to access the public API through a `use super::*` back door — tests that are [too low-level](https://refactoring.guru/refactoring/how-to) by refactoring.guru standards. Phase 3 of roadmap/15 resolves both issues by extracting `lib.rs` and decomposing the monolith into focused modules, following the same [Extract Class](https://refactoring.guru/extract-class) pattern proven in Phases 1-2.

## What Changes

- Extract `src/lib.rs` alongside `src/main.rs`, resolving the binary crate testing constraint.
- Decompose `main.rs` (2,780 lines) into ~16 focused modules: `types`, `fixtures`, `provenance`, `manifest`, `io`, `batch/report`, `batch/kpi`, `batch/safety`, `readiness/evidence`, `readiness/gates`, `readiness/runbooks`, `readiness/decision_packet`, `commands/run`, `commands/run_batch`, `commands/run_readiness`, `commands/create_subpage`.
- Slim `main.rs` to ~120 lines of CLI argument parsing and dispatch.
- `lib.rs` becomes a [Facade](https://refactoring.guru/design-patterns/facade) re-exporting the public API.
- Redistribute 14 of 15 tests from `src/tests.rs` to `tests/` as integration tests (they test the public API, not internals). 1 test stays inline in its module.
- Delete `src/tests.rs` and `src/test_helpers.rs` after redistribution.
- All extraction follows [Extract Class](https://refactoring.guru/extract-class) procedure: one module per commit, test after each move, start with zero-dependency modules.

## Capabilities

### New Capabilities
- `cli-module-structure`: Module boundaries, facade shape, dependency graph, and visibility rules for the `atlassy-cli` crate after modularization. Parallel to existing `pipeline-module-structure` and `leaf-crate-module-structure` specs.

### Modified Capabilities
- `test-placement-policy`: CLI test placement requirements change. Currently specifies tests in `src/tests.rs` and helpers in `src/test_helpers.rs` with `use super::*` access. After modularization: 14 tests move to `tests/` as integration tests using `use atlassy_cli::*`; 1 test stays inline in `commands/run.rs`; `src/tests.rs` and `src/test_helpers.rs` are deleted; test helper `fixture_path` moves to `tests/common/mod.rs`.

## Impact

- **Code**: `crates/atlassy-cli/src/` — all source files restructured; `main.rs` reduced from 2,780 to ~120 lines; ~18 new files created.
- **Tests**: 15 unit tests redistributed (14 to `tests/`, 1 inline). 2 existing integration tests in `tests/live_runtime_startup.rs` unchanged. Test count preserved exactly.
- **API**: No external API changes. CLI binary interface unchanged. Batch/readiness JSON output schemas unchanged.
- **Dependencies**: No new crate dependencies. Workspace dependency graph unchanged (`cli` depends on `pipeline`, `contracts`, `confluence`).
- **Quality gates**: `cargo fmt`, `cargo clippy`, `cargo test` must pass after every extraction step.
