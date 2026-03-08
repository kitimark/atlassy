## Context

Three leaf crates (`atlassy-adf`, `atlassy-contracts`, `atlassy-confluence`) sit at the bottom of the workspace dependency graph with zero workspace dependencies. Pipeline imports 50+ symbols from all three; CLI imports ~20 from contracts and confluence. Restructuring these crates first avoids dual-breakage when pipeline is later refactored (Phase 3).

```
  adf, contracts, confluence   ← leaf (zero workspace deps) — THIS CHANGE
           ^
           |   (50+ imports)
       pipeline                ← Phase 3
           ^
           |   (4-symbol API)
         cli                   ← Phase 4
```

Phase 1 (test extraction) is complete: no inline `mod tests { ... }` blocks remain, test count is preserved at 107, distribution is 25 in `src/` and 82 in `tests/`.

## Goals / Non-Goals

**Goals:**

- Split each leaf crate's `lib.rs` into focused single-responsibility modules.
- Preserve all public API surfaces via facade re-exports in `lib.rs` — zero downstream import changes.
- Preserve all existing test behavior — `cargo test --workspace` stays green.
- Establish a module structure pattern (facade + domain modules) that pipeline and CLI phases can follow.

**Non-Goals:**

- No behavior changes, output changes, or new features.
- No public API visibility changes (no new `pub` or `pub(crate)` items except within new module boundaries).
- No changes to `atlassy-pipeline` or `atlassy-cli` source code.
- No error taxonomy changes (Phase 5 target — `to_hard_error()` string matching, `&str` error constants, string error classes all stay as-is).
- No Cargo.toml or dependency changes.

## Decisions

### Decision 1: Facade re-export pattern

Each crate's `lib.rs` becomes a thin facade that declares submodules and re-exports all public items at the crate root.

```rust
// lib.rs pattern (example: atlassy-contracts)
mod constants;
mod types;
mod validation;

pub use constants::*;
pub use types::*;
pub use validation::*;
```

**Rationale:** Downstream crates use `use atlassy_contracts::RunSummary` today. Glob re-exports preserve this. The alternative — requiring downstream to use `use atlassy_contracts::types::RunSummary` — would be a breaking change touching 50+ import sites in pipeline alone.

**Alternative considered:** Selective re-exports (`pub use types::RunSummary;` per item). Rejected because it creates a maintenance burden — every new public type requires updating the re-export list in `lib.rs`. Glob re-exports are idiomatic for facade crates.

### Decision 2: atlassy-adf module structure (6 modules)

```
src/
├── lib.rs           facade: re-exports + types (AdfError, ScopeResolution,
│                    PatchCandidate, PatchOperation, TargetRoute) + constants
├── path.rs          JSON pointer utilities (8 functions)
├── index.rs         node path index + ADF tree walkers including collect_text (5 functions)
├── scope.rs         scope resolution (6 functions)
├── patch.rs         patch operations (4 functions)
├── table_guard.rs   target discovery + table classification (4 functions)
└── bootstrap.rs     emptiness detection + scaffolding (2 functions)
```

**Extraction order** (follows dependency direction):

1. `path` — zero internal deps, everything else depends on it
2. `index` — depends only on `path`
3. `scope` — depends on `path` + `index`
4. `patch` — depends on `path`
5. `table_guard` — depends on `path` + `index`
6. `bootstrap` — standalone (zero deps)

**Rationale:** Clusters match the 6 concern groups verified in codebase analysis. The dependency graph is acyclic and flows from `path` outward.

**Key placement decision:** `collect_text` goes in `index`, not `path`. It recursively walks ADF `Value` trees with schema awareness (`"type": "text"` node matching) — it is an ADF content utility, not a JSON pointer string utility. Its callers are `markdown_for_path` (in `table_guard`) and `find_heading_paths` (in `scope`), both of which already depend on `index`.

**Types and constants stay in `lib.rs`:** `AdfError`, `ScopeResolution`, `PatchCandidate`, `PatchOperation`, `TargetRoute`, `EDITABLE_PROSE_TYPES`, and `SCOPE_ANCHOR_TYPES` are referenced across multiple modules. Keeping them in `lib.rs` avoids circular imports and keeps the facade as the canonical location for cross-cutting types.

