## Why

The three leaf crates (`atlassy-adf`, `atlassy-contracts`, `atlassy-confluence`) each pack multiple unrelated concerns into a single `lib.rs` file (620, 660, and 495 lines respectively). This creates Divergent Change — each file changes for unrelated reasons — and makes it harder for both humans and AI to trace responsibility boundaries. Phase 1 (test extraction) is complete; Phase 2 (leaf crate modularization) is the next required step in the roadmap before pipeline or CLI can be safely restructured, because pipeline has 50+ deep imports from all three leaf crates.

## What Changes

- **`atlassy-adf`**: Split `src/lib.rs` (620 lines) into 6 focused modules: `path` (JSON pointer utilities), `index` (node path index + ADF tree walkers), `scope` (scope resolution), `patch` (patch operations), `table_guard` (target discovery + table classification), `bootstrap` (emptiness detection + scaffolding). `lib.rs` becomes a re-export facade.
- **`atlassy-contracts`**: Split `src/lib.rs` (660 lines) into 3 modules: `constants` (21 `pub const` declarations), `types` (30 structs, 5 enums, impl blocks), `validation` (8 validation functions + private helpers). `lib.rs` becomes a re-export facade.
- **`atlassy-confluence`**: Split `src/lib.rs` (495 lines) into 3 modules: `types` (response structs, error enum, `ConfluenceClient` trait), `stub` (in-memory test double), `live` (HTTP client). `lib.rs` becomes a re-export facade. Move 4 payload-builder tests from `src/tests.rs` into `live.rs` as `#[cfg(test)]` to avoid widening visibility.
- All public API surfaces preserved via facade re-exports — zero downstream import changes required.
- All existing test behavior preserved — `cargo test --workspace` remains green.

## Capabilities

### New Capabilities

- `leaf-crate-module-structure`: Defines the module boundaries, naming conventions, and re-export facade pattern for the three leaf crates. Establishes the structural rules that later phases (pipeline, CLI) will follow.

### Modified Capabilities

- `test-placement-policy`: The test relocation within `atlassy-confluence` (moving 4 payload-builder tests from `src/tests.rs` into `src/live.rs`) changes which file hosts those tests. The policy's requirement that private-access tests reside under `src/` is preserved, but the specific file changes from `tests.rs` to `live.rs`.

## Impact

- **Code**: `crates/atlassy-adf/src/`, `crates/atlassy-contracts/src/`, `crates/atlassy-confluence/src/` — each gains new module files; `lib.rs` shrinks to facade.
- **APIs**: No public API changes. All existing imports (`use atlassy_adf::resolve_scope`, `use atlassy_contracts::RunSummary`, etc.) continue to work unchanged.
- **Dependencies**: No new crate dependencies. No Cargo.toml changes.
- **Downstream crates**: `atlassy-pipeline` (50+ imports from leaf crates) and `atlassy-cli` (20+ imports from contracts + confluence) require zero changes due to facade re-exports.
- **CI**: Existing quality gates (`fmt`, `clippy`, `test`) are the acceptance check. No CI configuration changes.
