## Context

`atlassy-pipeline/src/lib.rs` is 1,534 lines containing the complete pipeline implementation: types (`RunMode`, `RunRequest`, `PipelineError`), infrastructure (`StateTracker`, `ArtifactStore`), error mapping (`to_hard_error`, `confluence_error_to_hard_error`, two `From` impls), the `Orchestrator<C>` struct with 15 methods (including 9 state methods), and 12 free helper functions.

Phase 1 established the modularization pattern on leaf crates (adf, contracts, confluence). This design applies the same patterns — facade `lib.rs`, single-responsibility modules, inline `#[cfg(test)] mod tests` — to the pipeline crate, which sits one level above in the dependency graph.

The pipeline's architecture is star-shaped: the orchestrator is the hub, individual states never call each other, and data flows linearly through `StateEnvelope<*Output>` structs. This makes extraction straightforward — each state can be lifted out independently.

## Goals / Non-Goals

**Goals:**
- Reduce `lib.rs` to a thin facade (~30 lines): `mod` declarations and re-exports.
- Each module has one primary responsibility and fits on one screen (~30-200 lines).
- State functions become independently readable and testable.
- Intra-crate dependency graph is acyclic and shallow.
- Existing 3 unit tests redistribute into destination modules as inline `#[cfg(test)] mod tests`.
- All quality gates pass after each extraction step.

**Non-Goals:**
- Changing pipeline behavior, state order, or semantics.
- Changing the public API (`Orchestrator`, `PipelineError`, `RunMode`, `RunRequest`, `StateTracker`).
- Adding new tests (this is a restructuring change; new coverage is a separate concern).
- Introducing abstractions (traits, macros, parameter objects) for state functions.
- Modifying integration tests in `tests/pipeline_integration.rs`.

## Decisions

### Decision 1: Free functions over split impl blocks

**Choice:** Extract state methods as `pub(crate) fn` free functions with explicit parameters, not as methods in separate `impl Orchestrator<C>` blocks.

**Rationale:** 7 of 9 state methods don't use `self.client` — they only use `self.artifact_store` and their input parameters. Keeping them as methods on `Orchestrator` hides this: every method appears to depend on the full `Orchestrator` struct. Free functions make the actual dependency surface explicit in the signature.

**Alternative considered:** Split `impl` blocks (one per file, all on `Orchestrator<C>`). This preserves method syntax (`self.run_fetch_state(...)`) but keeps the false coupling to `Orchestrator`. It also requires `Orchestrator` to be defined in a module visible to all state modules, adding a circular-feeling import pattern.

**Signature transformation:**

```
// Before (method):
fn run_classify_state(&mut self, request, tracker, fetch) -> Result<...>
    // uses: self.artifact_store.persist_state(...)

// After (free function):
pub(crate) fn run_classify_state(
    artifact_store: &ArtifactStore,
    request: &RunRequest,
    tracker: &mut StateTracker,
    fetch: &StateEnvelope<FetchOutput>,
) -> Result<...>
```

For the two states that need the client (fetch, publish):

```
pub(crate) fn run_fetch_state<C: ConfluenceClient>(
    client: &mut C,
    artifact_store: &ArtifactStore,
    request: &RunRequest,
    tracker: &mut StateTracker,
) -> Result<...>
```

### Decision 2: States as a submodule directory

**Choice:** State functions live under `src/states/` as a submodule directory with `src/states/mod.rs` re-exporting all state functions.

**Rationale:** 9 state modules at the top level alongside `orchestrator.rs`, `error_map.rs`, `util.rs`, etc. creates a flat directory with 14+ files where the states (which are the core pipeline concept) aren't visually grouped. A `states/` directory makes the hub-and-spoke architecture visible in the file tree:

```
src/
  lib.rs                 (facade)
  orchestrator.rs        (hub)
  error_map.rs
  artifact_store.rs
  state_tracker.rs
  util.rs
  states/
    mod.rs               (re-exports all state functions)
    fetch.rs
    classify.rs
    extract_prose.rs
    md_assist_edit.rs
    adf_table_edit.rs
    merge_candidates.rs
    patch.rs
    verify.rs
    publish.rs
```

**Alternative considered:** Flat layout with all 14 files at `src/` level. Simpler module declarations, but the states aren't visually distinguished from infrastructure modules.

### Decision 3: `hard_fail` stays on Orchestrator

**Choice:** `hard_fail` remains a private method on `Orchestrator`, not extracted as a free function.

**Rationale:** `hard_fail` mutates `summary` AND returns the error — it's used in `.map_err()` closures inside `run_internal()`. The orchestrator owns the summary and the error-to-summary bookkeeping. Extracting it would require passing `summary` through every call site with no clarity benefit, since it's only called from `run_internal()`.

### Decision 4: `meta()` and `estimate_tokens()` go to `util` before states

**Choice:** Extract `util` module (containing `meta`, `estimate_tokens`, `compute_section_bytes`, `add_duration_suffix`) at tier 3, before individual state modules.