### Decision 3: atlassy-contracts module structure (3 modules)

```
src/
├── lib.rs           facade: re-exports only
├── constants.rs     21 pub const declarations (versions, error codes, flows, patterns, runtimes)
├── types.rs         30 structs, 5 enums, 5 impl blocks (PipelineState, StateEnvelope, RunSummary, etc.)
└── validation.rs    8 pub fn validators + 2 private helpers
```

**Rationale:** The file is already organized in this exact order (constants L6-30, types L32-421, validation L423-660). Dependencies flow strictly one way: `constants ← types ← validation`. Extraction is mechanical — cut at the existing boundaries.

### Decision 4: atlassy-confluence module structure (3 modules)

```
src/
├── lib.rs           facade: re-exports only
├── types.rs         response structs, ConfluenceError, ConfluenceClient trait
├── stub.rs          StubPage, StubConfluenceClient + trait impl
└── live.rs          LiveConfluenceClient + trait impl + payload builders (private)
```

**Rationale:** Stub and live implementations have zero coupling — they are independent siblings sharing only the trait contract from `types`. The split mirrors the architectural boundary: `types` = contract, `stub` = test double, `live` = production.

**Payload builder test placement:** The 4 tests that directly call `LiveConfluenceClient::build_publish_payload` and `build_create_payload` move from `src/tests.rs` into a `#[cfg(test)] mod tests { ... }` block inside `live.rs`. This preserves private access without widening visibility to `pub(crate)`. The remaining 3 stub tests stay in `src/tests.rs` (accessed via `use super::*` from the crate root).

**Alternative considered:** Extracting payload builders to a separate `payload.rs` module. Rejected because they are private implementation details of `LiveConfluenceClient` with only 2 functions, and separating them would require `pub(crate)` visibility for no architectural benefit.

### Decision 5: Crate extraction order

All three leaf crates are independent (zero workspace deps between them), so they can technically be extracted in any order. The recommended sequence is:

1. **contracts** — simplest split (3 mechanical cuts at existing boundaries), provides confidence in the pattern
2. **confluence** — slightly more complex (test relocation), validates the pattern with trait/impl separation
3. **adf** — most modules (6), most internal coupling, benefits from pattern being proven on simpler crates

**Rationale:** Ascending complexity. Each crate validates the facade pattern before tackling the next. All three can also be done in a single pass if confidence is high.

### Decision 6: Module-internal visibility

Functions that are currently private (`fn`, not `pub fn`) and only used within their new module remain private. Functions that are called from other modules within the same crate become `pub(crate)`. Functions already `pub` stay `pub` and are re-exported from the facade.

No item becomes `pub` or `pub(crate)` solely for test access — tests use `#[cfg(test)] mod tests` within the same module file or `use super::*` from a sibling test file.

## Risks / Trade-offs

**[Risk] Name collisions in glob re-exports** → Mitigation: No two modules export items with the same name (verified during analysis). If a collision is found during extraction, use selective re-exports for the conflicting module.

**[Risk] Private helper functions used across module boundaries** → Mitigation: Functions like `is_json_pointer`, `escape_pointer_segment`, and `parent_path` in `atlassy-adf` are currently private but used by multiple concern clusters. These become `pub(crate)` in the `path` module. This is a visibility widening within the crate, not at the public API level.

**[Risk] Compilation order or circular dependency between new modules** → Mitigation: The dependency graphs for all three crates are acyclic (verified). Cross-cutting types stay in `lib.rs` to prevent any cycles.

**[Risk] Test breakage from moved test files in confluence** → Mitigation: The 4 payload-builder tests move into `live.rs` where they have `super::*` access to `LiveConfluenceClient` private methods. The 3 stub tests remain in `tests.rs` with `super::*` access to crate root re-exports. Both paths are verified to provide the same visibility as today.

**[Risk] Accidental behavior change during file moves** → Mitigation: This is purely structural — functions move between files with no logic changes. Quality gates (`fmt`, `clippy`, `test`) catch any regression. Each crate is extracted and verified independently before moving to the next.