**Rationale:** Every state function calls `meta()` to construct `EnvelopeMeta` for input and output envelopes. If states are extracted first, they can't reach `meta()` — it would still be a private function in `lib.rs`. Extracting `util` first (as `pub(crate)` functions) unblocks all state extractions.

The roadmap places `util` at tier 5, but the dependency is real: states depend on `meta()`. Reordering to tier 3 avoids temporary `pub(crate)` workarounds in `lib.rs`.

**Alternative considered:** Keep `meta()` in `lib.rs` as `pub(crate)` temporarily, extract to `util` later. This works but creates an intermediate state where `lib.rs` has orphaned helper functions — cleaner to move them first.

### Decision 5: Co-locate single-caller helpers with their state modules

**Choice:** Helpers that serve exactly one state module move into that module as private functions.

| Helper | Lines | Destination | Callers |
|---|---|---|---|
| `project_prose_candidate` | 38 | `states/md_assist_edit.rs` | `run_md_assist_edit_state` only |
| `project_table_candidate` | 53 | `states/adf_table_edit.rs` | `run_adf_table_edit_state` only |
| `route_for_node` | 17 | `states/classify.rs` | `run_classify_state` only |
| `has_table_ancestor` | 12 | `states/classify.rs` | `route_for_node` only |
| `parent_path` | 10 | `states/classify.rs` | `has_table_ancestor` only |
| `paths_overlap` | 9 | `states/merge_candidates.rs` | `run_merge_candidates_state` only |

**Rationale:** Placing single-caller helpers in `util` creates Feature Envy — a function that exists only to serve one module but lives elsewhere. Co-location keeps related code together and makes each state module self-contained.

### Decision 6: Bootstrap interlude stays in orchestrator

**Choice:** The empty-page detection and scaffold injection (~50 lines between fetch and classify in `run_internal`) stays in `orchestrator.rs`.

**Rationale:** This is a transition concern, not fetch logic. It reads and mutates the fetch output, injects scaffolding, and writes to the summary. Moving it into `run_fetch_state` would cause Divergent Change — fetch would need modification for both fetching logic changes and bootstrap policy changes. Moving it to its own module would over-fragment a 50-line interlude that has no callers outside `run_internal`.

### Decision 7: Extraction order

**Choice:** Bottom-up extraction in 6 tiers:

```
Tier 1:  state_tracker       (zero coupling — proof of concept)
Tier 2:  error_map            (foundational — PipelineError + converters + From impls)
Tier 3:  artifact_store + util (artifact_store depends on error_map; util unblocks states)
Tier 4:  states/*             (each depends on error_map + util + externals)
Tier 5:  orchestrator         (imports everything — extract last)
```

**Rationale:** Each tier compiles and passes tests independently. `state_tracker` first validates the mechanical pattern (mod declarations, re-exports, CI). `error_map` second because `ArtifactStore` and all state functions depend on `PipelineError`. `util` before states because states depend on `meta()`. Orchestrator last because it imports everything else.

Within tier 4, state order is flexible — no state depends on another. Alphabetical or pipeline-order both work.

### Decision 8: Visibility rules

| Item | Current | After extraction |
|---|---|---|
| `Orchestrator`, `PipelineError`, `RunMode`, `RunRequest`, `StateTracker` | `pub` | `pub` (re-exported from `lib.rs`) |
| `ArtifactStore` | `pub` | `pub` (re-exported — used by `Orchestrator::new()` callers indirectly) |
| State functions (`run_fetch_state`, etc.) | `fn` (private method) | `pub(crate) fn` (called from `orchestrator.rs`) |
| `meta`, `estimate_tokens`, `compute_section_bytes`, `add_duration_suffix` | `fn` (private) | `pub(crate) fn` (called from `orchestrator.rs` and state modules) |
| `to_hard_error`, `confluence_error_to_hard_error` | `fn` (private) | `pub(crate) fn` (called from state modules) |
| Single-caller helpers (`project_prose_candidate`, etc.) | `fn` (private) | `fn` (private in destination module) |
| `hard_fail` | `fn` (private method) | `fn` (private method — stays on `Orchestrator`) |

No item becomes `pub` solely for test access.

## Risks / Trade-offs

**Risk: Accidental behavior drift during extraction.**
Mitigation: Each tier is a single commit that must pass `cargo test --workspace`. Integration tests in `tests/pipeline_integration.rs` exercise the full pipeline end-to-end. State order and data flow are unchanged — only the file location of functions changes.

**Risk: `pub(crate)` proliferation widens internal visibility.**
Mitigation: Only functions that are called cross-module become `pub(crate)`. Single-caller helpers stay private in their destination modules. The public API surface (what downstream crates see) is unchanged.

**Risk: Util extraction before states deviates from roadmap order.**
Mitigation: The change is minor — moving `util` from tier 5 to tier 3. The dependency is real (`meta()` is called by every state). The roadmap notes the extraction order is "recommended", not mandatory.

**Risk: `states/mod.rs` re-export layer adds indirection.**
Mitigation: `mod.rs` is a thin barrel file (~20 lines of `pub(crate) use`). The alternative (9 `mod states_*;` declarations in `lib.rs`) is noisier. The indirection cost is one extra file to maintain.
